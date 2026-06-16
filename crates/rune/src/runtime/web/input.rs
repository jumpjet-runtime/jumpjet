//! `jumpjet:runtime/input` host import as Rust `#[wasm_bindgen]` classes.
//!
//! Keyboard and mouse state is tracked via DOM event listeners installed by
//! `input_install` (called from the run loop bootstrap); `input_end_frame` clears
//! the per-frame "just pressed" set. Gamepads use **gilrs** (its wasm backend
//! wraps the Gamepad API) for parity with native — the WIT `gamepad-button`/
//! `gamepad-axis` enums are gilrs's own naming, so they map directly. `input_poll`
//! drains gilrs events each frame to refresh state.

use std::cell::RefCell;
use std::collections::HashSet;

use gilrs::{Axis, Button, Gilrs, GamepadId};
use js_sys::{Array, Object, Reflect};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

#[derive(Default)]
struct InputState {
    keys_down: HashSet<String>,
    keys_just: HashSet<String>,
    mouse_buttons: u32,
}

thread_local! {
    static INPUT: RefCell<InputState> = RefCell::new(InputState::default());
    static INSTALLED: RefCell<bool> = RefCell::new(false);
    static GILRS: RefCell<Option<Gilrs>> = RefCell::new(None);
}

fn get(o: &JsValue, k: &str) -> JsValue {
    Reflect::get(o, &JsValue::from_str(k)).unwrap_or(JsValue::UNDEFINED)
}
fn set(o: &Object, k: &str, v: JsValue) {
    let _ = Reflect::set(o, &JsValue::from_str(k), &v);
}

/// `keyboard-key` named-variant tag <-> browser `KeyboardEvent.key`.
const KEYMAP: &[(&str, &str)] = &[
    ("arrow-up", "ArrowUp"), ("arrow-down", "ArrowDown"), ("arrow-left", "ArrowLeft"),
    ("arrow-right", "ArrowRight"), ("enter", "Enter"), ("tab", "Tab"), ("space", " "),
    ("escape", "Escape"), ("backspace", "Backspace"), ("delete", "Delete"), ("insert", "Insert"),
    ("home", "Home"), ("end", "End"), ("page-up", "PageUp"), ("page-down", "PageDown"),
    ("caps-lock", "CapsLock"), ("num-lock", "NumLock"), ("scroll-lock", "ScrollLock"),
    ("shift", "Shift"), ("control", "Control"), ("alt", "Alt"), ("super", "Meta"),
    ("context-menu", "ContextMenu"), ("fn", "Fn"),
    ("f1", "F1"), ("f2", "F2"), ("f3", "F3"), ("f4", "F4"), ("f5", "F5"), ("f6", "F6"),
    ("f7", "F7"), ("f8", "F8"), ("f9", "F9"), ("f10", "F10"), ("f11", "F11"), ("f12", "F12"),
];
/// Variants whose WIT payload is a `key-location`.
const LOCATION_TAGS: &[&str] = &["shift", "control", "alt", "super"];

fn tag_to_event_key(tag: &str) -> Option<&'static str> {
    KEYMAP.iter().find(|(t, _)| *t == tag).map(|(_, k)| *k)
}
fn event_key_to_variant(key: &str) -> Option<JsValue> {
    if let Some((tag, _)) = KEYMAP.iter().find(|(_, k)| *k == key) {
        let out = Object::new();
        set(&out, "tag", JsValue::from_str(tag));
        if LOCATION_TAGS.contains(tag) {
            let loc = Object::new();
            set(&loc, "tag", JsValue::from_str("standard"));
            set(&out, "val", loc.into());
        }
        Some(out.into())
    } else if key.chars().count() == 1 {
        let out = Object::new();
        set(&out, "tag", JsValue::from_str("character"));
        set(&out, "val", JsValue::from_str(key));
        Some(out.into())
    } else {
        None
    }
}
/// The browser `KeyboardEvent.key`(s) that satisfy a WIT `keyboard-key` query.
fn key_query_matches(key: &JsValue, down: &HashSet<String>) -> bool {
    let tag = get(key, "tag").as_string().unwrap_or_default();
    if tag == "character" {
        get(key, "val").as_string().map(|c| down.contains(&c)).unwrap_or(false)
    } else if let Some(ev) = tag_to_event_key(&tag) {
        down.contains(ev)
    } else {
        false
    }
}

fn add_listener(target: &web_sys::EventTarget, event: &str, cb: Closure<dyn FnMut(JsValue)>) {
    let _ = target.add_event_listener_with_callback(event, cb.as_ref().unchecked_ref());
    cb.forget();
}

/// Installs the DOM event listeners that track keyboard/mouse state. Idempotent.
#[wasm_bindgen(js_name = inputInstall)]
pub fn input_install() {
    if INSTALLED.with(|i| std::mem::replace(&mut *i.borrow_mut(), true)) {
        return;
    }
    let window = match web_sys::window() {
        Some(w) => w,
        None => return,
    };
    let target: web_sys::EventTarget = window.into();

    add_listener(&target, "keydown", Closure::wrap(Box::new(|e: JsValue| {
        if let Some(k) = get(&e, "key").as_string() {
            INPUT.with(|s| {
                let mut s = s.borrow_mut();
                if s.keys_down.insert(k.clone()) {
                    s.keys_just.insert(k);
                }
            });
        }
    }) as Box<dyn FnMut(JsValue)>));

    add_listener(&target, "keyup", Closure::wrap(Box::new(|e: JsValue| {
        if let Some(k) = get(&e, "key").as_string() {
            INPUT.with(|s| { s.borrow_mut().keys_down.remove(&k); });
        }
    }) as Box<dyn FnMut(JsValue)>));

    add_listener(&target, "mousedown", Closure::wrap(Box::new(|_e: JsValue| {
        INPUT.with(|s| { s.borrow_mut().mouse_buttons += 1; });
    }) as Box<dyn FnMut(JsValue)>));

    add_listener(&target, "mouseup", Closure::wrap(Box::new(|_e: JsValue| {
        INPUT.with(|s| {
            let mut s = s.borrow_mut();
            s.mouse_buttons = s.mouse_buttons.saturating_sub(1);
        });
    }) as Box<dyn FnMut(JsValue)>));

    // gilrs (its wasm backend wraps the Gamepad API) drives gamepad input.
    if let Ok(gilrs) = Gilrs::new() {
        GILRS.with(|g| *g.borrow_mut() = Some(gilrs));
    }
}

/// Drains pending gilrs events to refresh gamepad state. Called once per frame.
#[wasm_bindgen(js_name = inputPoll)]
pub fn input_poll() {
    GILRS.with(|g| {
        if let Some(gilrs) = g.borrow_mut().as_mut() {
            while gilrs.next_event().is_some() {}
        }
    });
}

/// Clears the per-frame "just pressed" set. Called by the run loop each frame.
#[wasm_bindgen(js_name = inputEndFrame)]
pub fn input_end_frame() {
    INPUT.with(|s| s.borrow_mut().keys_just.clear());
}

// ---- devices ----

#[wasm_bindgen]
pub struct KeyboardDevice {}
#[wasm_bindgen]
impl KeyboardDevice {
    #[wasm_bindgen(js_name = isPressed)]
    pub fn is_pressed(&self, key: JsValue) -> bool {
        INPUT.with(|s| key_query_matches(&key, &s.borrow().keys_down))
    }
    #[wasm_bindgen(js_name = justPressed)]
    pub fn just_pressed(&self, key: JsValue) -> bool {
        INPUT.with(|s| key_query_matches(&key, &s.borrow().keys_just))
    }
    #[wasm_bindgen(js_name = activeKeys)]
    pub fn active_keys(&self) -> JsValue {
        let out = Array::new();
        INPUT.with(|s| {
            for k in s.borrow().keys_down.iter() {
                if let Some(v) = event_key_to_variant(k) {
                    out.push(&v);
                }
            }
        });
        out.into()
    }
}

#[wasm_bindgen]
pub struct MouseDevice {}
#[wasm_bindgen]
impl MouseDevice {
    // mouse-button only has `unknown`, so this reports whether any button is down.
    #[wasm_bindgen(js_name = isPressed)]
    pub fn is_pressed(&self, _btn: JsValue) -> bool {
        INPUT.with(|s| s.borrow().mouse_buttons > 0)
    }
}

#[wasm_bindgen]
pub struct TouchDevice {}

#[wasm_bindgen]
pub struct GamepadDevice {
    id: GamepadId,
}
#[wasm_bindgen]
impl GamepadDevice {
    pub fn name(&self) -> String {
        GILRS.with(|g| {
            g.borrow().as_ref().map(|gilrs| gilrs.gamepad(self.id).name().to_owned()).unwrap_or_default()
        })
    }
    #[wasm_bindgen(js_name = isPressed)]
    pub fn is_pressed(&self, btn: JsValue) -> bool {
        let b = match button(&btn) { Some(b) => b, None => return false };
        GILRS.with(|g| g.borrow().as_ref().map(|gilrs| gilrs.gamepad(self.id).is_pressed(b)).unwrap_or(false))
    }
    pub fn value(&self, axis: JsValue) -> f32 {
        let a = match self::axis(&axis) { Some(a) => a, None => return 0.0 };
        GILRS.with(|g| g.borrow().as_ref().map(|gilrs| gilrs.gamepad(self.id).value(a)).unwrap_or(0.0))
    }
    #[wasm_bindgen(js_name = buttonData)]
    pub fn button_data(&self, btn: JsValue) -> JsValue {
        let b = match button(&btn) { Some(b) => b, None => return JsValue::UNDEFINED };
        GILRS.with(|g| {
            let g = g.borrow();
            let gilrs = match g.as_ref() { Some(g) => g, None => return JsValue::UNDEFINED };
            let pad = gilrs.gamepad(self.id);
            match pad.button_code(b).and_then(|c| pad.state().button_data(c)) {
                Some(d) => {
                    let out = Object::new();
                    set(&out, "isPressed", JsValue::from_bool(d.is_pressed()));
                    set(&out, "value", JsValue::from_f64(d.value() as f64));
                    set(&out, "isRepeating", JsValue::from_bool(d.is_repeating()));
                    set(&out, "counter", JsValue::from_f64(d.counter() as f64));
                    out.into()
                }
                None => JsValue::UNDEFINED,
            }
        })
    }
    #[wasm_bindgen(js_name = axisData)]
    pub fn axis_data(&self, axis: JsValue) -> JsValue {
        let a = match self::axis(&axis) { Some(a) => a, None => return JsValue::UNDEFINED };
        GILRS.with(|g| {
            let g = g.borrow();
            let gilrs = match g.as_ref() { Some(g) => g, None => return JsValue::UNDEFINED };
            let pad = gilrs.gamepad(self.id);
            match pad.axis_code(a).and_then(|c| pad.state().axis_data(c)) {
                Some(d) => {
                    let out = Object::new();
                    set(&out, "value", JsValue::from_f64(d.value() as f64));
                    set(&out, "counter", JsValue::from_f64(d.counter() as f64));
                    out.into()
                }
                None => JsValue::UNDEFINED,
            }
        })
    }
}

/// WIT `gamepad-button` -> `gilrs::Button` (identical naming).
fn button(btn: &JsValue) -> Option<Button> {
    Some(match btn.as_string().as_deref()? {
        "south" => Button::South, "east" => Button::East, "north" => Button::North, "west" => Button::West,
        "c" => Button::C, "z" => Button::Z,
        "left-trigger" => Button::LeftTrigger, "left-trigger2" => Button::LeftTrigger2,
        "right-trigger" => Button::RightTrigger, "right-trigger2" => Button::RightTrigger2,
        "select" => Button::Select, "start" => Button::Start, "mode" => Button::Mode,
        "left-thumb" => Button::LeftThumb, "right-thumb" => Button::RightThumb,
        "dpad-up" => Button::DPadUp, "dpad-down" => Button::DPadDown,
        "dpad-left" => Button::DPadLeft, "dpad-right" => Button::DPadRight,
        _ => return None,
    })
}
/// WIT `gamepad-axis` -> `gilrs::Axis` (identical naming).
fn axis(axis: &JsValue) -> Option<Axis> {
    Some(match axis.as_string().as_deref()? {
        "left-stick-x" => Axis::LeftStickX, "left-stick-y" => Axis::LeftStickY, "left-z" => Axis::LeftZ,
        "right-stick-x" => Axis::RightStickX, "right-stick-y" => Axis::RightStickY, "right-z" => Axis::RightZ,
        "dpad-x" => Axis::DPadX, "dpad-y" => Axis::DPadY,
        _ => return None,
    })
}

// ---- free functions ----

#[wasm_bindgen(js_name = inputKeyboard)]
pub fn keyboard() -> Option<KeyboardDevice> {
    Some(KeyboardDevice {})
}
#[wasm_bindgen(js_name = inputMouse)]
pub fn mouse() -> Option<MouseDevice> {
    Some(MouseDevice {})
}
#[wasm_bindgen(js_name = inputTouch)]
pub fn touch() -> Option<TouchDevice> {
    None
}
#[wasm_bindgen(js_name = inputGamepad)]
pub fn gamepad(id: u32) -> Option<GamepadDevice> {
    GILRS.with(|g| {
        let g = g.borrow();
        let gilrs = g.as_ref()?;
        // Map the guest's numeric index to the id-th connected gamepad.
        let (gid, _) = gilrs.gamepads().nth(id as usize)?;
        Some(GamepadDevice { id: gid })
    })
}
