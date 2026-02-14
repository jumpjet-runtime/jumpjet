use std::{path::PathBuf, time::Duration};

use anyhow::{Ok, Result};
use uuid::Uuid;
use winit::{dpi::PhysicalSize, window::Window};

use crate::{Runtime, RuntimePre};

pub use crate::runtime::RuneRuntimeState;

pub struct Game {
    pub path: String,
}

impl std::fmt::Debug for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("Game").field("path", &self.path).finish()
    }
}

impl Game {
    pub async fn from_binary(bytes: &[u8]) -> Result<Game> {
        let window = web_sys::window().unwrap();
        let jco = Reflect::get(&window, &JsValue::from_str("jco"))?;
        // Use JCO to instantiate the component
        let component_load_fn = Reflect::get(&jco, &JsValue::from_str("load"))?.dyn_into::<Function>()?;
        let component = JsFuture::from(
            component_load_fn.call1(&jco, &array_buffer)?
                .dyn_into::<js_sys::Promise>()?
        ).await?;
        
        // Initialize the component with potential imports (if needed)
        let component_init_fn = Reflect::get(&component, &JsValue::from_str("instantiate"))?.dyn_into::<Function>()?;
        let imports = Object::new();

        let debug = crate::runtime::debug::export();
        Reflect::set(&imports, &JsValue::from_str("rune:runtime/debug"), &debug)?;

        let gpu = crate::runtime::gpu::export();
        Reflect::set(&imports, &JsValue::from_str("rune:runtime/gpu"), &gpu)?;
        
        let instance = JsFuture::from(
            component_init_fn.call1(&component, &imports)?
                .dyn_into::<js_sys::Promise>()?
        ).await?;

        Ok(Self {
            path: "bytes".to_owned()
        })
    }

    pub async fn init(
        &mut self,
        window: &Window,
        input_path: PathBuf
    ) -> Result<(), anyhow::Error> {
        let window_size = window.inner_size();

        let runtime_state = RuneRuntimeState::new(
            Uuid::new_v4(),
            input_path,
            window_size
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
