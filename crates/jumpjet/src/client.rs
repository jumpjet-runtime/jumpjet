//! Host bindings for the **client/singleplayer** component (`client-runtime`
//! profile world, which exports the `game` interface).
//!
//! This is the *primary* `bindgen!` for the crate: it generates the host-side
//! bindings for every imported `jumpjet:runtime/*` interface at
//! `crate::client::jumpjet::runtime::*` (re-exported as `crate::jumpjet::…` from
//! `lib.rs`), plus `ClientRuntime`/`ClientRuntimePre` and the `game` export. The
//! server bindings (`server.rs`) reuse these generated interface modules so the
//! `Host` impls in `runtime/native/*` are written once.

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
