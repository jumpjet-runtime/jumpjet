use std::{path::PathBuf, time::Duration, sync::{Arc, Mutex}, net::TcpStream};

use anyhow::{Ok, Result};
use gilrs::Gilrs;
use uuid::Uuid;
use wasmtime::{
    component::{Component, Linker},
    Config, Engine, Store, DebugEvent, AsContextMut, DebugHandler,
};
use winit::{dpi::PhysicalSize, window::Window};

use crate::{Runtime, RuntimePre};

pub use crate::runtime::JumpjetRuntimeState;

#[derive(Clone)]
pub struct JumpjetDebugHandler {
    pub gdb_connection: Arc<Mutex<Option<TcpStream>>>,
    pub dap_connection: Arc<Mutex<Option<crate::debug::dap::DapConnection>>>,
    pub binary: Arc<Vec<u8>>,
}

impl DebugHandler for JumpjetDebugHandler {
    type Data = JumpjetRuntimeState;

    fn handle(
        &self,
        store: wasmtime::StoreContextMut<'_, Self::Data>,
        event: DebugEvent<'_>,
    ) -> impl std::future::Future<Output = ()> + Send {
        let conn_clone = self.gdb_connection.clone();
        let dap_conn_clone = self.dap_connection.clone();
        let binary_clone = self.binary.clone();
        async move {
            {
                let mut conn_lock = dap_conn_clone.lock().unwrap();
                if let Some(conn) = conn_lock.as_mut() {
                    // Map DebugEvent to StoppedEventReason
                    // For now, assume it's a Breakpoint since we are in debug handler
                    let reason = dapts::StoppedEventReason::Breakpoint;
                    
                    if let Err(e) = crate::debug::dap::handle_dap_event(store, conn, Some(reason), None, binary_clone) {
                        eprintln!("DAP handler error: {:?}", e);
                    }
                    return;
                }
            }

            let mut conn_lock = conn_clone.lock().unwrap();
            if let Some(conn) = conn_lock.as_mut() {
                if let Err(e) = crate::debug::gdb::handle_gdb_event(store, conn) {
                    log::error!("GDB handler error: {:?}", e);
                }
            }
        }
    }
}

pub struct Game {
    pub path: String,
    pub engine: Engine,
    pub instance_pre: RuntimePre<JumpjetRuntimeState>,
    pub runtime: Option<Runtime>,
    pub store: Option<Store<JumpjetRuntimeState>>,
    pub debug: bool,
    pub gdb_connection: Arc<Mutex<Option<TcpStream>>>,
    pub dap_connection: Arc<Mutex<Option<crate::debug::dap::DapConnection>>>,
    pub binary: Arc<Vec<u8>>,
}

impl std::fmt::Debug for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("Game").field("path", &self.path).finish()
    }
}

impl Game {
    pub fn from_binary(bytes: &[u8], debug: bool) -> Result<Game> {
        let mut config = Config::new();
        config.wasm_component_model(true);
        config.guest_debug(debug);

        let engine = Engine::new(&config)?;
        let component = Component::from_binary(&engine, bytes)?;
        let mut linker = Linker::new(&engine);

        wasmtime_wasi::p2::add_to_linker_sync(&mut linker)?;
        
        type Data = wasmtime::component::HasSelf<JumpjetRuntimeState>;
        Runtime::add_to_linker::<_, Data>(&mut linker, |state: &mut JumpjetRuntimeState| state)?;

        let instance_pre = RuntimePre::new(linker.instantiate_pre(&component)?)?;
        
        Ok(Self {
            path: "bytes".to_owned(),
            engine,
            instance_pre,
            runtime: None,
            store: None,
            debug,
            gdb_connection: Arc::new(Mutex::new(None)),
            dap_connection: Arc::new(Mutex::new(None)),
            binary: Arc::new(bytes.to_vec()),
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
        
        if self.debug {
            let handler = JumpjetDebugHandler {
                gdb_connection: self.gdb_connection.clone(),
                dap_connection: self.dap_connection.clone(),
                binary: self.binary.clone(),
            };
            store.set_debug_handler(handler);
        }

        let runtime = self.instance_pre.instantiate_async(&mut store).await?;

        if let Err(msg) = runtime.jumpjet_runtime_guest().call_init(&mut store).await {
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
            .jumpjet_runtime_guest()
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
            .jumpjet_runtime_guest()
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
            ctx.gpu_state.current_surface_texture = None;
        }

        Ok(())
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        if let Some(store) = self.store.as_mut() {
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
}
