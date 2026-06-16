use super::state::JumpjetRuntimeState;

impl crate::jumpjet::runtime::window::Host for JumpjetRuntimeState {
    async fn dimensions(&mut self) -> (u32, u32) {
        (self.window_size.width, self.window_size.height)
    }
}
