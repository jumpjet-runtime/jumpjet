import math
import struct
from jumpjet.runtime import gpu
from jumpjet.runtime import window

class Mat4:
    @staticmethod
    def identity():
        return [
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            0.0, 0.0, 0.0, 1.0,
        ]

    @staticmethod
    def perspective(fovy, aspect, near, far):
        f = 1.0 / math.tan(fovy / 2.0)
        nf = 1.0 / (near - far)
        return [
            f / aspect, 0.0, 0.0, 0.0,
            0.0, f, 0.0, 0.0,
            0.0, 0.0, (far + near) * nf, -1.0,
            0.0, 0.0, (2.0 * far * near) * nf, 0.0,
        ]

    @staticmethod
    def translation(x, y, z):
        return [
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            x, y, z, 1.0,
        ]

    @staticmethod
    def rotation_z(angle):
        c = math.cos(angle)
        s = math.sin(angle)
        return [
            c, s, 0.0, 0.0,
            -s, c, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            0.0, 0.0, 0.0, 1.0,
        ]
        
    @staticmethod
    def rotation_axis(axis, angle):
        x, y, z = axis
        c = math.cos(angle)
        s = math.sin(angle)
        t = 1 - c
        
        return [
            t*x*x + c,   t*x*y + z*s, t*x*z - y*s, 0.0,
            t*x*y - z*s, t*y*y + c,   t*y*z + x*s, 0.0,
            t*x*z + y*s, t*y*z - x*s, t*z*z + c,   0.0,
            0.0,         0.0,         0.0,         1.0
        ]

    @staticmethod
    def mul(a, b):
        out = [0.0] * 16
        for r in range(4):
            for c in range(4):
                out[c*4 + r] = (
                    a[0*4 + r] * b[c*4 + 0] +
                    a[1*4 + r] * b[c*4 + 1] +
                    a[2*4 + r] * b[c*4 + 2] +
                    a[3*4 + r] * b[c*4 + 3]
                )
        return out

CUBE_VERTICES = [
    # Front face
    1.0, -1.0, 1.0, 1.0, 1.0, 0.0, 1.0, 1.0, 0.0, 1.0, 
    -1.0, -1.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0, 
    -1.0, -1.0, 1.0, 0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 1.0, 
    # Right face
    -1.0, -1.0, 1.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 
    -1.0, 1.0, 1.0, 1.0, 0.0, 1.0, 1.0, 0.0, 1.0, -1.0, 
    -1.0, -1.0, 1.0, 0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 1.0, 
    # Back face
    1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 0.0, 1.0, 1.0, -1.0, 
    1.0, 1.0, 1.0, 0.0, 1.0, 1.0, 1.0, 1.0, -1.0, -1.0, 
    1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0, 1.0, 1.0, -1.0, 
    1.0, 1.0, 0.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0, 1.0, 
    # Left face
    1.0, -1.0, -1.0, 1.0, 1.0, 0.0, 1.0, 0.0, 0.0, 1.0, 
    1.0, 1.0, -1.0, 1.0, 1.0, 1.0, 1.0, 0.0, 1.0, 1.0, 
    -1.0, -1.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0, 1.0, 
    # Top face
    1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 
    1.0, -1.0, -1.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 1.0, 
    0.0, -1.0, -1.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0, 
    # Bottom face
    0.0, 1.0, 1.0, 1.0, 1.0, 0.0, 1.0, 1.0, -1.0, -1.0, 
    1.0, 1.0, 0.0, 0.0, 1.0, 0.0, 1.0, -1.0, -1.0, -1.0, 
    1.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, -1.0, 1.0, -1.0, 
    1.0, 0.0, 1.0, 0.0, 1.0, 1.0, 0.0, 1.0, 1.0, -1.0, 
    1.0, 1.0, 1.0, 0.0, 1.0, 0.0, 0.0, 1.0, -1.0, -1.0, 
    1.0, 1.0, 0.0, 0.0, 1.0, 0.0, 1.0, -1.0, 1.0, -1.0, 
    1.0, 0.0, 1.0, 0.0, 1.0, 1.0, 0.0,
]

VERTEX_SHADER = """
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
"""

FRAGMENT_SHADER = """
@fragment
fn main(
  @location(0) fragUV: vec2<f32>,
  @location(1) fragPosition: vec4<f32>
) -> @location(0) vec4<f32> {
  return fragPosition;
}
"""

class Guest:
    def __init__(self):
        self.render_pipeline = None
        self.bind_group = None
        self.vertices_buffer = None
        self.uniform_buffer = None
        self.depth_texture = None
        self.uniform_buffer_size = 4 * 16

    def init(self):
        print("Initializing Python Cube Demo")
        adapter = gpu.request_adapter()
        device = adapter.request_device()
        
        width, height = window.dimensions()
        
        vertex_data = struct.pack(f'{len(CUBE_VERTICES)}f', *CUBE_VERTICES)
        
        self.vertices_buffer = device.create_buffer(gpu.GpuBufferDescriptor(
            size=len(vertex_data),
            usage=gpu.GpuBufferUsage.VERTEX,
            mapped_at_creation=True,
            contents=list(vertex_data)
        ))
        
        self.depth_texture = device.create_texture(gpu.GpuTextureDescriptor(
            size=gpu.GpuExtentD3(width=width, height=height, depth_or_array_layers=1),
            format=gpu.GpuTextureFormat.DEPTH24PLUS,
            usage=gpu.GpuTextureUsage.RENDER_ATTACHMENT,
            dimension=gpu.GpuTextureDimension.D2,
            mip_level_count=1,
            sample_count=1,
            view_formats=[gpu.GpuTextureFormat.DEPTH24PLUS]
        ))
        
        self.uniform_buffer = device.create_buffer(gpu.GpuBufferDescriptor(
            size=self.uniform_buffer_size,
            usage=gpu.GpuBufferUsage.UNIFORM | gpu.GpuBufferUsage.COPY_DST,
            mapped_at_creation=False,
            contents=None
        ))
        
        vertex_module = device.create_shader_module(gpu.GpuShaderModuleDescriptor(
            code=VERTEX_SHADER,
            hints=[]
        ))
        fragment_module = device.create_shader_module(gpu.GpuShaderModuleDescriptor(
            code=FRAGMENT_SHADER,
            hints=[]
        ))
        
        pipeline_layout = device.create_pipeline_layout(gpu.GpuPipelineLayoutDescriptor(
            bind_group_layouts=[]
        ))

        self.render_pipeline = device.create_render_pipeline(gpu.GpuRenderPipelineDescriptor(
            layout=gpu.GpuLayout.AUTO,
            vertex=gpu.GpuVertexState(
                module=vertex_module,
                entry_point="main",
                buffers=[gpu.GpuVertexBufferLayout(
                    array_stride=4 * 10,
                    step_mode=gpu.GpuVertexStepMode.VERTEX,
                    attributes=[
                        gpu.GpuVertexAttribute(
                            format=gpu.GpuVertexFormat.FLOAT32X4,
                            offset=0,
                            shader_location=0
                        ),
                        gpu.GpuVertexAttribute(
                            format=gpu.GpuVertexFormat.FLOAT32X2,
                            offset=4 * 8, # uv offset
                            shader_location=1
                        )
                    ]
                )]
            ),
            fragment=gpu.GpuFragmentState(
                module=fragment_module,
                entry_point="main",
                targets=[gpu.GpuColorTargetState(
                    format=gpu.GpuTextureFormat.RGBA8UNORMSRGB,
                    blend=gpu.GpuBlendState(
                        color=gpu.GpuBlendComponent(
                            src_factor=gpu.GpuBlendFactor.ONE,
                            dst_factor=gpu.GpuBlendFactor.ZERO,
                            operation=gpu.GpuBlendOperation.ADD
                        ),
                        alpha=gpu.GpuBlendComponent(
                            src_factor=gpu.GpuBlendFactor.ONE,
                            dst_factor=gpu.GpuBlendFactor.ZERO,
                            operation=gpu.GpuBlendOperation.ADD
                        )
                    ),
                    write_mask=gpu.GpuColorWrite.ALL
                )]
            ),
            primitive=gpu.GpuPrimitiveState(
                topology=gpu.GpuPrimitiveTopology.TRIANGLE_LIST,
                front_face=gpu.GpuFrontFace.CCW,
                cull_mode=gpu.GpuCullMode.NONE
            ),
            depth_stencil=gpu.GpuDepthStencilState(
                format=gpu.GpuTextureFormat.DEPTH24PLUS,
                depth_write_enabled=True,
                depth_compare=gpu.GpuCompareFunction.LESS
            )
        ))
        
        bind_group_layout = self.render_pipeline.get_bind_group_layout(0)
        
        self.bind_group = device.create_bind_group(gpu.GpuBindGroupDescriptor(
            layout=bind_group_layout,
            entries=[gpu.GpuBindGroupEntry(
                binding=0,
                resource=gpu.GpuBindingResource.BUFFER(gpu.GpuBufferBinding(
                    buffer=self.uniform_buffer,
                    offset=0,
                    size=self.uniform_buffer_size
                ))
            )]
        ))
        
        return None

    def update(self, time, delta_time):
        pass

    def render(self, time, alpha):
        adapter = gpu.request_adapter()
        device = adapter.request_device()
        queue = device.queue()
        surface = gpu.surface()
        view = surface.current_texture().create_view()
        
        encoder = device.create_command_encoder(gpu.GpuCommandEncoderDescriptor(label=None))
        depth_view = self.depth_texture.create_view()
        
        projection = Mat4.perspective(
            (2.0 * math.pi) / 5.0,
            1600.0 / 1200.0,
            0.1,
            1000.0
        )
        
        rotation = Mat4.rotation_axis([math.sin(time), math.cos(time), 0.0], 1.0)
        translation = Mat4.translation(0.0, 0.0, -4.0)
        
        model_view = Mat4.mul(translation, rotation)
        mvp = Mat4.mul(projection, model_view)
        
        mvp_bytes = struct.pack('16f', *mvp)
        
        queue.write_buffer(
            self.uniform_buffer,
            0,
            list(mvp_bytes),
            0,
            len(mvp_bytes)
        )
        
        render_pass = encoder.begin_render_pass(gpu.GpuRenderPassDescriptor(
            color_attachments=[gpu.GpuRenderPassColorAttachment(
                view=view,
                load_op=gpu.GpuLoadOp.CLEAR,
                store_op=gpu.GpuStoreOp.STORE,
                clear_value=[0.5, 0.5, 0.5, 1.0]
            )],
            depth_stencil_attachment=gpu.GpuRenderPassDepthStencilAttachment(
                view=depth_view,
                depth_clear_value=1.0,
                depth_load_op=gpu.GpuLoadOp.CLEAR,
                depth_store_op=gpu.GpuStoreOp.STORE,
                stencil_clear_value=0,
                stencil_load_op=gpu.GpuLoadOp.CLEAR,
                stencil_store_op=gpu.GpuStoreOp.STORE
            )
        ))
        
        render_pass.set_pipeline(self.render_pipeline)
        render_pass.set_bind_group(0, self.bind_group, [])
        render_pass.set_vertex_buffer(0, self.vertices_buffer, 0, None)
        render_pass.draw(36, 1, 0, 0)
        render_pass.end()
        
        queue.submit([encoder.finish()])
