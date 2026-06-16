//! `jumpjet:runtime/debug` host import for the web (jco) build.

use js_sys::{Object, Reflect};
use wasm_bindgen::prelude::*;

use crate::runtime::web::state::SharedState;

/// Builds the JS import object passed to the jco-instantiated guest for the
/// `jumpjet:runtime/debug` interface. Each method is a closure that forwards to the
/// browser console. (Logging needs no runtime state.)
pub fn export(_state: SharedState) -> JsValue {
    let obj = Object::new();

    let log = Closure::<dyn Fn(String)>::new(|msg: String| {
        web_sys::console::log_1(&JsValue::from_str(&msg));
    });
    let warn = Closure::<dyn Fn(String)>::new(|msg: String| {
        web_sys::console::warn_1(&JsValue::from_str(&msg));
    });
    let error = Closure::<dyn Fn(String)>::new(|msg: String| {
        web_sys::console::error_1(&JsValue::from_str(&msg));
    });

    let _ = Reflect::set(&obj, &JsValue::from_str("log"), log.as_ref().unchecked_ref());
    let _ = Reflect::set(&obj, &JsValue::from_str("warn"), warn.as_ref().unchecked_ref());
    let _ = Reflect::set(&obj, &JsValue::from_str("error"), error.as_ref().unchecked_ref());

    // These closures must outlive this call (the guest may invoke them every
    // frame), so leak them into the JS world.
    log.forget();
    warn.forget();
    error.forget();

    obj.into()
}
