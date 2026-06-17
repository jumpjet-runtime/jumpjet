use wit_bindgen::generate;

use crate::exports::{{ namespace_snake }}::{{ package_snake }}::api::Guest;
use crate::jumpjet::runtime::debug::log;

generate!({
    world: "lib",
    path: "wit",
    generate_all
});
export!(Package);

struct Package;

impl Guest for Package {
    fn greet(name: String) -> String {
        // Packages can use Jumpjet's runtime APIs, just like games.
        log(&format!("[{{ name }}] greeting {name}"));
        format!("Hello, {name}!")
    }
}
