use wit_bindgen::generate;

use crate::exports::jumpjet::runtime::guest::Guest;
use crate::jumpjet::runtime::debug::log;

generate!({
    world: "game",
    path: ".jumpjet/wit/runtime",
    generate_all
});
export!(Game);

struct Game;

impl Guest for Game {
    fn init() -> Result<(), String> {
        log("init");
        Ok(())
    }

    fn update(time: f64, delta_time: f64) {
        log(&format!("update: {:?}", time));
    }

    fn render(time: f64, delta_time: f64) {
        log(&format!("render: {:?}", time));
    }
}
