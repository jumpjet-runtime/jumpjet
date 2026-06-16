use std::cell::RefCell;
use std::rc::Rc;

use winit::dpi::PhysicalSize;

/// Shared, interior-mutable handle to the web runtime state.
///
/// The frame loop (`runtime/web/run.rs`) and the `debug`/`window` host-import
/// closures hold a clone of this `Rc`. Because the browser is single-threaded and
/// host calls are synchronous and non-overlapping, `RefCell` borrows are always
/// short-lived: the frame loop must never hold a borrow across a call into guest
/// code (which re-enters the host closures), and each closure borrows only for the
/// duration of its call.
pub type SharedState = Rc<RefCell<JumpjetRuntimeState>>;

/// Web runtime state. Unlike native (`runtime/native/state.rs`), the web build does
/// not own a wasmtime `ResourceTable`/`WasiCtx` or wgpu_core ids — the guest runs in
/// the browser via jco and resource interfaces (gpu/audio/input/storage) are
/// self-contained `#[wasm_bindgen]` classes. This holds only the canvas size read
/// by the `window` interface.
pub struct JumpjetRuntimeState {
    pub window_size: PhysicalSize<u32>,
}

impl JumpjetRuntimeState {
    pub fn new(window_size: PhysicalSize<u32>) -> Self {
        Self { window_size }
    }
}
