use js_sys::Object;
use wasm_bindgen::prelude::*;

use crate::RuneRuntimeState;

static EXPORTS: OnceCell<Object> = OnceCell::new();

pub fn export() -> JsValue {
    let debug = Object::new();
    
    Reflect::set(&debug, &JsValue::from_str("log"), &Closure::wrap(Box::new(log) as Box<dyn FnMut(String)>).as_ref())?;
    Reflect::set(&debug, &JsValue::from_str("warn"), &Closure::wrap(Box::new(warn) as Box<dyn FnMut(String)>).as_ref())?;
    Reflect::set(&debug, &JsValue::from_str("error"), &Closure::wrap(Box::new(error) as Box<dyn FnMut(String)>).as_ref())?;

    debug
}

#[wasm_bindgen]
pub fn log(msg: String) {
    web_sys::console::log_1(&JsValue::from_str(&msg));
}

#[wasm_bindgen]
pub fn warn(msg: String) {
    web_sys::console::warn_1(&JsValue::from_str(&msg));
}

#[wasm_bindgen]
pub fn error(msg: String) {
    web_sys::console::error_1(&JsValue::from_str(&msg));
}

