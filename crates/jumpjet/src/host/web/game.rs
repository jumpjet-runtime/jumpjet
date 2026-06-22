//! Web `Game`: instantiates the guest component in the browser via jco and holds
//! the resulting guest export callables. The native equivalent
//! (`host/native/game.rs`) drives wasmtime instead.
//!
//! Harness contract (provided by the HTML harness as `window.jco`), matching the
//! validated jco instantiation-mode output:
//!   - `window.jco.instantiate(imports) -> Promise<instance>` — the harness binds
//!     jco's `instantiate(getCoreModule, imports)`, providing `getCoreModule` to
//!     fetch/compile the emitted `*.core*.wasm` files.
//!   - `instance["jumpjet:runtime/game"]` exposes a `Game` resource class with a
//!     static `init()` returning a game instance, plus `update`/`render` methods.
//!     The web host runs the `game` (client/singleplayer) component; the separate
//!     `server` component is driven by a future headless host.
//! The guest is transpiled at build time, so there is no runtime wasm binary.

use std::time::Duration;

use js_sys::{Function, Object, Reflect};
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

use crate::runtime::web::state::SharedState;
use crate::runtime::web::{debug, window};

pub struct Game {
    pub path: String,
    #[allow(dead_code)]
    state: SharedState,
    /// The guest's `Game` resource class (carries the static `init`).
    game_class: Function,
    /// The game instance returned by `init`; `update`/`render` are called on it.
    instance: Option<Object>,
    update_fn: Function,
    render_fn: Function,
}

impl std::fmt::Debug for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("Game").field("path", &self.path).finish()
    }
}

impl Game {
    /// Instantiates the (build-time-transpiled) guest component via the JS `jco`
    /// harness, wiring the host import objects and capturing the guest's exported
    /// functions.
    pub async fn instantiate(state: SharedState) -> Result<Game, JsValue> {
        let window_obj = web_sys::window().ok_or_else(|| JsValue::from_str("no global window"))?;
        let jco = Reflect::get(&window_obj, &JsValue::from_str("jco"))?;
        if jco.is_undefined() || jco.is_null() {
            return Err(JsValue::from_str("window.jco harness not found"));
        }

        // window.jco.instantiate(imports) -> Promise<instance>
        let instantiate_fn =
            Reflect::get(&jco, &JsValue::from_str("instantiate"))?.dyn_into::<Function>()?;
        let imports = build_imports(&state)?;
        let instance = JsFuture::from(
            instantiate_fn
                .call1(&jco, &imports)?
                .dyn_into::<js_sys::Promise>()?,
        )
        .await?;

        // instance["jumpjet:runtime/game"] -> { Game } (resource class)
        let guest: Object = Reflect::get(&instance, &JsValue::from_str("jumpjet:runtime/game"))?
            .dyn_into()
            .map_err(|_| JsValue::from_str("guest export `jumpjet:runtime/game` missing"))?;

        let game_class = Reflect::get(&guest, &JsValue::from_str("Game"))?
            .dyn_into::<Function>()
            .map_err(|_| JsValue::from_str("guest export `Game` resource missing"))?;
        // Instance methods live on the class prototype.
        let prototype = Reflect::get(&game_class, &JsValue::from_str("prototype"))?;
        let update_fn =
            Reflect::get(&prototype, &JsValue::from_str("update"))?.dyn_into::<Function>()?;
        let render_fn =
            Reflect::get(&prototype, &JsValue::from_str("render"))?.dyn_into::<Function>()?;

        Ok(Self {
            path: "bytes".to_owned(),
            state,
            game_class,
            instance: None,
            update_fn,
            render_fn,
        })
    }

    pub fn init(&mut self) -> Result<(), JsValue> {
        // Static `init` lives on the class; it constructs and returns the game.
        let init_fn = Reflect::get(&self.game_class, &JsValue::from_str("init"))?
            .dyn_into::<Function>()
            .map_err(|_| JsValue::from_str("guest export `Game.init` missing"))?;
        let instance: Object = init_fn.call0(&self.game_class)?.dyn_into()?;
        self.instance = Some(instance);
        Ok(())
    }

    pub fn update(&mut self, epoch: Duration, delta: Duration) -> Result<(), JsValue> {
        let instance = self
            .instance
            .as_ref()
            .ok_or_else(|| JsValue::from_str("game not initialized"))?;
        self.update_fn.call2(
            instance,
            &JsValue::from_f64(epoch.as_secs_f64()),
            &JsValue::from_f64(delta.as_secs_f64()),
        )?;
        Ok(())
    }

    pub fn render(&mut self, epoch: Duration, alpha: f64) -> Result<(), JsValue> {
        let instance = self
            .instance
            .as_ref()
            .ok_or_else(|| JsValue::from_str("game not initialized"))?;
        self.render_fn.call2(
            instance,
            &JsValue::from_f64(epoch.as_secs_f64()),
            &JsValue::from_f64(alpha),
        )?;
        // Phase 2: present the swapchain surface if state.gpu_state requests it.
        Ok(())
    }
}

/// Assembles the Rust-provided portion of the guest import object: the pure-data
/// interfaces (`debug`, `window`). The resource-bearing interfaces (gpu, audio,
/// input, storage) are JS classes provided by the harness, which merges them with
/// these and the WASI shim before calling jco's `instantiate` (see `web-runtime/`).
fn build_imports(state: &SharedState) -> Result<Object, JsValue> {
    let imports = Object::new();
    Reflect::set(
        &imports,
        &JsValue::from_str("jumpjet:runtime/debug"),
        &debug::export(state.clone()),
    )?;
    Reflect::set(
        &imports,
        &JsValue::from_str("jumpjet:runtime/window"),
        &window::export(state.clone()),
    )?;
    Ok(imports)
}
