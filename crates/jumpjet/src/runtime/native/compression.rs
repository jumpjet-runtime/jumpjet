use std::io::{Read, Write};

use wasmtime::component::Resource;

use super::state::JumpjetRuntimeState;
use crate::jumpjet::runtime::compression::*;
// `Buffer` is a type alias in this (consuming) module, which can't be used as a
// tuple constructor; reach for the concrete type to build one.
use crate::runtime::common::tasks::Buffer as BufferData;

impl Host for JumpjetRuntimeState {
    async fn compress(
        &mut self,
        data: Resource<Buffer>,
        format: CompressionFormat,
    ) -> core::result::Result<Resource<Buffer>, CompressionError> {
        let out = {
            let bytes = &self.table.get(&data).map_err(invalid)?.0;
            match format {
                CompressionFormat::Gzip => gzip_compress(bytes),
                CompressionFormat::Deflate => deflate_compress(bytes),
                CompressionFormat::Brotli => brotli_compress(bytes),
            }
            .map_err(invalid)?
        };
        self.table.push(BufferData(out)).map_err(invalid)
    }

    async fn decompress(
        &mut self,
        data: Resource<Buffer>,
        format: CompressionFormat,
    ) -> core::result::Result<Resource<Buffer>, CompressionError> {
        let out = {
            let bytes = &self.table.get(&data).map_err(invalid)?.0;
            match format {
                CompressionFormat::Gzip => gzip_decompress(bytes),
                CompressionFormat::Deflate => deflate_decompress(bytes),
                CompressionFormat::Brotli => brotli_decompress(bytes),
            }
            .map_err(invalid)?
        };
        self.table.push(BufferData(out)).map_err(invalid)
    }
}

fn invalid(e: impl std::fmt::Display) -> CompressionError {
    CompressionError::InvalidData(e.to_string())
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
