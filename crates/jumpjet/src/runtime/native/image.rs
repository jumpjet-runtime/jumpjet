use wasmtime::Result;
use wasmtime::component::{Accessor, HasSelf, Resource, StreamReader};

use super::state::JumpjetRuntimeState;
use super::stream::read_stream_to_vec;
use crate::jumpjet::runtime::image::*;
use crate::runtime::common::image::Bitmap;

// No non-store freestanding functions; `decode` is async and lives on
// `HostWithStore`.
impl Host for JumpjetRuntimeState {}

impl HostWithStore for HasSelf<JumpjetRuntimeState> {
    async fn decode<T: Send>(
        accessor: &Accessor<T, Self>,
        data: StreamReader<u8>,
    ) -> core::result::Result<Resource<ImageBitmap>, DecodeError> {
        let bytes = read_stream_to_vec(accessor, data)
            .await
            .map_err(|e| DecodeError::InvalidData(e.to_string()))?;

        let reader = image::ImageReader::new(std::io::Cursor::new(&bytes))
            .with_guessed_format()
            .map_err(|_| DecodeError::UnsupportedFormat)?;
        let rgba = reader
            .decode()
            .map_err(|e| DecodeError::InvalidData(e.to_string()))?
            .to_rgba8();
        let (width, height) = rgba.dimensions();

        accessor
            .with(|mut access| {
                access.get().table.push(Bitmap {
                    width,
                    height,
                    rgba: rgba.into_raw(),
                })
            })
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
