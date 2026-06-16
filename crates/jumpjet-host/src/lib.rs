#![cfg(target_arch = "wasm32")]

use jumpjet::runtime;
use wasm_bindgen::prelude::*;

/// Web entry point, called by the HTML harness after it has installed
/// `window.jco`. The guest is transpiled at build time (it is loaded by the
/// harness, not fetched here), so this just starts the web runtime, which
/// instantiates the guest via jco and drives the frame loop.
#[wasm_bindgen]
pub fn run() {
    runtime::run();
}
