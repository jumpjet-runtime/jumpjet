/// A decoded image held host-side, behind the `image-bitmap` resource. Pixels
/// are tightly packed RGBA8 (`width * height * 4` bytes) so they can be uploaded
/// to a texture as-is via `gpu-queue.copy-external-image-to-texture`.
pub struct Bitmap {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}
