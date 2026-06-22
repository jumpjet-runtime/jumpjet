use std::path::PathBuf;

use anyhow::Result;
use gilrs::Gilrs;
use slab::Slab;
use uuid::Uuid;
use vfs::VfsPath;
use wasmtime_wasi::{ResourceTable, WasiCtx, WasiCtxBuilder};
use wgpu_types::TextureFormat;
use winit::dpi::PhysicalSize;

use crate::{
    GamepadState, KeyboardState, MouseState,
    runtime::{audio::AudioState, gpu::GpuState, storage::Storage},
};

pub struct JumpjetRuntimeState {
    pub id: Uuid,
    pub generation: u64,
    pub input_path: PathBuf,
    pub window_size: PhysicalSize<u32>,
    pub instance: wgpu_core::global::Global,
    // Window-dependent: present only in the client runtime. A headless server has
    // no window/swapchain, so these are `None` (see [`new_headless`]).
    pub surface: Option<wgpu_core::id::SurfaceId>,
    pub surface_resource_id: Option<u32>,
    pub surface_config: Option<wgpu_types::SurfaceConfiguration<Vec<TextureFormat>>>,
    pub adapter: wgpu_core::id::AdapterId,
    pub adapter_resource_id: u32,
    pub device: wgpu_core::id::DeviceId,
    pub device_resource_id: u32,
    pub queue: wgpu_core::id::QueueId,
    pub queue_resource_id: u32,
    pub gilrs: Gilrs,
    pub gpu_state: GpuState,
    // Absent in a headless server (no audio output device).
    pub audio_state: Option<AudioState>,
    pub gamepad_state: GamepadState,
    pub keyboard_state: KeyboardState,
    pub mouse_state: MouseState,
    pub paths: Slab<VfsPath>,
    pub storages: Slab<Storage>,
    pub wasi_ctx: WasiCtx,
    pub table: ResourceTable,
}

impl JumpjetRuntimeState {
    pub fn new(
        id: Uuid,
        input_path: PathBuf,
        window_size: PhysicalSize<u32>,
        audio_device: cpal::Device,
        instance: wgpu_core::global::Global,
        surface: wgpu_core::id::SurfaceId,
        adapter: wgpu_core::id::AdapterId,
        device: wgpu_core::id::DeviceId,
        queue: wgpu_core::id::QueueId,
        gilrs: Gilrs,
    ) -> Self {
        let mut table = ResourceTable::new();

        let swapchain_capabilities = instance.surface_get_capabilities(surface, adapter).unwrap();
        let swapchain_format = swapchain_capabilities.formats[0];

        let surface_config = wgpu_types::SurfaceConfiguration {
            usage: wgpu_types::TextureUsages::RENDER_ATTACHMENT,
            format: swapchain_format,
            width: window_size.width,
            height: window_size.height,
            present_mode: swapchain_capabilities.present_modes[0],
            alpha_mode: swapchain_capabilities.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 1,
        };

        instance.surface_configure(surface, device, &surface_config);

        JumpjetRuntimeState {
            id,
            generation: 0,
            input_path,
            window_size,
            audio_state: Some(AudioState::new(audio_device)),
            instance,
            surface: Some(surface),
            surface_resource_id: Some(table.push(surface).unwrap().rep()),
            surface_config: Some(surface_config),
            adapter,
            adapter_resource_id: table.push(adapter).unwrap().rep(),
            device,
            device_resource_id: table.push(device).unwrap().rep(),
            queue,
            queue_resource_id: table.push(queue).unwrap().rep(),
            gilrs,
            gpu_state: GpuState::new(),
            gamepad_state: GamepadState::new(),
            keyboard_state: KeyboardState::new(),
            mouse_state: MouseState::new(),
            paths: Slab::new(),
            storages: Slab::new(),
            wasi_ctx: WasiCtxBuilder::new()
                .inherit_stderr()
                .inherit_stdout()
                .build(),
            table,
        }
    }

    /// Builds the state for a headless server: no window/swapchain (`surface*` and
    /// `surface_config` are `None`), no audio output (`audio_state` is `None`), and
    /// no local input devices used. The `server-runtime` world imports none of
    /// gpu/window/input/audio, so those host interfaces are never linked or called.
    ///
    /// A GPU adapter/device is still created (the `instance`/`adapter`/`device`/
    /// `queue` fields are non-optional). This requires an adapter to be available;
    /// making the server fully GPU-free is a follow-up (would mean threading
    /// `Option` through the gpu host impls).
    pub fn new_headless(id: Uuid, input_path: PathBuf) -> Result<Self> {
        let mut table = ResourceTable::new();

        let instance = wgpu_core::global::Global::new(
            "jumpjet-server",
            &wgpu_types::InstanceDescriptor {
                backends: wgpu_types::Backends::all(),
                flags: wgpu_types::InstanceFlags::from_build_config(),
                ..Default::default()
            },
        );
        // No surface to be compatible with — a server never presents.
        let adapter = instance
            .request_adapter(&Default::default(), wgpu_types::Backends::all(), None)
            .map_err(|e| anyhow::anyhow!("no GPU adapter available for headless server: {e:?}"))?;
        let adapter_limits = instance.adapter_limits(adapter);
        let (device, queue) = instance
            .adapter_request_device(
                adapter,
                &wgpu_types::DeviceDescriptor {
                    label: None,
                    required_features: wgpu_types::Features::empty(),
                    required_limits: wgpu_types::Limits::downlevel_webgl2_defaults()
                        .using_resolution(adapter_limits),
                    memory_hints: wgpu_types::MemoryHints::default(),
                },
                None,
                None,
                None,
            )
            .map_err(|e| anyhow::anyhow!("failed to create headless device: {e:?}"))?;

        let adapter_resource_id = table.push(adapter).unwrap().rep();
        let device_resource_id = table.push(device).unwrap().rep();
        let queue_resource_id = table.push(queue).unwrap().rep();

        Ok(JumpjetRuntimeState {
            id,
            generation: 0,
            input_path,
            window_size: PhysicalSize::new(0, 0),
            audio_state: None,
            instance,
            surface: None,
            surface_resource_id: None,
            surface_config: None,
            adapter,
            adapter_resource_id,
            device,
            device_resource_id,
            queue,
            queue_resource_id,
            gilrs: Gilrs::new()
                .map_err(|e| anyhow::anyhow!("failed to init gilrs (headless): {e}"))?,
            gpu_state: GpuState::new(),
            gamepad_state: GamepadState::new(),
            keyboard_state: KeyboardState::new(),
            mouse_state: MouseState::new(),
            paths: Slab::new(),
            storages: Slab::new(),
            wasi_ctx: WasiCtxBuilder::new()
                .inherit_stderr()
                .inherit_stdout()
                .build(),
            table,
        })
    }
}
