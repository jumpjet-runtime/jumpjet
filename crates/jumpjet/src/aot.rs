//! Ahead-of-time compilation to wasmtime's [Pulley] portable bytecode.
//!
//! iOS App Store apps cannot map writable+executable memory, so the default
//! Cranelift JIT path (`Component::from_binary`, which compiles to native code at
//! startup) is a non-starter there. Instead we precompile the guest component to
//! Pulley — an architecture-independent bytecode interpreted at runtime — at
//! *bundle* time on the dev machine, ship the resulting `.cwasm`, and load it on
//! device with `Component::deserialize` (mapping it read-only executable, no JIT).
//!
//! [`pulley_config`] is the single source of truth shared by the host precompile
//! step and the on-device loader (`host::native::game`): `Component::deserialize`
//! checks the artifact against the engine's configuration, so the two must agree.
//!
//! [Pulley]: https://docs.rs/wasmtime/latest/wasmtime/index.html#pulley

use anyhow::Result;
use wasmtime::{Config, Engine};

/// Triple selecting the 64-bit little-endian Pulley target. iOS and Android are
/// both arm64/little-endian, and Pulley bytecode is architecture-independent, so
/// one `.cwasm` precompiled here runs on either device.
pub const PULLEY_TARGET: &str = "pulley64";

/// The wasmtime [`Config`] used for both precompiling to Pulley and loading the
/// precompiled artifact. The component-model flags must mirror the JIT path in
/// `Game::from_binary` so guests behave identically across backends.
pub fn pulley_config() -> Result<Config> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    // WASI 0.3 (p3) is built on the component-model async ABI.
    config.wasm_component_model_async(true);
    // Interpret with Pulley rather than JIT to native code: no RWX memory, so the
    // artifact runs under iOS's code-signing restrictions.
    config.target(PULLEY_TARGET)?;
    Ok(config)
}

/// Precompiles a componentized guest `.wasm` to a Pulley `.cwasm` artifact.
///
/// Runs host-side at bundle time (needs the Cranelift→Pulley backend, which the
/// runtime crate's wasmtime build enables). The bytes it returns are loaded on
/// device with `Component::deserialize` under [`pulley_config`].
pub fn precompile_pulley(wasm: &[u8]) -> Result<Vec<u8>> {
    let engine = Engine::new(&pulley_config()?)?;
    Ok(engine.precompile_component(wasm)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasmtime::component::Component;

    /// A precompiled component round-trips: what `precompile_pulley` emits is
    /// accepted by `Component::deserialize` under the same `pulley_config`. Guards
    /// against the precompile and load configs drifting out of sync.
    #[test]
    fn precompiled_component_deserializes() {
        // Minimal valid component (empty), as WAT text — `precompile_component`
        // parses it via wasmtime's `wat` default feature.
        let cwasm = precompile_pulley(b"(component)").expect("precompile");

        let engine = Engine::new(&pulley_config().unwrap()).unwrap();
        // Safety: `cwasm` was just produced by `precompile_pulley` with the same
        // engine configuration, so it is trusted and compatible.
        unsafe { Component::deserialize(&engine, &cwasm).expect("deserialize") };
    }
}
