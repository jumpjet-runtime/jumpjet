//! `jumpjet:runtime/window` host import for the web (jco) build.
//!
//! jco import shape (validated): `{ dimensions: () => [width, height] }`.

use js_sys::{Array, Object, Reflect};
use wasm_bindgen::prelude::*;

use crate::runtime::web::state::SharedState;

/// Builds the JS import object for the `jumpjet:runtime/window` interface.
pub fn export(state: SharedState) -> JsValue {
    let obj = Object::new();

    let dims_state = state.clone();
    let dimensions = Closure::<dyn Fn() -> Array>::new(move || {
        let s = dims_state.borrow();
        let arr = Array::new();
        arr.push(&JsValue::from_f64(s.window_size.width as f64));
        arr.push(&JsValue::from_f64(s.window_size.height as f64));
        arr
    });
    let _ = Reflect::set(
        &obj,
        &JsValue::from_str("dimensions"),
        dimensions.as_ref().unchecked_ref(),
    );
    dimensions.forget();

    obj.into()
}
