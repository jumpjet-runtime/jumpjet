//! Web (wasm32) runtime. The guest component is instantiated in the browser via
//! jco; each host interface is exposed to the guest as a JS import object built
//! by the per-interface `export()` functions below. The Rune runtime itself is
//! compiled to wasm and drives the frame loop.

// Pure-data interfaces provided by the Rust host. The resource-bearing
// interfaces (gpu, audio, input, storage) are JS classes in `web-runtime/imports`
// (jco requires imported resources to be real instanceof-checked JS classes).
pub mod debug;
pub mod window;

// Resource interfaces as wasm-bindgen classes (jco requires instanceof-checked
// classes). `input` is the validation slice for the pattern; gpu/audio/storage
// follow once proven.
pub mod input;
pub mod gpu;
pub mod audio;
pub mod storage;

pub mod run;
pub mod state;

pub use run::run;
pub use state::RuneRuntimeState;
