use std::path::PathBuf;

use gilrs::Gilrs;
use slab::Slab;
use uuid::Uuid;
use vfs::VfsPath;
use wasmtime_wasi::{ResourceTable};
use wasmtime_wasi::p2::{WasiCtx, WasiCtxBuilder};
use wgpu_types::TextureFormat;
use winit::dpi::PhysicalSize;

use crate::{runtime::{audio::AudioState, gpu::GpuState, storage::Storage}, GamepadState, KeyboardState};

pub struct RuneRuntimeState {
    pub id: Uuid,
    pub generation: u64,
    pub input_path: PathBuf,
    pub window_size: PhysicalSize<u32>,
    pub instance: wgpu_core::global::Global,
    pub surface: wgpu_core::id::SurfaceId,
    pub surface_resource_id: u32,
    pub surface_config: wgpu_types::SurfaceConfiguration<Vec<TextureFormat>>,
    pub adapter: wgpu_core::id::AdapterId,
    pub adapter_resource_id: u32,
    pub device: wgpu_core::id::DeviceId,
    pub device_resource_id: u32,
    pub queue: wgpu_core::id::QueueId,
    pub queue_resource_id: u32,
    pub gilrs: Gilrs,
    pub gpu_state: GpuState,
    pub audio_state: AudioState,
    pub gamepad_state: GamepadState,
    pub keyboard_state: KeyboardState,
    pub paths: Slab<VfsPath>,
    pub storages: Slab<Storage>,
    pub wasi_ctx: WasiCtx,
    pub table: ResourceTable,
}

impl RuneRuntimeState {
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

        let swapchain_capabilities = instance
            .surface_get_capabilities(surface, adapter)
            .unwrap();
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

        RuneRuntimeState {
            id,
            generation: 0,
            input_path,
            window_size,
            audio_state: AudioState::new(audio_device),
            instance,
            surface,
            surface_resource_id: table.push(surface).unwrap().rep(),
            surface_config,
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
            paths: Slab::new(),
            storages: Slab::new(),
            wasi_ctx: WasiCtxBuilder::new()
                .inherit_stderr()
                .inherit_stdout()
                .build(),
            table,
        }
    }
}