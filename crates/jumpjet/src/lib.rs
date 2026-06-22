#[cfg(not(target_arch = "wasm32"))]
use anyhow::Result;
// Input tracking types are native-only; the web build tracks input in
// `runtime/web/input.rs` (DOM events + navigator.getGamepads).
#[cfg(not(target_arch = "wasm32"))]
use winit::keyboard::{Key, KeyLocation, PhysicalKey};

pub mod host;
pub mod runtime;
// Pulley AOT (precompile host-side, deserialize on iOS); shares `pulley_config`
// with the on-device loader.
#[cfg(not(target_arch = "wasm32"))]
pub mod aot;
#[cfg(not(target_arch = "wasm32"))]
pub mod debug;
#[cfg(not(target_arch = "wasm32"))]
pub mod tests;

// Re-exported so the generated Android bundle wrapper can name `AndroidApp`
// against the exact `winit`/`android-activity` version the runtime links (the
// glue symbol must come from a single version — see `runtime::run_android`).
#[cfg(not(target_arch = "wasm32"))]
pub use winit;

// needed for wasmtime::component::bindgen! as it only looks in the current crate.
#[cfg(not(target_arch = "wasm32"))]
pub(crate) use gilrs;
#[cfg(not(target_arch = "wasm32"))]
pub(crate) use wgpu_core;

pub type BufferSource = Vec<u8>;

#[cfg(not(target_arch = "wasm32"))]
pub fn wgpu_id<I: wgpu_core::id::Marker, E>(
    (id, error): (wgpu_core::id::Id<I>, Option<E>),
) -> Result<wgpu_core::id::Id<I>, E> {
    match error {
        Some(error) => Err(error),
        None => core::result::Result::Ok(id),
    }
}

#[cfg(not(target_arch = "wasm32"))]
wasmtime::component::bindgen!({
    world: "jumpjet:runtime/client-runtime",
    path: "wit/runtime",
    with: {
        "jumpjet:runtime/audio.audio-buffer": web_audio_api::AudioBuffer,
        "jumpjet:runtime/audio.audio-context": web_audio_api::context::AudioContext,
        "jumpjet:runtime/audio.audio-param": web_audio_api::AudioParam,
        "jumpjet:runtime/audio.analyzer-node": web_audio_api::node::AnalyserNode,
        "jumpjet:runtime/audio.audio-buffer-source-node": web_audio_api::node::AudioBufferSourceNode,
        "jumpjet:runtime/audio.audio-destination-node": web_audio_api::node::AudioDestinationNode,
        "jumpjet:runtime/audio.biquad-filter-node": web_audio_api::node::BiquadFilterNode,
        "jumpjet:runtime/audio.constant-source-node": web_audio_api::node::ConstantSourceNode,
        "jumpjet:runtime/audio.convolver-node": web_audio_api::node::ConvolverNode,
        "jumpjet:runtime/audio.channel-merger-node": web_audio_api::node::ChannelMergerNode,
        "jumpjet:runtime/audio.channel-splitter-node": web_audio_api::node::ChannelSplitterNode,
        "jumpjet:runtime/audio.delay-node": web_audio_api::node::DelayNode,
        "jumpjet:runtime/audio.dynamics-compressor-node": web_audio_api::node::DynamicsCompressorNode,
        "jumpjet:runtime/audio.gain-node": web_audio_api::node::GainNode,
        "jumpjet:runtime/audio.iir-filter-node": web_audio_api::node::IIRFilterNode,
        "jumpjet:runtime/audio.oscillator-node": web_audio_api::node::OscillatorNode,
        "jumpjet:runtime/audio.panner-node": web_audio_api::node::PannerNode,
        "jumpjet:runtime/audio.audio-render-capacity": web_audio_api::AudioRenderCapacity,
        "jumpjet:runtime/audio.stereo-panner-node": web_audio_api::node::StereoPannerNode,
        "jumpjet:runtime/audio.wave-shaper-node": web_audio_api::node::WaveShaperNode,
        "jumpjet:runtime/audio.audio-listener": web_audio_api::AudioListener,

        "jumpjet:runtime/gpu.gpu-adapter": wgpu_core::id::AdapterId,
        "jumpjet:runtime/gpu.gpu-device": wgpu_core::id::DeviceId,
        "jumpjet:runtime/gpu.gpu-queue": wgpu_core::id::QueueId,
        "jumpjet:runtime/gpu.gpu-buffer": wgpu_core::id::BufferId,
        "jumpjet:runtime/gpu.gpu-command-encoder": wgpu_core::id::CommandEncoderId,
        "jumpjet:runtime/gpu.gpu-compute-pass-encoder": wgpu_core::command::ComputePass,
        "jumpjet:runtime/gpu.gpu-render-pass-encoder": wgpu_core::command::RenderPass,
        "jumpjet:runtime/gpu.gpu-render-bundle": wgpu_core::id::RenderBundleId,
        "jumpjet:runtime/gpu.gpu-render-bundle-encoder": wgpu_core::command::RenderBundleEncoder,
        "jumpjet:runtime/gpu.gpu-shader-module": wgpu_core::id::ShaderModuleId,
        "jumpjet:runtime/gpu.gpu-bind-group": wgpu_core::id::BindGroupId,
        "jumpjet:runtime/gpu.gpu-bind-group-layout": wgpu_core::id::BindGroupLayoutId,
        "jumpjet:runtime/gpu.gpu-pipeline-layout": wgpu_core::id::PipelineLayoutId,
        "jumpjet:runtime/gpu.gpu-compute-pipeline": wgpu_core::id::ComputePipelineId,
        "jumpjet:runtime/gpu.gpu-render-pipeline": wgpu_core::id::RenderPipelineId,
        "jumpjet:runtime/gpu.gpu-command-buffer": wgpu_core::id::CommandBufferId,
        "jumpjet:runtime/gpu.gpu-sampler": wgpu_core::id::SamplerId,
        "jumpjet:runtime/gpu.gpu-texture": wgpu_core::id::TextureId,
        "jumpjet:runtime/gpu.gpu-texture-view": wgpu_core::id::TextureViewId,
        "jumpjet:runtime/gpu.gpu-query-set": wgpu_core::id::QuerySetId,
        // "jumpjet:runtime/gpu.buffer-source": BufferSource,

        // Host-resident asset resources live in the `ResourceTable`, mapped to
        // their backing Rust types (same pattern as audio/gpu) rather than a
        // bespoke `Slab` field on the state.
        "jumpjet:runtime/tasks.buffer": crate::runtime::common::tasks::Buffer,
        "jumpjet:runtime/tasks.task": crate::runtime::common::tasks::Task,
        "jumpjet:runtime/image.image-bitmap": crate::runtime::common::image::Bitmap,
        "jumpjet:runtime/model.gltf-model": crate::runtime::common::model::GltfModel,

        "jumpjet:runtime/input.gamepad-device": gilrs::GamepadId,

        // "jumpjet:runtime/network.network-client": crate::runtime::network::NetworkClient,
        // "jumpjet:runtime/network.network-server": crate::runtime::network::NetworkServer,
        // "jumpjet:runtime/network.network-connection": crate::runtime::network::NetworkConnection,
        // "jumpjet:runtime/network.network-http-client": crate::runtime::network::NetworkHttpClient,
    },
    imports: {
        default: async
    },
    exports: {
        default: async
    }
});

// wasmtime::component::bindgen!({
//     world: "jumpjet:tests/tests",
//     path: "wit/tests",
//     with: {
//         "jumpjet:runtime/debug": jumpjet::runtime::debug
//     }
// });

#[cfg(not(target_arch = "wasm32"))]
pub use exports::jumpjet::runtime::game;
#[cfg(not(target_arch = "wasm32"))]
pub use jumpjet::runtime::*;
#[cfg(not(target_arch = "wasm32"))]
use wasmtime_wasi::{ResourceTable, WasiCtxView, WasiView};

pub use runtime::JumpjetRuntimeState;

#[cfg(not(target_arch = "wasm32"))]
impl WasiView for JumpjetRuntimeState {
    fn ctx(&mut self) -> WasiCtxView<'_> {
        WasiCtxView {
            ctx: &mut self.wasi_ctx,
            table: &mut self.table,
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub struct KeyboardState {
    /// `(generation, physical_key, logical_key, location)`.
    ///
    /// Entries are matched for insert/remove on `physical_key`, which is
    /// modifier- and layout-invariant, so a key always clears on release even
    /// if modifier state changed since it was pressed (e.g. releasing Shift
    /// before the letter you were holding). The `logical_key`/`location` are
    /// retained only as the value reported back to the guest.
    pub active_keys: Vec<(u64, PhysicalKey, Key, KeyLocation)>,
}

#[cfg(not(target_arch = "wasm32"))]
impl KeyboardState {
    pub fn new() -> KeyboardState {
        Self {
            active_keys: Vec::new(),
        }
    }
}

/// Mouse cursor + pointer-lock state, mirrored from winit events by the run loop.
#[cfg(not(target_arch = "wasm32"))]
#[derive(Default)]
pub struct MouseState {
    /// Cursor position in physical pixels relative to the window, top-left origin.
    pub x: f32,
    pub y: f32,
    /// Raw movement accumulated since the last logic frame, in physical pixels.
    pub dx: f32,
    pub dy: f32,
    /// Mouse buttons currently held down, mirrored from winit press/release.
    pub buttons: Vec<winit::event::MouseButton>,
    /// Whether the pointer is currently locked (cursor grabbed + hidden).
    pub locked: bool,
    /// Pending lock (`Some(true)`) / unlock (`Some(false)`) request from the
    /// guest, applied to the window by the run loop and then cleared.
    pub lock_request: Option<bool>,
}

#[cfg(not(target_arch = "wasm32"))]
impl MouseState {
    pub fn new() -> MouseState {
        Self::default()
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub struct GamepadState {
    pub active_buttons: Vec<(u64, gilrs::GamepadId, gilrs::Button, bool)>,
}

#[cfg(not(target_arch = "wasm32"))]
impl GamepadState {
    pub fn new() -> GamepadState {
        Self {
            active_buttons: Vec::new(),
        }
    }
}
