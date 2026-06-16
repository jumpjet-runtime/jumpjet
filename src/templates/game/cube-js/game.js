import { log } from 'jumpjet:runtime/debug'
import * as gpu from 'jumpjet:runtime/gpu'
import * as window from 'jumpjet:runtime/window'

const disposeSymbol = Symbol.dispose || Symbol.for('dispose')

const cubeVertexSize = 4 * 10
const cubePositionOffset = 0
const cubeColorOffset = 4 * 4
const cubeUvOffset = 4 * 8
const cubeVertexCount = 36
const cubeVertices = new Float32Array([
  1.0, -1.0, 1.0, 1.0, 1.0, 0.0, 1.0, 1.0, 0.0, 1.0, -1.0, -1.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0,
  1.0, 1.0, -1.0, -1.0, -1.0, 1.0, 0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 1.0, -1.0, -1.0, 1.0, 1.0, 0.0,
  0.0, 1.0, 0.0, 0.0, 1.0, -1.0, 1.0, 1.0, 1.0, 0.0, 1.0, 1.0, 0.0, 1.0, -1.0, -1.0, -1.0, 1.0,
  0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 0.0, 1.0, 1.0, -1.0, 1.0,
  1.0, 1.0, 0.0, 1.0, 1.0, 1.0, 1.0, 1.0, -1.0, -1.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0, 1.0,
  1.0, -1.0, 1.0, 1.0, 1.0, 0.0, 1.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 0.0, 1.0,
  1.0, -1.0, -1.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0, -1.0, 1.0, 1.0, 1.0, 0.0, 1.0, 1.0, 1.0,
  0.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, -1.0, 1.0, 1.0, 1.0, 0.0,
  1.0, 1.0, 0.0, -1.0, 1.0, -1.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0, -1.0, 1.0, 1.0, 1.0, 0.0,
  1.0, 1.0, 1.0, 0.0, 1.0, 1.0, 1.0, -1.0, 1.0, 1.0, 1.0, 0.0, 1.0, 1.0, 0.0, -1.0, -1.0, 1.0,
  1.0, 0.0, 0.0, 1.0, 1.0, 0.0, 1.0, -1.0, 1.0, 1.0, 1.0, 0.0, 1.0, 1.0, 1.0, 1.0, 1.0, -1.0,
  1.0, -1.0, 1.0, 0.0, 1.0, 0.0, 1.0, 1.0, 0.0, -1.0, -1.0, -1.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0,
  0.0, -1.0, -1.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0, 1.0, -1.0, 1.0, -1.0, 1.0, 0.0, 1.0, 0.0,
  1.0, 1.0, 0.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 0.0, 1.0, -1.0, 1.0, 1.0, 1.0, 0.0, 1.0,
  1.0, 1.0, 1.0, 1.0, -1.0, -1.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 1.0, 0.0, -1.0, -1.0, 1.0, 1.0,
  0.0, 0.0, 1.0, 1.0, 1.0, 0.0, 1.0, -1.0, 1.0, 1.0, 1.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 1.0,
  1.0, 1.0, 1.0, 1.0, 1.0, 0.0, 1.0, 1.0, -1.0, -1.0, 1.0, 1.0, 0.0, 0.0, 1.0, 0.0, 1.0, -1.0,
  -1.0, -1.0, 1.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, -1.0, 1.0, -1.0, 1.0, 0.0, 1.0, 0.0, 1.0, 1.0,
  0.0, 1.0, 1.0, -1.0, 1.0, 1.0, 1.0, 0.0, 1.0, 0.0, 0.0, 1.0, -1.0, -1.0, 1.0, 1.0, 0.0, 0.0,
  1.0, 0.0, 1.0, -1.0, 1.0, -1.0, 1.0, 0.0, 1.0, 0.0, 1.0, 1.0, 0.0,
])

let verticesBuffer, depthTexture, uniformBindGroup, uniformBuffer, renderPipeline

export const guest = {
  init() {
    let adapter = gpu.requestAdapter()
    let device = adapter.requestDevice()
    
    let [windowWidth, windowHeight] = window.dimensions()

    verticesBuffer = device.createBuffer({
      size: cubeVertices.buffer.byteLength,
      usage: {
        vertex: true
      },
      mappedAtCreation: true,
      contents: cubeVertices.buffer
    })

    depthTexture = device.createTexture({
      size: {
        height: windowHeight,
        width: windowWidth,
        depthOrArrayLayers: 1
      },
      format: 'depth24plus',
      usage: {
        renderAttachment: true
      },
      dimension: 'd2',
      mipLevelCount: 1,
      sampleCount: 1,
      viewFormats: ['depth24plus']
    })

    let uniformBufferSize = 4 * 16;
    let uniformBuffer = device.createBuffer({
      size: uniformBufferSize,
      usage: {
        uniform: true,
        copyDst: true
      },
      mappedAtCreation: false
    })

    let vertexShader = device.createShaderModule({
      code: `
        struct Uniforms {
          modelViewProjectionMatrix : mat4x4<f32>,
        }
        @binding(0) @group(0) var<uniform> uniforms : Uniforms;
        
        struct VertexOutput {
          @builtin(position) Position : vec4<f32>,
          @location(0) fragUV : vec2<f32>,
          @location(1) fragPosition: vec4<f32>,
        }
        
        @vertex
        fn main(
          @location(0) position : vec4<f32>,
          @location(1) uv : vec2<f32>
        ) -> VertexOutput {
          var output : VertexOutput;
          output.Position = uniforms.modelViewProjectionMatrix * position;
          output.fragUV = uv;
          output.fragPosition = 0.5 * (position + vec4(1.0, 1.0, 1.0, 1.0));
          return output;
        }
      `,
      hints: []
    })

    let fragShader = device.createShaderModule({
      code: `
        @fragment
        fn main(
          @location(0) fragUV: vec2<f32>,
          @location(1) fragPosition: vec4<f32>
        ) -> @location(0) vec4<f32> {
          return fragPosition;
        }
      `,
      hints: []
    })

    renderPipeline = device.createRenderPipeline({
      layout: { tag: 'auto' },
      vertex: {
        module: vertexShader,
        entryPoint: 'main',
        constants: [],
        buffers: [
          {
            arrayStride: cubeVertexSize,
            stepMode: 'vertex',
            attributes: [
              // position
              {
                format: 'float32x4',
                offset: cubePositionOffset,
                shaderLocation: 0
              },
              // uv
              {
                format: 'float32x2',
                offset: cubeUvOffset,
                shaderLocation: 1
              }
            ]
          }
        ]
      },
      fragment: {
        module: fragShader,
        entryPoint: 'main',
        constants: [],
        targets: [
          {
            format: 'rgba8unormsrgb',
            blend: {
              color: {
                srcFactor: 'one',
                dstFactor: 'zero',
                operation: 'add'
              },
              alpha: {
                srcFactor: 'one',
                dstFactor: 'zero',
                operation: 'add'
              }
            },
            writeMask: {
              all: true
            }
          }
        ]
      },
      primitive: {
        topology: 'triangle-list',
        stripIndexFormat: null,
        frontFace: 'ccw',
        cullMode: 'none',
        unclippedDepth: false
      },
      depthStencil: {
        format: 'depth24plus',
        depthWriteEnabled: true,
        depthCompare: 'less',
        stencilFront: null,
        stencilBack: null,
        stencilReadMask: null,
        stencilWriteMask: null,
        depthBias: null,
        depthBiasSlopeScale: null,
        depthBiasClamp: null
      },
      multisample: null
    })

    let layout = renderPipeline.getBindGroupLayout(0)

    // uniformBindGroup = device.createBindGroup({
    //   layout,
    //   entries: [
    //     {
    //       binding: 0,
    //       resource: {
    //         tag: 'buffer',
    //         val: {
    //           buffer: uniformBuffer,
    //           offset: 0,
    //           size: uniformBufferSize
    //         }
    //       }
    //     }
    //   ]
    // })
  },
  update(time, deltaTime) {
    log('update')
  },
  render(time, deltaTime) {
    // let adapter = requestAdapter();
    // let device = adapter.requestDevice();
    // let queue = device.queue();
    // let view = surface().currentTexture().createView();

    // let encoder = device.createCommandEncoder({
    //   label: null
    // });

    // let renderPass = encoder.beginRenderPass({
    //   colorAttachments: [{
    //     view,
    //     resolveTarget: null,
    //     // loadOp: GpuLoadOp.Load,
    //     loadOp: 'load',
    //     // storeOp: GpuStoreOp.Store,
    //     storeOp: 'store',
    //     clearValue: null
    //   }],
    //   depthStencilAttachment: null,
    //   occlusionQuerySet: null,
    //   timestampWrites: null,
    //   maxDrawCount: null
    // });

    // renderPass.setPipeline(renderPipeline);
    // renderPass.draw(3, 1, 0, 0);
    // renderPass.end();

    // queue.submit([encoder.finish()]);

    // view[disposeSymbol]()
    // device[disposeSymbol]()
    // queue[disposeSymbol]()

    
  }
}
