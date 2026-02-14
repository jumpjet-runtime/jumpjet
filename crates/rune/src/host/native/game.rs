use std::{path::PathBuf, time::Duration};

use anyhow::{Ok, Result};
use gilrs::Gilrs;
use uuid::Uuid;
use wasmtime::{
    component::{Component, Linker},
    Config, Engine, Store,
};
use winit::{dpi::PhysicalSize, window::Window};

use crate::{Runtime, RuntimePre};

pub use crate::runtime::RuneRuntimeState;

/// Game is used to run wasm component

pub struct Game {
    pub path: String,
    pub engine: Engine,
    pub instance_pre: RuntimePre<RuneRuntimeState>,
    pub runtime: Option<Runtime>,
    pub store: Option<Store<RuneRuntimeState>>,
}

impl std::fmt::Debug for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("Game").field("path", &self.path).finish()
    }
}

impl Game {
    pub fn from_binary(bytes: &[u8]) -> Result<Game> {
        let mut config = Config::new();
        config.wasm_backtrace_details(wasmtime::WasmBacktraceDetails::Enable);
        config.async_support(true);
        config.wasm_component_model(true);
        let engine = Engine::new(&config)?;
        let component = Component::from_binary(&engine, &bytes)?;

        let mut linker = Linker::new(&engine);

        wasmtime_wasi::p2::add_to_linker_sync(&mut linker).expect("add wasmtime_wasi::preview2 failed");

        type Data = wasmtime::component::HasSelf<RuneRuntimeState>;

        Runtime::add_to_linker::<_, Data>(&mut linker, |state: &mut RuneRuntimeState| state)?;

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

        let runtime_state = RuneRuntimeState::new(
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

        if let Err(msg) = runtime.rune_runtime_guest().call_init(&mut store).await {
            panic!("{}", msg);
        }

        self.runtime = Some(runtime);
        self.store = Some(store);

        Ok(())
    }

    pub async fn update(
        &mut self,
        epoch_time: Duration,
        delta_time: Duration,
    ) -> Result<(), anyhow::Error> {
        let store = self.store.as_mut().unwrap();
        self.runtime
            .as_ref()
            .unwrap()
            .rune_runtime_guest()
            .call_update(store, epoch_time.as_secs_f64(), delta_time.as_secs_f64())
            .await?;

        Ok(())
    }

    pub async fn render(
        &mut self,
        epoch_time: Duration,
        alpha: f64,
    ) -> Result<(), anyhow::Error> {
        self.runtime
            .as_ref()
            .expect("Runtime must be initialized")
            .rune_runtime_guest()
            .call_render(
                self.store.as_mut().expect("Store must be initialized"),
                epoch_time.as_secs_f64(),
                alpha,
            )
            .await?;

        let store = self.store.as_mut().expect("Store must be initialized");
        let ctx = store.data_mut();
        let surface_id = ctx.surface;

        if ctx.gpu_state.present_surface {
            ctx.instance.surface_present(surface_id)?;
            ctx.gpu_state.present_surface = false;
        }

        Ok(())
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        let store = self.store.as_mut().unwrap();
        let ctx = store.data_mut();
        let surface_id = ctx.surface;
        let device_id = ctx.device;

        let surface_config = &mut ctx.surface_config;

        surface_config.width = size.width;
        surface_config.height = size.height;

        ctx.instance
            .surface_configure(surface_id, device_id, &surface_config);
    }
}
