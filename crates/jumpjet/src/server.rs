//! Host bindings for the **headless server** component (`server-runtime` profile
//! world, which exports the `server` interface).
//!
//! The server world imports only `debug`/`tasks`/`storage`. Rather than
//! regenerate those interface bindings (which would produce distinct `Host`
//! traits that the existing `runtime/native/*` impls wouldn't satisfy), we `with`
//! them onto the client's already-generated modules. So this `bindgen!` emits
//! only `ServerRuntime`/`ServerRuntimePre` and the `server` export, and the
//! shared `Host` impls on `JumpjetRuntimeState` cover this world too.

wasmtime::component::bindgen!({
    world: "jumpjet:runtime/server-runtime",
    path: "wit/runtime",
    with: {
        // Reuse the client's generated bindings for the shared imports.
        "jumpjet:runtime/debug": crate::jumpjet::runtime::debug,
        "jumpjet:runtime/tasks": crate::jumpjet::runtime::tasks,
        "jumpjet:runtime/storage": crate::jumpjet::runtime::storage,
    },
    imports: {
        default: async
    },
    exports: {
        default: async
    }
});
