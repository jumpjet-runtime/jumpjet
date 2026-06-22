//! The authoritative headless server. It targets the `server` world (the trimmed
//! server-runtime profile: no gpu/window/input/audio), so it only ticks `update`,
//! never renders. Runs via `jumpjet run --server`. Shared simulation lives in the
//! `common` crate, which the client runs too.

use std::sync::atomic::{AtomicU64, Ordering};

use wit_bindgen::generate;

use crate::exports::jumpjet::runtime::server::Guest;
use crate::jumpjet::runtime::debug::log;

generate!({
    world: "server",
    path: "../.jumpjet/wit",
    generate_all
});
export!(Server);

static TICK: AtomicU64 = AtomicU64::new(0);

struct Server;

impl Guest for Server {
    fn init() -> Result<(), String> {
        log(&format!("server init: {}", common::name()));
        Ok(())
    }

    fn update(time: f64, _delta_time: f64) {
        // The authoritative simulation step — the exact code the client predicts.
        let next = common::step(TICK.load(Ordering::Relaxed));
        TICK.store(next, Ordering::Relaxed);
        log(&format!("server tick {} @ {:?}", next, time));
    }
}
