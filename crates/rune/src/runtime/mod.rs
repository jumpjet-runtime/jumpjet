// `common` holds native-shaped shared types (wgpu_core/cpal/vfs); it is only
// part of the native build. The web runtime defines its own JS-handle state.
#[cfg(not(target_arch = "wasm32"))]
mod common;
#[cfg(target_arch = "wasm32")]
pub mod web;
#[cfg(any(target_os = "macos", target_os = "windows"))]
mod native;

#[cfg(not(target_arch = "wasm32"))]
pub use common::*;
#[cfg(target_arch = "wasm32")]
pub use web::*;
#[cfg(any(target_os = "macos", target_os = "windows"))]
pub use native::{
    run::run,
    run::test,
    state::RuneRuntimeState
};
