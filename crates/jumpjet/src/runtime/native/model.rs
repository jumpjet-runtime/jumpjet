use wasmtime::Result;
use wasmtime::component::{Accessor, HasSelf, Resource, StreamReader};

use super::state::JumpjetRuntimeState;
use super::stream::read_stream_to_vec;
use crate::jumpjet::runtime::model::*;

impl Host for JumpjetRuntimeState {}

impl HostGltfModel for JumpjetRuntimeState {
    async fn scene_count(&mut self, model: Resource<GltfModel>) -> u32 {
        self.table.get(&model).unwrap().scenes().count() as u32
    }

    async fn mesh_count(&mut self, model: Resource<GltfModel>) -> u32 {
        self.table.get(&model).unwrap().meshes().count() as u32
    }

    async fn drop(&mut self, rep: Resource<GltfModel>) -> Result<()> {
        self.table.delete(rep).ok();
        Ok(())
    }
}

impl HostWithStore for HasSelf<JumpjetRuntimeState> {
    async fn decode<T: Send>(
        accessor: &Accessor<T, Self>,
        data: StreamReader<u8>,
    ) -> core::result::Result<Model, DecodeError> {
        let bytes = read_stream_to_vec(accessor, data)
            .await
            .map_err(|e| DecodeError::InvalidData(e.to_string()))?;

        // `Gltf::from_slice` handles both `.gltf` (JSON) and `.glb` (binary).
        let gltf =
            gltf::Gltf::from_slice(&bytes).map_err(|e| DecodeError::InvalidData(e.to_string()))?;

        let model = accessor
            .with(|mut access| access.get().table.push(gltf))
            .map_err(|e| DecodeError::InvalidData(e.to_string()))?;
        Ok(Model::Gltf(model))
    }
}
