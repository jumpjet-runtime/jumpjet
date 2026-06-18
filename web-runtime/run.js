// Jumpjet web harness. Wires the wasm-bindgen host runtime (`web.js`) to the
// jco-transpiled guest (`guest/guest.js`) and the WASI shim, then starts the
// runtime. Emitted verbatim by `jumpjet build --target web`.
//
// Resource interfaces (gpu, input, …) are Rust #[wasm_bindgen] classes exported
// from web.js; this harness only references those constructors into the jco
// import object and performs the async GPU pre-resolution the sync WIT can't.

import initHost, * as host from './web.js';
import { instantiate } from './guest/guest.js';
import { cli, clocks, filesystem, http, io, random } from '@bytecodealliance/preview2-shim';

// WASI imports for the guest, satisfied by jco's preview2 browser shim.
const wasi = {
  'wasi:cli/environment': cli.environment,
  'wasi:cli/exit': cli.exit,
  'wasi:cli/stderr': cli.stderr,
  'wasi:cli/stdin': cli.stdin,
  'wasi:cli/stdout': cli.stdout,
  'wasi:cli/terminal-input': cli.terminalInput,
  'wasi:cli/terminal-output': cli.terminalOutput,
  'wasi:cli/terminal-stderr': cli.terminalStderr,
  'wasi:cli/terminal-stdin': cli.terminalStdin,
  'wasi:cli/terminal-stdout': cli.terminalStdout,
  'wasi:clocks/monotonic-clock': clocks.monotonicClock,
  'wasi:clocks/wall-clock': clocks.wallClock,
  'wasi:filesystem/preopens': filesystem.preopens,
  'wasi:filesystem/types': filesystem.types,
  'wasi:http/outgoing-handler': http.outgoingHandler,
  'wasi:http/types': http.types,
  'wasi:io/error': io.error,
  'wasi:io/poll': io.poll,
  'wasi:io/streams': io.streams,
  'wasi:random/random': random.random,
};

// Resource interfaces are wasm-bindgen classes exported from web.js; the import
// object just references those constructors + factory fns.
const inputImports = {
  KeyboardDevice: host.KeyboardDevice, MouseDevice: host.MouseDevice,
  TouchDevice: host.TouchDevice, GamepadDevice: host.GamepadDevice,
  keyboard: host.inputKeyboard, mouse: host.inputMouse,
  touch: host.inputTouch, gamepad: host.inputGamepad,
};
const audioImports = {
  output: host.audioOutput,
  AudioDevice: host.AudioDevice, AudioContext: host.AudioContext, AudioParam: host.AudioParam,
  AudioBuffer: host.AudioBuffer, AudioRenderCapacity: host.AudioRenderCapacity,
  AudioDestinationNode: host.AudioDestinationNode, AudioListener: host.AudioListener,
  PeriodicWave: host.PeriodicWave, AnalyzerNode: host.AnalyzerNode,
  BiquadFilterNode: host.BiquadFilterNode, AudioBufferSourceNode: host.AudioBufferSourceNode,
  ConstantSourceNode: host.ConstantSourceNode, ConvolverNode: host.ConvolverNode,
  ChannelMergerNode: host.ChannelMergerNode, ChannelSplitterNode: host.ChannelSplitterNode,
  DelayNode: host.DelayNode, DynamicsCompressorNode: host.DynamicsCompressorNode,
  GainNode: host.GainNode, IirFilterNode: host.IirFilterNode, OscillatorNode: host.OscillatorNode,
  PannerNode: host.PannerNode, StereoPannerNode: host.StereoPannerNode, WaveShaperNode: host.WaveShaperNode,
};
const storageImports = {
  StorageDevice: host.StorageDevice, Path: host.Path,
  local: host.storageLocal, cloud: host.storageCloud,
};
const gpuImports = {
  GpuAdapter: host.GpuAdapter, GpuDevice: host.GpuDevice, GpuQueue: host.GpuQueue,
  GpuBuffer: host.GpuBuffer, GpuTexture: host.GpuTexture, GpuTextureView: host.GpuTextureView,
  GpuShaderModule: host.GpuShaderModule, GpuBindGroupLayout: host.GpuBindGroupLayout,
  GpuBindGroup: host.GpuBindGroup, GpuPipelineLayout: host.GpuPipelineLayout,
  GpuSampler: host.GpuSampler, GpuQuerySet: host.GpuQuerySet, GpuCommandBuffer: host.GpuCommandBuffer,
  GpuRenderPipeline: host.GpuRenderPipeline, GpuComputePipeline: host.GpuComputePipeline,
  GpuCommandEncoder: host.GpuCommandEncoder, GpuRenderPassEncoder: host.GpuRenderPassEncoder,
  GpuComputePassEncoder: host.GpuComputePassEncoder, GpuRenderBundle: host.GpuRenderBundle,
  GpuRenderBundleEncoder: host.GpuRenderBundleEncoder, GpuSurface: host.GpuSurface,
  requestAdapter: host.gpuRequestAdapter, surface: host.gpuSurface,
};

async function getCoreModule(path) {
  const res = await fetch(new URL('./guest/' + path, import.meta.url));
  return WebAssembly.compile(await res.arrayBuffer());
}

async function main() {
  await initHost();

  window.jco = {
    // The Rust host (host/web/game.rs) creates the canvas, then calls
    // window.jco.instantiate(jumpjetImports) with the debug+window imports.
    instantiate: async (jumpjetImports) => {
      const canvas = document.getElementById('jumpjet-canvas');

      // request-adapter/request-device are sync in WIT but async in the browser,
      // so pre-resolve here and hand the device + configured context to Rust.
      const adapter = await navigator.gpu.requestAdapter();
      const device = await adapter.requestDevice();
      const context = canvas.getContext('webgpu');
      const surfaceViewFormat = 'rgba8unorm-srgb';
      context.configure({
        device,
        format: 'rgba8unorm',
        viewFormats: [surfaceViewFormat],
        alphaMode: 'opaque',
      });
      host.gpuSetContext(device, context, surfaceViewFormat);

      const imports = Object.assign(
        {},
        wasi,
        {
          'jumpjet:runtime/gpu': gpuImports,
          'jumpjet:runtime/input': inputImports,
          'jumpjet:runtime/audio': audioImports,
          'jumpjet:runtime/storage': storageImports,
        },
        jumpjetImports,
      );

      return instantiate(getCoreModule, imports);
    },
  };

  host.run();
}

main();
