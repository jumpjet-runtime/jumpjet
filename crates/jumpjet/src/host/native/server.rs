//! Headless server host: instantiates a `server-runtime` component and drives its
//! exported `server` interface (`init` + `update`, no render) on a fixed timestep.
//! The windowed client equivalent is [`super::game::Game`].

use std::{path::PathBuf, time::Duration};

use anyhow::Result;
use uuid::Uuid;
use wasmtime::{
    Config, Engine, Store,
    component::{Component, Linker},
};

use crate::{ServerRuntime, ServerRuntimePre};

pub use crate::runtime::JumpjetRuntimeState;

pub struct Server {
    pub engine: Engine,
    pub instance_pre: ServerRuntimePre<JumpjetRuntimeState>,
    pub runtime: Option<ServerRuntime>,
    pub store: Option<Store<JumpjetRuntimeState>>,
}

impl std::fmt::Debug for Server {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("Server").finish()
    }
}

impl Server {
    /// JIT-compile a server component. Servers run on desktop/Linux hosts, so —
    /// unlike the client — there's no iOS AOT path here.
    pub fn from_binary(bytes: &[u8]) -> Result<Server> {
        let mut config = Config::new();
        config.wasm_component_model(true);
        config.wasm_component_model_async(true);

        let engine = Engine::new(&config)?;
        let component = Component::from_binary(&engine, bytes)?;

        let mut linker = Linker::new(&engine);

        // Same WASI 0.3 + 0.2 linking as the client (see host/native/game.rs).
        wasmtime_wasi::p3::add_to_linker(&mut linker)?;
        wasmtime_wasi::p2::add_to_linker_async(&mut linker)?;

        type Data = wasmtime::component::HasSelf<JumpjetRuntimeState>;
        // Links only debug/tasks/storage — the interfaces the server world imports.
        ServerRuntime::add_to_linker::<_, Data>(&mut linker, |state: &mut JumpjetRuntimeState| {
            state
        })?;

        let instance_pre = ServerRuntimePre::new(linker.instantiate_pre(&component)?)?;

        Ok(Self {
            engine,
            instance_pre,
            runtime: None,
            store: None,
        })
    }

    pub async fn init(&mut self, input_path: PathBuf) -> Result<()> {
        let state = JumpjetRuntimeState::new_headless(Uuid::new_v4(), input_path)?;
        let mut store = Store::new(&self.engine, state);

        let runtime = self.instance_pre.instantiate_async(&mut store).await?;

        if let Err(msg) = runtime
            .jumpjet_runtime_server()
            .call_init(&mut store)
            .await?
        {
            panic!("{}", msg);
        }

        self.runtime = Some(runtime);
        self.store = Some(store);

        Ok(())
    }

    pub async fn update(&mut self, epoch_time: Duration, delta_time: Duration) -> Result<()> {
        let store = self.store.as_mut().expect("Store must be initialized");
        self.runtime
            .as_ref()
            .expect("Runtime must be initialized")
            .jumpjet_runtime_server()
            .call_update(store, epoch_time.as_secs_f64(), delta_time.as_secs_f64())
            .await?;

        Ok(())
    }
}
