use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use web_time::Instant;
use winit::dpi::PhysicalSize;

use crate::host::Game;
use crate::runtime::web::state::{JumpjetRuntimeState, SharedState};

/// Fixed logic timestep (~30 Hz), matching the native run loop.
const FIXED_TIMESTEP: Duration = Duration::from_millis(33);
/// Cap on accumulated time to avoid the "spiral of death" after a stall.
const MAX_ACCUMULATOR: Duration = Duration::from_millis(200);

/// Entry point for the web runtime, invoked by the `jumpjet-host` `web` cdylib after
/// the harness has installed `window.jco`.
///
/// The browser main thread cannot block, so unlike native (`pollster::block_on`
/// + winit `run_app`) the whole flow is async: bootstrap the canvas + guest, then
/// drive frames from `requestAnimationFrame`. The guest is transpiled at build
/// time, so no wasm binary is passed at runtime.
pub fn run() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    let _ = console_log::init_with_level(log::Level::Info);

    spawn_local(async move {
        if let Err(e) = bootstrap().await {
            web_sys::console::error_1(&e);
        }
    });
}

/// Creates the canvas, instantiates the guest via jco, runs `init`, and starts
/// the frame loop.
async fn bootstrap() -> Result<(), JsValue> {
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("no global window"))?;
    let document = window.document().ok_or_else(|| JsValue::from_str("no document"))?;
    let body = document.body().ok_or_else(|| JsValue::from_str("no document body"))?;

    let canvas = document
        .create_element("canvas")?
        .dyn_into::<web_sys::HtmlCanvasElement>()?;
    canvas.set_id("jumpjet-canvas");
    // Back the canvas with the full viewport at device resolution: the drawing
    // buffer is sized in physical pixels (CSS pixels * devicePixelRatio) for crisp
    // rendering on HiDPI displays, while CSS (see the embedded index.html) stretches
    // the element across the viewport. The guest sees this physical size via
    // `window::dimensions()`, so its projection fills the surface exactly.
    let dpr = window.device_pixel_ratio().max(1.0);
    let css_width = window.inner_width()?.as_f64().unwrap_or(800.0);
    let css_height = window.inner_height()?.as_f64().unwrap_or(600.0);
    let width = (css_width * dpr).max(1.0) as u32;
    let height = (css_height * dpr).max(1.0) as u32;
    canvas.set_width(width);
    canvas.set_height(height);
    body.append_child(&canvas)?;
    let size = PhysicalSize::new(width, height);

    let state: SharedState = Rc::new(RefCell::new(JumpjetRuntimeState::new(size)));

    // Install keyboard/mouse DOM listeners before the guest runs.
    crate::runtime::web::input::input_install();

    let mut game = Game::instantiate(state).await?;
    game.init()?;

    start_frame_loop(game);
    Ok(())
}

/// Drives `update`/`render` from `requestAnimationFrame` using the standard
/// recursive-closure pattern. All guest calls are synchronous (the WIT exports
/// are sync), so no awaiting happens inside the frame callback.
fn start_frame_loop(game: Game) {
    let game = Rc::new(RefCell::new(game));

    let callback: Rc<RefCell<Option<Closure<dyn FnMut()>>>> = Rc::new(RefCell::new(None));
    let scheduler = callback.clone();

    let start = Instant::now();
    let mut last_update = start;
    let mut accumulator = Duration::ZERO;

    *callback.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        let now = Instant::now();
        accumulator += now.duration_since(last_update);
        last_update = now;
        if accumulator > MAX_ACCUMULATOR {
            accumulator = MAX_ACCUMULATOR;
        }

        // Borrow the game only for the duration of the frame. The host-import
        // closures borrow `state` (a separate RefCell), never `game`, so there
        // is no re-entrant double-borrow.
        let mut game = game.borrow_mut();

        // Refresh gamepad state from gilrs at the start of the frame.
        crate::runtime::web::input::input_poll();

        while accumulator >= FIXED_TIMESTEP {
            accumulator -= FIXED_TIMESTEP;
            let epoch = now.duration_since(start);
            if let Err(e) = game.update(epoch, FIXED_TIMESTEP) {
                web_sys::console::error_1(&e);
            }
            // Clear per-frame "just pressed" input after each logic tick.
            crate::runtime::web::input::input_end_frame();
        }

        let epoch = now.duration_since(start);
        let alpha = accumulator.as_secs_f64() / FIXED_TIMESTEP.as_secs_f64();
        if let Err(e) = game.render(epoch, alpha) {
            web_sys::console::error_1(&e);
        }

        drop(game);
        request_animation_frame(scheduler.borrow().as_ref().unwrap());
    }) as Box<dyn FnMut()>));

    request_animation_frame(callback.borrow().as_ref().unwrap());
}

fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    web_sys::window()
        .expect("no global window")
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("request_animation_frame failed");
}
