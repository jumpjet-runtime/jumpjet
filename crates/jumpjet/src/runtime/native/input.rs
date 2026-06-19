use gilrs::{Axis, Button};
use wasmtime::Result;
use wasmtime::component::Resource;
use winit::keyboard::{Key, NamedKey, SmolStr};

use super::state::JumpjetRuntimeState;
use crate::jumpjet::runtime::input::*;

impl Host for JumpjetRuntimeState {
    async fn gamepad(&mut self, id: u32) -> Option<Resource<GamepadDevice>> {
        let mut gilrs = self.gilrs.gamepads();
        if let Some((gamepad_id, _)) = gilrs.nth(id as usize) {
            let idx = self.table.push(gamepad_id).unwrap();
            Some(Resource::new_own(idx.rep()))
        } else {
            None
        }
    }

    async fn keyboard(&mut self) -> Option<Resource<KeyboardDevice>> {
        Some(Resource::new_own(0))
    }

    async fn mouse(&mut self) -> Option<Resource<MouseDevice>> {
        Some(Resource::new_own(0))
    }

    async fn touch(&mut self) -> Option<Resource<TouchDevice>> {
        Some(Resource::new_own(0))
    }
}

impl HostGamepadDevice for JumpjetRuntimeState {
    async fn name(&mut self, gamepad: Resource<GamepadDevice>) -> String {
        let gamepad_id = self.table.get(&gamepad).unwrap();
        let gamepad = self.gilrs.connected_gamepad(*gamepad_id).unwrap();

        gamepad.name().to_owned()
    }

    async fn is_pressed(&mut self, gamepad: Resource<GamepadDevice>, btn: GamepadButton) -> bool {
        let gamepad_id = *self.table.get(&gamepad).unwrap();
        let btn_code: Button = btn.into();
        self.gamepad_state
            .active_buttons
            .iter()
            .any(|k| k.1 == gamepad_id && k.2 == btn_code)
    }

    async fn value(&mut self, _gamepad: Resource<GamepadDevice>, _axis: GamepadAxis) -> f32 {
        // let gamepad_id = self.table.get(&gamepad).unwrap();
        // let gamepad = self.gilrs.connected_gamepad(*gamepad_id).unwrap();

        // Ok(gamepad.value(axis.into()))
        0.0
    }

    async fn button_data(
        &mut self,
        gamepad: Resource<GamepadDevice>,
        btn: GamepadButton,
    ) -> Option<GamepadButtonData> {
        let gamepad_id = *self.table.get(&gamepad).unwrap();
        let btn_code: Button = btn.into();

        let button_state = self
            .gamepad_state
            .active_buttons
            .iter()
            .find(|k| k.1 == gamepad_id && k.2 == btn_code);

        if let Some((_, _, _, is_repeating)) = button_state {
            Some(GamepadButtonData {
                is_pressed: true,
                value: 1.0, // TODO: Get actual value from gilrs if possible, or support analog buttons in active_buttons
                is_repeating: *is_repeating,
                counter: 0, // TODO: Implement counter
            })
        } else {
            None
        }
    }

    async fn axis_data(
        &mut self,
        _gamepad: Resource<GamepadDevice>,
        _axis: GamepadAxis,
    ) -> Option<GamepadAxisData> {
        // let gamepad_id = self.table.get(&gamepad).unwrap();
        // let gamepad = self.gilrs.connected_gamepad(*gamepad_id).unwrap();

        // if let Some(axis_data) = gamepad.axis_data(axis.into()) {
        //     Ok(Some(GamepadAxisData {
        //         value: axis_data.value(),
        //         counter: u32::try_from(axis_data.counter()).ok().unwrap()
        //     }))
        // } else {
        //     Ok(None)
        // }
        None
    }

    async fn drop(&mut self, _rep: Resource<GamepadDevice>) -> Result<()> {
        Ok(())
    }
}

impl HostKeyboardDevice for JumpjetRuntimeState {
    async fn is_pressed(&mut self, _device: Resource<KeyboardDevice>, key: KeyboardKey) -> bool {
        let (target_key, target_location): (Key, winit::keyboard::KeyLocation) = key.into();
        self.keyboard_state
            .active_keys
            .iter()
            .any(|k| k.2 == target_key && k.3 == target_location)
    }

    async fn just_pressed(&mut self, _device: Resource<KeyboardDevice>, key: KeyboardKey) -> bool {
        let (target_key, target_location): (Key, winit::keyboard::KeyLocation) = key.into();
        let generation = self.generation;
        self.keyboard_state
            .active_keys
            .iter()
            .any(|k| k.0 == generation && k.2 == target_key && k.3 == target_location)
    }

    async fn active_keys(&mut self, _device: Resource<KeyboardDevice>) -> Vec<KeyboardKey> {
        self.keyboard_state
            .active_keys
            .iter()
            .map(|k| (k.2.clone(), k.3.clone()).into())
            .collect()
    }

    async fn drop(&mut self, _rep: Resource<KeyboardDevice>) -> Result<()> {
        Ok(())
    }
}

impl HostMouseDevice for JumpjetRuntimeState {
    async fn is_pressed(&mut self, _device: Resource<MouseDevice>, btn: MouseButton) -> bool {
        use winit::event::MouseButton as W;
        self.mouse_state.buttons.iter().any(|b| {
            matches!(
                (btn, b),
                (MouseButton::Left, W::Left)
                    | (MouseButton::Right, W::Right)
                    | (MouseButton::Middle, W::Middle)
                    | (MouseButton::Back, W::Back)
                    | (MouseButton::Forward, W::Forward)
                    | (MouseButton::Other, W::Other(_))
            )
        })
    }

    async fn position(&mut self, _device: Resource<MouseDevice>) -> MousePosition {
        MousePosition {
            x: self.mouse_state.x,
            y: self.mouse_state.y,
        }
    }

    async fn delta(&mut self, _device: Resource<MouseDevice>) -> MousePosition {
        MousePosition {
            x: self.mouse_state.dx,
            y: self.mouse_state.dy,
        }
    }

    async fn lock(&mut self, _device: Resource<MouseDevice>) {
        self.mouse_state.lock_request = Some(true);
    }

    async fn unlock(&mut self, _device: Resource<MouseDevice>) {
        self.mouse_state.lock_request = Some(false);
    }

    async fn is_locked(&mut self, _device: Resource<MouseDevice>) -> bool {
        self.mouse_state.locked
    }

    async fn drop(&mut self, _rep: Resource<MouseDevice>) -> Result<()> {
        Ok(())
    }
}

impl HostTouchDevice for JumpjetRuntimeState {
    async fn drop(&mut self, _rep: Resource<TouchDevice>) -> Result<()> {
        Ok(())
    }
}

impl Into<gilrs::Button> for crate::jumpjet::runtime::input::GamepadButton {
    fn into(self) -> gilrs::Button {
        match self {
            crate::input::GamepadButton::C => Button::C,
            crate::input::GamepadButton::DpadDown => Button::DPadDown,
            crate::input::GamepadButton::DpadLeft => Button::DPadLeft,
            crate::input::GamepadButton::DpadRight => Button::DPadRight,
            crate::input::GamepadButton::DpadUp => Button::DPadUp,
            crate::input::GamepadButton::East => Button::East,
            crate::input::GamepadButton::LeftThumb => Button::LeftThumb,
            crate::input::GamepadButton::LeftTrigger => Button::LeftTrigger,
            crate::input::GamepadButton::LeftTrigger2 => Button::LeftTrigger2,
            crate::input::GamepadButton::Mode => Button::Mode,
            crate::input::GamepadButton::North => Button::North,
            crate::input::GamepadButton::RightThumb => Button::RightThumb,
            crate::input::GamepadButton::RightTrigger => Button::RightTrigger,
            crate::input::GamepadButton::RightTrigger2 => Button::RightTrigger2,
            crate::input::GamepadButton::Select => Button::Select,
            crate::input::GamepadButton::South => Button::South,
            crate::input::GamepadButton::Start => Button::Start,
            crate::input::GamepadButton::West => Button::West,
            crate::input::GamepadButton::Z => Button::Z,
            crate::input::GamepadButton::Unknown => Button::Unknown,
        }
    }
}

impl Into<gilrs::Axis> for crate::jumpjet::runtime::input::GamepadAxis {
    fn into(self) -> gilrs::Axis {
        match self {
            crate::input::GamepadAxis::DpadX => Axis::DPadX,
            crate::input::GamepadAxis::DpadY => Axis::DPadY,
            crate::input::GamepadAxis::LeftStickX => Axis::LeftStickX,
            crate::input::GamepadAxis::LeftStickY => Axis::LeftStickY,
            crate::input::GamepadAxis::LeftZ => Axis::LeftZ,
            crate::input::GamepadAxis::RightStickX => Axis::RightStickX,
            crate::input::GamepadAxis::RightStickY => Axis::RightStickY,
            crate::input::GamepadAxis::RightZ => Axis::RightZ,
            crate::input::GamepadAxis::Unknown => Axis::Unknown,
        }
    }
}

impl Into<(Key, winit::keyboard::KeyLocation)> for crate::jumpjet::runtime::input::KeyboardKey {
    fn into(self) -> (Key, winit::keyboard::KeyLocation) {
        match self {
            crate::input::KeyboardKey::Character(c) => (
                Key::Character(SmolStr::new(c)),
                winit::keyboard::KeyLocation::Standard,
            ),
            crate::input::KeyboardKey::Dead(c) => (
                Key::Dead(match c {
                    Some(c) => c.chars().next(),
                    None => None,
                }),
                winit::keyboard::KeyLocation::Standard,
            ),
            crate::input::KeyboardKey::Alt(location) => {
                (Key::Named(NamedKey::Alt), location.into())
            }
            crate::input::KeyboardKey::CapsLock => (
                Key::Named(NamedKey::CapsLock),
                winit::keyboard::KeyLocation::Standard,
            ),
            crate::input::KeyboardKey::Control(location) => {
                (Key::Named(NamedKey::Control), location.into())
            }
            crate::input::KeyboardKey::Super(location) => {
                (Key::Named(NamedKey::Super), location.into())
            }
            crate::input::KeyboardKey::Fn => (
                Key::Named(NamedKey::Fn),
                winit::keyboard::KeyLocation::Standard,
            ),
            crate::input::KeyboardKey::FnLock => (
                Key::Named(NamedKey::FnLock),
                winit::keyboard::KeyLocation::Standard,
            ),
            crate::input::KeyboardKey::NumLock => (
                Key::Named(NamedKey::NumLock),
                winit::keyboard::KeyLocation::Standard,
            ),
            crate::input::KeyboardKey::ScrollLock => (
                Key::Named(NamedKey::ScrollLock),
                winit::keyboard::KeyLocation::Standard,
            ),
            crate::input::KeyboardKey::Shift(location) => {
                (Key::Named(NamedKey::Shift), location.into())
            }
            crate::input::KeyboardKey::Symbol => (
                Key::Named(NamedKey::Symbol),
                winit::keyboard::KeyLocation::Standard,
            ),
            crate::input::KeyboardKey::SymbolLock => (
                Key::Named(NamedKey::SymbolLock),
                winit::keyboard::KeyLocation::Standard,
            ),
            crate::input::KeyboardKey::Enter => (
                Key::Named(NamedKey::Enter),
                winit::keyboard::KeyLocation::Standard,
            ),
            crate::input::KeyboardKey::Tab => (
                Key::Named(NamedKey::Tab),
                winit::keyboard::KeyLocation::Standard,
            ),
            crate::input::KeyboardKey::Space => (
                Key::Named(NamedKey::Space),
                winit::keyboard::KeyLocation::Standard,
            ),
            crate::input::KeyboardKey::ArrowDown => (
                Key::Named(NamedKey::ArrowDown),
                winit::keyboard::KeyLocation::Standard,
            ),
            crate::input::KeyboardKey::ArrowLeft => (
                Key::Named(NamedKey::ArrowLeft),
                winit::keyboard::KeyLocation::Standard,
            ),
            crate::input::KeyboardKey::ArrowRight => (
                Key::Named(NamedKey::ArrowRight),
                winit::keyboard::KeyLocation::Standard,
            ),
            crate::input::KeyboardKey::ArrowUp => (
                Key::Named(NamedKey::ArrowUp),
                winit::keyboard::KeyLocation::Standard,
            ),
            crate::input::KeyboardKey::End => (
                Key::Named(NamedKey::End),
                winit::keyboard::KeyLocation::Standard,
            ),
            crate::input::KeyboardKey::Home => (
                Key::Named(NamedKey::Home),
                winit::keyboard::KeyLocation::Standard,
            ),
            crate::input::KeyboardKey::PageDown => (
                Key::Named(NamedKey::PageDown),
                winit::keyboard::KeyLocation::Standard,
            ),
            crate::input::KeyboardKey::PageUp => (
                Key::Named(NamedKey::PageUp),
                winit::keyboard::KeyLocation::Standard,
            ),
            crate::input::KeyboardKey::Backspace => (
                Key::Named(NamedKey::Backspace),
                winit::keyboard::KeyLocation::Standard,
            ),
            crate::input::KeyboardKey::Delete => (
                Key::Named(NamedKey::Delete),
                winit::keyboard::KeyLocation::Standard,
            ),
            crate::input::KeyboardKey::Insert => (
                Key::Named(NamedKey::Insert),
                winit::keyboard::KeyLocation::Standard,
            ),
            crate::input::KeyboardKey::Cancel => (
                Key::Named(NamedKey::Cancel),
                winit::keyboard::KeyLocation::Standard,
            ),
            crate::input::KeyboardKey::ContextMenu => (
                Key::Named(NamedKey::ContextMenu),
                winit::keyboard::KeyLocation::Standard,
            ),
            crate::input::KeyboardKey::Escape => (
                Key::Named(NamedKey::Escape),
                winit::keyboard::KeyLocation::Standard,
            ),
            crate::input::KeyboardKey::Props => (
                Key::Named(NamedKey::Props),
                winit::keyboard::KeyLocation::Standard,
            ),
            crate::input::KeyboardKey::Select => (
                Key::Named(NamedKey::Select),
                winit::keyboard::KeyLocation::Standard,
            ),
            crate::input::KeyboardKey::ZoomIn => (
                Key::Named(NamedKey::ZoomIn),
                winit::keyboard::KeyLocation::Standard,
            ),
            crate::input::KeyboardKey::ZoomOut => (
                Key::Named(NamedKey::ZoomOut),
                winit::keyboard::KeyLocation::Standard,
            ),
            crate::input::KeyboardKey::F1 => (
                Key::Named(NamedKey::F1),
                winit::keyboard::KeyLocation::Standard,
            ),
            crate::input::KeyboardKey::F2 => (
                Key::Named(NamedKey::F2),
                winit::keyboard::KeyLocation::Standard,
            ),
            crate::input::KeyboardKey::F3 => (
                Key::Named(NamedKey::F3),
                winit::keyboard::KeyLocation::Standard,
            ),
            crate::input::KeyboardKey::F4 => (
                Key::Named(NamedKey::F4),
                winit::keyboard::KeyLocation::Standard,
            ),
            crate::input::KeyboardKey::F5 => (
                Key::Named(NamedKey::F5),
                winit::keyboard::KeyLocation::Standard,
            ),
            crate::input::KeyboardKey::F6 => (
                Key::Named(NamedKey::F6),
                winit::keyboard::KeyLocation::Standard,
            ),
            crate::input::KeyboardKey::F7 => (
                Key::Named(NamedKey::F7),
                winit::keyboard::KeyLocation::Standard,
            ),
            crate::input::KeyboardKey::F8 => (
                Key::Named(NamedKey::F8),
                winit::keyboard::KeyLocation::Standard,
            ),
            crate::input::KeyboardKey::F9 => (
                Key::Named(NamedKey::F9),
                winit::keyboard::KeyLocation::Standard,
            ),
            crate::input::KeyboardKey::F10 => (
                Key::Named(NamedKey::F10),
                winit::keyboard::KeyLocation::Standard,
            ),
            crate::input::KeyboardKey::F11 => (
                Key::Named(NamedKey::F11),
                winit::keyboard::KeyLocation::Standard,
            ),
            crate::input::KeyboardKey::F12 => (
                Key::Named(NamedKey::F12),
                winit::keyboard::KeyLocation::Standard,
            ),
            crate::input::KeyboardKey::Unidentified(_) => (
                Key::Unidentified(winit::keyboard::NativeKey::Unidentified),
                winit::keyboard::KeyLocation::Standard,
            ),
        }
    }
}

impl Into<crate::jumpjet::runtime::input::KeyboardKey> for (Key, winit::keyboard::KeyLocation) {
    fn into(self) -> crate::jumpjet::runtime::input::KeyboardKey {
        match self {
            (Key::Character(str), _) => {
                crate::jumpjet::runtime::input::KeyboardKey::Character(str.to_string())
            }
            (Key::Unidentified(_value), _) => {
                crate::jumpjet::runtime::input::KeyboardKey::Unidentified(0)
            }
            (Key::Dead(c), _) => crate::jumpjet::runtime::input::KeyboardKey::Dead(match c {
                Some(c) => Some(c.to_string()),
                None => None,
            }),
            (Key::Named(NamedKey::Alt), winit::keyboard::KeyLocation::Standard) => {
                crate::jumpjet::runtime::input::KeyboardKey::Alt(KeyLocation::Left)
            }
            (Key::Named(NamedKey::AltGraph), winit::keyboard::KeyLocation::Standard) => {
                crate::jumpjet::runtime::input::KeyboardKey::Alt(KeyLocation::Right)
            }
            (Key::Named(NamedKey::CapsLock), winit::keyboard::KeyLocation::Standard) => {
                crate::jumpjet::runtime::input::KeyboardKey::CapsLock
            }
            (Key::Named(NamedKey::Control), location) => {
                crate::jumpjet::runtime::input::KeyboardKey::Control(location.into())
            }
            (Key::Named(NamedKey::Super), location) => {
                crate::jumpjet::runtime::input::KeyboardKey::Super(location.into())
            }
            (Key::Named(NamedKey::Fn), winit::keyboard::KeyLocation::Standard) => {
                crate::jumpjet::runtime::input::KeyboardKey::Fn
            }
            (Key::Named(NamedKey::FnLock), winit::keyboard::KeyLocation::Standard) => {
                crate::jumpjet::runtime::input::KeyboardKey::FnLock
            }
            (Key::Named(NamedKey::NumLock), winit::keyboard::KeyLocation::Standard) => {
                crate::jumpjet::runtime::input::KeyboardKey::NumLock
            }
            (Key::Named(NamedKey::ScrollLock), winit::keyboard::KeyLocation::Standard) => {
                crate::jumpjet::runtime::input::KeyboardKey::ScrollLock
            }
            (Key::Named(NamedKey::Shift), location) => {
                crate::jumpjet::runtime::input::KeyboardKey::Shift(location.into())
            }
            (Key::Named(NamedKey::Symbol), winit::keyboard::KeyLocation::Standard) => {
                crate::jumpjet::runtime::input::KeyboardKey::Symbol
            }
            (Key::Named(NamedKey::SymbolLock), winit::keyboard::KeyLocation::Standard) => {
                crate::jumpjet::runtime::input::KeyboardKey::SymbolLock
            }
            (Key::Named(NamedKey::Enter), winit::keyboard::KeyLocation::Standard) => {
                crate::jumpjet::runtime::input::KeyboardKey::Enter
            }
            (Key::Named(NamedKey::Tab), winit::keyboard::KeyLocation::Standard) => {
                crate::jumpjet::runtime::input::KeyboardKey::Tab
            }
            (Key::Named(NamedKey::Space), winit::keyboard::KeyLocation::Standard) => {
                crate::jumpjet::runtime::input::KeyboardKey::Space
            }
            (Key::Named(NamedKey::ArrowDown), winit::keyboard::KeyLocation::Standard) => {
                crate::jumpjet::runtime::input::KeyboardKey::ArrowDown
            }
            (Key::Named(NamedKey::ArrowLeft), winit::keyboard::KeyLocation::Standard) => {
                crate::jumpjet::runtime::input::KeyboardKey::ArrowLeft
            }
            (Key::Named(NamedKey::ArrowRight), winit::keyboard::KeyLocation::Standard) => {
                crate::jumpjet::runtime::input::KeyboardKey::ArrowRight
            }
            (Key::Named(NamedKey::ArrowUp), winit::keyboard::KeyLocation::Standard) => {
                crate::jumpjet::runtime::input::KeyboardKey::ArrowUp
            }
            (Key::Named(NamedKey::End), winit::keyboard::KeyLocation::Standard) => {
                crate::jumpjet::runtime::input::KeyboardKey::End
            }
            (Key::Named(NamedKey::Home), winit::keyboard::KeyLocation::Standard) => {
                crate::jumpjet::runtime::input::KeyboardKey::Home
            }
            (Key::Named(NamedKey::PageDown), winit::keyboard::KeyLocation::Standard) => {
                crate::jumpjet::runtime::input::KeyboardKey::PageDown
            }
            (Key::Named(NamedKey::PageUp), winit::keyboard::KeyLocation::Standard) => {
                crate::jumpjet::runtime::input::KeyboardKey::PageUp
            }
            (Key::Named(NamedKey::Backspace), winit::keyboard::KeyLocation::Standard) => {
                crate::jumpjet::runtime::input::KeyboardKey::Backspace
            }
            (Key::Named(NamedKey::Delete), winit::keyboard::KeyLocation::Standard) => {
                crate::jumpjet::runtime::input::KeyboardKey::Delete
            }
            (Key::Named(NamedKey::Insert), winit::keyboard::KeyLocation::Standard) => {
                crate::jumpjet::runtime::input::KeyboardKey::Insert
            }
            (Key::Named(NamedKey::Cancel), winit::keyboard::KeyLocation::Standard) => {
                crate::jumpjet::runtime::input::KeyboardKey::Cancel
            }
            (Key::Named(NamedKey::ContextMenu), winit::keyboard::KeyLocation::Standard) => {
                crate::jumpjet::runtime::input::KeyboardKey::ContextMenu
            }
            (Key::Named(NamedKey::Escape), winit::keyboard::KeyLocation::Standard) => {
                crate::jumpjet::runtime::input::KeyboardKey::Escape
            }
            (Key::Named(NamedKey::Props), winit::keyboard::KeyLocation::Standard) => {
                crate::jumpjet::runtime::input::KeyboardKey::Props
            }
            (Key::Named(NamedKey::Select), winit::keyboard::KeyLocation::Standard) => {
                crate::jumpjet::runtime::input::KeyboardKey::Select
            }
            (Key::Named(NamedKey::ZoomIn), winit::keyboard::KeyLocation::Standard) => {
                crate::jumpjet::runtime::input::KeyboardKey::ZoomIn
            }
            (Key::Named(NamedKey::ZoomOut), winit::keyboard::KeyLocation::Standard) => {
                crate::jumpjet::runtime::input::KeyboardKey::ZoomOut
            }
            (Key::Named(NamedKey::F1), winit::keyboard::KeyLocation::Standard) => {
                crate::jumpjet::runtime::input::KeyboardKey::F1
            }
            (Key::Named(NamedKey::F2), winit::keyboard::KeyLocation::Standard) => {
                crate::jumpjet::runtime::input::KeyboardKey::F2
            }
            (Key::Named(NamedKey::F3), winit::keyboard::KeyLocation::Standard) => {
                crate::jumpjet::runtime::input::KeyboardKey::F3
            }
            (Key::Named(NamedKey::F4), winit::keyboard::KeyLocation::Standard) => {
                crate::jumpjet::runtime::input::KeyboardKey::F4
            }
            (Key::Named(NamedKey::F5), winit::keyboard::KeyLocation::Standard) => {
                crate::jumpjet::runtime::input::KeyboardKey::F5
            }
            (Key::Named(NamedKey::F6), winit::keyboard::KeyLocation::Standard) => {
                crate::jumpjet::runtime::input::KeyboardKey::F6
            }
            (Key::Named(NamedKey::F7), winit::keyboard::KeyLocation::Standard) => {
                crate::jumpjet::runtime::input::KeyboardKey::F7
            }
            (Key::Named(NamedKey::F8), winit::keyboard::KeyLocation::Standard) => {
                crate::jumpjet::runtime::input::KeyboardKey::F8
            }
            (Key::Named(NamedKey::F9), winit::keyboard::KeyLocation::Standard) => {
                crate::jumpjet::runtime::input::KeyboardKey::F9
            }
            (Key::Named(NamedKey::F10), winit::keyboard::KeyLocation::Standard) => {
                crate::jumpjet::runtime::input::KeyboardKey::F10
            }
            (Key::Named(NamedKey::F11), winit::keyboard::KeyLocation::Standard) => {
                crate::jumpjet::runtime::input::KeyboardKey::F11
            }
            (Key::Named(NamedKey::F12), winit::keyboard::KeyLocation::Standard) => {
                crate::jumpjet::runtime::input::KeyboardKey::F12
            }
            (_, _) => crate::jumpjet::runtime::input::KeyboardKey::Unidentified(0),
        }
    }
}

impl Into<winit::keyboard::KeyLocation> for crate::jumpjet::runtime::input::KeyLocation {
    fn into(self) -> winit::keyboard::KeyLocation {
        match self {
            crate::input::KeyLocation::Standard => winit::keyboard::KeyLocation::Standard,
            crate::input::KeyLocation::Left => winit::keyboard::KeyLocation::Left,
            crate::input::KeyLocation::Right => winit::keyboard::KeyLocation::Right,
            crate::input::KeyLocation::Numpad => winit::keyboard::KeyLocation::Numpad,
        }
    }
}

impl Into<crate::jumpjet::runtime::input::KeyLocation> for winit::keyboard::KeyLocation {
    fn into(self) -> crate::input::KeyLocation {
        match self {
            winit::keyboard::KeyLocation::Standard => crate::input::KeyLocation::Standard,
            winit::keyboard::KeyLocation::Left => crate::input::KeyLocation::Left,
            winit::keyboard::KeyLocation::Right => crate::input::KeyLocation::Right,
            winit::keyboard::KeyLocation::Numpad => crate::input::KeyLocation::Numpad,
        }
    }
}
