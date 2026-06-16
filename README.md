# 🚧 Jumpjet 🚧

[jumpjet.dev](https://jumpjet.dev)

A runtime for cross-language, cross-platform game development. A chassis without an engine.

🚧 Jumpjet is not production ready.

## Get Started

A quick install script is available [on the website](https://jumpjet.dev).

## What is Jumpjet?
Jumpjet provides a host environment for guest WebAssembly applications (ie. your game.)

Guest applications have access to the Jumpjet Runtime API, which includes access to features like GPU, audio, storage, input, networking, UI, window management and more. Think of it like SDL for WebAssembly. Jumpjet _is not_ a web environment - method invocations from your application are executed against "native" APIs.

Jumpjet models much of its APIs heavily on the Web platform because of its familiarity and foundational design principles; for example the GPU API is basically WebGPU and the audio API is basically WebAudio.

## How does it work
Guest applications implement the simple Jumpjet Guest protocol, which includes `init`, `update`, `render` methods to be implemented. The protocol is provided as a collection of `.wit` files, which can be compiled to a library in the language of your choice. (We may release official packages for various languges in the future.) Once you've implemented the protocol, you build your application targetting WebAssembly, with WASI support. You can then run your application in the Jumpjet CLI. When you're ready to publish your game, Jumpjet CLI can bundle your application into various platform outputs such as `.app` for Mac, `.msi` for Windows, etc.

## What platforms are supported?
Jumpjet is built on Wasmtime with Rust, and will be able to theoretically support any platform and any language in the future. For now we're limiting the scope of our support to Windows, Mac, and Linux, with support for iOS and Android coming later.

## Example Guest Application (Javascript)

```js
import { log } from 'jumpjet:runtime/debug'
import {
    requestAdapter,
    GpuTextureFormat,
    GpuBlendFactor,
    GpuColorWrite,
    GpuPrimitiveTopology,
    GpuFrontFace,
    GpuCullMode,
    GpuLoadOp,
    GpuStoreOp
} from 'jumpjet:runtime/gpu'
import { requestAdapter } from 'jumpjet:runtime/gpu'

export const guest = {
  init() {
    let adapter = requestAdapter();
    let device = adapter.requestDevice();
    
    let shader = device.createShaderModule({
      label: null,
      code: `
          @vertex
          fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> @builtin(position) vec4<f32> {
              let x = f32(i32(in_vertex_index) - 1);
              let y = f32(i32(in_vertex_index & 1u) * 2 - 1);
              return vec4<f32>(x, y, 0.0, 1.0);
          }

          @fragment
          fn fs_main() -> @location(0) vec4<f32> {
              return vec4<f32>(1.0, 0.0, 0.0, 1.0);
          }
      `,
      hints: []
    });

    let pipelineLayout = device.createPipelineLayout({
      bindGroupLayouts: []
    });

    renderPipeline = device.createRenderPipeline({
      layout: { tag: 'pipeline', val: pipelineLayout },
      vertex: {
        module: shader,
        entryPoint: 'vs_main',
        constants: [],
        buffers: null
      },
      fragment: {
        module: shader,
        entryPoint: 'fs_main',
        constants: [],
        targets: [{
          format: GpuTextureFormat.Bgra8unormsrgb,
          blend: {
            color: {
              srcFactor: GpuBlendFactor.One,
              dstFactor: GpuBlendFactor.Zero,
              operation: GpuBlendOperation.Add,
            },
            alpha: {
              srcFactor: GpuBlendFactor.One,
              dstFactor: GpuBlendFactor.Zero,
              operation: GpuBlendOperation.Add,
            }
          },
          writeMask: GpuColorWrite.ALL,
        }],
      },
      primitive: {
        topology: GpuPrimitiveTopology.TriangleList,
        stripIndexFormat: null,
        frontFace: GpuFrontFace.Ccw,
        cullMode: GpuCullMode.None,
        unclippedDepth: false  
      },
      depthStencil: null,
      multisample: null
    });
  },
  update(time, deltaTime) {
    log(time);
  },
  render(view, device, queue) {
    let encoder = device.createCommandEncoder({
      label: null
    });

    let renderPass = encoder.beginRenderPass({
      colorAttachments: [{
        view,
        resolveTarget: null,
        loadOp: GpuLoadOp.Load,
        storeOp: GpuStoreOp.Store,
        clearValue: null
      }],
      depthStencilAttachment: null,
      occlusionQuerySet: null,
      timestampWrites: null,
      maxDrawCount: null
    });

    renderPass.setPipeline(renderPipeline);
    renderPass.draw(3, 1, 0, 0);
    renderPass.end();

    queue.submit([encoder.finish()]);
  }
}

```

## Future Plans
Jumpjet is a commercial endeavour, as such commercial features are planned to be integrated into the Runtime API. The core runtime will remain free and will be continually improved. You will always be able to publish games on the Jumpjet runtime without a licensing fee. Some future functionality will be incorporated into the Jumpjet commercial platform. More information will be made available later on the commercial platform and its features, but our intent is to provide fair pricing that scales with your game's success.

## Libraries in use
Jumpjet would not be possible without the dedicated efforts of the people behind these incredible projects. Please consider supporting them if you're enjoying Jumpjet.

- [wasmtime](https://github.com/bytecodealliance/wasmtime)
- [wgpu](https://github.com/gfx-rs/wgpu)
- [web-audio-api-rs](https://github.com/orottier/web-audio-api-rs)
- [winit](https://github.com/rust-windowing/winit)
- [gilrs](https://gitlab.com/gilrs-project/gilrs)
- [cargo-bundle](https://github.com/burtonageo/cargo-bundle)
