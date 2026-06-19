use std::{path::PathBuf, net::TcpListener};
use wasmtime::AsContextMut;

use anyhow::Result;
use cpal::traits::HostTrait;
use libtest_mimic::{Arguments, Trial};
use pollster;

use crate::host::Game;
pub use crate::runtime::common::*;

pub use super::state::JumpjetRuntimeState;

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

/// Hide/show the hardware cursor while the pointer is locked. winit's
/// `set_cursor_visible` hides via cursor-rects, which the system stops
/// re-evaluating once the pointer is locked (the cursor is decoupled and
/// frozen), so on macOS the cursor never disappears. `NSCursor`'s `hide`/
/// `unhide` are app-wide and unaffected by that. The calls are balanced
/// (stack-counted), so the lock loop only ever calls this on a real
/// lock <-> unlock transition. No-op off macOS, where `set_cursor_visible` works.
#[cfg(target_os = "macos")]
fn set_macos_cursor_hidden(hidden: bool) {
    use objc2::{class, msg_send};
    // `+[NSCursor hide]`/`+[NSCursor unhide]` take no arguments and must run on
    // the main thread, which the winit run loop guarantees.
    unsafe {
        if hidden {
            let _: () = msg_send![class!(NSCursor), hide];
        } else {
            let _: () = msg_send![class!(NSCursor), unhide];
        }
    }
}

#[cfg(not(target_os = "macos"))]
fn set_macos_cursor_hidden(_hidden: bool) {}

struct App {
    input_path: PathBuf,
    binary: Vec<u8>,
    window: Option<Window>,
    game: Option<Game>,
    start_time: Option<std::time::Instant>,
    last_update: Option<std::time::Instant>,
    last_render_update: Option<std::time::Instant>,
    accumulator: std::time::Duration,
    debug: bool,
    gdb_server: Option<TcpListener>,
    dap_server: Option<TcpListener>,
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

            // iOS forbids JIT, so `self.binary` is a Pulley `.cwasm` loaded
            // interpreted; every other native target JIT-compiles the component.
            #[cfg(target_os = "ios")]
            let mut game = Game::from_cwasm(&self.binary).unwrap();
            #[cfg(not(target_os = "ios"))]
            let mut game = Game::from_binary(&self.binary, self.debug).unwrap();

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
            WindowEvent::CursorMoved { position, .. } => {
                let mouse_state = &mut game.store.as_mut().unwrap().data_mut().mouse_state;
                mouse_state.x = position.x as f32;
                mouse_state.y = position.y as f32;
            }
            WindowEvent::MouseInput { state, button, .. } => {
                let mouse_state = &mut game.store.as_mut().unwrap().data_mut().mouse_state;
                mouse_state.buttons.retain(|b| *b != button);
                if state.is_pressed() {
                    mouse_state.buttons.push(button);
                }
            }
            WindowEvent::KeyboardInput { event: key_event, .. } => {
                let generation = game.store.as_ref().unwrap().data().generation;
                let keyboard_state = &mut game.store.as_mut().unwrap().data_mut().keyboard_state;

                // Match on `physical_key` so a key clears on release even when
                // the modifier state differs from when it was pressed (the
                // logical key changes with Shift, the physical key does not).
                if !key_event.state.is_pressed() || key_event.repeat {
                    keyboard_state
                        .active_keys
                        .retain(|key| key.1 != key_event.physical_key);

                    if key_event.repeat {
                        keyboard_state.active_keys.push((
                            generation,
                            key_event.physical_key,
                            key_event.logical_key,
                            key_event.location,
                        ));
                    }
                } else if !keyboard_state
                    .active_keys
                    .iter()
                    .any(|k| k.1 == key_event.physical_key)
                {
                    keyboard_state.active_keys.push((
                        generation,
                        key_event.physical_key,
                        key_event.logical_key,
                        key_event.location,
                    ));
                }
            }
            WindowEvent::Focused(false) => {
                // winit stops delivering key events once the window loses focus
                // (e.g. alt-tab), so any keys held at that point would never see
                // a release. Flush them to avoid stranding.
                let keyboard_state = &mut game.store.as_mut().unwrap().data_mut().keyboard_state;
                keyboard_state.active_keys.clear();
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

    fn device_event(&mut self, _event_loop: &ActiveEventLoop, _device_id: winit::event::DeviceId, event: winit::event::DeviceEvent) {
        // Raw mouse motion keeps reporting while the cursor is grabbed/locked,
        // so it (not CursorMoved) is the source for the guest's `delta`.
        if let winit::event::DeviceEvent::MouseMotion { delta } = event {
            if let Some(game) = &mut self.game {
                if let Some(store) = game.store.as_mut() {
                    let mouse_state = &mut store.data_mut().mouse_state;
                    mouse_state.dx += delta.0 as f32;
                    mouse_state.dy += delta.1 as f32;
                }
            }
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() || self.game.is_none() {
            return;
        }

        let window = self.window.as_ref().unwrap();
        let game = self.game.as_mut().unwrap();

        // Apply any pending pointer-lock request from the guest. `Locked` gives
        // true relative-only motion where supported; fall back to `Confined`
        // (e.g. on platforms without grab-lock, like X11) so the cursor at least
        // stays inside the window.
        {
            let mouse_state = &mut game.store.as_mut().unwrap().data_mut().mouse_state;
            if let Some(lock) = mouse_state.lock_request.take() {
                if lock {
                    let grabbed = window
                        .set_cursor_grab(winit::window::CursorGrabMode::Locked)
                        .or_else(|_| window.set_cursor_grab(winit::window::CursorGrabMode::Confined))
                        .is_ok();
                    if grabbed && !mouse_state.locked {
                        window.set_cursor_visible(false);
                        set_macos_cursor_hidden(true);
                        mouse_state.locked = true;
                    }
                } else if mouse_state.locked {
                    let _ = window.set_cursor_grab(winit::window::CursorGrabMode::None);
                    window.set_cursor_visible(true);
                    set_macos_cursor_hidden(false);
                    mouse_state.locked = false;
                }
            }
        }

        // Debugger connection polling only matters when debugging; skip the
        // socket accepts and mutex/peek entirely on the hot path otherwise.
        if self.debug {
            // Check for new GDB connections
            if let Some(server) = &self.gdb_server {
                if let Ok((stream, _)) = server.accept() {
                    let mut conn = game.gdb_connection.lock().unwrap();
                    *conn = Some(stream);
                }
            }

            // Check for new DAP connections
            if let Some(server) = &self.dap_server {
                if let Ok((stream, addr)) = server.accept() {
                    eprintln!("Accepted new DAP connection from {}", addr);
                    stream.set_nonblocking(true).ok();
                    let mut conn = game.dap_connection.lock().unwrap();
                    *conn = Some(crate::debug::dap::DapConnection::new(stream));
                }
            }

            // Check for incoming DAP data
            let mut dap_needs_handling = false;
            {
                let lock = game.dap_connection.lock().unwrap();
                if let Some(conn) = lock.as_ref() {
                    let mut buf = [0u8; 1];
                    match conn.stream.peek(&mut buf) {
                        Ok(n) if n > 0 => dap_needs_handling = true,
                        _ => {}
                    }
                }
            }

            if dap_needs_handling {
                let mut lock = game.dap_connection.lock().unwrap();
                if let Some(conn) = lock.as_mut() {
                    let store = game.store.as_mut().unwrap();
                    if let Err(e) = crate::debug::dap::handle_dap_event(store.as_context_mut(), conn, None, None, game.binary.clone()) {
                        eprintln!("DAP handler error: {:?}", e);
                    }
                }
            }
        }

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

            while let Some(gilrs::Event { id, event, .. }) = game_store.data_mut().gilrs.next_event() {
                let gamepad_state = &mut game_store.data_mut().gamepad_state;
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

            let update_result = pollster::block_on(game.update(epoch_time, fixed_time_step));
            
            if let Err(e) = update_result {
                let mut handled_by_dap = false;
                {
                    let mut lock = game.dap_connection.lock().unwrap();
                    if let Some(conn) = lock.as_mut() {
                        eprintln!("Game update error (trapped): {:?}. Entering Debugger Loop.", e);
                        let store = game.store.as_mut().unwrap();
                        
                        // We use Breakpoint reason if it looks like a breakpoint? 
                        // Or just Exception.
                        // For now use Exception.
                        let reason = dapts::StoppedEventReason::Exception;
                        
                        let bt = e.downcast_ref::<wasmtime::WasmBacktrace>();
                        
                        if let Err(dap_err) = crate::debug::dap::handle_dap_event(
                            store.as_context_mut(), 
                            conn, 
                            Some(reason),
                            bt,
                            game.binary.clone()
                        ) {
                            eprintln!("Failed to handle DAP event during trap: {:?}", dap_err);
                        }
                        handled_by_dap = true;
                    }
                }
                if !handled_by_dap {
                    panic!("Game update failed: {:?}", e);
                }
            }

            // Reset per-frame mouse movement now that the guest has consumed it.
            let mouse_state = &mut game.store.as_mut().unwrap().data_mut().mouse_state;
            mouse_state.dx = 0.0;
            mouse_state.dy = 0.0;
        }

        // Cap render rate at 60Hz.
        let render_frame_time = std::time::Duration::from_millis(1000 / 60);
        let last_render_update = self.last_render_update.unwrap();

        let mut next_render = last_render_update + render_frame_time;
        if now >= next_render {
            window.request_redraw();
            self.last_render_update = Some(now);
            next_render = now + render_frame_time;
        }

        // Instead of busy-spinning with ControlFlow::Poll (which burns a full
        // core between the 30Hz logic step and 60Hz render), sleep until the
        // next deadline: whichever of the next logic tick or next render is due
        // first.
        let until_next_logic = fixed_time_step.saturating_sub(self.accumulator);
        let next_logic = now + until_next_logic;
        let next_deadline = next_logic.min(next_render);
        event_loop.set_control_flow(ControlFlow::WaitUntil(next_deadline));
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, _event: GameEvent) {
        // Handle user events
    }
}

/// Drives a pre-built event loop. The loop is built by the caller because its
/// construction differs per platform: desktop/iOS use the default builder, while
/// Android must thread the `AndroidApp` through `with_android_app`.
async fn run_loop(
    event_loop: EventLoop<GameEvent>,
    input_path: PathBuf,
    binary: Vec<u8>,
    debug: bool,
) -> Result<(), EventLoopError> {
    let gdb_server = if debug {
        Some(crate::debug::gdb::start_gdb_server(crate::debug::gdb::DEFAULT_GDB_PORT).unwrap())
    } else {
        None
    };

    let dap_server = if debug {
        eprintln!("Starting DAP server on port {}", crate::debug::dap::DEFAULT_DAP_PORT);
        Some(crate::debug::dap::start_dap_server(crate::debug::dap::DEFAULT_DAP_PORT).unwrap())
    } else {
        None
    };

    let mut app = App {
        input_path,
        binary,
        window: None,
        game: None,
        start_time: None,
        last_update: None,
        last_render_update: None,
        accumulator: std::time::Duration::ZERO,
        debug,
        gdb_server,
        dap_server,
    };

    event_loop.run_app(&mut app)
}

pub fn run(input_path: PathBuf, binary: Vec<u8>, debug: bool) {
    let event_loop = EventLoop::<GameEvent>::with_user_event().build().unwrap();
    pollster::block_on(run_loop(event_loop, input_path, binary, debug)).ok();
}

/// Android entry. The OS hands the bundle wrapper's `android_main` an
/// `AndroidApp`, which winit needs to build its event loop; everything after
/// that is the shared [`run_loop`]. Guest input is extracted from the APK by
/// [`prepare_android_input`] before this is called.
#[cfg(target_os = "android")]
pub fn run_android(
    app: winit::platform::android::activity::AndroidApp,
    input_path: PathBuf,
    binary: Vec<u8>,
    debug: bool,
) {
    use winit::platform::android::EventLoopBuilderExtAndroid;
    let event_loop = EventLoop::<GameEvent>::with_user_event()
        .with_android_app(app)
        .build()
        .unwrap();
    pollster::block_on(run_loop(event_loop, input_path, binary, debug)).ok();
}

/// Name of the single archive asset the Android bundle packs the guest tree into.
/// Created by the bundler from the build output; see `src/commands/bundle.rs`.
#[cfg(target_os = "android")]
pub const ANDROID_INPUT_ARCHIVE: &str = "input.tar";

/// Extracts the bundled guest tree from the APK into app-private storage and
/// returns that path, so the rest of the runtime reads guest files through the
/// normal filesystem/VFS path instead of the Android `AssetManager`.
///
/// The tree is packed as a single [`ANDROID_INPUT_ARCHIVE`] asset rather than
/// loose files because Android's `AAssetDir` enumeration cannot list
/// subdirectories at runtime — walking an arbitrary asset tree is unreliable, so
/// we open one known asset and unpack it.
#[cfg(target_os = "android")]
pub fn prepare_android_input(app: &winit::platform::android::activity::AndroidApp) -> PathBuf {
    use std::ffi::CString;

    let dest = app
        .internal_data_path()
        .expect("Android internal data path unavailable")
        .join("input");
    std::fs::create_dir_all(&dest).expect("create Android input dir");

    let name = CString::new(ANDROID_INPUT_ARCHIVE).unwrap();
    // `Asset` implements `Read`, so stream it straight into the tar extractor.
    let asset = app
        .asset_manager()
        .open(&name)
        .unwrap_or_else(|| panic!("bundled '{ANDROID_INPUT_ARCHIVE}' missing from APK assets"));
    tar::Archive::new(asset)
        .unpack(&dest)
        .expect("extract bundled guest input");

    dest
}


pub async fn test(_input_path: PathBuf, _binary: Vec<u8>) {
    // Parse command line arguments
    let args = Arguments::from_args();

    // Create a list of tests and/or benchmarks (in this case: two dummy tests).
    let tests = vec![
        Trial::test("succeeding_test", move || Ok(())),
        Trial::test("failing_test", move || Err("Woops".into())),
    ];

    // TODO: Setup Jumpjet host in test mode

    // TODO: Guest implements Jumpjet "test guest", which has a setup_tests() function

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
