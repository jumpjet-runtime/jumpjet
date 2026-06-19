use std::collections::VecDeque;
use std::sync::Mutex;
use rand::Rng;
use once_cell::sync::{Lazy, OnceCell};
use wit_bindgen::generate;
use glam::{Mat4};

use crate::exports::jumpjet::runtime::guest::Guest;
use crate::jumpjet::runtime::gpu::*;
use crate::jumpjet::runtime::window;
use crate::jumpjet::runtime::input::{keyboard, KeyboardKey};

generate!({
    world: "game",
    path: ".jumpjet/wit",
    generate_all
});
export!(Game);

struct Game;

#[derive(PartialEq, Clone, Copy, Debug)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

struct SnakeState {
    body: VecDeque<(i32, i32)>,
    direction: Direction,
    next_direction: Direction,
    food: (i32, i32),
    score: u32,
    game_over: bool,
    width_tiles: i32,
    height_tiles: i32,
    last_move_time: f64,
}

static STATE: Lazy<Mutex<SnakeState>> = Lazy::new(|| Mutex::new(SnakeState {
    body: VecDeque::from(vec![(5, 5), (4, 5), (3, 5)]),
    direction: Direction::Right,
    next_direction: Direction::Right,
    food: (10, 10),
    score: 0,
    game_over: false,
    width_tiles: 20,
    height_tiles: 20,
    last_move_time: 0.0,
}));

static RENDER_PIPELINE: OnceCell<GpuRenderPipeline> = OnceCell::new();
static UNIFORM_BIND_GROUP_LAYOUT: OnceCell<GpuBindGroupLayout> = OnceCell::new();
static UNIFORM_BUFFER: OnceCell<GpuBuffer> = OnceCell::new();
static DYNAMIC_VERTEX_BUFFER: OnceCell<GpuBuffer> = OnceCell::new();

struct AudioState {
    context: Option<crate::jumpjet::runtime::audio::AudioContext>,
}


static AUDIO: Lazy<Mutex<AudioState>> = Lazy::new(|| Mutex::new(AudioState {
    context: None,
}));

struct Graphics {
    #[allow(dead_code)]
    surface: GpuSurface,
    device: GpuDevice,
    queue: GpuQueue,
}

static GRAPHICS: OnceCell<Graphics> = OnceCell::new();

const TILE_SIZE: f32 = 32.0;
const MOVE_INTERVAL: f64 = 0.15;

impl Guest for Game {
    fn init() -> Result<(), String> {
        let adapter = crate::jumpjet::runtime::gpu::request_adapter();
        let device = adapter.request_device();
        let queue = device.queue();
        let surface = crate::jumpjet::runtime::gpu::surface();

        // Initialize grid size based on window
        let (window_width, window_height) = window::dimensions();
        let mut state = STATE.lock().unwrap();
        state.width_tiles = (window_width as f32 / TILE_SIZE) as i32;
        state.height_tiles = (window_height as f32 / TILE_SIZE) as i32;
        drop(state);

        // Initialize audio
        let audio_device = crate::jumpjet::runtime::audio::output();
        if let Some(dev) = audio_device {
            let context = dev.create_context();
            let mut audio_state = AUDIO.lock().unwrap();
            audio_state.context = Some(context);
        }



        let format = surface.get_texture_format();

        let shader = device.create_shader_module(&GpuShaderModuleDescriptor {
            label: Some("shader".to_owned()),
            code: r#"
                struct Uniforms {
                    view_proj: mat4x4<f32>,
                }
                @group(0) @binding(0) var<uniform> uniforms: Uniforms;

                struct VertexInput {
                    @location(0) position: vec2<f32>,
                    @location(1) color: vec4<f32>,
                }

                struct VertexOutput {
                    @builtin(position) clip_position: vec4<f32>,
                    @location(0) color: vec4<f32>,
                }

                @vertex
                fn vs_main(model: VertexInput) -> VertexOutput {
                    var out: VertexOutput;
                    out.color = model.color;
                    out.clip_position = uniforms.view_proj * vec4<f32>(model.position, 0.0, 1.0);
                    return out;
                }

                @fragment
                fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
                    return in.color;
                }
            "#.to_owned(),
            hints: vec![],
        });

        // Create uniform buffer
        let uniform_buffer_size = 64;
        let uniform_buffer = device.create_buffer(&GpuBufferDescriptor {
            label: Some("Uniform Buffer".to_owned()),
            size: uniform_buffer_size,
            usage: GpuBufferUsage::UNIFORM | GpuBufferUsage::COPY_DST,
            mapped_at_creation: false,
            contents: None,
        });
        UNIFORM_BUFFER.set(uniform_buffer).unwrap();

        // Create dynamic vertex buffer
        let max_quads = 1000;
        let vertex_buffer_size = max_quads * 6 * (2 + 4) * 4; // 6 verts * 6 floats * 4 bytes
        let vertex_buffer = device.create_buffer(&GpuBufferDescriptor {
            label: Some("Vertex Buffer".to_owned()),
            size: vertex_buffer_size,
            usage: GpuBufferUsage::VERTEX | GpuBufferUsage::COPY_DST,
            mapped_at_creation: false,
            contents: None,
        });
        DYNAMIC_VERTEX_BUFFER.set(vertex_buffer).unwrap();

        let bind_group_layout = device.create_bind_group_layout(&GpuBindGroupLayoutDescriptor {
            entries: vec![GpuBindGroupLayoutEntry {
                binding: 0,
                visibility: GpuShaderStage::VERTEX | GpuShaderStage::FRAGMENT,
                buffer: Some(GpuBufferBindingLayout {
                    type_: GpuBufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: uniform_buffer_size,
                }),
                sampler: None,
                texture: None,
                storage_texture: None,
            }],
        });
        UNIFORM_BIND_GROUP_LAYOUT.set(bind_group_layout).unwrap();

        let pipeline_layout = device.create_pipeline_layout(&GpuPipelineLayoutDescriptor {
            bind_group_layouts: vec![UNIFORM_BIND_GROUP_LAYOUT.get().unwrap()],
        });

        let pipeline = device.create_render_pipeline(&GpuRenderPipelineDescriptor {
            layout: GpuLayout::Pipeline(&pipeline_layout),
            vertex: GpuVertexState {
                module: &shader,
                entry_point: "vs_main".to_owned(),
                constants: vec![],
                buffers: Some(vec![GpuVertexBufferLayout {
                    array_stride: 24, // 6 floats * 4 bytes
                    step_mode: GpuVertexStepMode::Vertex,
                    attributes: vec![
                        GpuVertexAttribute {
                            format: GpuVertexFormat::Float32x2,
                            offset: 0,
                            shader_location: 0,
                        },
                        GpuVertexAttribute {
                            format: GpuVertexFormat::Float32x4,
                            offset: 8,
                            shader_location: 1,
                        },
                    ],
                }]),
            },
            fragment: Some(GpuFragmentState {
                module: &shader,
                entry_point: "fs_main".to_owned(),
                constants: vec![],
                targets: vec![GpuColorTargetState {
                    format,

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
            depth_stencil: None,
            multisample: None,
        });
        RENDER_PIPELINE.set(pipeline).unwrap();

        GRAPHICS.set(Graphics {
            surface,
            device,
            queue,
        }).map_err(|_| "Failed to set GRAPHICS".to_owned())?;

        Ok(())
    }

    fn update(time: f64, _delta_time: f64) {
        let mut state = STATE.lock().unwrap();

        // Input
        if let Some(kb) = keyboard() {
            if kb.is_pressed(&KeyboardKey::ArrowUp) && state.direction != Direction::Down {
                state.next_direction = Direction::Up;
            } else if kb.is_pressed(&KeyboardKey::ArrowDown) && state.direction != Direction::Up {
                state.next_direction = Direction::Down;
            } else if kb.is_pressed(&KeyboardKey::ArrowLeft) && state.direction != Direction::Right {
                state.next_direction = Direction::Left;
            } else if kb.is_pressed(&KeyboardKey::ArrowRight) && state.direction != Direction::Left {
                state.next_direction = Direction::Right;
            }
        }

        if state.game_over {
             // Restart on space
             if let Some(kb) = keyboard() {
                 if kb.is_pressed(&KeyboardKey::Space) {
                     // Reset state
                     state.body = VecDeque::from(vec![(5, 5), (4, 5), (3, 5)]);
                     state.direction = Direction::Right;
                     state.next_direction = Direction::Right;
                     state.score = 0;
                     state.game_over = false;
                     // spawn food
                     loop {
                        let x = rand::thread_rng().gen_range(0..state.width_tiles);
                        let y = rand::thread_rng().gen_range(0..state.height_tiles);
                        if !state.body.contains(&(x, y)) {
                            state.food = (x, y);
                            break;
                        }
                    }
                 }
             }
             return;
        }

        // Move interval
        if time - state.last_move_time < MOVE_INTERVAL {
            return;
        }
        state.last_move_time = time;

        // Apply direction
        state.direction = state.next_direction;

        let mut new_head = *state.body.front().unwrap();
        match state.direction {
            Direction::Up => new_head.1 -= 1,
            Direction::Down => new_head.1 += 1,
            Direction::Left => new_head.0 -= 1,
            Direction::Right => new_head.0 += 1,
        }

        // Wall collision
        if new_head.0 < 0 || new_head.0 >= state.width_tiles || new_head.1 < 0 || new_head.1 >= state.height_tiles {
            state.game_over = true;
            play_sound(440.0, 0.5); // Crash sound
            return;
        }

        // Self collision
        if state.body.contains(&new_head) {
            state.game_over = true;
            play_sound(440.0, 0.5); // Crash sound
            return;
        }

        state.body.push_front(new_head);

        // Eat food
        if new_head == state.food {
            state.score += 1;
            play_sound(880.0, 0.1); // Eat sound
            
            // Spawn new food
             loop {
                let x = rand::thread_rng().gen_range(0..state.width_tiles);
                let y = rand::thread_rng().gen_range(0..state.height_tiles);
                if !state.body.contains(&(x, y)) {
                    state.food = (x, y);
                    break;
                }
            }
        } else {
            state.body.pop_back();
        }
    }

    fn render(_time: f64, _delta_time: f64) {
        let graphics = GRAPHICS.get().unwrap();
        let device = &graphics.device;
        let queue = &graphics.queue;
        let surface = &graphics.surface;
        let view = surface.current_texture().create_view();
        
        let mut encoder = device.create_command_encoder(&GpuCommandEncoderDescriptor { label: None });

        let (window_width, window_height) = window::dimensions();
        let projection = Mat4::orthographic_rh(0.0, window_width as f32, window_height as f32, 0.0, -1.0, 1.0);
        
        // Update uniforms
        let uniform_buffer = UNIFORM_BUFFER.get().unwrap();
        let uniform_data = bytemuck::cast_slice(&projection.to_cols_array()).to_vec();
        queue.write_buffer(
            uniform_buffer,
            0,
            &uniform_data,
            0,
            64
        );
        
        // Build vertices
        let mut vertices: Vec<f32> = Vec::new();
        let state = STATE.lock().unwrap();

        let mut add_quad = |x: i32, y: i32, r: f32, g: f32, b: f32| {
            let x0 = x as f32 * TILE_SIZE;
            let y0 = y as f32 * TILE_SIZE;
            let x1 = x0 + TILE_SIZE;
            let y1 = y0 + TILE_SIZE;
            
            // Triangle 1
            vertices.extend_from_slice(&[x0, y0, r, g, b, 1.0]);
            vertices.extend_from_slice(&[x0, y1, r, g, b, 1.0]);
            vertices.extend_from_slice(&[x1, y1, r, g, b, 1.0]);
            
            // Triangle 2
            vertices.extend_from_slice(&[x0, y0, r, g, b, 1.0]);
            vertices.extend_from_slice(&[x1, y1, r, g, b, 1.0]);
            vertices.extend_from_slice(&[x1, y0, r, g, b, 1.0]);
        };

        // Food detected
        add_quad(state.food.0, state.food.1, 1.0, 0.2, 0.2); // Redish food

        // Snake
        for (i, segment) in state.body.iter().enumerate() {
            let c = if i == 0 { 0.8 } else { 0.6 }; // Head is brighter
            if state.game_over {
                add_quad(segment.0, segment.1, 0.5, 0.5, 0.5); // Grey when dead
            } else {
                add_quad(segment.0, segment.1, 0.2, c, 0.2); // Green
            }
        }
        
        // Write vertices
        let vertex_buffer = DYNAMIC_VERTEX_BUFFER.get().unwrap();
        let vertex_data = bytemuck::cast_slice(&vertices).to_vec();
        queue.write_buffer(
            vertex_buffer,
            0,
            &vertex_data,
            0,
            (vertices.len() * 4) as u64
        );
        
        let vertex_count = (vertices.len() / 6) as u32;

        {
            let mut render_pass = encoder.begin_render_pass(&GpuRenderPassDescriptor {
                color_attachments: vec![GpuRenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    load_op: GpuLoadOp::Clear,
                    store_op: GpuStoreOp::Store,
                    clear_value: Some(vec![0.1, 0.1, 0.1, 1.0]),
                }],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
                max_draw_count: None,
            });

            render_pass.set_pipeline(RENDER_PIPELINE.get().unwrap());
            
            let bind_group = device.create_bind_group(&GpuBindGroupDescriptor {
                label: None,
                layout: UNIFORM_BIND_GROUP_LAYOUT.get().unwrap(),
                entries: vec![GpuBindGroupEntry {
                    binding: 0,
                    resource: GpuBindingResource::Buffer(GpuBufferBinding {
                        buffer: uniform_buffer,
                        offset: 0,
                        size: Some(64),
                    }), 
                }],
            });
            render_pass.set_bind_group(0, Some(&bind_group), None);
            
            render_pass.set_vertex_buffer(0, vertex_buffer, 0, None);
            render_pass.draw(vertex_count, 1, 0, 0);
            render_pass.end();
        }
        
        queue.submit(vec![encoder.finish()]);
    }
}

fn play_sound(freq: f32, duration: f32) {
    let mut audio_state = AUDIO.lock().unwrap();
    if let Some(ctx) = &mut audio_state.context {
        // Create a simple samples buffer
        let sample_rate = 44100.0;
        let num_samples = (duration * sample_rate) as u32;
        let mut samples = Vec::with_capacity(num_samples as usize);
        
        let angular_freq = 2.0 * std::f32::consts::PI * freq;
        for i in 0..num_samples {
            let t = i as f32 / sample_rate;
            let sample = (t * angular_freq).sin();
            samples.push(sample);
        }
        
        let channel_data = vec![samples];
        let audio_buffer = crate::jumpjet::runtime::audio::AudioBuffer::new(&channel_data, sample_rate);
        
        let source = ctx.create_buffer_source();
        source.set_buffer(audio_buffer);
        source.connect(&crate::jumpjet::runtime::audio::AudioNode::Destination(&ctx.destination()));
        source.start();
    }
}
