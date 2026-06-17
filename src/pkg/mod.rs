//! Jumpjet package manager.
//!
//! A jumpjet *package* (`type = "lib"`) is a WebAssembly component that imports the
//! `jumpjet:runtime/*` host APIs and exports its own WIT interface. A *consumer* (a
//! game, or another package) declares it under `[dependencies]`; at build time the
//! dependency component is composed into the consumer (see [`compose`]) while the
//! host imports are left for the runtime `Linker` to satisfy. The package's WIT is
//! staged into `.jumpjet/wit/deps/...` so guest bindgen can resolve the imports.
//!
//! Identities use the wasm-pkg style `namespace:name@version` so they map cleanly
//! onto wasm-pkg-tools / WIT when the registry source lands.

pub mod compose;
pub mod lock;
pub mod manifest;
pub mod resolve;
pub mod source;
pub mod stage;
pub mod store;

pub use manifest::{Manifest, PackageId, PackageName};
