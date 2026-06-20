use std::io::{Read, Write};

use wasmtime::component::{Accessor, HasSelf};

use super::state::JumpjetRuntimeState;
use super::stream::{read_stream_to_vec, vec_to_stream};
use crate::jumpjet::runtime::compression::*;

// All ops are async (they touch streams) and live on `HostWithStore`.
impl Host for JumpjetRuntimeState {}

impl crate::jumpjet::runtime::compression::HostWithStore for HasSelf<JumpjetRuntimeState> {
    async fn compress<T: Send>(
        accessor: &Accessor<T, Self>,
        input: CompressionData,
        output: OutputKind,
        format: CompressionFormat,
    ) -> core::result::Result<CompressionData, CompressionError> {
        let bytes = input_bytes(accessor, input).await?;
        let out = match format {
            CompressionFormat::Gzip => gzip_compress(&bytes),
            CompressionFormat::Deflate => deflate_compress(&bytes),
            CompressionFormat::Brotli => brotli_compress(&bytes),
            // Draco bridges bytes <-> model and needs an encoder we don't ship yet.
            CompressionFormat::Draco => return Err(CompressionError::UnsupportedFormat),
        }
        .map_err(|e| CompressionError::InvalidData(e.to_string()))?;
        output_data(accessor, output, out)
    }

    async fn decompress<T: Send>(
        accessor: &Accessor<T, Self>,
        input: CompressionData,
        output: OutputKind,
        format: CompressionFormat,
    ) -> core::result::Result<CompressionData, CompressionError> {
        let bytes = input_bytes(accessor, input).await?;
        let out = match format {
            CompressionFormat::Gzip => gzip_decompress(&bytes),
            CompressionFormat::Deflate => deflate_decompress(&bytes),
            CompressionFormat::Brotli => brotli_decompress(&bytes),
            CompressionFormat::Draco => return Err(CompressionError::UnsupportedFormat),
        }
        .map_err(|e| CompressionError::InvalidData(e.to_string()))?;
        output_data(accessor, output, out)
    }
}

/// Pull raw bytes out of a `compression-data` input. Only `bytes` is supported;
/// `model` input would require draco encode, which isn't wired up.
async fn input_bytes<T>(
    accessor: &Accessor<T, HasSelf<JumpjetRuntimeState>>,
    input: CompressionData,
) -> core::result::Result<Vec<u8>, CompressionError> {
    match input {
        CompressionData::Bytes(stream) => read_stream_to_vec(accessor, stream)
            .await
            .map_err(|e| CompressionError::InvalidData(e.to_string())),
        CompressionData::Model(_) => Err(CompressionError::UnsupportedFormat),
    }
}

/// Wrap produced bytes in the requested output form. Only `bytes` is produced;
/// `model` output (draco decode) isn't wired up.
fn output_data<T>(
    accessor: &Accessor<T, HasSelf<JumpjetRuntimeState>>,
    output: OutputKind,
    bytes: Vec<u8>,
) -> core::result::Result<CompressionData, CompressionError> {
    match output {
        OutputKind::Bytes => vec_to_stream(accessor, bytes)
            .map(CompressionData::Bytes)
            .map_err(|e| CompressionError::InvalidData(e.to_string())),
        OutputKind::Model => Err(CompressionError::UnsupportedFormat),
    }
}

fn gzip_compress(bytes: &[u8]) -> std::io::Result<Vec<u8>> {
    let mut e = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
    e.write_all(bytes)?;
    e.finish()
}

fn gzip_decompress(bytes: &[u8]) -> std::io::Result<Vec<u8>> {
    let mut out = Vec::new();
    flate2::read::GzDecoder::new(bytes).read_to_end(&mut out)?;
    Ok(out)
}

fn deflate_compress(bytes: &[u8]) -> std::io::Result<Vec<u8>> {
    let mut e = flate2::write::DeflateEncoder::new(Vec::new(), flate2::Compression::default());
    e.write_all(bytes)?;
    e.finish()
}

fn deflate_decompress(bytes: &[u8]) -> std::io::Result<Vec<u8>> {
    let mut out = Vec::new();
    flate2::read::DeflateDecoder::new(bytes).read_to_end(&mut out)?;
    Ok(out)
}

fn brotli_compress(bytes: &[u8]) -> std::io::Result<Vec<u8>> {
    let mut out = Vec::new();
    let params = brotli::enc::BrotliEncoderParams::default();
    brotli::BrotliCompress(&mut &bytes[..], &mut out, &params)?;
    Ok(out)
}

fn brotli_decompress(bytes: &[u8]) -> std::io::Result<Vec<u8>> {
    let mut out = Vec::new();
    brotli::BrotliDecompress(&mut &bytes[..], &mut out)?;
    Ok(out)
}
