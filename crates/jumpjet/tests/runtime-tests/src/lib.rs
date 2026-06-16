use wasm_bindgen::prelude::*;

wit_bindgen::generate!({
    world: "jumpjet:tests/tests",
    path: [
        "../../wit/runtime",
        "../../wit/tests",
    ],
    generate_all
});

use crate::exports::jumpjet::tests::guest::Guest;
use crate::jumpjet::runtime::debug::log;
// use crate::exports::jumpjet::tests::guest::

struct RuntimeTests;

impl Guest for RuntimeTests {
    fn register() -> Vec<String> {
        [
            "example_test"
        ].iter().map(|&s| s.to_owned()).collect()
    }
}

#[wasm_bindgen]
pub fn example_test() {
    log("example test");
}

export!(RuntimeTests);
