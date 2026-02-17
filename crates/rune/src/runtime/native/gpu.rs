use std::{borrow::Cow, collections::HashMap, num::NonZeroU64};

use wasmtime::{component::Resource, Result};
use wgpu_core::{
    binding_model::{BindGroupLayoutDescriptor, PipelineLayoutDescriptor},
    command::PassChannel,
    pipeline::VertexBufferLayout,
    resource::{BufferMapOperation, TextureViewDescriptor},
};
use wgpu_types::{
    BindGroupLayoutEntry, Color, ColorTargetState, ColorWrites, DepthBiasState, DepthStencilState,
    DynamicOffset, Face, FrontFace, ImageSubresourceRange, MultisampleState, Origin3d, PolygonMode,
    PrimitiveState, PrimitiveTopology, StencilFaceState, StencilState, TexelCopyBufferLayout,
    TexelCopyTextureInfo, TextureDimension, TextureUsages, VertexAttribute,
};

use crate::{
    rune::runtime::gpu::*,
    runtime::gpu::{Buffer, QuerySet, Texture},
    wgpu_id,
};

use super::state::RuneRuntimeState;

use super::utilities::{convert_bind_group_entry, vec_to_color};

impl RuneRuntimeState {}

impl Host for RuneRuntimeState {
    async fn surface(&mut self) -> Resource<GpuSurface> {
        Resource::new_own(self.surface_resource_id)
    }

    async fn request_adapter(&mut self) -> Resource<GpuAdapter> {
        // let adapter_id = self.instance.request_adapter(
        //     &RequestAdapterOptions {
        //         power_preference: wgpu_core::resource::PowerPreference::HighPerformance,
        //         force_fallback_adapter: false,
        //         compatible_surface: None
        //     },
        //     AdapterInputs::Mask(Backends::all(), |_| ())
        // ).unwrap();
        // Ok(self.table.push(adapter_id).unwrap())
        Resource::new_own(self.adapter_resource_id)
    }
}

impl HostGpuSurface for RuneRuntimeState {
    async fn current_texture(&mut self, _surface: Resource<GpuSurface>) -> Resource<GpuTexture> {
        let texture_id = if let Some(texture_id) = self.gpu_state.current_surface_texture {
            texture_id
        } else {
            let surface_output = self
                .instance
                .surface_get_current_texture(self.surface, None)
                .unwrap();

            self.gpu_state.present_surface = true;
            let texture_id = surface_output.texture_id.unwrap();

            self.gpu_state.textures.insert(
                texture_id,
                Texture {
                    width: self.surface_config.width,
                    height: self.surface_config.height,
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: TextureDimension::D2,
                    format: self.surface_config.format,
                    usage: TextureUsages::RENDER_ATTACHMENT,
                },
            );

            self.gpu_state.current_surface_texture = Some(texture_id);
            texture_id
        };

        self.table.push(texture_id).unwrap()
    }

    async fn get_texture_format(&mut self, _surface: Resource<GpuSurface>) -> GpuTextureFormat {
        self.surface_config.format.into()
    }

    async fn drop(&mut self, _rep: Resource<GpuSurface>) -> Result<()> {
        Ok(())
    }
}

impl HostGpuQuerySet for RuneRuntimeState {
    async fn type_(&mut self, query_set: Resource<GpuQuerySet>) -> GpuQueryType {
        let query_set_id = self.table.get(&query_set).unwrap();
        let query_set = self.gpu_state.query_sets.get(&query_set_id).unwrap();
        query_set.type_.into()
    }

    async fn count(&mut self, query_set: Resource<GpuQuerySet>) -> GpuSizeU32Out {
        let query_set_id = self.table.get(&query_set).unwrap();
        let query_set = self.gpu_state.query_sets.get(&query_set_id).unwrap();
        query_set.count
    }

    async fn destroy(&mut self, _self_: Resource<GpuQuerySet>) {}

    async fn drop(&mut self, rep: Resource<GpuQuerySet>) -> Result<()> {
        let query_set_id = self.table.delete(rep).unwrap();
        self.instance.query_set_drop(query_set_id);
        Ok(())
    }
}

impl HostGpuAdapter for RuneRuntimeState {
    async fn request_device(&mut self, _adapter: Resource<GpuAdapter>) -> Resource<GpuDevice> {
        // let adapter_id = self.table.get(&adapter).unwrap();
        // let (device_id, queue_id) = self.instance.adapter_request_device(
        //     *adapter_id,
        //     &DeviceDescriptor {
        //         label: None,
        //         required_features: Features::default(),
        //         required_limits: Limits::default()
        //     },
        //     None,
        //     (),
        //     ()
        // ).unwrap();
        // self.table.push(queue_id).unwrap();
        // Ok(self.table.push(device_id).unwrap())
        Resource::new_own(self.device_resource_id)
    }

    async fn drop(&mut self, _rep: Resource<GpuAdapter>) -> Result<()> {
        // let adapter_id = self.table.delete(rep).unwrap();
        // self.instance.adapter_drop(adapter_id);
        Ok(())
    }
}

impl HostGpuDevice for RuneRuntimeState {
    async fn create_buffer(
        &mut self,
        device: Resource<GpuDevice>,
        descriptor: GpuBufferDescriptor,
    ) -> Resource<GpuBuffer> {
        let device_id = self.table.get(&device).unwrap();

        let buffer_descriptor;
        let buffer_id = if let Some(contents) = descriptor.contents {
            let unpadded_size = contents.len() as wgpu_types::BufferAddress;
            // Valid vulkan usage is
            // 1. buffer size must be a multiple of COPY_BUFFER_ALIGNMENT.
            // 2. buffer size must be greater than 0.
            // Therefore we round the value up to the nearest multiple, and ensure it's at least COPY_BUFFER_ALIGNMENT.
            let align_mask = wgpu_types::COPY_BUFFER_ALIGNMENT - 1;
            let padded_size =
                ((unpadded_size + align_mask) & !align_mask).max(wgpu_types::COPY_BUFFER_ALIGNMENT);

            buffer_descriptor = wgpu_core::resource::BufferDescriptor {
                label: descriptor.label.map(|s| s.into()),
                size: padded_size,
                usage: descriptor.usage.into(),
                mapped_at_creation: true,
            };

            let buffer_id = wgpu_id(self.instance.device_create_buffer(
                *device_id,
                &buffer_descriptor,
                None,
            ))
            .unwrap();

            let (buffer, buffer_length) = self
                .instance
                .buffer_get_mapped_range(buffer_id, 0, Some(unpadded_size))
                .unwrap();

            unsafe {
                assert!(buffer_length as usize >= contents.len());
                std::ptr::copy_nonoverlapping(contents.as_ptr(), buffer.as_ptr(), contents.len());
            }

            self.instance.buffer_unmap(buffer_id).ok();

            buffer_id
        } else {
            buffer_descriptor = wgpu_core::resource::BufferDescriptor {
                label: None,
                size: descriptor.size,
                usage: descriptor.usage.into(),
                mapped_at_creation: false,
            };

            wgpu_id(self.instance.device_create_buffer(
                *device_id,
                &buffer_descriptor,
                None,
            ))
            .unwrap()
        };

        self.gpu_state.buffers.insert(
            buffer_id,
            Buffer {
                size: buffer_descriptor.size,
                usage: buffer_descriptor.usage,
                map_state: GpuBufferMapState::Unmapped
            },
        );

        self.table.push_child(buffer_id, &device).unwrap()
    }

    async fn create_texture(
        &mut self,
        device: Resource<GpuDevice>,
        descriptor: GpuTextureDescriptor,
    ) -> Resource<GpuTexture> {
        let device_id = self.table.get(&device).unwrap();

        let texture_descriptor = wgpu_core::resource::TextureDescriptor {
            label: None,
            size: descriptor.size.into(),
            mip_level_count: descriptor.mip_level_count,
            sample_count: descriptor.sample_count,
            dimension: descriptor.dimension.into(),
            format: descriptor.format.into(),
            usage: descriptor.usage.into(),
            view_formats: descriptor
                .view_formats
                .iter()
                .map(|vf| (*vf).into())
                .collect(),
        };

        let texture_id = wgpu_id(self.instance.device_create_texture(
            *device_id,
            &texture_descriptor,
            None,
        ))
        .unwrap();

        self.gpu_state.textures.insert(
            texture_id,
            Texture {
                height: texture_descriptor.size.height,
                width: texture_descriptor.size.width,
                dimension: texture_descriptor.dimension,
                format: texture_descriptor.format,
                mip_level_count: texture_descriptor.mip_level_count,
                sample_count: texture_descriptor.sample_count,
                usage: texture_descriptor.usage,
            },
        );

        self.table.push_child(texture_id, &device).unwrap()
    }

    async fn create_sampler(
        &mut self,
        device: Resource<GpuDevice>,
        descriptor: GpuSamplerDescriptor,
    ) -> Resource<GpuSampler> {
        let device_id = self.table.get(&device).unwrap();

        let sampler_id = wgpu_id(self.instance.device_create_sampler(
            *device_id,
            &wgpu_core::resource::SamplerDescriptor {
                label: None,
                address_modes: [
                    descriptor.address_mode_u.into(),
                    descriptor.address_mode_v.into(),
                    descriptor.address_mode_w.into(),
                ],
                mag_filter: descriptor.mag_filter.into(),
                min_filter: descriptor.min_filter.into(),
                mipmap_filter: descriptor.mipmap_filter.into(),
                lod_min_clamp: descriptor.lod_min_clamp,
                lod_max_clamp: descriptor.lod_max_clamp,
                compare: descriptor.compare.map(|c| c.into()),
                anisotropy_clamp: descriptor.max_anisotrophy,
                border_color: None,
            },
            None,
        ))
        .unwrap();

        self.table.push_child(sampler_id, &device).unwrap()
    }

    async fn create_bind_group_layout(
        &mut self,
        device: Resource<GpuDevice>,
        descriptor: GpuBindGroupLayoutDescriptor,
    ) -> Resource<GpuBindGroupLayout> {
        let device_id = self.table.get(&device).unwrap();
        let bind_group_layout_entries: Vec<_> = descriptor
            .entries
            .iter()
            .map(|entry| BindGroupLayoutEntry {
                binding: entry.binding,
                visibility: entry.visibility.into(),
                ty: if let Some(buffer) = entry.buffer {
                    wgpu_types::BindingType::Buffer {
                        ty: match buffer.type_ {
                            GpuBufferBindingType::Uniform => wgpu_types::BufferBindingType::Uniform,
                            GpuBufferBindingType::Storage => {
                                wgpu_types::BufferBindingType::Storage { read_only: false }
                            }
                            GpuBufferBindingType::ReadOnlyStorage => {
                                wgpu_types::BufferBindingType::Storage { read_only: true }
                            }
                        },
                        has_dynamic_offset: buffer.has_dynamic_offset,
                        min_binding_size: std::num::NonZeroU64::new(buffer.min_binding_size),
                    }
                } else if let Some(sampler) = entry.sampler {
                    wgpu_types::BindingType::Sampler(match sampler.type_ {
                        GpuSamplerBindingType::Filtering => wgpu_types::SamplerBindingType::Filtering,
                        GpuSamplerBindingType::NonFiltering => {
                            wgpu_types::SamplerBindingType::NonFiltering
                        }
                        GpuSamplerBindingType::Comparison => {
                            wgpu_types::SamplerBindingType::Comparison
                        }
                    })
                } else if let Some(texture) = entry.texture {
                    wgpu_types::BindingType::Texture {
                        sample_type: match texture.sample_type {
                            GpuTextureSampleType::Float => {
                                wgpu_types::TextureSampleType::Float { filterable: true }
                            }
                            GpuTextureSampleType::UnfilterableFloat => {
                                wgpu_types::TextureSampleType::Float { filterable: false }
                            }
                            GpuTextureSampleType::Depth => wgpu_types::TextureSampleType::Depth,
                            GpuTextureSampleType::Sint => wgpu_types::TextureSampleType::Sint,
                            GpuTextureSampleType::Uint => wgpu_types::TextureSampleType::Uint,
                        },
                        view_dimension: texture.view_dimension.into(),
                        multisampled: texture.multisampled,
                    }
                } else if let Some(storage_texture) = entry.storage_texture {
                    wgpu_types::BindingType::StorageTexture {
                        access: match storage_texture.access {
                            GpuStorageTextureAccess::WriteOnly => {
                                wgpu_types::StorageTextureAccess::WriteOnly
                            }
                        },
                        format: storage_texture.format.into(),
                        view_dimension: storage_texture.view_dimension.into(),
                    }
                } else {
                    panic!("Invalid bind group layout entry");
                },
                count: None,
            })
            .collect();

        let bind_group_layout_id = wgpu_id(
            self.instance
                .device_create_bind_group_layout(
                    *device_id,
                    &BindGroupLayoutDescriptor {
                        label: None,
                        entries: Cow::Owned(bind_group_layout_entries),
                    },
                    None,
                ),
        )
        .unwrap();

        self.table
            .push_child(bind_group_layout_id, &device)
            .unwrap()
    }

    async fn create_pipeline_layout(
        &mut self,
        device: Resource<GpuDevice>,
        descriptor: GpuPipelineLayoutDescriptor,
    ) -> Resource<GpuPipelineLayout> {
        let device_id = self.table.get(&device).unwrap();
        let bind_group_layouts: Vec<_> = descriptor
            .bind_group_layouts
            .iter()
            .map(|layout| *self.table.get(&layout).unwrap())
            .collect();

        let pipeline_layout_id = wgpu_id(
            self.instance
                .device_create_pipeline_layout(
                    *device_id,
                    &PipelineLayoutDescriptor {
                        label: None,
                        bind_group_layouts: Cow::Owned(bind_group_layouts),
                        push_constant_ranges: Cow::Owned(Vec::new()),
                    },
                    None,
                ),
        )
        .unwrap();

        self.table.push_child(pipeline_layout_id, &device).unwrap()
    }

    async fn create_bind_group(
        &mut self,
        device: Resource<GpuDevice>,
        descriptor: GpuBindGroupDescriptor,
    ) -> Resource<GpuBindGroup> {
        let device_id = self.table.get(&device).unwrap();
        let bind_group_layout_id = self.table.get(&descriptor.layout).unwrap();

        let bind_group_entries: Vec<_> = descriptor
            .entries
            .into_iter()
            .map(|entry| convert_bind_group_entry(&self.table, entry))
            .collect();

        let bind_group_id = wgpu_id(self.instance.device_create_bind_group(
            *device_id,
            &wgpu_core::binding_model::BindGroupDescriptor {
                label: descriptor.label.map(|label| label.into()),
                layout: *bind_group_layout_id,
                entries: Cow::Owned(bind_group_entries),
            },
            None,
        ))
        .unwrap();

        self.table.push_child(bind_group_id, &device).unwrap()
    }

    async fn create_shader_module(
        &mut self,
        device: Resource<GpuDevice>,
        descriptor: GpuShaderModuleDescriptor,
    ) -> Resource<GpuShaderModule> {
        let device_id = self.table.get(&device).unwrap();

        let shader_module = wgpu_id(self.instance.device_create_shader_module(
            *device_id,
            &wgpu_core::pipeline::ShaderModuleDescriptor {
                label: descriptor.label.map(|label| label.into()),
                runtime_checks: Default::default()
            },
            wgpu_core::pipeline::ShaderModuleSource::Wgsl(Cow::Owned(descriptor.code)),
            None,
        ))
        .unwrap();

        self.table.push(shader_module).unwrap()
    }

    async fn create_compute_pipeline(
        &mut self,
        _self_: Resource<GpuDevice>,
        _descriptor: GpuComputePipelineDescriptor,
    ) -> Resource<GpuComputePipeline> {
        todo!()
    }

    async fn create_render_pipeline(
        &mut self,
        device: Resource<GpuDevice>,
        descriptor: GpuRenderPipelineDescriptor,
    ) -> Resource<GpuRenderPipeline> {
        let layout = match descriptor.layout {
            GpuLayout::Auto => None,
            GpuLayout::Pipeline(pipeline_layout) => {
                Some(*self.table.get(&pipeline_layout).unwrap())
            }
        };

        let fragment_targets = match descriptor.fragment {
            Some(ref fragment) => fragment
                .targets
                .iter()
                .map(|c| {
                    Some(ColorTargetState {
                        format: c.format.into(),
                        blend: match c.blend {
                            None => None,
                            Some(blend) => Some(blend.into()),
                        },
                        write_mask: match c.write_mask {
                            None => ColorWrites::default(),
                            Some(write_mask) => write_mask.into(),
                        },
                    })
                })
                .collect::<Vec<Option<ColorTargetState>>>(),
            None => Vec::<Option<ColorTargetState>>::new(),
        };

        let vertex_buffer_attributes = match descriptor.vertex.buffers {
            Some(ref buffers) => {
                let mut attrs = Vec::<Vec<VertexAttribute>>::with_capacity(buffers.len());
                buffers.iter().enumerate().for_each(|(i, v)| {
                    attrs.insert(
                        i,
                        v.attributes
                            .iter()
                            .map(|a| VertexAttribute {
                                format: a.format.into(),
                                offset: a.offset,
                                shader_location: a.shader_location,
                            })
                            .collect::<Vec<VertexAttribute>>(),
                    );
                });
                attrs
            }
            None => Vec::<Vec<VertexAttribute>>::new(),
        };

        let vertex_buffers = match descriptor.vertex.buffers {
            Some(ref buffers) => buffers
                .iter()
                .enumerate()
                .map(|(i, v)| {
                    let attributes = vertex_buffer_attributes.get(i).unwrap();

                    VertexBufferLayout {
                        array_stride: v.array_stride,
                        step_mode: v.step_mode.into(),
                        attributes: Cow::Borrowed(&attributes.as_slice()),
                    }
                })
                .collect::<Vec<VertexBufferLayout>>(),
            None => Vec::<VertexBufferLayout>::new(),
        };

        let vertex_module = self.table.get(&descriptor.vertex.module).unwrap();
        let vertex = wgpu_core::pipeline::VertexState {
            stage: wgpu_core::pipeline::ProgrammableStageDescriptor {
                module: *vertex_module,
                entry_point: Some(descriptor.vertex.entry_point.into()),
                constants: Cow::Owned(HashMap::new()),
                zero_initialize_workgroup_memory: false,
            },
            buffers: Cow::Borrowed(vertex_buffers.as_slice()),
        };

        let fragment = descriptor.fragment.map(|fragment| {
            let fragment_module = self.table.get(&fragment.module).unwrap();
            wgpu_core::pipeline::FragmentState {
                stage: wgpu_core::pipeline::ProgrammableStageDescriptor {
                    module: *fragment_module,
                    entry_point: Some(fragment.entry_point.into()),
                    constants: Cow::Owned(HashMap::new()),
                    zero_initialize_workgroup_memory: false,
                },
                targets: Cow::Borrowed(fragment_targets.as_slice()),
            }
        });

        let device_id = self.table.get(&device).unwrap();

        let desc = &wgpu_core::pipeline::RenderPipelineDescriptor {
            vertex,
            fragment,
            primitive: match descriptor.primitive {
                None => PrimitiveState::default(),
                Some(primitive) => PrimitiveState {
                    topology: match primitive.topology {
                        None => PrimitiveTopology::default(),
                        Some(topology) => unsafe { std::mem::transmute(topology as u32) },
                    },
                    strip_index_format: match primitive.strip_index_format {
                        None => None,
                        Some(strip_index_format) => unsafe {
                            std::mem::transmute(strip_index_format as u32)
                        },
                    },
                    front_face: match primitive.front_face {
                        None => FrontFace::default(),
                        Some(front_face) => unsafe { std::mem::transmute(front_face as u32) },
                    },
                    cull_mode: match primitive.cull_mode {
                        GpuCullMode::None => None,
                        GpuCullMode::Front => Some(Face::Front),
                        GpuCullMode::Back => Some(Face::Back),
                    },
                    unclipped_depth: primitive.unclipped_depth,
                    polygon_mode: PolygonMode::default(),
                    conservative: false,
                },
            },
            depth_stencil: match descriptor.depth_stencil {
                None => None,
                Some(depth_stencil) => Some(DepthStencilState {
                    format: depth_stencil.format.into(),
                    depth_write_enabled: depth_stencil.depth_write_enabled,
                    depth_compare: depth_stencil.depth_compare.into(),
                    stencil: StencilState {
                        front: match depth_stencil.stencil_front {
                            None => StencilFaceState::default(),
                            Some(front) => front.into(),
                        },
                        back: match depth_stencil.stencil_back {
                            None => StencilFaceState::default(),
                            Some(front) => front.into(),
                        },
                        read_mask: match depth_stencil.stencil_read_mask {
                            Some(read_mask) => read_mask,
                            None => 0,
                        },
                        write_mask: match depth_stencil.stencil_write_mask {
                            Some(write_mask) => write_mask,
                            None => 0,
                        },
                    },
                    bias: DepthBiasState {
                        constant: depth_stencil.depth_bias.unwrap_or_default(),
                        slope_scale: depth_stencil.depth_bias_slope_scale.unwrap_or_default(),
                        clamp: depth_stencil.depth_bias_clamp.unwrap_or_default(),
                    },
                }),
            },
            multisample: match descriptor.multisample {
                None => MultisampleState::default(),
                Some(ref multisample) => MultisampleState {
                    count: multisample.count,
                    mask: multisample.mask as u64,
                    alpha_to_coverage_enabled: multisample.alpha_to_coverage_enabled,
                },
            },
            multiview: Default::default(),
            label: Default::default(),
            layout,
            cache: None,
        };

        let render_pipeline_id = wgpu_id(
            self.instance
                .device_create_render_pipeline(*device_id, desc, None, None),
        )
        .unwrap();

        self.table.push_child(render_pipeline_id, &device).unwrap()
    }

    async fn create_command_encoder(
        &mut self,
        device: Resource<GpuDevice>,
        descriptor: GpuCommandEncoderDescriptor,
    ) -> Resource<GpuCommandEncoder> {
        let device_id = self.table.get(&device).unwrap();

        let command_encoder_id = wgpu_id(
            self.instance
                .device_create_command_encoder(
                    *device_id,
                    &wgpu_types::CommandEncoderDescriptor {
                        label: descriptor.label.map(|label| label.into())
                    },
                    None,
                ),
        )
        .unwrap();

        self.table.push_child(command_encoder_id, &device).unwrap()
    }

    async fn create_render_bundle_encoder(
        &mut self,
        device: Resource<GpuDevice>,
        _descriptor: GpuRenderBundleDescriptor,
    ) -> Resource<GpuRenderBundleEncoder> {
        let device_id = self.table.get(&device).unwrap();

        #[allow(unused_variables)]
        let (render_bundle_encoder, _) = self.instance.device_create_render_bundle_encoder(
            *device_id,
            &wgpu_core::command::RenderBundleEncoderDescriptor {
                label: todo!(),
                #[allow(unreachable_code)]
                color_formats: todo!(),
                #[allow(unreachable_code)]
                depth_stencil: todo!(),
                #[allow(unreachable_code)]
                sample_count: todo!(),
                #[allow(unreachable_code)]
                multiview: todo!(),
            },
        );

        self.table
            .push_child(unsafe { *Box::from_raw(render_bundle_encoder) }, &device)
            .unwrap()
    }

    async fn create_query_set(
        &mut self,
        device: Resource<GpuDevice>,
        descriptor: GpuQuerySetDescriptor,
    ) -> Resource<GpuQuerySet> {
        let device_id = self.table.get(&device).unwrap();

        let query_set_descriptor = wgpu_core::resource::QuerySetDescriptor {
            label: Some(descriptor.label.into()),
            ty: descriptor.type_.into(),
            count: descriptor.count,
        };

        let query_set_id = wgpu_id(self.instance.device_create_query_set(
            *device_id,
            &query_set_descriptor,
            None,
        ))
        .unwrap();

        self.gpu_state.query_sets.insert(
            query_set_id,
            QuerySet {
                type_: query_set_descriptor.ty,
                count: query_set_descriptor.count,
            },
        );

        self.table.push_child(query_set_id, &device).unwrap()
    }

    async fn queue(&mut self, _device: Resource<GpuDevice>) -> Resource<GpuQueue> {
        Resource::new_own(self.queue_resource_id)
    }

    async fn drop(&mut self, _rep: Resource<GpuDevice>) -> Result<()> {
        Ok(())
    }
}

impl HostGpuQueue for RuneRuntimeState {
    async fn submit(
        &mut self,
        queue: Resource<GpuQueue>,
        command_buffers: Vec<Resource<GpuCommandBuffer>>,
    ) -> () {
        let command_buffers = command_buffers
            .into_iter()
            .map(|buffer| self.table.delete(buffer).unwrap())
            .collect::<Vec<_>>();

        let queue_id = self.table.get(&queue).unwrap();
        self.instance
            .queue_submit(*queue_id, &command_buffers)
            .unwrap();

        ()
    }

    async fn write_buffer(
        &mut self,
        queue: Resource<GpuQueue>,
        buffer: Resource<GpuBuffer>,
        buffer_offset: GpuSizeU64,
        data: BufferSource,
        _data_offset: GpuSizeU64,
        _size: GpuSizeU64,
    ) -> () {
        let queue_id = self.table.get(&queue).unwrap();
        let buffer_id = self.table.get(&buffer).unwrap();

        self.instance
            .queue_write_buffer(*queue_id, *buffer_id, buffer_offset, &data)
            .unwrap();

        ()
    }

    async fn write_texture(
        &mut self,
        queue: Resource<GpuQueue>,
        destination: GpuImageCopyTexture,
        data: BufferSource,
        data_layout: GpuImageDataLayout,
        size: GpuExtentD3,
    ) -> () {
        let queue_id = self.table.get(&queue).unwrap();
        let texture_id = self.table.get(&destination.texture).unwrap();

        self.instance
            .queue_write_texture(
                *queue_id,
                &TexelCopyTextureInfo {
                    texture: *texture_id,
                    mip_level: destination.mip_level,
                    origin: Origin3d {
                        x: destination.origin[0],
                        y: destination.origin[1],
                        z: destination.origin[2],
                    },
                    aspect: destination.aspect.into(),
                },
                &data,
                &TexelCopyBufferLayout {
                    offset: data_layout.offset,
                    bytes_per_row: Some(data_layout.bytes_per_row),
                    rows_per_image: Some(data_layout.rows_per_image),
                },
                &size.into(),
            )
            .unwrap();

        ()
    }

    async fn drop(&mut self, _rep: Resource<GpuQueue>) -> Result<()> {
        Ok(())
    }
}

impl HostGpuBuffer for RuneRuntimeState {
    async fn size(&mut self, buffer: Resource<GpuBuffer>) -> GpuSizeU64 {
        let buffer_id = self.table.get(&buffer).unwrap();
        let buffer = self.gpu_state.buffers.get(&buffer_id).unwrap();
        buffer.size
    }

    async fn usage(&mut self, buffer: Resource<GpuBuffer>) -> GpuBufferUsage {
        let buffer_id = self.table.get(&buffer).unwrap();
        let buffer = self.gpu_state.buffers.get(&buffer_id).unwrap();
        buffer.usage.into()
    }

    async fn map_state(&mut self, buffer: Resource<GpuBuffer>) -> GpuBufferMapState {
        let buffer_id = self.table.get(&buffer).unwrap();
        let buffer = self.gpu_state.buffers.get(&buffer_id).unwrap();
        buffer.map_state
    }

    async fn map(
        &mut self,
        buffer: Resource<GpuBuffer>,
        mode: GpuMapMode,
        offset: GpuSizeU64,
        size: GpuSizeU64,
    ) -> () {
        let buffer_id = self.table.get(&buffer).unwrap();
        let buffer = self.gpu_state.buffers.get_mut(buffer_id).unwrap();
        
        buffer.map_state = GpuBufferMapState::Pending;
        self.instance
            .buffer_map_async(
                *buffer_id,
                offset,
                Some(size),
                BufferMapOperation {
                    host: mode.into(),
                    callback: None
                }
            )
            .unwrap();
        buffer.map_state = GpuBufferMapState::Mapped;
        ()
    }

    async fn get_mapped_range(
        &mut self,
        buffer: Resource<GpuBuffer>,
        offset: GpuSizeU64,
        size: GpuSizeU64,
    ) -> Vec<u8> {
        let buffer_id = self.table.get(&buffer).unwrap();
        let (mapped_range, range_length) = self
            .instance
            .buffer_get_mapped_range(*buffer_id, offset, Some(size))
            .unwrap();

        unsafe {
            Vec::from_raw_parts(
                mapped_range.as_ptr(),
                range_length.try_into().unwrap(),
                range_length.try_into().unwrap(),
            )
        }
    }

    async fn unmap(&mut self, buffer: Resource<GpuBuffer>) -> () {
        let buffer_id = self.table.get(&buffer).unwrap();
        let buffer = self.gpu_state.buffers.get_mut(buffer_id).unwrap();
        self.instance
            .buffer_unmap(*buffer_id)
            .ok();
        buffer.map_state = GpuBufferMapState::Unmapped;
        ()
    }

    async fn destroy(&mut self, buffer: Resource<GpuBuffer>) -> () {
        let buffer_id = self.table.get(&buffer).unwrap();
        self.instance
            .buffer_destroy(*buffer_id)
            .ok();
        ()
    }

    async fn drop(&mut self, rep: Resource<GpuBuffer>) -> Result<()> {
        let buffer_id = self.table.delete(rep).unwrap();
        self.gpu_state.buffers.remove(&buffer_id);
        self.instance.buffer_drop(buffer_id);
        Ok(())
    }
}

impl HostGpuTexture for RuneRuntimeState {
    async fn width(&mut self, texture: Resource<GpuTexture>) -> GpuIntegerCoordinate {
        let texture_id = self.table.get(&texture).unwrap();
        let texture = self.gpu_state.textures.get(&texture_id).unwrap();
        texture.width
    }

    async fn height(&mut self, texture: Resource<GpuTexture>) -> GpuIntegerCoordinate {
        let texture_id = self.table.get(&texture).unwrap();
        let texture = self.gpu_state.textures.get(&texture_id).unwrap();
        texture.height
    }

    async fn depth_or_array_layers(
        &mut self,
        _self_: Resource<GpuTexture>,
    ) -> GpuIntegerCoordinate {
        todo!()
    }

    async fn mip_level_count(&mut self, texture: Resource<GpuTexture>) -> GpuIntegerCoordinate {
        let texture_id = self.table.get(&texture).unwrap();
        let texture = self.gpu_state.textures.get(&texture_id).unwrap();
        texture.mip_level_count
    }

    async fn sample_count(&mut self, texture: Resource<GpuTexture>) -> GpuSizeU32 {
        let texture_id = self.table.get(&texture).unwrap();
        let texture = self.gpu_state.textures.get(&texture_id).unwrap();
        texture.sample_count
    }

    async fn dimension(&mut self, texture: Resource<GpuTexture>) -> GpuTextureDimension {
        let texture_id = self.table.get(&texture).unwrap();
        let texture = self.gpu_state.textures.get(&texture_id).unwrap();
        texture.dimension.into()
    }

    async fn format(&mut self, texture: Resource<GpuTexture>) -> GpuTextureFormat {
        let texture_id = self.table.get(&texture).unwrap();
        let texture = self.gpu_state.textures.get(&texture_id).unwrap();
        texture.format.into()
    }

    async fn usage(&mut self, texture: Resource<GpuTexture>) -> GpuTextureUsage {
        let texture_id = self.table.get(&texture).unwrap();
        let texture = self.gpu_state.textures.get(&texture_id).unwrap();
        texture.usage.into()
    }

    async fn create_view(
        &mut self,
        texture_resource: Resource<GpuTexture>,
    ) -> Resource<GpuTextureView> {
        let texture_id = self.table.get(&texture_resource).unwrap();

        let texture_view_descriptor = if let Some(texture) = self.gpu_state.textures.get(texture_id)
        {
            TextureViewDescriptor {
                label: None,
                format: Some(texture.format),
                dimension: None,
                range: ImageSubresourceRange::default(),
                ..Default::default()
            }
        } else {
            TextureViewDescriptor::default()
        };

        let texture_view_id = wgpu_id(self.instance.texture_create_view(
            *texture_id,
            &texture_view_descriptor,
            None,
        ))
        .unwrap();

        self.table.push(texture_view_id).unwrap()
    }

    async fn destroy(&mut self, texture: Resource<GpuTexture>) -> () {
        let texture_id = self.table.get(&texture).unwrap();
        self.instance
            .texture_destroy(*texture_id)
            .unwrap();
        self.table.delete(texture).ok();
        ()
    }

    async fn drop(&mut self, texture: Resource<GpuTexture>) -> Result<()> {
        let texture_id = self.table.delete(texture).ok();
        if let Some(texture_id) = texture_id {
            self.instance.texture_drop(texture_id);
        }
        Ok(())
    }
}

impl HostGpuTextureView for RuneRuntimeState {
    async fn drop(&mut self, rep: Resource<GpuTextureView>) -> Result<()> {
        let texture_view_id = self.table.delete(rep).ok();
        if let Some(texture_view_id) = texture_view_id {
            self.instance
                .texture_view_drop(texture_view_id)
                .unwrap();
        }
        Ok(())
    }
}

impl HostGpuSampler for RuneRuntimeState {
    async fn drop(&mut self, rep: Resource<GpuSampler>) -> Result<()> {
        let sampler_id = self.table.delete(rep).unwrap();
        self.instance.sampler_drop(sampler_id);
        Ok(())
    }
}

impl HostGpuBindGroupLayout for RuneRuntimeState {
    async fn drop(&mut self, rep: Resource<GpuBindGroupLayout>) -> Result<()> {
        let bind_group_layout_id = self.table.delete(rep).unwrap();
        self.instance
            .bind_group_layout_drop(bind_group_layout_id);
        Ok(())
    }
}

impl HostGpuBindGroup for RuneRuntimeState {
    async fn drop(&mut self, rep: Resource<GpuBindGroup>) -> Result<()> {
        let bind_group_id = self.table.delete(rep).unwrap();
        self.instance
            .bind_group_drop(bind_group_id);
        Ok(())
    }
}

impl HostGpuPipelineLayout for RuneRuntimeState {
    async fn drop(&mut self, rep: Resource<GpuPipelineLayout>) -> Result<()> {
        let pipeline_layout_id = self.table.delete(rep).unwrap();
        self.instance
            .pipeline_layout_drop(pipeline_layout_id);
        Ok(())
    }
}

impl HostGpuShaderModule for RuneRuntimeState {
    async fn get_compilation_info(
        &mut self,
        _self_: Resource<GpuShaderModule>,
    ) -> GpuCompilationInfo {
        todo!()
    }

    async fn drop(&mut self, rep: Resource<GpuShaderModule>) -> Result<()> {
        let shader_module_id = self.table.delete(rep).unwrap();
        self.instance
            .shader_module_drop(shader_module_id);
        Ok(())
    }
}

impl HostGpuComputePipeline for RuneRuntimeState {
    async fn get_bind_group_layout(
        &mut self,
        pipeline: Resource<GpuComputePipeline>,
        index: u32,
    ) -> Resource<GpuBindGroupLayout> {
        let pipeline_id = self.table.get(&pipeline).unwrap();
        let bind_group_layout_id = wgpu_id(
            self.instance
                .compute_pipeline_get_bind_group_layout(
                    *pipeline_id,
                    index,
                    None,
                ),
        )
        .unwrap();
        self.table
            .push_child(bind_group_layout_id, &pipeline)
            .unwrap()
    }

    async fn drop(&mut self, rep: Resource<GpuComputePipeline>) -> Result<()> {
        let pipeline_id = self.table.delete(rep).unwrap();
        self.instance
            .compute_pipeline_drop(pipeline_id);
        Ok(())
    }
}

impl HostGpuRenderPipeline for RuneRuntimeState {
    async fn get_bind_group_layout(
        &mut self,
        pipeline: Resource<GpuRenderPipeline>,
        index: u32,
    ) -> Resource<GpuBindGroupLayout> {
        let pipeline_id = self.table.get(&pipeline).unwrap();
        let bind_group_layout_id = wgpu_id(
            self.instance
                .render_pipeline_get_bind_group_layout(*pipeline_id, index, None),
        )
        .unwrap();
        self.table
            .push_child(bind_group_layout_id, &pipeline)
            .unwrap()
    }

    async fn drop(&mut self, rep: Resource<GpuRenderPipeline>) -> Result<()> {
        let render_pipeline_id = self.table.delete(rep).unwrap();
        self.instance
            .render_pipeline_drop(render_pipeline_id);
        Ok(())
    }
}

impl HostGpuCommandBuffer for RuneRuntimeState {
    async fn drop(&mut self, _: Resource<GpuCommandBuffer>) -> Result<()> {
        Ok(())
    }
}

impl HostGpuCommandEncoder for RuneRuntimeState {
    async fn begin_render_pass(
        &mut self,
        command_encoder: Resource<GpuCommandEncoder>,
        descriptor: GpuRenderPassDescriptor,
    ) -> Resource<GpuRenderPassEncoder> {
        let command_encoder = self.table.get(&command_encoder).unwrap();
        let views = descriptor
            .color_attachments
            .iter()
            .map(|color_attachment| *self.table.get(&color_attachment.view).unwrap())
            .collect::<Vec<_>>();

        let mut color_attachments = vec![];
        for (i, color_attachment) in descriptor.color_attachments.iter().enumerate() {
            let resolve_target = color_attachment
                .resolve_target
                .as_ref()
                .map(|t| *self.table.get(t).unwrap());
            color_attachments.push(Some(wgpu_core::command::RenderPassColorAttachment {
                view: views[i],
                resolve_target,
                load_op: color_attachment.load_op.into_wgt(color_attachment
                    .clear_value
                    .as_ref()
                    .map(|v| vec_to_color(v))
                    .unwrap_or(Color::BLACK)),
                store_op: color_attachment.store_op.into(),
            }));
        }

        let depth_stencil_attachment =
            descriptor
                .depth_stencil_attachment
                .map(|depth_stencil_attachment| {
                    wgpu_core::command::RenderPassDepthStencilAttachment {
                        view: *self.table.get(&depth_stencil_attachment.view).unwrap(),
                        depth: PassChannel {
                            load_op: Some(depth_stencil_attachment.depth_load_op.into_wgt(Some(depth_stencil_attachment.depth_clear_value))),
                            store_op: Some(depth_stencil_attachment.depth_store_op.into()),
                            read_only: depth_stencil_attachment.depth_read_only,
                        },
                        stencil: PassChannel {
                            load_op: Some(depth_stencil_attachment.stencil_load_op.into_wgt(Some(depth_stencil_attachment.stencil_clear_value))),
                            store_op: Some(depth_stencil_attachment.stencil_store_op.into()),
                            read_only: depth_stencil_attachment.stencil_read_only,
                        },
                    }
                });

        let (render_pass, _) = self
            .instance
            .command_encoder_create_render_pass(
                *command_encoder,
                &wgpu_core::command::RenderPassDescriptor {
                    label: None,
                    color_attachments: color_attachments.into(),
                    depth_stencil_attachment: depth_stencil_attachment.as_ref(),
                    timestamp_writes: None,
                    occlusion_query_set: None,
                },
            );

        self.table.push(render_pass).unwrap()
    }

    async fn begin_compute_pass(
        &mut self,
        command_encoder: wasmtime::component::Resource<GpuCommandEncoder>,
        _descriptor: Option<GpuComputePassDescriptor>,
    ) -> Resource<GpuComputePassEncoder> {
        let command_encoder = self.table.get(&command_encoder).unwrap();

        let (compute_pass, _) = self.instance.command_encoder_create_compute_pass(
            *command_encoder,
            &wgpu_core::command::ComputePassDescriptor {
                label: None,
                timestamp_writes: None,
            },
        );

        self.table.push(compute_pass).unwrap()
    }

    async fn copy_buffer_to_buffer(
        &mut self,
        command_encoder: wasmtime::component::Resource<GpuCommandEncoder>,
        source: wasmtime::component::Resource<GpuBuffer>,
        source_offset: GpuSizeU64,
        destination: wasmtime::component::Resource<GpuBuffer>,
        destination_offset: GpuSizeU64,
        size: GpuSizeU64,
    ) -> () {
        let command_encoder_id = self.table.get(&command_encoder).unwrap();
        let source_id = self.table.get(&source).unwrap();
        let destination_id = self.table.get(&destination).unwrap();

        self.instance
            .command_encoder_copy_buffer_to_buffer(
                *command_encoder_id,
                *source_id,
                source_offset,
                *destination_id,
                destination_offset,
                size,
            )
            .ok();
        ()
    }

    async fn copy_buffer_to_texture(
        &mut self,
        command_encoder: wasmtime::component::Resource<GpuCommandEncoder>,
        source: GpuImageCopyBuffer,
        destination: GpuImageCopyTexture,
        copy_size: GpuExtentD3,
    ) -> () {
        let command_encoder_id = self.table.get(&command_encoder).unwrap();
        let source_buffer_id = self.table.get(&source.buffer).unwrap();
        let destination_texture_id = self.table.get(&destination.texture).unwrap();

        let source = wgpu_core::command::TexelCopyBufferInfo {
            buffer: *source_buffer_id,
            layout: wgpu_types::TexelCopyBufferLayout {
                offset: source.layout.offset,
                bytes_per_row: Some(source.layout.bytes_per_row),
                rows_per_image: Some(source.layout.rows_per_image),
            },
        };
        let destination = wgpu_core::command::TexelCopyTextureInfo {
            texture: *destination_texture_id,
            mip_level: destination.mip_level,
            origin: Origin3d {
                x: destination.origin[0],
                y: destination.origin[1],
                z: destination.origin[2],
            },
            aspect: destination.aspect.into(),
        };

        self.instance
            .command_encoder_copy_buffer_to_texture(
                *command_encoder_id,
                &source,
                &destination,
                &copy_size.into(),
            )
            .ok();
        ()
    }

    async fn copy_texture_to_buffer(
        &mut self,
        command_encoder: wasmtime::component::Resource<GpuCommandEncoder>,
        source: GpuImageCopyTexture,
        destination: GpuImageCopyBuffer,
        copy_size: GpuExtentD3,
    ) -> () {
        let command_encoder_id = self.table.get(&command_encoder).unwrap();
        let source_texture_id = self.table.get(&source.texture).unwrap();
        let destination_texture_id = self.table.get(&destination.buffer).unwrap();

        let source = wgpu_core::command::TexelCopyTextureInfo {
            texture: *source_texture_id,
            mip_level: source.mip_level,
            origin: Origin3d {
                x: source.origin[0],
                y: source.origin[1],
                z: source.origin[2],
            },
            aspect: source.aspect.into(),
        };
        let destination = wgpu_core::command::TexelCopyBufferInfo {
            buffer: *destination_texture_id,
            layout: wgpu_types::TexelCopyBufferLayout {
                offset: destination.layout.offset,
                bytes_per_row: Some(destination.layout.bytes_per_row),
                rows_per_image: Some(destination.layout.rows_per_image),
            },
        };

        self.instance
            .command_encoder_copy_texture_to_buffer(
                *command_encoder_id,
                &source,
                &destination,
                &copy_size.into(),
            )
            .ok();
        ()
    }

    async fn copy_texture_to_texture(
        &mut self,
        command_encoder: wasmtime::component::Resource<GpuCommandEncoder>,
        source: GpuImageCopyTexture,
        destination: GpuImageCopyTexture,
        copy_size: GpuExtentD3,
    ) -> () {
        let command_encoder_id = self.table.get(&command_encoder).unwrap();
        let source_texture_id = self.table.get(&source.texture).unwrap();
        let destination_texture_id = self.table.get(&destination.texture).unwrap();

        let source = wgpu_core::command::TexelCopyTextureInfo {
            texture: *source_texture_id,
            mip_level: source.mip_level,
            origin: Origin3d {
                x: source.origin[0],
                y: source.origin[1],
                z: source.origin[2],
            },
            aspect: source.aspect.into(),
        };
        let destination = wgpu_core::command::TexelCopyTextureInfo {
            texture: *destination_texture_id,
            mip_level: destination.mip_level,
            origin: Origin3d {
                x: destination.origin[0],
                y: destination.origin[1],
                z: destination.origin[2],
            },
            aspect: destination.aspect.into(),
        };

        self.instance
            .command_encoder_copy_texture_to_texture(
                *command_encoder_id,
                &source,
                &destination,
                &copy_size.into(),
            )
            .ok();
        ()
    }

    async fn clear_buffer(
        &mut self,
        command_encoder: wasmtime::component::Resource<GpuCommandEncoder>,
        buffer: wasmtime::component::Resource<GpuBuffer>,
        offset: Option<GpuSizeU64>,
        size: Option<GpuSizeU64>,
    ) -> () {
        let command_encoder_id = self.table.get(&command_encoder).unwrap();
        let buffer_id = self.table.get(&buffer).unwrap();

        self.instance
            .command_encoder_clear_buffer(
                *command_encoder_id,
                *buffer_id,
                offset.unwrap_or(0),
                size,
            )
            .ok();
        ()
    }

    async fn write_timestamp(
        &mut self,
        command_encoder: wasmtime::component::Resource<GpuCommandEncoder>,
        query_set: wasmtime::component::Resource<GpuQuerySet>,
        query_index: GpuSizeU32,
    ) -> () {
        let command_encoder_id = self.table.get(&command_encoder).unwrap();
        let query_set_id = self.table.get(&query_set).unwrap();

        self.instance
            .command_encoder_write_timestamp(
                *command_encoder_id,
                *query_set_id,
                query_index,
            )
            .ok();
        ()
    }

    async fn resolve_query_set(
        &mut self,
        command_encoder: wasmtime::component::Resource<GpuCommandEncoder>,
        query_set: wasmtime::component::Resource<GpuQuerySet>,
        first_query: GpuSizeU32,
        query_count: GpuSizeU32,
        destination: wasmtime::component::Resource<GpuBuffer>,
        destination_offset: GpuSizeU64,
    ) -> () {
        let command_encoder_id = self.table.get(&command_encoder).unwrap();
        let query_set_id = self.table.get(&query_set).unwrap();
        let destination_id = self.table.get(&destination).unwrap();

        self.instance
            .command_encoder_resolve_query_set(
                *command_encoder_id,
                *query_set_id,
                first_query,
                query_count,
                *destination_id,
                destination_offset,
            )
            .ok();
        ()
    }

    async fn finish(
        &mut self,
        command_encoder: wasmtime::component::Resource<GpuCommandEncoder>,
    ) -> Resource<GpuCommandBuffer> {
        let command_encoder_id = self.table.get(&command_encoder).unwrap();

        let command_buffer_id = wgpu_id(self.instance.command_encoder_finish(
            *command_encoder_id,
            &wgpu_types::CommandBufferDescriptor { label: None },
        ))
        .unwrap();

        self.table
            .push_child(command_buffer_id, &command_encoder)
            .unwrap()
    }

    async fn drop(
        &mut self,
        rep: wasmtime::component::Resource<GpuCommandEncoder>,
    ) -> wasmtime::Result<()> {
        let command_encoder_id = self.table.delete(rep).unwrap();
        self.instance
            .command_encoder_drop(command_encoder_id);
        Ok(())
    }
}

impl HostGpuComputePassEncoder for RuneRuntimeState {
    async fn set_pipeline(
        &mut self,
        compute_pass_encoder: Resource<GpuComputePassEncoder>,
        pipeline: Resource<GpuComputePipeline>,
    ) -> () {
        let pipeline_id = *self.table.get(&pipeline).unwrap();
        let compute_pass_encoder_id = self.table.get_mut(&compute_pass_encoder).unwrap();

        self.instance
            .compute_pass_set_pipeline(compute_pass_encoder_id, pipeline_id)
            .unwrap();
        ()
    }

    async fn dispatch_workgroups(
        &mut self,
        compute_pass_encoder: Resource<GpuComputePassEncoder>,
        workgroup_count_x: GpuSizeU32,
        workgroup_count_y: Option<GpuSizeU32>,
        workgroup_count_z: Option<GpuSizeU32>,
    ) -> () {
        let compute_pass_encoder_id = self.table.get_mut(&compute_pass_encoder).unwrap();

        self.instance
            .compute_pass_dispatch_workgroups(
                compute_pass_encoder_id,
                workgroup_count_x,
                workgroup_count_y.unwrap_or(1),
                workgroup_count_z.unwrap_or(1),
            )
            .unwrap();

        ()
    }

    async fn dispatch_workgroups_indirect(
        &mut self,
        compute_pass_encoder: Resource<GpuComputePassEncoder>,
        indirect_buffer: Resource<GpuBuffer>,
        indirect_offset: GpuSizeU64,
    ) -> () {
        let indirect_buffer_id = *self.table.get(&indirect_buffer).unwrap();
        let compute_pass_encoder_id = self.table.get_mut(&compute_pass_encoder).unwrap();

        self.instance
            .compute_pass_dispatch_workgroups_indirect(
                compute_pass_encoder_id,
                indirect_buffer_id,
                indirect_offset,
            )
            .unwrap();

        ()
    }

    async fn end(&mut self, compute_pass_encoder: Resource<GpuComputePassEncoder>) -> () {
        let compute_pass_encoder_id = self.table.get_mut(&compute_pass_encoder).unwrap();

        self.instance
            .compute_pass_end(compute_pass_encoder_id)
            .unwrap();
        ()
    }

    async fn set_bind_group(
        &mut self,
        compute_pass_encoder: Resource<GpuComputePassEncoder>,
        index: GpuIndexU32,
        bind_group: Option<Resource<GpuBindGroup>>,
        dynamic_offsets: Option<Vec<GpuBufferDynamicOffset>>,
    ) -> () {
        let bind_group_id = bind_group.map(|bg| *self.table.get(&bg).unwrap());
        let compute_pass_encoder_id = self.table.get_mut(&compute_pass_encoder).unwrap();

        let dynamic_offsets_slice = dynamic_offsets.as_deref().unwrap_or(&[]);

        self.instance
            .compute_pass_set_bind_group(
                compute_pass_encoder_id,
                index,
                bind_group_id,
                dynamic_offsets_slice,
            )
            .unwrap();
        ()
    }

    async fn set_bind_group_with_data(
        &mut self,
        _self_: Resource<GpuComputePassEncoder>,
        _index: GpuIndexU32,
        _bind_group: Option<Resource<GpuBindGroup>>,
        _dynamic_offsets_data: Vec<u32>,
        _dynamic_offsets_data_start: GpuSizeU64,
        _dynamic_offsets_data_lengh: GpuSizeU32,
    ) -> () {
        let bind_group_id = _bind_group.map(|bg| *self.table.get(&bg).unwrap());
        let compute_pass_encoder_id = self.table.get_mut(&_self_).unwrap();

        let dynamic_offsets_slice = if _dynamic_offsets_data_lengh > 0 {
            let start = _dynamic_offsets_data_start as usize;
            let len = _dynamic_offsets_data_lengh as usize;
            // Assuming start and len are valid and won't cause out-of-bounds panic.
            // WIT generated code or guest logic should ensure this.
            &_dynamic_offsets_data[start..start + len]
        } else {
            &[]
        };

        self.instance
            .compute_pass_set_bind_group(
                compute_pass_encoder_id,
                _index,
                bind_group_id,
                dynamic_offsets_slice,
            )
            .unwrap();
        ()
    }

    async fn push_debug_group(
        &mut self,
        compute_pass_encoder: Resource<GpuComputePassEncoder>,
        group_label: String,
    ) -> () {
        let compute_pass_encoder_id = self.table.get_mut(&compute_pass_encoder).unwrap();
        self.instance
            .compute_pass_push_debug_group(compute_pass_encoder_id, &group_label, 0)
            .unwrap();
        ()
    }

    async fn pop_debug_group(&mut self, compute_pass_encoder: Resource<GpuComputePassEncoder>) -> () {
        let compute_pass_encoder_id = self.table.get_mut(&compute_pass_encoder).unwrap();
        self.instance
            .compute_pass_pop_debug_group(compute_pass_encoder_id)
            .unwrap();
        ()
    }

    async fn insert_debug_marker(
        &mut self,
        compute_pass_encoder: Resource<GpuComputePassEncoder>,
        marker_label: String,
    ) -> () {
        let compute_pass_encoder_id = self.table.get_mut(&compute_pass_encoder).unwrap();
        self.instance
            .compute_pass_insert_debug_marker(
                compute_pass_encoder_id,
                &marker_label,
                0,
            )
            .unwrap();
        ()
    }

    async fn drop(&mut self, rep: Resource<GpuComputePassEncoder>) -> Result<()> {
        self.table.delete(rep).unwrap();
        Ok(())
    }
}

impl HostGpuRenderPassEncoder for RuneRuntimeState {
    async fn set_pipeline(
        &mut self,
        render_pass_encoder: Resource<GpuRenderPassEncoder>,
        pipeline: Resource<GpuRenderPipeline>,
    ) -> () {
        let render_pipeline_id = *self.table.get(&pipeline).unwrap();
        let render_pass_encoder_id = self.table.get_mut(&render_pass_encoder).unwrap();

        self.instance
            .render_pass_set_pipeline(render_pass_encoder_id, render_pipeline_id)
            .unwrap();
        ()
    }

    async fn set_index_buffer(
        &mut self,
        render_pass_encoder: Resource<GpuRenderPassEncoder>,
        buffer: Resource<GpuBuffer>,
        index_format: GpuIndexFormat,
        offset: GpuSizeU64,
        size: Option<GpuSizeU64>,
    ) -> () {
        let buffer_id = *self.table.get(&buffer).unwrap();
        let render_pass_encoder_id = self.table.get_mut(&render_pass_encoder).unwrap();

        self.instance
            .render_pass_set_index_buffer(
                render_pass_encoder_id,
                buffer_id,
                index_format.into(),
                offset,
                size.map(|s| NonZeroU64::new(s).unwrap()),
            )
            .unwrap();

        ()
    }

    async fn set_vertex_buffer(
        &mut self,
        render_pass_encoder: Resource<GpuRenderPassEncoder>,
        slot: GpuIndexU32,
        buffer: Resource<GpuBuffer>,
        offset: GpuSizeU64,
        size: Option<GpuSizeU64>,
    ) -> () {
        let buffer_id = *self.table.get(&buffer).unwrap();
        let render_pass_encoder_id = self.table.get_mut(&render_pass_encoder).unwrap();

        self.instance
            .render_pass_set_vertex_buffer(
                render_pass_encoder_id,
                slot,
                buffer_id,
                offset,
                size.map(|s| NonZeroU64::new(s).unwrap()),
            )
            .unwrap();

        ()
    }

    async fn draw(
        &mut self,
        render_pass_encoder: Resource<GpuRenderPassEncoder>,
        vertex_count: GpuSizeU32,
        instance_count: GpuSizeU32,
        first_vertex: GpuSizeU32,
        first_instance: GpuSizeU32,
    ) -> () {
        let render_pass_encoder_id = self.table.get_mut(&render_pass_encoder).unwrap();

        self.instance
            .render_pass_draw(
                render_pass_encoder_id,
                vertex_count,
                instance_count,
                first_vertex,
                first_instance,
            )
            .unwrap();

        ()
    }

    async fn draw_indexed(
        &mut self,
        render_pass_encoder: Resource<GpuRenderPassEncoder>,
        index_count: GpuSizeU32,
        instance_count: GpuSizeU32,
        first_index: GpuSizeU32,
        base_vertex: GpuSignedOffsetS32,
        first_instance: GpuSizeU32,
    ) -> () {
        let render_pass_encoder_id = self.table.get_mut(&render_pass_encoder).unwrap();

        self.instance
            .render_pass_draw_indexed(
                render_pass_encoder_id,
                index_count,
                instance_count,
                first_index,
                base_vertex,
                first_instance,
            )
            .unwrap();

        ()
    }

    async fn draw_indirect(
        &mut self,
        render_pass_encoder: Resource<GpuRenderPassEncoder>,
        indirect_buffer: Resource<GpuBuffer>,
        indirect_offset: GpuSizeU64,
    ) -> () {
        let buffer_id = *self.table.get(&indirect_buffer).unwrap();
        let render_pass_encoder_id = self.table.get_mut(&render_pass_encoder).unwrap();

        self.instance
            .render_pass_draw_indirect(
                render_pass_encoder_id,
                buffer_id,
                indirect_offset,
            )
            .unwrap();

        ()
    }

    async fn draw_indexed_indirect(
        &mut self,
        render_pass_encoder: Resource<GpuRenderPassEncoder>,
        indirect_buffer: Resource<GpuBuffer>,
        indirect_offset: GpuSizeU64,
    ) -> () {
        let buffer_id = *self.table.get(&indirect_buffer).unwrap();
        let render_pass_encoder_id = self.table.get_mut(&render_pass_encoder).unwrap();

        self.instance
            .render_pass_draw_indexed_indirect(
                render_pass_encoder_id,
                buffer_id,
                indirect_offset,
            )
            .unwrap();

        ()
    }

    async fn set_viewport(
        &mut self,
        render_pass_encoder: Resource<GpuRenderPassEncoder>,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        min_depth: f32,
        max_depth: f32,
    ) -> () {
        let render_pass_encoder_id = self.table.get_mut(&render_pass_encoder).unwrap();
        self.instance
            .render_pass_set_viewport(
                render_pass_encoder_id,
                x,
                y,
                width,
                height,
                min_depth,
                max_depth,
            )
            .unwrap();

        ()
    }

    async fn set_scissor_rect(
        &mut self,
        render_pass_encoder: Resource<GpuRenderPassEncoder>,
        x: GpuIntegerCoordinate,
        y: GpuIntegerCoordinate,
        width: GpuIntegerCoordinate,
        height: GpuIntegerCoordinate,
    ) -> () {
        let render_pass_encoder_id = self.table.get_mut(&render_pass_encoder).unwrap();
        self.instance
            .render_pass_set_scissor_rect(
                render_pass_encoder_id,
                x,
                y,
                width,
                height,
            )
            .unwrap();

        ()
    }

    async fn set_blend_constant(
        &mut self,
        render_pass_encoder: Resource<GpuRenderPassEncoder>,
        color: GpuColor,
    ) -> () {
        let render_pass_encoder_id = self.table.get_mut(&render_pass_encoder).unwrap();
        self.instance
            .render_pass_set_blend_constant(
                render_pass_encoder_id,
                Color {
                    r: color[0],
                    g: color[1],
                    b: color[2],
                    a: color[3],
                },
            )
            .unwrap();

        ()
    }

    async fn set_stencil_reference(
        &mut self,
        render_pass_encoder: Resource<GpuRenderPassEncoder>,
        reference: GpuStencilValue,
    ) -> () {
        let render_pass_encoder_id = self.table.get_mut(&render_pass_encoder).unwrap();
        self.instance
            .render_pass_set_stencil_reference(render_pass_encoder_id, reference)
            .unwrap();

        ()
    }

    async fn begin_occlusion_query(
        &mut self,
        render_pass_encoder: Resource<GpuRenderPassEncoder>,
        query_index: GpuSizeU32,
    ) -> () {
        let render_pass_encoder_id = self.table.get_mut(&render_pass_encoder).unwrap();
        self.instance
            .render_pass_begin_occlusion_query(
                render_pass_encoder_id,
                query_index,
            )
            .unwrap();

        ()
    }

    async fn end_occlusion_query(
        &mut self,
        render_pass_encoder: Resource<GpuRenderPassEncoder>,
    ) -> () {
        let render_pass_encoder_id = self.table.get_mut(&render_pass_encoder).unwrap();
        self.instance
            .render_pass_end_occlusion_query(render_pass_encoder_id)
            .unwrap();

        ()
    }

    async fn execute_bundles(
        &mut self,
        render_pass_encoder: Resource<GpuRenderPassEncoder>,
        bundles: Vec<Resource<GpuRenderBundle>>,
    ) -> () {
        let render_bundle_ids: Vec<_> = bundles
            .iter()
            .map(|b| *self.table.get(b).unwrap())
            .collect();

        let render_pass_encoder_id = self.table.get_mut(&render_pass_encoder).unwrap();

        self.instance
            .render_pass_execute_bundles(
                render_pass_encoder_id,
                &render_bundle_ids[..],
            )
            .unwrap();

        ()
    }

    async fn end(&mut self, render_pass: Resource<GpuRenderPassEncoder>) -> () {
        let render_pass = self.table.get_mut(&render_pass).unwrap();

        self.instance
            .render_pass_end(render_pass)
            .unwrap();
        ()
    }

    async fn set_bind_group(
        &mut self,
        render_pass: Resource<GpuRenderPassEncoder>,
        index: GpuIndexU32,
        bind_group: Option<Resource<GpuBindGroup>>,
        dynamic_offsets: Option<Vec<GpuBufferDynamicOffset>>,
    ) -> () {
        let bind_group_id = *self.table.get(&bind_group.unwrap()).unwrap();
        let render_pass = self.table.get_mut(&render_pass).unwrap();

        let dynamic_offsets = if let Some(dynamic_offsets) = dynamic_offsets {
            dynamic_offsets
                .into_iter()
                .map(|o| o as DynamicOffset)
                .collect()
        } else {
            Vec::new()
        };

        self.instance
            .render_pass_set_bind_group(
                render_pass,
                index,
                Some(bind_group_id),
                &dynamic_offsets,
            )
            .unwrap();

        ()
    }

    async fn set_bind_group_with_data(
        &mut self,
        _render_pass_encoder: Resource<GpuRenderPassEncoder>,
        _index: GpuIndexU32,
        _bind_group: Option<Resource<GpuBindGroup>>,
        _dynamic_offsets_data: Vec<u32>,
        _dynamic_offsets_data_start: GpuSizeU64,
        _dynamic_offsets_data_length: GpuSizeU32,
    ) -> () {
        let bind_group_id = _bind_group.map(|bg| *self.table.get(&bg).unwrap());
        let render_pass_encoder_id = self.table.get_mut(&_render_pass_encoder).unwrap();

        let dynamic_offsets_slice = if _dynamic_offsets_data_length > 0 {
            let start = _dynamic_offsets_data_start as usize;
            let len = _dynamic_offsets_data_length as usize;
            // Assuming start and len are valid and won't cause out-of-bounds panic.
            // WIT generated code or guest logic should ensure this.
            &_dynamic_offsets_data[start..start + len]
        } else {
            &[]
        };

        self.instance
            .render_pass_set_bind_group(
                render_pass_encoder_id,
                _index,
                bind_group_id,
                dynamic_offsets_slice,
            )
            .unwrap();
        ()
    }

    async fn push_debug_group(
        &mut self,
        render_pass_encoder: Resource<GpuRenderPassEncoder>,
        group_label: String,
    ) -> () {
        let render_pass_encoder_id = self.table.get_mut(&render_pass_encoder).unwrap();
        self.instance
            .render_pass_push_debug_group(render_pass_encoder_id, &group_label, 0)
            .unwrap();

        ()
    }

    async fn pop_debug_group(&mut self, render_pass_encoder: Resource<GpuRenderPassEncoder>) -> () {
        let render_pass_encoder_id = self.table.get_mut(&render_pass_encoder).unwrap();
        self.instance
            .render_pass_pop_debug_group(render_pass_encoder_id)
            .unwrap();

        ()
    }

    async fn insert_debug_marker(
        &mut self,
        render_pass_encoder: Resource<GpuRenderPassEncoder>,
        marker_label: String,
    ) -> () {
        let render_pass_encoder_id = self.table.get_mut(&render_pass_encoder).unwrap();
        self.instance
            .render_pass_insert_debug_marker(
                render_pass_encoder_id,
                &marker_label,
                0,
            )
            .unwrap();

        ()
    }

    async fn drop(&mut self, rep: Resource<GpuRenderPassEncoder>) -> Result<()> {
        self.table.delete(rep).unwrap();
        Ok(())
    }
}

impl HostGpuRenderBundle for RuneRuntimeState {
    async fn drop(&mut self, rep: Resource<GpuRenderBundle>) -> Result<()> {
        let render_bundle_id = self.table.delete(rep).unwrap();
        self.instance
            .render_bundle_drop(render_bundle_id);
        Ok(())
    }
}

impl HostGpuRenderBundleEncoder for RuneRuntimeState {
    async fn finish(
        &mut self,
        render_bundle_encoder_resource: Resource<GpuRenderBundleEncoder>,
        descriptor: GpuRenderBundleDescriptor,
    ) -> Resource<GpuRenderBundle> {
        let render_bundle_encoder = self.table.delete(render_bundle_encoder_resource).unwrap();
        let render_bundle_id = wgpu_id(
            self.instance
                .render_bundle_encoder_finish(
                    render_bundle_encoder,
                    &wgpu_core::command::RenderBundleDescriptor {
                        label: Some(descriptor.label.into()),
                    },
                    None,
                ),
        )
        .unwrap();

        self.table.push(render_bundle_id).unwrap()
    }

    async fn drop(&mut self, rep: Resource<GpuRenderBundleEncoder>) -> Result<()> {
        self.table.delete(rep).unwrap();
        Ok(())
    }
}
