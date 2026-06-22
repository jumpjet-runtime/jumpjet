use wit_bindgen::generate;

use crate::exports::jumpjet::runtime::game::{Game, Guest, GuestGame};
use crate::jumpjet::runtime::debug::log;

generate!({
    world: "game",
    path: ".jumpjet/wit",
    generate_all
});
export!(Component);

struct Component;

impl Guest for Component {
    type Game = MyGame;
}

struct MyGame;

impl GuestGame for MyGame {
    fn init() -> Result<Game, String> {
        log("init");
        Ok(Game::new(MyGame))
    }

    fn update(&self, time: f64, delta_time: f64) {
        log(&format!("update: {:?}", time));
    }

    fn render(&self, time: f64, alpha: f64) {
        log(&format!("render: {:?}", time));
    }
}
