use std::{borrow::Cow, num::NonZeroU64};

use wasmtime::component::ResourceTable;
use wgpu_core::{
    binding_model::{BindGroupEntry, BindingResource, BufferBinding},
    command::{LoadOp, StoreOp},
    device::HostMap
};

use crate::gpu::{
    GpuAddressMode, GpuBindGroupEntry, GpuBlendComponent, GpuBlendFactor, GpuBlendOperation,
    GpuBlendState, GpuBufferUsage, GpuColorWrite, GpuCompareFunction, GpuExtentD3, GpuFilterMode,
    GpuIndexFormat, GpuLoadOp, GpuMapMode, GpuQueryType, GpuShaderStage, GpuStencilFaceState,
    GpuStencilOperation, GpuStoreOp, GpuTextureAspect, GpuTextureDimension, GpuTextureFormat,
    GpuTextureUsage, GpuTextureViewDimension, GpuVertexFormat, GpuVertexStepMode,
};

// use crate::renderer::{GpuCompareFunction, GpuTextureFormat, GpuStencilOperation, GpuStencilFaceState, GpuColorWrite, GpuBlendState, GpuBlendComponent, GpuBlendFactor, GpuBlendOperation};

pub fn vec_to_color(vec: &Vec<f64>) -> wgpu_types::Color {
    assert_eq!(vec.len(), 4);

    wgpu_types::Color {
        r: vec[0],
        g: vec[1],
        b: vec[2],
        a: vec[3],
    }
}

impl Into<wgpu_types::AddressMode> for GpuAddressMode {
    fn into(self) -> wgpu_types::AddressMode {
        match self {
            GpuAddressMode::ClampToEdge => wgpu_types::AddressMode::ClampToEdge,
            GpuAddressMode::MirrorRepeat => wgpu_types::AddressMode::MirrorRepeat,
            GpuAddressMode::Repeat => wgpu_types::AddressMode::Repeat,
        }
    }
}

impl Into<wgpu_types::Extent3d> for GpuExtentD3 {
    fn into(self) -> wgpu_types::Extent3d {
        wgpu_types::Extent3d {
            width: self.width,
            height: self.height,
            depth_or_array_layers: self.depth_or_array_layers,
        }
    }
}

impl Into<wgpu_types::CompareFunction> for GpuCompareFunction {
    fn into(self) -> wgpu_types::CompareFunction {
        match self {
            GpuCompareFunction::Never => wgpu_types::CompareFunction::Never,
            GpuCompareFunction::Less => wgpu_types::CompareFunction::Less,
            GpuCompareFunction::Equal => wgpu_types::CompareFunction::Equal,
            GpuCompareFunction::LessEqual => wgpu_types::CompareFunction::LessEqual,
            GpuCompareFunction::Greater => wgpu_types::CompareFunction::Greater,
            GpuCompareFunction::NotEqual => wgpu_types::CompareFunction::NotEqual,
            GpuCompareFunction::GreaterEqual => wgpu_types::CompareFunction::GreaterEqual,
            GpuCompareFunction::Always => wgpu_types::CompareFunction::Always,
        }
    }
}

impl Into<wgpu_types::FilterMode> for GpuFilterMode {
    fn into(self) -> wgpu_types::FilterMode {
        match self {
            GpuFilterMode::Linear => wgpu_types::FilterMode::Linear,
            GpuFilterMode::Nearest => wgpu_types::FilterMode::Nearest,
        }
    }
}

impl Into<wgpu_types::IndexFormat> for GpuIndexFormat {
    fn into(self) -> wgpu_types::IndexFormat {
        match self {
            GpuIndexFormat::Uint16 => wgpu_types::IndexFormat::Uint16,
            GpuIndexFormat::Uint32 => wgpu_types::IndexFormat::Uint32,
        }
    }
}

impl Into<wgpu_types::ShaderStages> for GpuShaderStage {
    fn into(self) -> wgpu_types::ShaderStages {
        let mut ss = wgpu_types::ShaderStages::empty();
        if self.contains(GpuShaderStage::COMPUTE) {
            ss |= wgpu_types::ShaderStages::COMPUTE;
        }
        if self.contains(GpuShaderStage::FRAGMENT) {
            ss |= wgpu_types::ShaderStages::FRAGMENT;
        }
        if self.contains(GpuShaderStage::VERTEX) {
            ss |= wgpu_types::ShaderStages::VERTEX;
        }

        ss
    }
}

impl Into<wgpu_types::TextureAspect> for GpuTextureAspect {
    fn into(self) -> wgpu_types::TextureAspect {
        match self {
            GpuTextureAspect::All => wgpu_types::TextureAspect::All,
            GpuTextureAspect::DepthOnly => wgpu_types::TextureAspect::DepthOnly,
            GpuTextureAspect::StencilOnly => wgpu_types::TextureAspect::StencilOnly,
        }
    }
}

impl Into<wgpu_types::TextureFormat> for GpuTextureFormat {
    fn into(self) -> wgpu_types::TextureFormat {
        match self {
            GpuTextureFormat::R8unorm => wgpu_types::TextureFormat::R8Unorm,
            GpuTextureFormat::R8snorm => wgpu_types::TextureFormat::R8Snorm,
            GpuTextureFormat::R8uint => wgpu_types::TextureFormat::R8Uint,
            GpuTextureFormat::R8sint => wgpu_types::TextureFormat::R8Sint,
            GpuTextureFormat::R16uint => wgpu_types::TextureFormat::R16Uint,
            GpuTextureFormat::R16sint => wgpu_types::TextureFormat::R16Sint,
            GpuTextureFormat::R16float => wgpu_types::TextureFormat::R16Float,
            GpuTextureFormat::Rg8unorm => wgpu_types::TextureFormat::Rg8Unorm,
            GpuTextureFormat::Rg8snorm => wgpu_types::TextureFormat::Rg8Snorm,
            GpuTextureFormat::Rg8uint => wgpu_types::TextureFormat::Rg8Uint,
            GpuTextureFormat::Rg8sint => wgpu_types::TextureFormat::Rg8Sint,
            GpuTextureFormat::R32uint => wgpu_types::TextureFormat::R32Uint,
            GpuTextureFormat::R32sint => wgpu_types::TextureFormat::R32Sint,
            GpuTextureFormat::R32float => wgpu_types::TextureFormat::R32Float,
            GpuTextureFormat::Rg16uint => wgpu_types::TextureFormat::Rg16Uint,
            GpuTextureFormat::Rg16sint => wgpu_types::TextureFormat::Rg16Sint,
            GpuTextureFormat::Rg16float => wgpu_types::TextureFormat::Rg16Float,
            GpuTextureFormat::Rgba8unorm => wgpu_types::TextureFormat::Rgba8Unorm,
            GpuTextureFormat::Rgba8unormsrgb => wgpu_types::TextureFormat::Rgba8UnormSrgb,
            GpuTextureFormat::Rgba8snorm => wgpu_types::TextureFormat::Rgba8Snorm,
            GpuTextureFormat::Rgba8uint => wgpu_types::TextureFormat::Rgba8Uint,
            GpuTextureFormat::Rgba8sint => wgpu_types::TextureFormat::Rgba8Sint,
            GpuTextureFormat::Bgra8unorm => wgpu_types::TextureFormat::Bgra8Unorm,
            GpuTextureFormat::Bgra8unormsrgb => wgpu_types::TextureFormat::Bgra8UnormSrgb,
            GpuTextureFormat::Rgb9e5ufloat => wgpu_types::TextureFormat::Rgb9e5Ufloat,
            GpuTextureFormat::Rgb10a2unorm => wgpu_types::TextureFormat::Rgb10a2Unorm,
            GpuTextureFormat::Rg11b10ufloat => wgpu_types::TextureFormat::Rg11b10Ufloat,
            GpuTextureFormat::Rg32uint => wgpu_types::TextureFormat::Rg32Uint,
            GpuTextureFormat::Rg32sint => wgpu_types::TextureFormat::Rg32Sint,
            GpuTextureFormat::Rg32float => wgpu_types::TextureFormat::Rg32Float,
            GpuTextureFormat::Rgba16uint => wgpu_types::TextureFormat::Rgba16Uint,
            GpuTextureFormat::Rgba16sint => wgpu_types::TextureFormat::Rgba16Sint,
            GpuTextureFormat::Rgba16float => wgpu_types::TextureFormat::Rgba16Float,
            GpuTextureFormat::Rgba32uint => wgpu_types::TextureFormat::Rgba32Uint,
            GpuTextureFormat::Rgba32sint => wgpu_types::TextureFormat::Rgba32Sint,
            GpuTextureFormat::Rgba32float => wgpu_types::TextureFormat::Rgba32Float,
            GpuTextureFormat::Stencil8 => wgpu_types::TextureFormat::Stencil8,
            GpuTextureFormat::Depth16unorm => wgpu_types::TextureFormat::Depth16Unorm,
            GpuTextureFormat::Depth24plus => wgpu_types::TextureFormat::Depth24Plus,
            GpuTextureFormat::Depth24plusstencil8 => wgpu_types::TextureFormat::Depth24PlusStencil8,
            GpuTextureFormat::Depth32float => wgpu_types::TextureFormat::Depth32Float,
            GpuTextureFormat::Depth32floatstencil8 => wgpu_types::TextureFormat::Depth32FloatStencil8,
            GpuTextureFormat::Bc1rgbaunorm => wgpu_types::TextureFormat::Bc1RgbaUnorm,
            GpuTextureFormat::Bc1rgbaunormsrgb => wgpu_types::TextureFormat::Bc1RgbaUnormSrgb,
            GpuTextureFormat::Bc2rgbaunorm => wgpu_types::TextureFormat::Bc2RgbaUnorm,
            GpuTextureFormat::Bc2rgbaunormsrgb => wgpu_types::TextureFormat::Bc2RgbaUnormSrgb,
            GpuTextureFormat::Bc3rgbaunorm => wgpu_types::TextureFormat::Bc3RgbaUnorm,
            GpuTextureFormat::Bc3rgbaunormsrgb => wgpu_types::TextureFormat::Bc3RgbaUnormSrgb,
            GpuTextureFormat::Bc4runorm => wgpu_types::TextureFormat::Bc4RUnorm,
            GpuTextureFormat::Bc4rsnorm => wgpu_types::TextureFormat::Bc4RSnorm,
            GpuTextureFormat::Bc5rgunorm => wgpu_types::TextureFormat::Bc5RgUnorm,
            GpuTextureFormat::Bc5rgsnorm => wgpu_types::TextureFormat::Bc5RgSnorm,
            GpuTextureFormat::Bc6hrgbufloat => wgpu_types::TextureFormat::Bc6hRgbUfloat,
            GpuTextureFormat::Bc6hrgbfloat => wgpu_types::TextureFormat::Bc6hRgbFloat,
            GpuTextureFormat::Bc7rgbaunorm => wgpu_types::TextureFormat::Bc7RgbaUnorm,
            GpuTextureFormat::Bc7rgbaunormsrgb => wgpu_types::TextureFormat::Bc7RgbaUnormSrgb,
            GpuTextureFormat::Etc2rgb8unorm => wgpu_types::TextureFormat::Etc2Rgb8Unorm,
            GpuTextureFormat::Etc2rgb8unormsrgb => wgpu_types::TextureFormat::Etc2Rgb8UnormSrgb,
            GpuTextureFormat::Etc2rgb8a1unorm => wgpu_types::TextureFormat::Etc2Rgb8A1Unorm,
            GpuTextureFormat::Etc2rgb8a1unormsrgb => wgpu_types::TextureFormat::Etc2Rgb8A1UnormSrgb,
            GpuTextureFormat::Etc2rgba8unorm => wgpu_types::TextureFormat::Etc2Rgba8Unorm,
            GpuTextureFormat::Etc2rgba8unormsrgb => wgpu_types::TextureFormat::Etc2Rgba8UnormSrgb,
            GpuTextureFormat::Eacr11unorm => wgpu_types::TextureFormat::EacR11Unorm,
            GpuTextureFormat::Eacr11snorm => wgpu_types::TextureFormat::EacR11Snorm,
            GpuTextureFormat::Eacrg11unorm => wgpu_types::TextureFormat::EacRg11Unorm,
            GpuTextureFormat::Eacrg11snorm => wgpu_types::TextureFormat::EacRg11Snorm,
            GpuTextureFormat::Astc4x4unorm => wgpu_types::TextureFormat::Astc {
                block: wgpu_types::AstcBlock::B4x4,
                channel: wgpu_types::AstcChannel::Unorm,
            },
            GpuTextureFormat::Astc4x4unormsrgb => wgpu_types::TextureFormat::Astc {
                block: wgpu_types::AstcBlock::B4x4,
                channel: wgpu_types::AstcChannel::UnormSrgb,
            },
            GpuTextureFormat::Astc5x4unorm => wgpu_types::TextureFormat::Astc {
                block: wgpu_types::AstcBlock::B5x4,
                channel: wgpu_types::AstcChannel::Unorm,
            },
            GpuTextureFormat::Astc5x4unormsrgb => wgpu_types::TextureFormat::Astc {
                block: wgpu_types::AstcBlock::B5x4,
                channel: wgpu_types::AstcChannel::UnormSrgb,
            },
            GpuTextureFormat::Astc5x5unorm => wgpu_types::TextureFormat::Astc {
                block: wgpu_types::AstcBlock::B5x5,
                channel: wgpu_types::AstcChannel::Unorm,
            },
            GpuTextureFormat::Astc5x5unormsrgb => wgpu_types::TextureFormat::Astc {
                block: wgpu_types::AstcBlock::B5x5,
                channel: wgpu_types::AstcChannel::UnormSrgb,
            },
            GpuTextureFormat::Astc6x5unorm => wgpu_types::TextureFormat::Astc {
                block: wgpu_types::AstcBlock::B6x5,
                channel: wgpu_types::AstcChannel::Unorm,
            },
            GpuTextureFormat::Astc6x5unormsrgb => wgpu_types::TextureFormat::Astc {
                block: wgpu_types::AstcBlock::B6x5,
                channel: wgpu_types::AstcChannel::UnormSrgb,
            },
            GpuTextureFormat::Astc6x6unorm => wgpu_types::TextureFormat::Astc {
                block: wgpu_types::AstcBlock::B6x6,
                channel: wgpu_types::AstcChannel::Unorm,
            },
            GpuTextureFormat::Astc6x6unormsrgb => wgpu_types::TextureFormat::Astc {
                block: wgpu_types::AstcBlock::B6x6,
                channel: wgpu_types::AstcChannel::UnormSrgb,
            },
            GpuTextureFormat::Astc8x5unorm => wgpu_types::TextureFormat::Astc {
                block: wgpu_types::AstcBlock::B8x5,
                channel: wgpu_types::AstcChannel::Unorm,
            },
            GpuTextureFormat::Astc8x5unormsrgb => wgpu_types::TextureFormat::Astc {
                block: wgpu_types::AstcBlock::B8x5,
                channel: wgpu_types::AstcChannel::UnormSrgb,
            },
            GpuTextureFormat::Astc8x6unorm => wgpu_types::TextureFormat::Astc {
                block: wgpu_types::AstcBlock::B8x6,
                channel: wgpu_types::AstcChannel::Unorm,
            },
            GpuTextureFormat::Astc8x6unormsrgb => wgpu_types::TextureFormat::Astc {
                block: wgpu_types::AstcBlock::B8x6,
                channel: wgpu_types::AstcChannel::UnormSrgb,
            },
            GpuTextureFormat::Astc8x8unorm => wgpu_types::TextureFormat::Astc {
                block: wgpu_types::AstcBlock::B8x8,
                channel: wgpu_types::AstcChannel::Unorm,
            },
            GpuTextureFormat::Astc8x8unormsrgb => wgpu_types::TextureFormat::Astc {
                block: wgpu_types::AstcBlock::B8x8,
                channel: wgpu_types::AstcChannel::UnormSrgb,
            },
            GpuTextureFormat::Astc10x5unorm => wgpu_types::TextureFormat::Astc {
                block: wgpu_types::AstcBlock::B10x5,
                channel: wgpu_types::AstcChannel::Unorm,
            },
            GpuTextureFormat::Astc10x5unormsrgb => wgpu_types::TextureFormat::Astc {
                block: wgpu_types::AstcBlock::B10x5,
                channel: wgpu_types::AstcChannel::UnormSrgb,
            },
            GpuTextureFormat::Astc10x6unorm => wgpu_types::TextureFormat::Astc {
                block: wgpu_types::AstcBlock::B10x6,
                channel: wgpu_types::AstcChannel::Unorm,
            },
            GpuTextureFormat::Astc10x6unormsrgb => wgpu_types::TextureFormat::Astc {
                block: wgpu_types::AstcBlock::B10x6,
                channel: wgpu_types::AstcChannel::UnormSrgb,
            },
            GpuTextureFormat::Astc10x8unorm => wgpu_types::TextureFormat::Astc {
                block: wgpu_types::AstcBlock::B10x8,
                channel: wgpu_types::AstcChannel::Unorm,
            },
            GpuTextureFormat::Astc10x8unormsrgb => wgpu_types::TextureFormat::Astc {
                block: wgpu_types::AstcBlock::B10x8,
                channel: wgpu_types::AstcChannel::UnormSrgb,
            },
            GpuTextureFormat::Astc10x10unorm => wgpu_types::TextureFormat::Astc {
                block: wgpu_types::AstcBlock::B10x10,
                channel: wgpu_types::AstcChannel::Unorm,
            },
            GpuTextureFormat::Astc10x10unormsrgb => wgpu_types::TextureFormat::Astc {
                block: wgpu_types::AstcBlock::B10x10,
                channel: wgpu_types::AstcChannel::UnormSrgb,
            },
            GpuTextureFormat::Astc12x10unorm => wgpu_types::TextureFormat::Astc {
                block: wgpu_types::AstcBlock::B12x10,
                channel: wgpu_types::AstcChannel::Unorm,
            },
            GpuTextureFormat::Astc12x10unormsrgb => wgpu_types::TextureFormat::Astc {
                block: wgpu_types::AstcBlock::B12x10,
                channel: wgpu_types::AstcChannel::UnormSrgb,
            },
            GpuTextureFormat::Astc12x12unorm => wgpu_types::TextureFormat::Astc {
                block: wgpu_types::AstcBlock::B12x12,
                channel: wgpu_types::AstcChannel::Unorm,
            },
            GpuTextureFormat::Astc12x12unormsrgb => wgpu_types::TextureFormat::Astc {
                block: wgpu_types::AstcBlock::B12x12,
                channel: wgpu_types::AstcChannel::UnormSrgb,
            },
        }
    }
}

impl Into<GpuTextureFormat> for wgpu_types::TextureFormat {
    fn into(self) -> GpuTextureFormat {
        match self {
            wgpu_types::TextureFormat::R8Unorm => GpuTextureFormat::R8unorm,
            wgpu_types::TextureFormat::R8Snorm => GpuTextureFormat::R8snorm,
            wgpu_types::TextureFormat::R8Uint => GpuTextureFormat::R8uint,
            wgpu_types::TextureFormat::R8Sint => GpuTextureFormat::R8sint,
            wgpu_types::TextureFormat::R16Uint => GpuTextureFormat::R16uint,
            wgpu_types::TextureFormat::R16Sint => GpuTextureFormat::R16sint,
            wgpu_types::TextureFormat::R16Float => GpuTextureFormat::R16float,
            wgpu_types::TextureFormat::Rg8Unorm => GpuTextureFormat::Rg8unorm,
            wgpu_types::TextureFormat::Rg8Snorm => GpuTextureFormat::Rg8snorm,
            wgpu_types::TextureFormat::Rg8Uint => GpuTextureFormat::Rg8uint,
            wgpu_types::TextureFormat::Rg8Sint => GpuTextureFormat::Rg8sint,
            wgpu_types::TextureFormat::R32Uint => GpuTextureFormat::R32uint,
            wgpu_types::TextureFormat::R32Sint => GpuTextureFormat::R32sint,
            wgpu_types::TextureFormat::R32Float => GpuTextureFormat::R32float,
            wgpu_types::TextureFormat::Rg16Uint => GpuTextureFormat::Rg16uint,
            wgpu_types::TextureFormat::Rg16Sint => GpuTextureFormat::Rg16sint,
            wgpu_types::TextureFormat::Rg16Float => GpuTextureFormat::Rg16float,
            wgpu_types::TextureFormat::Rgba8Unorm => GpuTextureFormat::Rgba8unorm,
            wgpu_types::TextureFormat::Rgba8UnormSrgb => GpuTextureFormat::Rgba8unormsrgb,
            wgpu_types::TextureFormat::Rgba8Snorm => GpuTextureFormat::Rgba8snorm,
            wgpu_types::TextureFormat::Rgba8Uint => GpuTextureFormat::Rgba8uint,
            wgpu_types::TextureFormat::Rgba8Sint => GpuTextureFormat::Rgba8sint,
            wgpu_types::TextureFormat::Bgra8Unorm => GpuTextureFormat::Bgra8unorm,
            wgpu_types::TextureFormat::Bgra8UnormSrgb => GpuTextureFormat::Bgra8unormsrgb,
            wgpu_types::TextureFormat::Rgb9e5Ufloat => GpuTextureFormat::Rgb9e5ufloat,
            wgpu_types::TextureFormat::Rgb10a2Unorm => GpuTextureFormat::Rgb10a2unorm,
            wgpu_types::TextureFormat::Rg11b10Ufloat => GpuTextureFormat::Rg11b10ufloat,
            wgpu_types::TextureFormat::Rg32Uint => GpuTextureFormat::Rg32uint,
            wgpu_types::TextureFormat::Rg32Sint => GpuTextureFormat::Rg32sint,
            wgpu_types::TextureFormat::Rg32Float => GpuTextureFormat::Rg32float,
            wgpu_types::TextureFormat::Rgba16Uint => GpuTextureFormat::Rgba16uint,
            wgpu_types::TextureFormat::Rgba16Sint => GpuTextureFormat::Rgba16sint,
            wgpu_types::TextureFormat::Rgba16Float => GpuTextureFormat::Rgba16float,
            wgpu_types::TextureFormat::Rgba32Uint => GpuTextureFormat::Rgba32uint,
            wgpu_types::TextureFormat::Rgba32Sint => GpuTextureFormat::Rgba32sint,
            wgpu_types::TextureFormat::Rgba32Float => GpuTextureFormat::Rgba32float,
            wgpu_types::TextureFormat::Stencil8 => GpuTextureFormat::Stencil8,
            wgpu_types::TextureFormat::Depth16Unorm => GpuTextureFormat::Depth16unorm,
            wgpu_types::TextureFormat::Depth24Plus => GpuTextureFormat::Depth24plus,
            wgpu_types::TextureFormat::Depth24PlusStencil8 => GpuTextureFormat::Depth24plusstencil8,
            wgpu_types::TextureFormat::Depth32Float => GpuTextureFormat::Depth32float,
            wgpu_types::TextureFormat::Depth32FloatStencil8 => GpuTextureFormat::Depth32floatstencil8,
            wgpu_types::TextureFormat::Bc1RgbaUnorm => GpuTextureFormat::Bc1rgbaunorm,
            wgpu_types::TextureFormat::Bc1RgbaUnormSrgb => GpuTextureFormat::Bc1rgbaunormsrgb,
            wgpu_types::TextureFormat::Bc2RgbaUnorm => GpuTextureFormat::Bc2rgbaunorm,
            wgpu_types::TextureFormat::Bc2RgbaUnormSrgb => GpuTextureFormat::Bc2rgbaunormsrgb,
            wgpu_types::TextureFormat::Bc3RgbaUnorm => GpuTextureFormat::Bc3rgbaunorm,
            wgpu_types::TextureFormat::Bc3RgbaUnormSrgb => GpuTextureFormat::Bc3rgbaunormsrgb,
            wgpu_types::TextureFormat::Bc4RUnorm => GpuTextureFormat::Bc4runorm,
            wgpu_types::TextureFormat::Bc4RSnorm => GpuTextureFormat::Bc4rsnorm,
            wgpu_types::TextureFormat::Bc5RgUnorm => GpuTextureFormat::Bc5rgunorm,
            wgpu_types::TextureFormat::Bc5RgSnorm => GpuTextureFormat::Bc5rgsnorm,
            wgpu_types::TextureFormat::Bc6hRgbUfloat => GpuTextureFormat::Bc6hrgbufloat,
            wgpu_types::TextureFormat::Bc6hRgbFloat => GpuTextureFormat::Bc6hrgbfloat,
            wgpu_types::TextureFormat::Bc7RgbaUnorm => GpuTextureFormat::Bc7rgbaunorm,
            wgpu_types::TextureFormat::Bc7RgbaUnormSrgb => GpuTextureFormat::Bc7rgbaunormsrgb,
            wgpu_types::TextureFormat::Etc2Rgb8Unorm => GpuTextureFormat::Etc2rgb8unorm,
            wgpu_types::TextureFormat::Etc2Rgb8UnormSrgb => GpuTextureFormat::Etc2rgb8unormsrgb,
            wgpu_types::TextureFormat::Etc2Rgb8A1Unorm => GpuTextureFormat::Etc2rgb8a1unorm,
            wgpu_types::TextureFormat::Etc2Rgb8A1UnormSrgb => GpuTextureFormat::Etc2rgb8a1unormsrgb,
            wgpu_types::TextureFormat::Etc2Rgba8Unorm => GpuTextureFormat::Etc2rgba8unorm,
            wgpu_types::TextureFormat::Etc2Rgba8UnormSrgb => GpuTextureFormat::Etc2rgba8unormsrgb,
            wgpu_types::TextureFormat::EacR11Unorm => GpuTextureFormat::Eacr11unorm,
            wgpu_types::TextureFormat::EacR11Snorm => GpuTextureFormat::Eacr11snorm,
            wgpu_types::TextureFormat::EacRg11Unorm => GpuTextureFormat::Eacrg11unorm,
            wgpu_types::TextureFormat::EacRg11Snorm => GpuTextureFormat::Eacrg11snorm,
            wgpu_types::TextureFormat::Astc {
                block: wgpu_types::AstcBlock::B4x4,
                channel: wgpu_types::AstcChannel::Unorm,
            } => GpuTextureFormat::Astc4x4unorm,
            wgpu_types::TextureFormat::Astc {
                block: wgpu_types::AstcBlock::B4x4,
                channel: wgpu_types::AstcChannel::UnormSrgb,
            } => GpuTextureFormat::Astc4x4unormsrgb,
            wgpu_types::TextureFormat::Astc {
                block: wgpu_types::AstcBlock::B5x4,
                channel: wgpu_types::AstcChannel::Unorm,
            } => GpuTextureFormat::Astc5x4unorm,
            wgpu_types::TextureFormat::Astc {
                block: wgpu_types::AstcBlock::B5x4,
                channel: wgpu_types::AstcChannel::UnormSrgb,
            } => GpuTextureFormat::Astc5x4unormsrgb,
            wgpu_types::TextureFormat::Astc {
                block: wgpu_types::AstcBlock::B5x5,
                channel: wgpu_types::AstcChannel::Unorm,
            } => GpuTextureFormat::Astc5x5unorm,
            wgpu_types::TextureFormat::Astc {
                block: wgpu_types::AstcBlock::B5x5,
                channel: wgpu_types::AstcChannel::UnormSrgb,
            } => GpuTextureFormat::Astc5x5unormsrgb,
            wgpu_types::TextureFormat::Astc {
                block: wgpu_types::AstcBlock::B6x5,
                channel: wgpu_types::AstcChannel::Unorm,
            } => GpuTextureFormat::Astc6x5unorm,
            wgpu_types::TextureFormat::Astc {
                block: wgpu_types::AstcBlock::B6x5,
                channel: wgpu_types::AstcChannel::UnormSrgb,
            } => GpuTextureFormat::Astc6x5unormsrgb,
            wgpu_types::TextureFormat::Astc {
                block: wgpu_types::AstcBlock::B6x6,
                channel: wgpu_types::AstcChannel::Unorm,
            } => GpuTextureFormat::Astc6x6unorm,
            wgpu_types::TextureFormat::Astc {
                block: wgpu_types::AstcBlock::B6x6,
                channel: wgpu_types::AstcChannel::UnormSrgb,
            } => GpuTextureFormat::Astc6x6unormsrgb,
            wgpu_types::TextureFormat::Astc {
                block: wgpu_types::AstcBlock::B8x5,
                channel: wgpu_types::AstcChannel::Unorm,
            } => GpuTextureFormat::Astc8x5unorm,
            wgpu_types::TextureFormat::Astc {
                block: wgpu_types::AstcBlock::B8x5,
                channel: wgpu_types::AstcChannel::UnormSrgb,
            } => GpuTextureFormat::Astc8x5unormsrgb,
            wgpu_types::TextureFormat::Astc {
                block: wgpu_types::AstcBlock::B8x6,
                channel: wgpu_types::AstcChannel::Unorm,
            } => GpuTextureFormat::Astc8x6unorm,
            wgpu_types::TextureFormat::Astc {
                block: wgpu_types::AstcBlock::B8x6,
                channel: wgpu_types::AstcChannel::UnormSrgb,
            } => GpuTextureFormat::Astc8x6unormsrgb,
            wgpu_types::TextureFormat::Astc {
                block: wgpu_types::AstcBlock::B8x8,
                channel: wgpu_types::AstcChannel::Unorm,
            } => GpuTextureFormat::Astc8x8unorm,
            wgpu_types::TextureFormat::Astc {
                block: wgpu_types::AstcBlock::B8x8,
                channel: wgpu_types::AstcChannel::UnormSrgb,
            } => GpuTextureFormat::Astc8x8unormsrgb,
            wgpu_types::TextureFormat::Astc {
                block: wgpu_types::AstcBlock::B10x5,
                channel: wgpu_types::AstcChannel::Unorm,
            } => GpuTextureFormat::Astc10x5unorm,
            wgpu_types::TextureFormat::Astc {
                block: wgpu_types::AstcBlock::B10x5,
                channel: wgpu_types::AstcChannel::UnormSrgb,
            } => GpuTextureFormat::Astc10x5unormsrgb,
            wgpu_types::TextureFormat::Astc {
                block: wgpu_types::AstcBlock::B10x6,
                channel: wgpu_types::AstcChannel::Unorm,
            } => GpuTextureFormat::Astc10x6unorm,
            wgpu_types::TextureFormat::Astc {
                block: wgpu_types::AstcBlock::B10x6,
                channel: wgpu_types::AstcChannel::UnormSrgb,
            } => GpuTextureFormat::Astc10x6unormsrgb,
            wgpu_types::TextureFormat::Astc {
                block: wgpu_types::AstcBlock::B10x8,
                channel: wgpu_types::AstcChannel::Unorm,
            } => GpuTextureFormat::Astc10x8unorm,
            wgpu_types::TextureFormat::Astc {
                block: wgpu_types::AstcBlock::B10x8,
                channel: wgpu_types::AstcChannel::UnormSrgb,
            } => GpuTextureFormat::Astc10x8unormsrgb,
            wgpu_types::TextureFormat::Astc {
                block: wgpu_types::AstcBlock::B10x10,
                channel: wgpu_types::AstcChannel::Unorm,
            } => GpuTextureFormat::Astc10x10unorm,
            wgpu_types::TextureFormat::Astc {
                block: wgpu_types::AstcBlock::B10x10,
                channel: wgpu_types::AstcChannel::UnormSrgb,
            } => GpuTextureFormat::Astc10x10unormsrgb,
            wgpu_types::TextureFormat::Astc {
                block: wgpu_types::AstcBlock::B12x10,
                channel: wgpu_types::AstcChannel::Unorm,
            } => GpuTextureFormat::Astc12x10unorm,
            wgpu_types::TextureFormat::Astc {
                block: wgpu_types::AstcBlock::B12x10,
                channel: wgpu_types::AstcChannel::UnormSrgb,
            } => GpuTextureFormat::Astc12x10unormsrgb,
            wgpu_types::TextureFormat::Astc {
                block: wgpu_types::AstcBlock::B12x12,
                channel: wgpu_types::AstcChannel::Unorm,
            } => GpuTextureFormat::Astc12x12unorm,
            wgpu_types::TextureFormat::Astc {
                block: wgpu_types::AstcBlock::B12x12,
                channel: wgpu_types::AstcChannel::UnormSrgb,
            } => GpuTextureFormat::Astc12x12unormsrgb,
            _ => panic!("Texture format not supported"),
        }
    }
}

impl Into<wgpu_types::TextureDimension> for GpuTextureDimension {
    fn into(self) -> wgpu_types::TextureDimension {
        match self {
            GpuTextureDimension::D1 => wgpu_types::TextureDimension::D1,
            GpuTextureDimension::D2 => wgpu_types::TextureDimension::D2,
            GpuTextureDimension::D3 => wgpu_types::TextureDimension::D3,
        }
    }
}

impl Into<GpuTextureDimension> for wgpu_types::TextureDimension {
    fn into(self) -> GpuTextureDimension {
        match self {
            wgpu_types::TextureDimension::D1 => GpuTextureDimension::D1,
            wgpu_types::TextureDimension::D2 => GpuTextureDimension::D2,
            wgpu_types::TextureDimension::D3 => GpuTextureDimension::D3,
        }
    }
}

impl Into<wgpu_types::TextureViewDimension> for GpuTextureViewDimension {
    fn into(self) -> wgpu_types::TextureViewDimension {
        match self {
            GpuTextureViewDimension::D1 => wgpu_types::TextureViewDimension::D1,
            GpuTextureViewDimension::D2 => wgpu_types::TextureViewDimension::D2,
            GpuTextureViewDimension::D2Array => wgpu_types::TextureViewDimension::D2Array,
            GpuTextureViewDimension::Cube => wgpu_types::TextureViewDimension::Cube,
            GpuTextureViewDimension::CubeArray => wgpu_types::TextureViewDimension::CubeArray,
            GpuTextureViewDimension::D3 => wgpu_types::TextureViewDimension::D3,
        }
    }
}

impl Into<wgpu_types::VertexFormat> for GpuVertexFormat {
    fn into(self) -> wgpu_types::VertexFormat {
        match self {
            GpuVertexFormat::Uint8 => wgpu_types::VertexFormat::Uint8,
            GpuVertexFormat::Uint8x2 => wgpu_types::VertexFormat::Uint8x2,
            GpuVertexFormat::Uint8x4 => wgpu_types::VertexFormat::Uint8x4,
            GpuVertexFormat::Sint8 => wgpu_types::VertexFormat::Sint8,
            GpuVertexFormat::Sint8x2 => wgpu_types::VertexFormat::Sint8x2,
            GpuVertexFormat::Sint8x4 => wgpu_types::VertexFormat::Sint8x4,
            GpuVertexFormat::Unorm8 => wgpu_types::VertexFormat::Unorm8,
            GpuVertexFormat::Unorm8x2 => wgpu_types::VertexFormat::Unorm8x2,
            GpuVertexFormat::Unorm8x4 => wgpu_types::VertexFormat::Unorm8x4,
            GpuVertexFormat::Snorm8 => wgpu_types::VertexFormat::Snorm8,
            GpuVertexFormat::Snorm8x2 => wgpu_types::VertexFormat::Snorm8x2,
            GpuVertexFormat::Snorm8x4 => wgpu_types::VertexFormat::Snorm8x4,
            GpuVertexFormat::Uint16 => wgpu_types::VertexFormat::Uint16,
            GpuVertexFormat::Uint16x2 => wgpu_types::VertexFormat::Uint16x2,
            GpuVertexFormat::Uint16x4 => wgpu_types::VertexFormat::Uint16x4,
            GpuVertexFormat::Sint16 => wgpu_types::VertexFormat::Sint16,
            GpuVertexFormat::Sint16x2 => wgpu_types::VertexFormat::Sint16x2,
            GpuVertexFormat::Sint16x4 => wgpu_types::VertexFormat::Sint16x4,
            GpuVertexFormat::Unorm16 => wgpu_types::VertexFormat::Unorm16,
            GpuVertexFormat::Unorm16x2 => wgpu_types::VertexFormat::Unorm16x2,
            GpuVertexFormat::Unorm16x4 => wgpu_types::VertexFormat::Unorm16x4,
            GpuVertexFormat::Snorm16 => wgpu_types::VertexFormat::Snorm16,
            GpuVertexFormat::Snorm16x2 => wgpu_types::VertexFormat::Snorm16x2,
            GpuVertexFormat::Snorm16x4 => wgpu_types::VertexFormat::Snorm16x4,
            GpuVertexFormat::Float16 => wgpu_types::VertexFormat::Float16,
            GpuVertexFormat::Float16x2 => wgpu_types::VertexFormat::Float16x2,
            GpuVertexFormat::Float16x4 => wgpu_types::VertexFormat::Float16x4,
            GpuVertexFormat::Float32 => wgpu_types::VertexFormat::Float32,
            GpuVertexFormat::Float32x2 => wgpu_types::VertexFormat::Float32x2,
            GpuVertexFormat::Float32x3 => wgpu_types::VertexFormat::Float32x3,
            GpuVertexFormat::Float32x4 => wgpu_types::VertexFormat::Float32x4,
            GpuVertexFormat::Uint32 => wgpu_types::VertexFormat::Uint32,
            GpuVertexFormat::Uint32x2 => wgpu_types::VertexFormat::Uint32x2,
            GpuVertexFormat::Uint32x3 => wgpu_types::VertexFormat::Uint32x3,
            GpuVertexFormat::Uint32x4 => wgpu_types::VertexFormat::Uint32x4,
            GpuVertexFormat::Sint32 => wgpu_types::VertexFormat::Sint32,
            GpuVertexFormat::Sint32x2 => wgpu_types::VertexFormat::Sint32x2,
            GpuVertexFormat::Sint32x3 => wgpu_types::VertexFormat::Sint32x3,
            GpuVertexFormat::Sint32x4 => wgpu_types::VertexFormat::Sint32x4,
        }
    }
}

impl Into<wgpu_types::VertexStepMode> for GpuVertexStepMode {
    fn into(self) -> wgpu_types::VertexStepMode {
        match self {
            GpuVertexStepMode::Vertex => wgpu_types::VertexStepMode::Vertex,
            GpuVertexStepMode::Instance => wgpu_types::VertexStepMode::Instance,
        }
    }
}

impl Into<wgpu_types::StencilOperation> for GpuStencilOperation {
    fn into(self) -> wgpu_types::StencilOperation {
        match self {
            GpuStencilOperation::Keep => wgpu_types::StencilOperation::Keep,
            GpuStencilOperation::Zero => wgpu_types::StencilOperation::Zero,
            GpuStencilOperation::Replace => wgpu_types::StencilOperation::Replace,
            GpuStencilOperation::Invert => wgpu_types::StencilOperation::Invert,
            GpuStencilOperation::IncrementClamp => wgpu_types::StencilOperation::IncrementClamp,
            GpuStencilOperation::DecrementClamp => wgpu_types::StencilOperation::DecrementClamp,
            GpuStencilOperation::IncrementWrap => wgpu_types::StencilOperation::IncrementWrap,
            GpuStencilOperation::DecrementWrap => wgpu_types::StencilOperation::DecrementWrap,
        }
    }
}

impl GpuLoadOp {
    pub fn into_wgt<V>(self, clear: V) -> LoadOp<V> {
        match self {
            GpuLoadOp::Clear => LoadOp::Clear(clear),
            GpuLoadOp::Load => LoadOp::Load,
        }
    }
}

impl<V> Into<GpuLoadOp> for LoadOp<V> {
    fn into(self) -> GpuLoadOp {
        match self {
            LoadOp::Clear(_clear) => GpuLoadOp::Clear,
            LoadOp::Load => GpuLoadOp::Load,
        }
    }
}

impl Into<StoreOp> for GpuStoreOp {
    fn into(self) -> StoreOp {
        match self {
            GpuStoreOp::Discard => StoreOp::Discard,
            GpuStoreOp::Store => StoreOp::Store,
        }
    }
}

impl Into<GpuStoreOp> for StoreOp {
    fn into(self) -> GpuStoreOp {
        match self {
            StoreOp::Discard => GpuStoreOp::Discard,
            StoreOp::Store => GpuStoreOp::Store,
        }
    }
}

impl Into<wgpu_types::StencilFaceState> for GpuStencilFaceState {
    fn into(self) -> wgpu_types::StencilFaceState {
        wgpu_types::StencilFaceState {
            compare: match self.compare {
                None => wgpu_types::CompareFunction::Always,
                Some(compare) => compare.into(),
            },
            fail_op: match self.fail_op {
                None => wgpu_types::StencilOperation::default(),
                Some(fail_op) => fail_op.into(),
            },
            depth_fail_op: match self.depth_fail_op {
                None => wgpu_types::StencilOperation::default(),
                Some(depth_fail_op) => depth_fail_op.into(),
            },
            pass_op: match self.pass_op {
                None => wgpu_types::StencilOperation::default(),
                Some(pass_op) => pass_op.into(),
            },
        }
    }
}

impl Into<wgpu_types::BlendState> for GpuBlendState {
    fn into(self) -> wgpu_types::BlendState {
        wgpu_types::BlendState {
            color: self.color.into(),
            alpha: self.alpha.into(),
        }
    }
}

impl Into<wgpu_types::BlendComponent> for GpuBlendComponent {
    fn into(self) -> wgpu_types::BlendComponent {
        wgpu_types::BlendComponent {
            src_factor: match self.src_factor {
                None => wgpu_types::BlendFactor::One,
                Some(src_factor) => src_factor.into(),
            },
            dst_factor: match self.dst_factor {
                None => wgpu_types::BlendFactor::Zero,
                Some(src_factor) => src_factor.into(),
            },
            operation: match self.operation {
                None => wgpu_types::BlendOperation::Add,
                Some(operation) => operation.into(),
            },
        }
    }
}

impl Into<wgpu_types::BlendFactor> for GpuBlendFactor {
    fn into(self) -> wgpu_types::BlendFactor {
        match self {
            GpuBlendFactor::Zero => wgpu_types::BlendFactor::Zero,
            GpuBlendFactor::One => wgpu_types::BlendFactor::One,
            GpuBlendFactor::Src => wgpu_types::BlendFactor::Src,
            GpuBlendFactor::OneMinusSrc => wgpu_types::BlendFactor::OneMinusSrc,
            GpuBlendFactor::SrcAlpha => wgpu_types::BlendFactor::SrcAlpha,
            GpuBlendFactor::OneMinusSrcAlpha => wgpu_types::BlendFactor::OneMinusSrcAlpha,
            GpuBlendFactor::Dst => wgpu_types::BlendFactor::Dst,
            GpuBlendFactor::OneMinusDst => wgpu_types::BlendFactor::OneMinusDst,
            GpuBlendFactor::DstAlpha => wgpu_types::BlendFactor::DstAlpha,
            GpuBlendFactor::OneMinusDstAlpha => wgpu_types::BlendFactor::OneMinusDstAlpha,
            GpuBlendFactor::SrcAlphaSaturated => wgpu_types::BlendFactor::SrcAlphaSaturated,
            GpuBlendFactor::Constant => wgpu_types::BlendFactor::Constant,
            GpuBlendFactor::OneMinusConstant => wgpu_types::BlendFactor::OneMinusConstant,
        }
    }
}

impl Into<wgpu_types::BlendOperation> for GpuBlendOperation {
    fn into(self) -> wgpu_types::BlendOperation {
        match self {
            GpuBlendOperation::Add => wgpu_types::BlendOperation::Add,
            GpuBlendOperation::Subtract => wgpu_types::BlendOperation::Subtract,
            GpuBlendOperation::ReverseSubtract => wgpu_types::BlendOperation::ReverseSubtract,
            GpuBlendOperation::Min => wgpu_types::BlendOperation::Min,
            GpuBlendOperation::Max => wgpu_types::BlendOperation::Max,
        }
    }
}

impl Into<wgpu_types::ColorWrites> for GpuColorWrite {
    fn into(self) -> wgpu_types::ColorWrites {
        let mut cw = wgpu_types::ColorWrites::empty();
        if self.contains(GpuColorWrite::RED) {
            cw |= wgpu_types::ColorWrites::RED;
        }
        if self.contains(GpuColorWrite::GREEN) {
            cw |= wgpu_types::ColorWrites::GREEN;
        }
        if self.contains(GpuColorWrite::BLUE) {
            cw |= wgpu_types::ColorWrites::BLUE;
        }
        if self.contains(GpuColorWrite::ALPHA) {
            cw |= wgpu_types::ColorWrites::ALPHA;
        }

        if self.contains(GpuColorWrite::ALL) {
            cw = wgpu_types::ColorWrites::all();
        }

        cw
    }
}

impl Into<wgpu_types::BufferUsages> for GpuBufferUsage {
    fn into(self) -> wgpu_types::BufferUsages {
        let mut cw = wgpu_types::BufferUsages::empty();

        if self.contains(GpuBufferUsage::MAP_READ) {
            cw |= wgpu_types::BufferUsages::MAP_READ;
        }
        if self.contains(GpuBufferUsage::MAP_WRITE) {
            cw |= wgpu_types::BufferUsages::MAP_WRITE;
        }
        if self.contains(GpuBufferUsage::COPY_SRC) {
            cw |= wgpu_types::BufferUsages::COPY_SRC;
        }
        if self.contains(GpuBufferUsage::COPY_DST) {
            cw |= wgpu_types::BufferUsages::COPY_DST;
        }
        if self.contains(GpuBufferUsage::INDEX) {
            cw |= wgpu_types::BufferUsages::INDEX;
        }
        if self.contains(GpuBufferUsage::VERTEX) {
            cw |= wgpu_types::BufferUsages::VERTEX;
        }
        if self.contains(GpuBufferUsage::UNIFORM) {
            cw |= wgpu_types::BufferUsages::UNIFORM;
        }
        if self.contains(GpuBufferUsage::INDIRECT) {
            cw |= wgpu_types::BufferUsages::INDIRECT;
        }
        if self.contains(GpuBufferUsage::QUERY_RESOLVE) {
            cw |= wgpu_types::BufferUsages::QUERY_RESOLVE;
        }
        if self.contains(GpuBufferUsage::COPY_DST) {
            cw |= wgpu_types::BufferUsages::COPY_DST;
        }

        cw
    }
}

impl Into<GpuBufferUsage> for wgpu_types::BufferUsages {
    fn into(self) -> GpuBufferUsage {
        let mut cw = GpuBufferUsage::empty();

        if self.contains(wgpu_types::BufferUsages::MAP_READ) {
            cw |= GpuBufferUsage::MAP_READ;
        }
        if self.contains(wgpu_types::BufferUsages::MAP_WRITE) {
            cw |= GpuBufferUsage::MAP_WRITE;
        }
        if self.contains(wgpu_types::BufferUsages::COPY_SRC) {
            cw |= GpuBufferUsage::COPY_SRC;
        }
        if self.contains(wgpu_types::BufferUsages::COPY_DST) {
            cw |= GpuBufferUsage::COPY_DST;
        }
        if self.contains(wgpu_types::BufferUsages::INDEX) {
            cw |= GpuBufferUsage::INDEX;
        }
        if self.contains(wgpu_types::BufferUsages::VERTEX) {
            cw |= GpuBufferUsage::VERTEX;
        }
        if self.contains(wgpu_types::BufferUsages::UNIFORM) {
            cw |= GpuBufferUsage::UNIFORM;
        }
        if self.contains(wgpu_types::BufferUsages::INDIRECT) {
            cw |= GpuBufferUsage::INDIRECT;
        }
        if self.contains(wgpu_types::BufferUsages::QUERY_RESOLVE) {
            cw |= GpuBufferUsage::QUERY_RESOLVE;
        }
        if self.contains(wgpu_types::BufferUsages::COPY_DST) {
            cw |= GpuBufferUsage::COPY_DST;
        }

        cw
    }
}

impl Into<wgpu_types::TextureUsages> for GpuTextureUsage {
    fn into(self) -> wgpu_types::TextureUsages {
        let mut cw = wgpu_types::TextureUsages::empty();
        if self.contains(GpuTextureUsage::COPY_SRC) {
            cw |= wgpu_types::TextureUsages::COPY_SRC;
        }
        if self.contains(GpuTextureUsage::COPY_DST) {
            cw |= wgpu_types::TextureUsages::COPY_DST;
        }
        if self.contains(GpuTextureUsage::TEXTURE_BINDING) {
            cw |= wgpu_types::TextureUsages::TEXTURE_BINDING;
        }
        if self.contains(GpuTextureUsage::STORAGE_BINDING) {
            cw |= wgpu_types::TextureUsages::STORAGE_BINDING;
        }
        if self.contains(GpuTextureUsage::RENDER_ATTACHMENT) {
            cw |= wgpu_types::TextureUsages::RENDER_ATTACHMENT;
        }

        cw
    }
}

impl Into<GpuTextureUsage> for wgpu_types::TextureUsages {
    fn into(self) -> GpuTextureUsage {
        let mut cw = GpuTextureUsage::empty();
        if self.contains(wgpu_types::TextureUsages::COPY_SRC) {
            cw |= GpuTextureUsage::COPY_SRC;
        }
        if self.contains(wgpu_types::TextureUsages::COPY_DST) {
            cw |= GpuTextureUsage::COPY_DST;
        }
        if self.contains(wgpu_types::TextureUsages::TEXTURE_BINDING) {
            cw |= GpuTextureUsage::TEXTURE_BINDING;
        }
        if self.contains(wgpu_types::TextureUsages::STORAGE_BINDING) {
            cw |= GpuTextureUsage::STORAGE_BINDING;
        }
        if self.contains(wgpu_types::TextureUsages::RENDER_ATTACHMENT) {
            cw |= GpuTextureUsage::RENDER_ATTACHMENT;
        }

        cw
    }
}

impl Into<wgpu_types::QueryType> for GpuQueryType {
    fn into(self) -> wgpu_types::QueryType {
        match self {
            GpuQueryType::Occlusion => wgpu_types::QueryType::Occlusion,
            GpuQueryType::Timestamp => wgpu_types::QueryType::Timestamp,
        }
    }
}

impl Into<GpuQueryType> for wgpu_types::QueryType {
    fn into(self) -> GpuQueryType {
        match self {
            wgpu_types::QueryType::Occlusion => GpuQueryType::Occlusion,
            wgpu_types::QueryType::Timestamp => GpuQueryType::Timestamp,
            _ => panic!("Query type not supported"),
        }
    }
}

impl Into<HostMap> for GpuMapMode {
    fn into(self) -> HostMap {
        if self.contains(GpuMapMode::READ) {
            HostMap::Read
        } else {
            HostMap::Write
        }
    }
}

pub fn convert_bind_group_entry(
    resource_table: &ResourceTable,
    entry: GpuBindGroupEntry,
) -> BindGroupEntry {
    BindGroupEntry {
        binding: entry.binding,
        resource: match entry.resource {
            crate::gpu::GpuBindingResource::Buffer(buffer_binding) => {
                BindingResource::Buffer(BufferBinding {
                    buffer_id: resource_table
                        .get(&buffer_binding.buffer)
                        .unwrap()
                        .to_owned(),
                    offset: buffer_binding.offset,
                    size: buffer_binding.size.map(|s| NonZeroU64::new(s).unwrap()),
                })
            }
            crate::gpu::GpuBindingResource::BufferArray(buffers) => {
                BindingResource::BufferArray(Cow::Owned(
                    buffers
                        .iter()
                        .map(|b| BufferBinding {
                            buffer_id: resource_table.get(&b.buffer).unwrap().to_owned(),
                            offset: b.offset,
                            size: b.size.map(|s| NonZeroU64::new(s).unwrap()),
                        })
                        .collect(),
                ))
            }
            crate::gpu::GpuBindingResource::Sampler(sampler) => {
                BindingResource::Sampler(resource_table.get(&sampler).unwrap().to_owned())
            }
            crate::gpu::GpuBindingResource::SamplerArray(samplers) => {
                BindingResource::SamplerArray(Cow::Owned(
                    samplers
                        .iter()
                        .map(|s| resource_table.get(&s).unwrap().to_owned())
                        .collect(),
                ))
            }
            crate::gpu::GpuBindingResource::TextureView(texture_view) => {
                BindingResource::TextureView(resource_table.get(&texture_view).unwrap().to_owned())
            }
            crate::gpu::GpuBindingResource::TextureViewArray(texture_views) => {
                BindingResource::TextureViewArray(Cow::Owned(
                    texture_views
                        .iter()
                        .map(|v| resource_table.get(&v).unwrap().to_owned())
                        .collect(),
                ))
            }
        },
    }
}

impl Into<crate::audio::BiquadFilterType> for web_audio_api::node::BiquadFilterType {
    fn into(self) -> crate::audio::BiquadFilterType {
        match self {
            web_audio_api::node::BiquadFilterType::Allpass => {
                crate::audio::BiquadFilterType::Allpass
            }
            web_audio_api::node::BiquadFilterType::Lowpass => {
                crate::audio::BiquadFilterType::Lowpass
            }
            web_audio_api::node::BiquadFilterType::Highpass => {
                crate::audio::BiquadFilterType::Highpass
            }
            web_audio_api::node::BiquadFilterType::Bandpass => {
                crate::audio::BiquadFilterType::Bandpass
            }
            web_audio_api::node::BiquadFilterType::Notch => crate::audio::BiquadFilterType::Notch,
            web_audio_api::node::BiquadFilterType::Peaking => {
                crate::audio::BiquadFilterType::Peaking
            }
            web_audio_api::node::BiquadFilterType::Lowshelf => {
                crate::audio::BiquadFilterType::Lowshelf
            }
            web_audio_api::node::BiquadFilterType::Highshelf => {
                crate::audio::BiquadFilterType::Highshelf
            }
        }
    }
}

impl Into<web_audio_api::node::BiquadFilterType> for crate::audio::BiquadFilterType {
    fn into(self) -> web_audio_api::node::BiquadFilterType {
        match self {
            crate::audio::BiquadFilterType::Allpass => {
                web_audio_api::node::BiquadFilterType::Allpass
            }
            crate::audio::BiquadFilterType::Lowpass => {
                web_audio_api::node::BiquadFilterType::Lowpass
            }
            crate::audio::BiquadFilterType::Highpass => {
                web_audio_api::node::BiquadFilterType::Highpass
            }
            crate::audio::BiquadFilterType::Bandpass => {
                web_audio_api::node::BiquadFilterType::Bandpass
            }
            crate::audio::BiquadFilterType::Notch => web_audio_api::node::BiquadFilterType::Notch,
            crate::audio::BiquadFilterType::Peaking => {
                web_audio_api::node::BiquadFilterType::Peaking
            }
            crate::audio::BiquadFilterType::Lowshelf => {
                web_audio_api::node::BiquadFilterType::Lowshelf
            }
            crate::audio::BiquadFilterType::Highshelf => {
                web_audio_api::node::BiquadFilterType::Highshelf
            }
        }
    }
}

impl Into<crate::audio::AutomationRate> for web_audio_api::AutomationRate {
    fn into(self) -> crate::audio::AutomationRate {
        match self {
            web_audio_api::AutomationRate::A => crate::audio::AutomationRate::A,
            web_audio_api::AutomationRate::K => crate::audio::AutomationRate::K,
        }
    }
}

impl Into<web_audio_api::AutomationRate> for crate::audio::AutomationRate {
    fn into(self) -> web_audio_api::AutomationRate {
        match self {
            crate::audio::AutomationRate::A => web_audio_api::AutomationRate::A,
            crate::audio::AutomationRate::K => web_audio_api::AutomationRate::K,
        }
    }
}

impl Into<crate::audio::OscillatorType> for web_audio_api::node::OscillatorType {
    fn into(self) -> crate::audio::OscillatorType {
        match self {
            web_audio_api::node::OscillatorType::Sine => crate::audio::OscillatorType::Sine,
            web_audio_api::node::OscillatorType::Square => crate::audio::OscillatorType::Square,
            web_audio_api::node::OscillatorType::Sawtooth => crate::audio::OscillatorType::Sawtooth,
            web_audio_api::node::OscillatorType::Triangle => crate::audio::OscillatorType::Triangle,
            web_audio_api::node::OscillatorType::Custom => crate::audio::OscillatorType::Custom,
        }
    }
}

impl Into<web_audio_api::node::OscillatorType> for crate::audio::OscillatorType {
    fn into(self) -> web_audio_api::node::OscillatorType {
        match self {
            crate::audio::OscillatorType::Sine => web_audio_api::node::OscillatorType::Sine,
            crate::audio::OscillatorType::Square => web_audio_api::node::OscillatorType::Square,
            crate::audio::OscillatorType::Sawtooth => web_audio_api::node::OscillatorType::Sawtooth,
            crate::audio::OscillatorType::Triangle => web_audio_api::node::OscillatorType::Triangle,
            crate::audio::OscillatorType::Custom => web_audio_api::node::OscillatorType::Custom,
        }
    }
}

impl Into<crate::audio::DistanceModelType> for web_audio_api::node::DistanceModelType {
    fn into(self) -> crate::audio::DistanceModelType {
        match self {
            web_audio_api::node::DistanceModelType::Linear => {
                crate::audio::DistanceModelType::Linear
            }
            web_audio_api::node::DistanceModelType::Inverse => {
                crate::audio::DistanceModelType::Inverse
            }
            web_audio_api::node::DistanceModelType::Exponential => {
                crate::audio::DistanceModelType::Exponential
            }
        }
    }
}

impl Into<web_audio_api::node::DistanceModelType> for crate::audio::DistanceModelType {
    fn into(self) -> web_audio_api::node::DistanceModelType {
        match self {
            crate::audio::DistanceModelType::Linear => {
                web_audio_api::node::DistanceModelType::Linear
            }
            crate::audio::DistanceModelType::Inverse => {
                web_audio_api::node::DistanceModelType::Inverse
            }
            crate::audio::DistanceModelType::Exponential => {
                web_audio_api::node::DistanceModelType::Exponential
            }
        }
    }
}

impl Into<web_audio_api::node::PanningModelType> for crate::audio::PanningModelType {
    fn into(self) -> web_audio_api::node::PanningModelType {
        match self {
            crate::audio::PanningModelType::EqualPower => {
                web_audio_api::node::PanningModelType::EqualPower
            }
            crate::audio::PanningModelType::Hrtf => web_audio_api::node::PanningModelType::HRTF,
        }
    }
}

impl Into<crate::audio::PanningModelType> for web_audio_api::node::PanningModelType {
    fn into(self) -> crate::audio::PanningModelType {
        match self {
            web_audio_api::node::PanningModelType::EqualPower => {
                crate::audio::PanningModelType::EqualPower
            }
            web_audio_api::node::PanningModelType::HRTF => crate::audio::PanningModelType::Hrtf,
        }
    }
}

impl Into<web_audio_api::node::OverSampleType> for crate::audio::OverSampleType {
    fn into(self) -> web_audio_api::node::OverSampleType {
        match self {
            crate::audio::OverSampleType::None => web_audio_api::node::OverSampleType::None,
            crate::audio::OverSampleType::X2 => web_audio_api::node::OverSampleType::X2,
            crate::audio::OverSampleType::X4 => web_audio_api::node::OverSampleType::X4,
        }
    }
}

impl Into<crate::audio::OverSampleType> for web_audio_api::node::OverSampleType {
    fn into(self) -> crate::audio::OverSampleType {
        match self {
            web_audio_api::node::OverSampleType::None => crate::audio::OverSampleType::None,
            web_audio_api::node::OverSampleType::X2 => crate::audio::OverSampleType::X2,
            web_audio_api::node::OverSampleType::X4 => crate::audio::OverSampleType::X4,
        }
    }
}

impl Into<web_audio_api::PeriodicWaveOptions> for crate::audio::PeriodicWaveOptions {
    fn into(self) -> web_audio_api::PeriodicWaveOptions {
        web_audio_api::PeriodicWaveOptions {
            real: self.real,
            imag: self.imag,
            disable_normalization: self.disable_normalization,
        }
    }
}

impl Into<crate::audio::PeriodicWaveOptions> for web_audio_api::PeriodicWaveOptions {
    fn into(self) -> crate::audio::PeriodicWaveOptions {
        crate::audio::PeriodicWaveOptions {
            real: self.real,
            imag: self.imag,
            disable_normalization: self.disable_normalization,
        }
    }
}

impl Into<crate::audio::AudioContextState> for web_audio_api::context::AudioContextState {
    fn into(self) -> crate::audio::AudioContextState {
        match self {
            web_audio_api::context::AudioContextState::Suspended => {
                crate::audio::AudioContextState::Suspended
            }
            web_audio_api::context::AudioContextState::Running => {
                crate::audio::AudioContextState::Running
            }
            web_audio_api::context::AudioContextState::Closed => {
                crate::audio::AudioContextState::Closed
            }
        }
    }
}

impl Into<web_audio_api::context::AudioContextState> for crate::audio::AudioContextState {
    fn into(self) -> web_audio_api::context::AudioContextState {
        match self {
            crate::audio::AudioContextState::Suspended => {
                web_audio_api::context::AudioContextState::Suspended
            }
            crate::audio::AudioContextState::Running => {
                web_audio_api::context::AudioContextState::Running
            }
            crate::audio::AudioContextState::Closed => {
                web_audio_api::context::AudioContextState::Closed
            }
        }
    }
}
