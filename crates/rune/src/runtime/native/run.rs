use std::path::PathBuf;

use anyhow::Result;
use cpal::traits::HostTrait;
use libtest_mimic::{Arguments, Trial};
use pollster;

use crate::host::Game;
pub use crate::runtime::common::*;

#[cfg(any(target_os = "macos", target_os = "windows"))]
pub use super::state::RuneRuntimeState;

use winit::{
    application::ApplicationHandler,
    error::EventLoopError,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop, EventLoopBuilder},
    raw_window_handle::{HasDisplayHandle, HasWindowHandle},
    window::{Window, WindowAttributes},
};

#[cfg(target_os = "macos")]
use winit::platform::macos::WindowAttributesExtMacOS;

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
            let mut window_attributes = WindowAttributes::default()
                .with_title("Game");

            #[cfg(target_os = "macos")]
            {
                window_attributes = window_attributes.with_titlebar_hidden(true);
            }

            let window = event_loop.create_window(window_attributes).unwrap();

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
                    // Make sure we use the texture resolution limits from the adapter, so we can support images the size of the swapchain.
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
                             gamepad_state.active_buttons.push((generation, id, button, false));
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

            // Generation is logical frame count
            let generation = &mut game.store.as_mut().unwrap().data_mut().generation;
            *generation = *generation + 1;

            // TODO: Track total logic time separately from wall clock?
            // For now, epoch can be wall clock, but strictly it should be logic time.
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

        // Don't wait, run as fast as possible (vsync will handle partial throttling if enabled, or we burn CPU)
        // event_loop.set_control_flow(ControlFlow::Poll); 
        // Or wait a tiny bit to be nice to CPU if vsync is off? 
        // ControlFlow::Poll is standard for games.
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
    pollster::block_on(run_loop(input_path, binary)).ok();
}


pub async fn test(_input_path: PathBuf, _binary: Vec<u8>) {
    // Parse command line arguments
    let args = Arguments::from_args();

    // Create a list of tests and/or benchmarks (in this case: two dummy tests).
    let tests = vec![
        Trial::test("succeeding_test", move || Ok(())),
        Trial::test("failing_test", move || Err("Woops".into())),
    ];

    // TODO: Setup Rune host in test mode

    // TODO: Guest implements Rune "test guest", which has a setup_tests() function

    // TODO: Guest calls add-test method which registers the test

    // TODO: Registered tests go into tests vec above. Test function invokes the "run_test(name)" method on the guest.
    // Guest invokes the test associated with the name passed to run_test.

    // let event_loop = EventLoopBuilder::<GameEvent>::with_user_event()
    //     .build()
    //     .unwrap();

    // let window = WindowBuilder::new()
    //     .with_title("Game")
    //     .with_titlebar_hidden(true)
    //     .build(&event_loop)
    //     .unwrap();

    // let instance = wgpu_core::global::Global::new(
    //     "webgpu",
    //     wgpu_types::InstanceDescriptor {
    //         backends: wgpu_types::Backends::all(),
    //         flags: wgpu_types::InstanceFlags::from_build_config(),
    //         dx12_shader_compiler: wgpu_types::Dx12Compiler::Fxc,
    //         gles_minor_version: wgpu_types::Gles3MinorVersion::default(),
    //     },
    // );
    // let surface_id = unsafe {
    //     instance.instance_create_surface(
    //         window.raw_display_handle().unwrap(),
    //         window.raw_window_handle().unwrap(),
    //         None,
    //     ).unwrap()
    // };
    // let adapter_id = instance
    //     .request_adapter(
    //         &Default::default(),
    //         wgpu_core::instance::AdapterInputs::Mask(wgpu_types::Backends::all(), |_| None),
    //     )
    //     .unwrap();

    // let adapter_limits = instance.adapter_limits::<crate::Backend>(adapter_id).unwrap();

    // // Create the logical device and command queue
    // let (device_id, queue_id) = instance.adapter_request_device::<crate::Backend>(
    //     adapter_id,
    //     &wgpu_types::DeviceDescriptor {
    //         label: None,
    //         required_features: wgpu_types::Features::empty(),
    //         // Make sure we use the texture resolution limits from the adapter, so we can support images the size of the swapchain.
    //         required_limits: wgpu_types::Limits::downlevel_webgl2_defaults()
    //             .using_resolution(adapter_limits),
    //     },
    //     None,
    //     None,
    //     None
    // )
    // .unwrap();

    // let audio_host = cpal::default_host();
    // let audio_device = audio_host.default_output_device().unwrap();

    // let mut gilrs = gilrs::Gilrs::new().unwrap();

    // let mut test = Tests::from_binary(&binary).unwrap();
    // test.init(&window, input_path, audio_device, instance, surface_id, adapter_id, device_id, queue_id).await.expect("Tests didn't initialize");

    libtest_mimic::run(&args, tests).exit_code();
}
