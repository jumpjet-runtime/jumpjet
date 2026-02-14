use std::path::PathBuf;

use anyhow::Result;
use cpal::traits::HostTrait;
use libtest_mimic::{Arguments, Trial};
use pollster;

pub use common::*;

pub use native::web::RuneRuntimeState;

use winit::{
    application::ApplicationHandler,
    error::EventLoopError,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop, EventLoopBuilder},
    raw_window_handle::{HasDisplayHandle, HasWindowHandle},
    window::{Window, WindowAttributes},
};

#[cfg(target_arch = "wasm32")]
use winit::platform::web::WindowExtWebSys;

use crate::runtime::web::run;

use crate::{game::Game};

struct App {
    input_path: PathBuf,
    binary: Vec<u8>,
    window: Option<Window>,
    game: Option<Game>,
    start_time: Option<std::time::Instant>,
    last_update: Option<std::time::Instant>,
    last_render_update: Option<std::time::Instant>,
    accumulator: std::time::Duration,
}

impl ApplicationHandler<GameEvent> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let window_attributes = WindowAttributes::default()
                .with_title("Game");

            let window = event_loop.create_window(window_attributes).unwrap();

            // Web-specific: insert canvas into DOM
            #[cfg(target_arch = "wasm32")]
            {
                use wasm_bindgen::JsCast;
                let canvas = window.canvas();
                let window_web = web_sys::window().expect("no global `window` exists");
                let document = window_web.document().expect("should have a document on window");
                let body = document.body().expect("document should have a body");
                body.append_child(&web_sys::Element::from(canvas)).expect("couldn't append canvas to document body");
            }

            // Initialize the game with the window
            let instance = wgpu_core::global::Global::new(
                "webgpu",
                &wgpu_types::InstanceDescriptor {
                    backends: wgpu_types::Backends::all(),
                    flags: wgpu_types::InstanceFlags::from_build_config(),
                    ..Default::default()
                },
            );
            let surface_id = unsafe {
                instance
                    .instance_create_surface(
                        window.display_handle().unwrap().into(),
                        window.window_handle().unwrap().into(),
                        None,
                    )
                    .unwrap()
            };
            let adapter_id = instance
                .request_adapter(
                    &Default::default(),
                    wgpu_types::Backends::all(),
                    None
                )
                .unwrap();

            let adapter_limits = instance
                .adapter_limits(adapter_id);

            // Create the logical device and command queue
            let (device_id, queue_id) = instance.adapter_request_device(
                adapter_id,
                &wgpu_types::DeviceDescriptor {
                    label: None,
                    required_features: wgpu_types::Features::empty(),
                    required_limits:
                        wgpu_types::Limits::downlevel_webgl2_defaults().using_resolution(adapter_limits),
                    memory_hints: wgpu_types::MemoryHints::default(),
                },
                None,
                None,
                None,
            ).unwrap();

            let audio_host = cpal::default_host();
            let audio_device = audio_host.default_output_device().unwrap();

            let gilrs = gilrs::Gilrs::new().unwrap();

            let mut game = Game::from_binary(&self.binary).unwrap();
            pollster::block_on(game.init(
                &window,
                self.input_path.clone(),
                audio_device,
                instance,
                surface_id,
                adapter_id,
                device_id,
                queue_id,
                gilrs,
            )).expect("Game didn't initialize");

            let start_time = std::time::Instant::now();
            self.start_time = Some(start_time);
            self.last_update = Some(start_time);
            self.last_render_update = Some(start_time);
            self.accumulator = std::time::Duration::ZERO;

            self.game = Some(game);
            self.window = Some(window);
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: winit::window::WindowId, event: WindowEvent) {
        if self.window.is_none() {
            return;
        }

        let window = self.window.as_ref().unwrap();
        let game = match &mut self.game {
            Some(game) => game,
            None => return,
        };

        match event {
            WindowEvent::Resized(size) => {
                game.resize(size);
            }
            WindowEvent::KeyboardInput { event: key_event, .. } => {
                let generation = game.store.as_ref().unwrap().data().generation;
                let keyboard_state = &mut game.store.as_mut().unwrap().data_mut().keyboard_state;

                if !key_event.state.is_pressed() || key_event.repeat {
                    keyboard_state.active_keys.retain(|key| {
                        !(key.1.eq(&key_event.logical_key) && key.2.eq(&key_event.location))
                    });

                    if key_event.repeat {
                        keyboard_state.active_keys.push((
                            generation,
                            key_event.logical_key,
                            key_event.location,
                        ));
                    }
                } else if !keyboard_state
                    .active_keys
                    .iter()
                    .any(|k| k.1.eq(&key_event.logical_key) && k.2.eq(&key_event.location))
                {
                    keyboard_state.active_keys.push((
                        generation,
                        key_event.logical_key,
                        key_event.location,
                    ));
                }
            }
            WindowEvent::RedrawRequested => {
                let now = std::time::Instant::now();
                let start_time = self.start_time.unwrap();
                let epoch_time = now - start_time;
                let fixed_time_step = std::time::Duration::from_millis(33);
                let alpha = self.accumulator.as_secs_f64() / fixed_time_step.as_secs_f64();
                pollster::block_on(game.render(epoch_time, alpha)).unwrap();
            }
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() || self.game.is_none() {
            return;
        }

        let window = self.window.as_ref().unwrap();
        let game = self.game.as_mut().unwrap();

        let now = std::time::Instant::now();
        let last_update = self.last_update.unwrap();
        let delta_time = now - last_update;
        self.last_update = Some(now);

        self.accumulator += delta_time;

        // Fixed timestep: 30 FPS logic
        let fixed_time_step = std::time::Duration::from_millis(33);

        // Limit accumulator to avoid spiral of death
        if self.accumulator > std::time::Duration::from_millis(200) {
            self.accumulator = std::time::Duration::from_millis(200);
        }

        while self.accumulator >= fixed_time_step {
            self.accumulator -= fixed_time_step;

            // start gamepad handling
            let generation = game.store.as_ref().unwrap().data().generation;
            let game_store = game.store.as_mut().unwrap();
            let gilrs_event = { game_store.data_mut().gilrs.next_event() };
            let gamepad_state = &mut game_store.data_mut().gamepad_state;

            while let Some(gilrs::Event { id, event, .. }) = gilrs_event {
                match event {
                    gilrs::EventType::ButtonPressed(button, _) => {
                         if !gamepad_state.active_buttons.iter().any(|b| b.1 == id && b.2 == button) {
                             gamepad_state.active_buttons.push((*generation, id, button, false));
                         }
                    }
                    gilrs::EventType::ButtonRepeated(button, _) => {
                         if let Some(idx) = gamepad_state.active_buttons.iter().position(|b| b.1 == id && b.2 == button) {
                             gamepad_state.active_buttons[idx].3 = true;
                         }
                    }
                    gilrs::EventType::ButtonReleased(button, _) => {
                         gamepad_state.active_buttons.retain(|b| !(b.1 == id && b.2 == button));
                    }
                    _ => {}
                }
            }
            // end gamepad handling

            let generation = &mut game.store.as_mut().unwrap().data_mut().generation;
            *generation = *generation + 1;

            let epoch_time = now - self.start_time.unwrap();

            pollster::block_on(game.update(epoch_time, fixed_time_step)).unwrap();
        }

        // Logic to cap render rate at 60Hz
        let render_frame_time = std::time::Duration::from_millis(1000 / 60);
        let last_render_update = self.last_render_update.unwrap();
        
        if now - last_render_update >= render_frame_time {
            window.request_redraw();
            // Note: last_render_update is updated in processing RedrawRequested
        }

        event_loop.set_control_flow(ControlFlow::Poll);
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, _event: GameEvent) {
        // Handle user events
    }
}

async fn run_loop(
    input_path: PathBuf,
    binary: Vec<u8>,
) -> Result<(), EventLoopError> {
    let event_loop = EventLoop::<GameEvent>::with_user_event().build().unwrap();
    
    let mut app = App {
        input_path,
        binary,
        window: None,
        game: None,
        start_time: None,
        last_update: None,
        last_render_update: None,
        accumulator: std::time::Duration::ZERO,
    };

    event_loop.run_app(&mut app)
}

pub fn run(input_path: PathBuf, binary: Vec<u8>) {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init_with_level(log::Level::Info).expect("failed to initialize logger");
    pollster::block_on(run_loop(input_path, binary)).ok();
}
