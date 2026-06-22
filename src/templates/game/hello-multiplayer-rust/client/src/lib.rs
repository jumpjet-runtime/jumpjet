//! The client: renders and runs on each player's machine. It targets the `game`
//! world (the full client-runtime profile), so it can use gpu/window/input/audio.
//! Shared simulation lives in the `common` crate, which the server runs too.

use std::sync::atomic::{AtomicU64, Ordering};

use wit_bindgen::generate;

use crate::exports::jumpjet::runtime::game::{Game, Guest, GuestGame};
use crate::jumpjet::runtime::debug::log;

generate!({
    world: "game",
    path: "../.jumpjet/wit",
    generate_all
});
export!(Component);

static TICK: AtomicU64 = AtomicU64::new(0);

struct Component;

impl Guest for Component {
    type Game = MyGame;
}

struct MyGame;

impl GuestGame for MyGame {
    fn init() -> Result<Game, String> {
        log(&format!("client init: {}", common::name()));
        Ok(Game::new(MyGame))
    }

    fn update(&self, _time: f64, _delta_time: f64) {
        // Advance the shared simulation — the exact code the server runs.
        let next = common::step(TICK.load(Ordering::Relaxed));
        TICK.store(next, Ordering::Relaxed);
    }

    fn render(&self, _time: f64, _alpha: f64) {
        log(&format!("client render @ tick {}", TICK.load(Ordering::Relaxed)));
    }
}
