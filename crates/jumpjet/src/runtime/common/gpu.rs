use std::collections::HashMap;

use wgpu_core::id::{BufferId, QuerySetId, TextureId};
use wgpu_types::{BufferUsages, QueryType, TextureDimension, TextureFormat, TextureUsages};

use crate::gpu::GpuBufferMapState;

pub struct Buffer {
    pub usage: BufferUsages,
    pub map_state: GpuBufferMapState,
    pub size: u64,
}

pub struct QuerySet {
    pub count: u32,
    pub type_: QueryType,
}

pub struct Texture {
    pub height: u32,
    pub width: u32,
    pub depth_or_array_layers: u32,
    pub mip_level_count: u32,
    pub sample_count: u32,
    pub dimension: TextureDimension,
    pub format: TextureFormat,
    pub usage: TextureUsages,
}

pub struct GpuState {
    pub buffers: HashMap<BufferId, Buffer>,
    pub query_sets: HashMap<QuerySetId, QuerySet>,
    pub textures: HashMap<TextureId, Texture>,
    pub present_surface: bool,
    pub current_surface_texture: Option<TextureId>,
}

impl GpuState {
    pub fn new() -> GpuState {
        GpuState {
            buffers: HashMap::new(),
            query_sets: HashMap::new(),
            textures: HashMap::new(),
            present_surface: false,
            current_surface_texture: None,
        }
    }
}
