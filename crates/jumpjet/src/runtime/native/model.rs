use wasmtime::Result;
use wasmtime::component::Resource;

use super::state::JumpjetRuntimeState;
use crate::jumpjet::runtime::model::*;
use crate::runtime::common::image::Bitmap;
use crate::runtime::common::model::GltfModel as GltfDoc;

impl Host for JumpjetRuntimeState {
    async fn decode(
        &mut self,
        data: Resource<Buffer>,
    ) -> core::result::Result<Model, DecodeError> {
        // `import_slice` parses .gltf/.glb and resolves embedded buffers/images.
        let doc = {
            let bytes = &self
                .table
                .get(&data)
                .map_err(|e| DecodeError::InvalidData(e.to_string()))?
                .0;
            let (document, buffers, images) =
                gltf::import_slice(bytes).map_err(|e| DecodeError::InvalidData(e.to_string()))?;
            GltfDoc {
                document,
                buffers,
                images,
            }
        };
        let model = self
            .table
            .push(doc)
            .map_err(|e| DecodeError::InvalidData(e.to_string()))?;
        Ok(Model::Gltf(model))
    }
}

impl HostGltfModel for JumpjetRuntimeState {
    async fn scene_count(&mut self, model: Resource<GltfModel>) -> u32 {
        self.table.get(&model).unwrap().document.scenes().count() as u32
    }

    async fn default_scene(&mut self, model: Resource<GltfModel>) -> Option<u32> {
        self.table
            .get(&model)
            .unwrap()
            .document
            .default_scene()
            .map(|s| s.index() as u32)
    }

    async fn scene_nodes(&mut self, model: Resource<GltfModel>, scene: u32) -> Vec<u32> {
        let doc = &self.table.get(&model).unwrap().document;
        doc.scenes()
            .nth(scene as usize)
            .map(|s| s.nodes().map(|n| n.index() as u32).collect())
            .unwrap_or_default()
    }

    async fn node_count(&mut self, model: Resource<GltfModel>) -> u32 {
        self.table.get(&model).unwrap().document.nodes().count() as u32
    }

    async fn node(&mut self, model: Resource<GltfModel>, index: u32) -> Option<Node> {
        let doc = &self.table.get(&model).unwrap().document;
        let n = doc.nodes().nth(index as usize)?;
        let (t, r, s) = n.transform().decomposed();
        Some(Node {
            name: n.name().map(str::to_string),
            transform: Transform {
                translation: v3(t),
                rotation: v4(r),
                scale: v3(s),
            },
            mesh: n.mesh().map(|m| m.index() as u32),
            children: n.children().map(|c| c.index() as u32).collect(),
        })
    }

    async fn mesh_count(&mut self, model: Resource<GltfModel>) -> u32 {
        self.table.get(&model).unwrap().document.meshes().count() as u32
    }

    async fn mesh_name(&mut self, model: Resource<GltfModel>, index: u32) -> Option<String> {
        let doc = &self.table.get(&model).unwrap().document;
        doc.meshes()
            .nth(index as usize)?
            .name()
            .map(str::to_string)
    }

    async fn primitive_count(&mut self, model: Resource<GltfModel>, mesh: u32) -> u32 {
        let doc = &self.table.get(&model).unwrap().document;
        doc.meshes()
            .nth(mesh as usize)
            .map(|m| m.primitives().count() as u32)
            .unwrap_or(0)
    }

    async fn primitive(
        &mut self,
        model: Resource<GltfModel>,
        mesh: u32,
        primitive: u32,
    ) -> Option<Primitive> {
        let m = self.table.get(&model).unwrap();
        let prim = m
            .document
            .meshes()
            .nth(mesh as usize)?
            .primitives()
            .nth(primitive as usize)?;

        let buffers = &m.buffers;
        let reader = prim.reader(|b| buffers.get(b.index()).map(|d| d.0.as_slice()));

        let positions: Vec<Vec3> = reader.read_positions()?.map(v3).collect();
        Some(Primitive {
            mode: topology(prim.mode()),
            material: prim.material().index().map(|i| i as u32),
            positions,
            normals: reader.read_normals().map(|it| it.map(v3).collect()),
            tangents: reader.read_tangents().map(|it| it.map(v4).collect()),
            tex_coords: reader
                .read_tex_coords(0)
                .map(|tc| tc.into_f32().map(v2).collect()),
            colors: reader
                .read_colors(0)
                .map(|c| c.into_rgba_f32().map(v4).collect()),
            indices: reader.read_indices().map(|i| i.into_u32().collect()),
        })
    }

    async fn material_count(&mut self, model: Resource<GltfModel>) -> u32 {
        self.table.get(&model).unwrap().document.materials().count() as u32
    }

    async fn material(&mut self, model: Resource<GltfModel>, index: u32) -> Option<Material> {
        let doc = &self.table.get(&model).unwrap().document;
        let mat = doc.materials().nth(index as usize)?;
        let pbr = mat.pbr_metallic_roughness();
        let image_of = |info: gltf::texture::Info| info.texture().source().index() as u32;
        Some(Material {
            name: mat.name().map(str::to_string),
            base_color_factor: v4(pbr.base_color_factor()),
            base_color_texture: pbr.base_color_texture().map(image_of),
            metallic_factor: pbr.metallic_factor(),
            roughness_factor: pbr.roughness_factor(),
            metallic_roughness_texture: pbr.metallic_roughness_texture().map(image_of),
            normal_texture: mat
                .normal_texture()
                .map(|t| t.texture().source().index() as u32),
            occlusion_texture: mat
                .occlusion_texture()
                .map(|t| t.texture().source().index() as u32),
            emissive_factor: v3(mat.emissive_factor()),
            emissive_texture: mat.emissive_texture().map(image_of),
            alpha_mode: match mat.alpha_mode() {
                gltf::material::AlphaMode::Opaque => AlphaMode::Opaque,
                gltf::material::AlphaMode::Mask => AlphaMode::Mask,
                gltf::material::AlphaMode::Blend => AlphaMode::Blend,
            },
            alpha_cutoff: mat.alpha_cutoff().unwrap_or(0.5),
            double_sided: mat.double_sided(),
        })
    }

    async fn image_count(&mut self, model: Resource<GltfModel>) -> u32 {
        self.table.get(&model).unwrap().images.len() as u32
    }

    async fn image(
        &mut self,
        model: Resource<GltfModel>,
        index: u32,
    ) -> Option<Resource<ImageBitmap>> {
        // Build the RGBA8 bitmap first (releases the immutable `table` borrow),
        // then push it as a new resource.
        let bitmap = {
            let m = self.table.get(&model).ok()?;
            to_rgba8(m.images.get(index as usize)?)?
        };
        self.table.push(bitmap).ok()
    }

    async fn drop(&mut self, rep: Resource<GltfModel>) -> Result<()> {
        self.table.delete(rep).ok();
        Ok(())
    }
}

fn v2(a: [f32; 2]) -> Vec2 {
    Vec2 { x: a[0], y: a[1] }
}
fn v3(a: [f32; 3]) -> Vec3 {
    Vec3 {
        x: a[0],
        y: a[1],
        z: a[2],
    }
}
fn v4(a: [f32; 4]) -> Vec4 {
    Vec4 {
        x: a[0],
        y: a[1],
        z: a[2],
        w: a[3],
    }
}

fn topology(mode: gltf::mesh::Mode) -> PrimitiveTopology {
    use gltf::mesh::Mode;
    match mode {
        Mode::Points => PrimitiveTopology::Points,
        Mode::Lines => PrimitiveTopology::Lines,
        Mode::LineLoop => PrimitiveTopology::LineLoop,
        Mode::LineStrip => PrimitiveTopology::LineStrip,
        Mode::Triangles => PrimitiveTopology::Triangles,
        Mode::TriangleStrip => PrimitiveTopology::TriangleStrip,
        Mode::TriangleFan => PrimitiveTopology::TriangleFan,
    }
}

/// Convert a decoded glTF image to an RGBA8 `Bitmap`. Returns `None` for the
/// 16-/32-bit formats (uncommon in game assets) which we don't repack yet.
fn to_rgba8(img: &gltf::image::Data) -> Option<Bitmap> {
    use gltf::image::Format;
    let rgba = match img.format {
        Format::R8G8B8A8 => img.pixels.clone(),
        Format::R8G8B8 => img
            .pixels
            .chunks_exact(3)
            .flat_map(|c| [c[0], c[1], c[2], 255])
            .collect(),
        Format::R8 => img.pixels.iter().flat_map(|&g| [g, g, g, 255]).collect(),
        // Two-channel is treated as luminance + alpha.
        Format::R8G8 => img
            .pixels
            .chunks_exact(2)
            .flat_map(|c| [c[0], c[0], c[0], c[1]])
            .collect(),
        _ => return None,
    };
    Some(Bitmap {
        width: img.width,
        height: img.height,
        rgba,
    })
}
