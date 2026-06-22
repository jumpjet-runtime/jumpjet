use wasmtime::Result;
use wasmtime::component::Resource;

use super::state::JumpjetRuntimeState;
use crate::jumpjet::runtime::image::*;
use crate::runtime::common::image::Bitmap;

impl Host for JumpjetRuntimeState {
    async fn decode(
        &mut self,
        data: Resource<Buffer>,
    ) -> core::result::Result<Resource<ImageBitmap>, DecodeError> {
        // Decode within a scope so the immutable `table` borrow on the source
        // buffer is released before pushing the bitmap resource.
        let bitmap = {
            let bytes = &self
                .table
                .get(&data)
                .map_err(|e| DecodeError::InvalidData(e.to_string()))?
                .0;
            let reader = image::ImageReader::new(std::io::Cursor::new(bytes.as_slice()))
                .with_guessed_format()
                .map_err(|_| DecodeError::UnsupportedFormat)?;
            let rgba = reader
                .decode()
                .map_err(|e| DecodeError::InvalidData(e.to_string()))?
                .to_rgba8();
            let (width, height) = rgba.dimensions();
            Bitmap {
                width,
                height,
                rgba: rgba.into_raw(),
            }
        };
        self.table
            .push(bitmap)
            .map_err(|e| DecodeError::InvalidData(e.to_string()))
    }
}

impl HostImageBitmap for JumpjetRuntimeState {
    async fn width(&mut self, bitmap: Resource<ImageBitmap>) -> u32 {
        self.table.get(&bitmap).unwrap().width
    }

    async fn height(&mut self, bitmap: Resource<ImageBitmap>) -> u32 {
        self.table.get(&bitmap).unwrap().height
    }

    async fn format(&mut self, _bitmap: Resource<ImageBitmap>) -> PixelFormat {
        PixelFormat::Rgba8
    }

    async fn drop(&mut self, rep: Resource<ImageBitmap>) -> Result<()> {
        self.table.delete(rep).ok();
        Ok(())
    }
}
