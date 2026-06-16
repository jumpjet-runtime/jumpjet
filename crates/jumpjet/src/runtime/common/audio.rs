use cpal::Device;

pub struct AudioState {
    pub device: Device,
}

impl AudioState {
    pub fn new(device: Device) -> AudioState {
        AudioState { device }
    }
}
