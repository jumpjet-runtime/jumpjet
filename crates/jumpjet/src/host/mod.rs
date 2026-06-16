#[cfg(target_arch = "wasm32")]
mod web;
#[cfg(any(target_os = "macos", target_os = "windows"))]
mod native;

#[cfg(target_arch = "wasm32")]
pub use web::game::Game;
#[cfg(any(target_os = "macos", target_os = "windows"))]
pub use native::game::Game;
