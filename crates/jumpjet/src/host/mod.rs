#[cfg(not(target_arch = "wasm32"))]
mod native;
#[cfg(target_arch = "wasm32")]
mod web;

#[cfg(not(target_arch = "wasm32"))]
pub use native::game::Game;
#[cfg(not(target_arch = "wasm32"))]
pub use native::server::Server;
#[cfg(target_arch = "wasm32")]
pub use web::game::Game;
