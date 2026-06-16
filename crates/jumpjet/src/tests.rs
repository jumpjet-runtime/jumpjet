use std::path::PathBuf;

use anyhow::{Ok, Result};
use gilrs::Gilrs;
use uuid::Uuid;
use wasmtime::{
    component::{Component, Linker},
    Config, Engine, Store,
};
use winit::window::Window;

use crate::{JumpjetRuntimeState, Runtime, RuntimePre};

/// Test is used to run wasm component

pub struct Tests {
    pub path: String,
    pub engine: Engine,
    pub instance_pre: RuntimePre<JumpjetRuntimeState>,
    pub runtime: Option<Runtime>,
    pub store: Option<Store<JumpjetRuntimeState>>,
}

impl std::fmt::Debug for Tests {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("Test").field("path", &self.path).finish()
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl Tests {
    pub fn from_binary(bytes: &[u8]) -> Result<Tests> {
        let mut config = Config::new();
        config.wasm_backtrace_details(wasmtime::WasmBacktraceDetails::Enable);
        config.wasm_component_model(true);
        let engine = Engine::new(&config)?;
        let component = Component::from_binary(&engine, &bytes)?;

        let mut linker = Linker::new(&engine);

        type Data = wasmtime::component::HasSelf<JumpjetRuntimeState>;

        wasmtime_wasi::p2::add_to_linker_sync(&mut linker).expect("add wasmtime_wasi::preview2 failed");

        Runtime::add_to_linker::<_, Data>(&mut linker, |state: &mut JumpjetRuntimeState| state)?;

        Ok(Self {
            path: "bytes".to_owned(),
            engine,
            instance_pre: RuntimePre::new(linker.instantiate_pre(&component)?)?,
            runtime: None,
            store: None,
        })
    }

    pub async fn init(
        &mut self,
        window: &Window,
        input_path: PathBuf,
        audio_device: cpal::Device,
        instance: wgpu_core::global::Global,
        surface: wgpu_core::id::SurfaceId,
        adapter: wgpu_core::id::AdapterId,
        device: wgpu_core::id::DeviceId,
        queue: wgpu_core::id::QueueId,
        gilrs: Gilrs,
    ) -> Result<(), anyhow::Error> {
        let window_size = window.inner_size();

        let runtime_state = JumpjetRuntimeState::new(
            Uuid::new_v4(),
            input_path,
            window_size,
            audio_device,
            instance,
            surface,
            adapter,
            device,
            queue,
            gilrs,
        );

        let mut store = Store::new(&self.engine, runtime_state);

        let runtime = self.instance_pre.instantiate_async(&mut store).await?;

        self.runtime = Some(runtime);
        self.store = Some(store);

        Ok(())
    }

    pub async fn run(&mut self, _test_name: String) {}
}
