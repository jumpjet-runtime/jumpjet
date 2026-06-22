//! The client: renders and runs on each player's machine. It targets the `game`
//! world (the full client-runtime profile), so it can use gpu/window/input/audio.
//! Shared simulation lives in the `common` crate, which the server runs too.

use std::sync::atomic::{AtomicU64, Ordering};

use wit_bindgen::generate;

use crate::exports::jumpjet::runtime::client::Guest;
use crate::jumpjet::runtime::debug::log;

generate!({
    world: "game",
    path: "../.jumpjet/wit",
    generate_all
});
export!(Game);

static TICK: AtomicU64 = AtomicU64::new(0);

struct Game;

impl Guest for Game {
    fn init() -> Result<(), String> {
        log(&format!("client init: {}", common::name()));
        Ok(())
    }

    fn update(_time: f64, _delta_time: f64) {
        // Advance the shared simulation — the exact code the server runs.
        let next = common::step(TICK.load(Ordering::Relaxed));
        TICK.store(next, Ordering::Relaxed);
    }

    fn render(_time: f64, _alpha: f64) {
        log(&format!("client render @ tick {}", TICK.load(Ordering::Relaxed)));
    }
}
