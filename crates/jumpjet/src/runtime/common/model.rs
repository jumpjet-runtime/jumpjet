/// A decoded glTF/GLB document with its buffers and images resolved, backing the
/// `gltf-model` resource. `gltf::import_slice` handles embedded (GLB blob /
/// data-URI) buffers and images; external-file URIs aren't reachable from a
/// stream and simply won't be present.
pub struct GltfModel {
    pub document: gltf::Document,
    pub buffers: Vec<gltf::buffer::Data>,
    pub images: Vec<gltf::image::Data>,
}
