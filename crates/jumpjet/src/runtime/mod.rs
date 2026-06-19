// `common` holds native-shaped shared types (wgpu_core/cpal/vfs); it is only
// part of the native build. The web runtime defines its own JS-handle state.
#[cfg(not(target_arch = "wasm32"))]
mod common;
#[cfg(target_arch = "wasm32")]
pub mod web;
#[cfg(not(target_arch = "wasm32"))]
mod native;

#[cfg(not(target_arch = "wasm32"))]
pub use common::*;
#[cfg(target_arch = "wasm32")]
pub use web::*;
#[cfg(not(target_arch = "wasm32"))]
pub use native::{
    run::run,
    run::test,
    state::JumpjetRuntimeState
};

// Android has no `main()`; the OS calls into `android_main`, which the generated
// bundle wrapper exports and forwards to these entries along with the `AndroidApp`.
#[cfg(target_os = "android")]
pub use native::run::{prepare_android_input, run_android};
