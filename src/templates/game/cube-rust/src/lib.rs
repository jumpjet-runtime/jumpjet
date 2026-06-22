use glam::{Mat4, Vec3};
use once_cell::sync::OnceCell;
use wit_bindgen::generate;

use crate::exports::jumpjet::runtime::game::{Game, Guest, GuestGame};
use crate::jumpjet::runtime::gpu::*;

generate!({
    world: "game",
    path: ".jumpjet/wit",
    generate_all
});
export!(Component);

struct Component;

impl Guest for Component {
    type Game = MyGame;
}

struct MyGame;

static RENDER_PIPELINE: OnceCell<GpuRenderPipeline> = OnceCell::new();
static UNIFORM_BIND_GROUP: OnceCell<GpuBindGroup> = OnceCell::new();
static VERTICES_BUFFER: OnceCell<GpuBuffer> = OnceCell::new();
static UNIFORM_BUFFER: OnceCell<GpuBuffer> = OnceCell::new();
static DEPTH_TEXTURE: OnceCell<GpuTexture> = OnceCell::new();

static cube_vertex_size: u64 = 4 * 10; // Byte size of one cube vertex.
static cube_position_offset: u64 = 0;
static cube_color_offset: u64 = 4 * 4; // Byte offset of cube vertex color attribute.
static cube_uv_offset: u64 = 4 * 8;
static cube_vertex_count: u32 = 36;

static cube_vertices: &'static [f32] = &[
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
];

fn get_transform_matrix(projection_matrix: Mat4, time: f64) -> Mat4 {
    let translation = Mat4::from_translation(Vec3::new(0.0f32, 0.0f32, -4.0f32));
    let rotation = Mat4::from_axis_angle(
        Vec3::new((time as f32).sin(), (time as f32).cos(), 0.0f32),
        1.0f32,
    );

    let view_matrix = Mat4::IDENTITY;
    let transformed = view_matrix.mul_mat4(&translation).mul_mat4(&rotation);

    projection_matrix.mul_mat4(&transformed)
}

impl GuestGame for MyGame {
    fn init() -> Result<Game, String> {
        let adapter = crate::jumpjet::runtime::gpu::request_adapter();
        let device = adapter.request_device();
        let (window_width, window_height) = crate::jumpjet::runtime::window::dimensions();

        let cube_vertices_buffer = bytemuck::cast_slice(&cube_vertices).to_vec();

        VERTICES_BUFFER
            .set(device.create_buffer(&GpuBufferDescriptor {
                label: None,
                size: (cube_vertices.len() * std::mem::size_of::<f32>()) as u64,
                usage: GpuBufferUsage::VERTEX,
                mapped_at_creation: true,
                contents: Some(cube_vertices_buffer),
            }))
            .unwrap();

        DEPTH_TEXTURE
            .set(device.create_texture(&GpuTextureDescriptor {
                size: GpuExtentD3 {
                    height: window_height,
                    width: window_width,
                    depth_or_array_layers: 1,
                },
                format: GpuTextureFormat::Depth24plus,
                usage: GpuTextureUsage::RENDER_ATTACHMENT,
                dimension: GpuTextureDimension::D2,
                mip_level_count: 1,
                sample_count: 1,
                view_formats: vec![GpuTextureFormat::Depth24plus],
            }))
            .unwrap();

        let uniform_buffer_size: u64 = 4 * 16; // 4x4 matrix
        let uniform_buffer = device.create_buffer(&GpuBufferDescriptor {
            label: None,
            size: uniform_buffer_size,
            usage: GpuBufferUsage::UNIFORM | GpuBufferUsage::COPY_DST,
            mapped_at_creation: false,
            contents: None,
        });

        let vertex_shader = device.create_shader_module(&GpuShaderModuleDescriptor {
            label: None,
            code: "
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
            "
            .to_string(),
            hints: vec![],
        });

        let surface_format = crate::jumpjet::runtime::gpu::surface().get_texture_format();

        let frag_shader = device.create_shader_module(&GpuShaderModuleDescriptor {
            label: None,
            code: "
              @fragment
              fn main(
                @location(0) fragUV: vec2<f32>,
                @location(1) fragPosition: vec4<f32>
              ) -> @location(0) vec4<f32> {
                return fragPosition;
              }            
            "
            .to_string(),
            hints: vec![],
        });

        let pipeline = device.create_render_pipeline(&GpuRenderPipelineDescriptor {
            layout: GpuLayout::Auto,
            vertex: GpuVertexState {
                module: &vertex_shader,
                entry_point: "main".to_owned(),
                constants: Vec::new(),
                buffers: Some(vec![GpuVertexBufferLayout {
                    array_stride: cube_vertex_size,
                    step_mode: GpuVertexStepMode::Vertex,
                    attributes: vec![
                        // position
                        GpuVertexAttribute {
                            format: GpuVertexFormat::Float32x4,
                            offset: cube_position_offset,
                            shader_location: 0,
                        },
                        // uv
                        GpuVertexAttribute {
                            format: GpuVertexFormat::Float32x2,
                            offset: cube_uv_offset,
                            shader_location: 1,
                        },
                    ],
                }]),
            },
            fragment: Some(GpuFragmentState {
                module: &frag_shader,
                entry_point: "main".to_owned(),
                constants: Vec::new(),
                targets: vec![GpuColorTargetState {
                    format: surface_format,
                    blend: Some(GpuBlendState {
                        color: GpuBlendComponent {
                            src_factor: Some(GpuBlendFactor::One),
                            dst_factor: Some(GpuBlendFactor::Zero),
                            operation: Some(GpuBlendOperation::Add),
                        },
                        alpha: GpuBlendComponent {
                            src_factor: Some(GpuBlendFactor::One),
                            dst_factor: Some(GpuBlendFactor::Zero),
                            operation: Some(GpuBlendOperation::Add),
                        },
                    }),
                    write_mask: Some(GpuColorWrite::ALL),
                }],
            }),
            primitive: Some(GpuPrimitiveState {
                topology: Some(GpuPrimitiveTopology::TriangleList),
                strip_index_format: None,
                front_face: Some(GpuFrontFace::Ccw),
                cull_mode: GpuCullMode::None,
                unclipped_depth: false,
            }),
            depth_stencil: Some(GpuDepthStencilState {
                format: GpuTextureFormat::Depth24plus,
                depth_write_enabled: true,
                depth_compare: GpuCompareFunction::Less,
                stencil_front: None,
                stencil_back: None,
                stencil_read_mask: None,
                stencil_write_mask: None,
                depth_bias: None,
                depth_bias_slope_scale: None,
                depth_bias_clamp: None,
            }),
            multisample: None,
        });

        UNIFORM_BIND_GROUP
            .set(device.create_bind_group(&GpuBindGroupDescriptor {
                label: None,
                layout: &pipeline.get_bind_group_layout(0),
                entries: vec![GpuBindGroupEntry {
                    binding: 0,
                    resource: GpuBindingResource::Buffer(GpuBufferBinding {
                        buffer: &uniform_buffer,
                        offset: 0,
                        size: Some(uniform_buffer_size),
                    }),
                }],
            }))
            .unwrap();

        UNIFORM_BUFFER.set(uniform_buffer).unwrap();

        RENDER_PIPELINE.set(pipeline).unwrap();

        Ok(Game::new(MyGame))
    }

    fn update(&self, time: f64, delta_time: f64) { }

    fn render(&self, time: f64, alpha: f64) {
        let adapter = crate::jumpjet::runtime::gpu::request_adapter();
        let device = adapter.request_device();
        let queue = device.queue();
        let view = crate::jumpjet::runtime::gpu::surface()
            .current_texture()
            .create_view();

        let mut encoder =
            device.create_command_encoder(&GpuCommandEncoderDescriptor { label: None });

        let depth_texture_view = DEPTH_TEXTURE.get().unwrap().create_view();

        let projection_matrix = Mat4::perspective_rh(
            (2.0 * std::f32::consts::PI) / 5.0,
            1600.0 / 1200.0,
            0.1,
            1000.0,
        );

        let model_view_projection_matrix =
            get_transform_matrix(projection_matrix, time).to_cols_array();
        let model_view_projection_matrix_bytes =
            bytemuck::cast_slice(&model_view_projection_matrix).to_vec();

        let uniform_buffer = UNIFORM_BUFFER.get().unwrap();

        queue.write_buffer(
            &uniform_buffer,
            0,
            &model_view_projection_matrix_bytes,
            0,
            model_view_projection_matrix_bytes.len() as u64,
        );

        let mut render_pass = encoder.begin_render_pass(&GpuRenderPassDescriptor {
            color_attachments: vec![GpuRenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                load_op: GpuLoadOp::Clear,
                store_op: GpuStoreOp::Store,
                clear_value: Some(vec![0.5, 0.5, 0.5, 1.0]),
            }],
            depth_stencil_attachment: Some(GpuRenderPassDepthStencilAttachment {
                view: &depth_texture_view,
                depth_clear_value: 1.0,
                depth_load_op: GpuLoadOp::Clear,
                depth_store_op: GpuStoreOp::Store,
                depth_read_only: false,
                stencil_clear_value: 1,
                stencil_load_op: GpuLoadOp::Clear,
                stencil_store_op: GpuStoreOp::Store,
                stencil_read_only: false,
            }),
            occlusion_query_set: None,
            timestamp_writes: None,
            max_draw_count: None,
        });

        render_pass.set_pipeline(RENDER_PIPELINE.get().unwrap());
        render_pass.set_bind_group(0, Some(UNIFORM_BIND_GROUP.get().unwrap()), None);
        render_pass.set_vertex_buffer(0, VERTICES_BUFFER.get().unwrap(), 0, None);
        render_pass.draw(cube_vertex_count, 1, 0, 0);
        render_pass.end();

        queue.submit(vec![encoder.finish()]);
    }
}
