#[cfg(not(target_arch = "wasm32"))]
use anyhow::Result;
// Input tracking types are native-only; the web build tracks input in
// `runtime/web/input.rs` (DOM events + navigator.getGamepads).
#[cfg(not(target_arch = "wasm32"))]
use winit::keyboard::{Key, KeyLocation, PhysicalKey};

pub mod host;
pub mod runtime;
// Pulley AOT (precompile host-side, deserialize on iOS); shares `pulley_config`
// with the on-device loader.
#[cfg(not(target_arch = "wasm32"))]
pub mod aot;
#[cfg(not(target_arch = "wasm32"))]
pub mod debug;
#[cfg(not(target_arch = "wasm32"))]
pub mod tests;

// Re-exported so the generated Android bundle wrapper can name `AndroidApp`
// against the exact `winit`/`android-activity` version the runtime links (the
// glue symbol must come from a single version — see `runtime::run_android`).
#[cfg(not(target_arch = "wasm32"))]
pub use winit;

// needed for wasmtime::component::bindgen! as it only looks in the current crate.
#[cfg(not(target_arch = "wasm32"))]
pub(crate) use gilrs;
#[cfg(not(target_arch = "wasm32"))]
pub(crate) use wgpu_core;

pub type BufferSource = Vec<u8>;

#[cfg(not(target_arch = "wasm32"))]
pub fn wgpu_id<I: wgpu_core::id::Marker, E>(
    (id, error): (wgpu_core::id::Id<I>, Option<E>),
) -> Result<wgpu_core::id::Id<I>, E> {
    match error {
        Some(error) => Err(error),
        None => core::result::Result::Ok(id),
    }
}

// Host bindings are split by component so each `bindgen!` targets its own profile
// world. `client` is the primary generator (every imported interface module lives
// at `crate::client::jumpjet::runtime::*`, re-exported below as `crate::jumpjet::…`);
// `server` reuses those modules via `with:` (see `server.rs`).
#[cfg(not(target_arch = "wasm32"))]
mod client;
#[cfg(not(target_arch = "wasm32"))]
mod server;

#[cfg(not(target_arch = "wasm32"))]
pub use client::{ClientRuntime, ClientRuntimePre, exports, jumpjet};
#[cfg(not(target_arch = "wasm32"))]
pub use server::{ServerRuntime, ServerRuntimePre};

#[cfg(not(target_arch = "wasm32"))]
pub use exports::jumpjet::runtime::game;
#[cfg(not(target_arch = "wasm32"))]
pub use jumpjet::runtime::*;
#[cfg(not(target_arch = "wasm32"))]
use wasmtime_wasi::{ResourceTable, WasiCtxView, WasiView};

pub use runtime::JumpjetRuntimeState;

#[cfg(not(target_arch = "wasm32"))]
impl WasiView for JumpjetRuntimeState {
    fn ctx(&mut self) -> WasiCtxView<'_> {
        WasiCtxView {
            ctx: &mut self.wasi_ctx,
            table: &mut self.table,
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub struct KeyboardState {
    /// `(generation, physical_key, logical_key, location)`.
    ///
    /// Entries are matched for insert/remove on `physical_key`, which is
    /// modifier- and layout-invariant, so a key always clears on release even
    /// if modifier state changed since it was pressed (e.g. releasing Shift
    /// before the letter you were holding). The `logical_key`/`location` are
    /// retained only as the value reported back to the guest.
    pub active_keys: Vec<(u64, PhysicalKey, Key, KeyLocation)>,
}

#[cfg(not(target_arch = "wasm32"))]
impl KeyboardState {
    pub fn new() -> KeyboardState {
        Self {
            active_keys: Vec::new(),
        }
    }
}

/// Mouse cursor + pointer-lock state, mirrored from winit events by the run loop.
#[cfg(not(target_arch = "wasm32"))]
#[derive(Default)]
pub struct MouseState {
    /// Cursor position in physical pixels relative to the window, top-left origin.
    pub x: f32,
    pub y: f32,
    /// Raw movement accumulated since the last logic frame, in physical pixels.
    pub dx: f32,
    pub dy: f32,
    /// Mouse buttons currently held down, mirrored from winit press/release.
    pub buttons: Vec<winit::event::MouseButton>,
    /// Whether the pointer is currently locked (cursor grabbed + hidden).
    pub locked: bool,
    /// Pending lock (`Some(true)`) / unlock (`Some(false)`) request from the
    /// guest, applied to the window by the run loop and then cleared.
    pub lock_request: Option<bool>,
}

#[cfg(not(target_arch = "wasm32"))]
impl MouseState {
    pub fn new() -> MouseState {
        Self::default()
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub struct GamepadState {
    pub active_buttons: Vec<(u64, gilrs::GamepadId, gilrs::Button, bool)>,
}

#[cfg(not(target_arch = "wasm32"))]
impl GamepadState {
    pub fn new() -> GamepadState {
        Self {
            active_buttons: Vec::new(),
        }
    }
}
