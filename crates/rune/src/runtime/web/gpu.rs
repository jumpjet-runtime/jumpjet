//! `rune:runtime/gpu` host import as Rust `#[wasm_bindgen]` classes wrapping the
//! browser's WebGPU objects. jco requires imported resources to be real classes
//! (instanceof-checked); these are, and methods dispatch on `&self`.
//!
//! Each wrapper holds the underlying browser object as a `JsValue` and forwards
//! calls via `js_sys`. The WIT API mirrors WebGPU so most descriptor fields pass
//! through (`lower()` handles `u64`->Number, resource-unwrap, enum remaps, and
//! `visibility` flags); buffer/texture `usage`, color `writeMask`, and the
//! `gpu-layout`/`gpu-binding-resource` variants are handled explicitly.
//!
//! `request-adapter`/`request-device` are sync in WIT but async in the browser,
//! so the harness pre-resolves the device + canvas context and hands them in via
//! `gpu_set_context` before instantiating the guest.

use std::cell::RefCell;

use js_sys::{Array, Function, Object, Reflect, Uint8Array};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

// ---- pre-resolved GPU context (set by the harness before guest init) ----

struct GpuContext {
    device: JsValue,
    context: JsValue,
    surface_view_format: String,
}

thread_local! {
    static GPU_CTX: RefCell<Option<GpuContext>> = RefCell::new(None);
}

/// Called by the harness after `await`ing the adapter/device and configuring the
/// canvas context, before instantiating the guest.
#[wasm_bindgen(js_name = gpuSetContext)]
pub fn gpu_set_context(device: JsValue, context: JsValue, surface_view_format: String) {
    GPU_CTX.with(|c| {
        *c.borrow_mut() = Some(GpuContext { device, context, surface_view_format });
    });
}

fn device_handle() -> JsValue {
    GPU_CTX.with(|c| c.borrow().as_ref().expect("gpu context not set").device.clone())
}
fn context_handle() -> JsValue {
    GPU_CTX.with(|c| c.borrow().as_ref().expect("gpu context not set").context.clone())
}
fn surface_view_format() -> String {
    GPU_CTX.with(|c| c.borrow().as_ref().expect("gpu context not set").surface_view_format.clone())
}

// ---- js_sys helpers ----

fn get(o: &JsValue, k: &str) -> JsValue {
    Reflect::get(o, &JsValue::from_str(k)).unwrap_or(JsValue::UNDEFINED)
}
fn set(o: &Object, k: &str, v: JsValue) {
    let _ = Reflect::set(o, &JsValue::from_str(k), &v);
}
fn is_some(v: &JsValue) -> bool {
    !v.is_undefined() && !v.is_null()
}
fn f64v(n: f64) -> JsValue {
    JsValue::from_f64(n)
}
/// Coerce a JS number-or-bigint to f64 (jco lowers WIT u64 to bigint).
fn num(v: &JsValue) -> f64 {
    // `Number::new` (`new Number(v)`) coerces a bigint to a number; the suggested
    // `Number::from` throws on bigint, so we keep `new` despite its deprecation.
    #[allow(deprecated)]
    v.as_f64().unwrap_or_else(|| js_sys::Number::new(v).value_of())
}
fn call(obj: &JsValue, method: &str, args: &[JsValue]) -> JsValue {
    let f: Function = get(obj, method).unchecked_into();
    let arr = Array::new();
    for a in args {
        arr.push(a);
    }
    Reflect::apply(&f, obj, &arr).unwrap_or(JsValue::UNDEFINED)
}

/// Extracts the wrapped browser handle from a resource-wrapper instance passed
/// (nested in a descriptor) by jco. wasm-bindgen exported structs don't impl
/// `JsCast`, so each wrapper exposes its handle via a `__h` getter called here.
fn handle(v: &JsValue) -> JsValue {
    call(v, "__h", &[])
}

// ---- enum / flag conversions ----

fn fmt(v: &JsValue) -> JsValue {
    match v.as_string().as_deref() {
        Some("rgba8unormsrgb") => JsValue::from_str("rgba8unorm-srgb"),
        Some("bgra8unormsrgb") => JsValue::from_str("bgra8unorm-srgb"),
        Some("depth24plusstencil8") => JsValue::from_str("depth24plus-stencil8"),
        Some("depth32floatstencil8") => JsValue::from_str("depth32float-stencil8"),
        _ => v.clone(),
    }
}
fn dim(v: &JsValue) -> JsValue {
    match v.as_string().as_deref() {
        Some("d1") => JsValue::from_str("1d"),
        Some("d2") => JsValue::from_str("2d"),
        Some("d3") => JsValue::from_str("3d"),
        _ => v.clone(),
    }
}
fn remap_enum(v: &JsValue) -> JsValue {
    dim(&fmt(v))
}
/// Reverse of `fmt`/`dim`, for getters that return WebGPU strings the guest
/// expects in WIT spelling.
fn rev_enum(s: &str) -> String {
    match s {
        "rgba8unorm-srgb" => "rgba8unormsrgb",
        "bgra8unorm-srgb" => "bgra8unormsrgb",
        "depth24plus-stencil8" => "depth24plusstencil8",
        "depth32float-stencil8" => "depth32floatstencil8",
        "1d" => "d1",
        "2d" => "d2",
        "3d" => "d3",
        other => other,
    }
    .to_owned()
}
fn bits(o: &JsValue, fields: &[(&str, u32)]) -> u32 {
    let mut m = 0u32;
    for (k, bit) in fields {
        if get(o, k).as_bool().unwrap_or(false) {
            m |= bit;
        }
    }
    m
}
const BUFFER_USAGE: &[(&str, u32)] = &[
    ("mapRead", 1), ("mapWrite", 2), ("copySrc", 4), ("copyDst", 8), ("index", 16),
    ("vertex", 32), ("uniform", 64), ("storage", 128), ("indirect", 256), ("queryResolve", 512),
];
const TEXTURE_USAGE: &[(&str, u32)] = &[
    ("copySrc", 1), ("copyDst", 2), ("textureBinding", 4), ("storageBinding", 8), ("renderAttachment", 16),
];
fn buffer_usage(o: &JsValue) -> u32 {
    bits(o, BUFFER_USAGE)
}
fn texture_usage(o: &JsValue) -> u32 {
    bits(o, TEXTURE_USAGE)
}
fn color_write(o: &JsValue) -> u32 {
    if get(o, "all").as_bool().unwrap_or(false) {
        return 0xf;
    }
    bits(o, &[("red", 1), ("green", 2), ("blue", 4), ("alpha", 8)])
}
fn shader_stage(o: &JsValue) -> u32 {
    bits(o, &[("vertex", 1), ("fragment", 2), ("compute", 4)])
}
fn map_mode(o: &JsValue) -> u32 {
    bits(o, &[("read", 1), ("write", 2)])
}
/// Build a WIT flags-object (booleans) from a WebGPU numeric bitmask, for getters.
fn unbits(n: u32, fields: &[(&str, u32)]) -> JsValue {
    let o = Object::new();
    for (k, bit) in fields {
        set(&o, k, JsValue::from_bool(n & bit != 0));
    }
    o.into()
}

/// Generic descriptor lowering: bigint->Number, resource-wrapper->handle, enum
/// string remaps, `visibility` flags->bitmask, recursing arrays/objects. Used for
/// descriptors whose only flag field is `visibility`; callers handle `usage`,
/// `writeMask`, variants, and typed-array `data` fields explicitly.
fn lower(v: &JsValue) -> JsValue {
    if !is_some(v) {
        return v.clone();
    }
    match v.js_typeof().as_string().as_deref() {
        Some("bigint") => return f64v(num(v)),
        Some("string") => return remap_enum(v),
        Some("object") => {}
        _ => return v.clone(),
    }
    if Array::is_array(v) {
        let out = Array::new();
        Array::from(v).for_each(&mut |e, _, _| {
            out.push(&lower(&e));
        });
        return out.into();
    }
    if get(v, "__h").is_function() {
        return handle(v);
    }
    let out = Object::new();
    Object::keys(v.unchecked_ref::<Object>()).for_each(&mut |k, _, _| {
        let key = k.as_string().unwrap_or_default();
        let val = get(v, &key);
        let lowered = match key.as_str() {
            "visibility" => f64v(shader_stage(&val) as f64),
            _ => lower(&val),
        };
        set(&out, &key, lowered);
    });
    out.into()
}

/// Lowers a `gpu-layout` variant (`{tag:'auto'}` | `{tag:'pipeline',val}`).
fn lower_layout(v: &JsValue) -> JsValue {
    if get(v, "tag").as_string().as_deref() == Some("pipeline") {
        handle(&get(v, "val"))
    } else {
        JsValue::from_str("auto")
    }
}

// ---- resource classes ----

#[wasm_bindgen]
pub struct GpuAdapter {}
#[wasm_bindgen]
impl GpuAdapter {
    #[wasm_bindgen(js_name = requestDevice)]
    pub fn request_device(&self) -> GpuDevice {
        GpuDevice { inner: device_handle() }
    }
}

#[wasm_bindgen]
pub struct GpuDevice {
    inner: JsValue,
}
#[wasm_bindgen]
impl GpuDevice {
    pub fn queue(&self) -> GpuQueue {
        GpuQueue { inner: get(&self.inner, "queue") }
    }

    #[wasm_bindgen(js_name = createBuffer)]
    pub fn create_buffer(&self, d: JsValue) -> GpuBuffer {
        let desc = Object::new();
        set(&desc, "size", f64v(num(&get(&d, "size"))));
        set(&desc, "usage", f64v(buffer_usage(&get(&d, "usage")) as f64));
        let mapped = get(&d, "mappedAtCreation").as_bool().unwrap_or(false);
        set(&desc, "mappedAtCreation", JsValue::from_bool(mapped));
        let buf = call(&self.inner, "createBuffer", &[desc.into()]);

        // WIT adds a `contents` field for convenient initial upload.
        let contents = get(&d, "contents");
        if is_some(&contents) {
            let range = call(&buf, "getMappedRange", &[]);
            Uint8Array::new(&range).set(&Uint8Array::new(&contents), 0);
            call(&buf, "unmap", &[]);
        }
        GpuBuffer { inner: buf }
    }

    #[wasm_bindgen(js_name = createTexture)]
    pub fn create_texture(&self, d: JsValue) -> GpuTexture {
        let size_in = get(&d, "size");
        let size = Object::new();
        set(&size, "width", get(&size_in, "width"));
        set(&size, "height", get(&size_in, "height"));
        set(&size, "depthOrArrayLayers", get(&size_in, "depthOrArrayLayers"));

        let desc = Object::new();
        set(&desc, "size", size.into());
        set(&desc, "mipLevelCount", get(&d, "mipLevelCount"));
        set(&desc, "sampleCount", get(&d, "sampleCount"));
        set(&desc, "dimension", dim(&get(&d, "dimension")));
        set(&desc, "format", fmt(&get(&d, "format")));
        set(&desc, "usage", f64v(texture_usage(&get(&d, "usage")) as f64));
        let view_formats = get(&d, "viewFormats");
        if is_some(&view_formats) {
            let mapped = Array::new();
            Array::from(&view_formats).for_each(&mut |v, _, _| {
                mapped.push(&fmt(&v));
            });
            set(&desc, "viewFormats", mapped.into());
        }
        GpuTexture { inner: call(&self.inner, "createTexture", &[desc.into()]), is_surface: false }
    }

    #[wasm_bindgen(js_name = createSampler)]
    pub fn create_sampler(&self, d: JsValue) -> GpuSampler {
        GpuSampler { inner: call(&self.inner, "createSampler", &[lower(&d)]) }
    }

    #[wasm_bindgen(js_name = createBindGroupLayout)]
    pub fn create_bind_group_layout(&self, d: JsValue) -> GpuBindGroupLayout {
        GpuBindGroupLayout { inner: call(&self.inner, "createBindGroupLayout", &[lower(&d)]) }
    }

    #[wasm_bindgen(js_name = createPipelineLayout)]
    pub fn create_pipeline_layout(&self, d: JsValue) -> GpuPipelineLayout {
        GpuPipelineLayout { inner: call(&self.inner, "createPipelineLayout", &[lower(&d)]) }
    }

    #[wasm_bindgen(js_name = createShaderModule)]
    pub fn create_shader_module(&self, d: JsValue) -> GpuShaderModule {
        let desc = Object::new();
        set(&desc, "code", get(&d, "code"));
        GpuShaderModule { inner: call(&self.inner, "createShaderModule", &[desc.into()]) }
    }

    #[wasm_bindgen(js_name = createBindGroup)]
    pub fn create_bind_group(&self, d: JsValue) -> GpuBindGroup {
        let entries = Array::new();
        Array::from(&get(&d, "entries")).for_each(&mut |e, _, _| {
            let out = Object::new();
            set(&out, "binding", get(&e, "binding"));
            set(&out, "resource", binding_resource(&get(&e, "resource")));
            entries.push(&out);
        });
        let desc = Object::new();
        set(&desc, "layout", handle(&get(&d, "layout")));
        set(&desc, "entries", entries.into());
        GpuBindGroup { inner: call(&self.inner, "createBindGroup", &[desc.into()]) }
    }

    #[wasm_bindgen(js_name = createComputePipeline)]
    pub fn create_compute_pipeline(&self, d: JsValue) -> GpuComputePipeline {
        let cin = get(&d, "compute");
        let compute = Object::new();
        set(&compute, "module", handle(&get(&cin, "module")));
        set(&compute, "entryPoint", get(&cin, "entryPoint"));
        let desc = Object::new();
        set(&desc, "layout", lower_layout(&get(&d, "layout")));
        set(&desc, "compute", compute.into());
        GpuComputePipeline { inner: call(&self.inner, "createComputePipeline", &[desc.into()]) }
    }

    #[wasm_bindgen(js_name = createRenderPipeline)]
    pub fn create_render_pipeline(&self, d: JsValue) -> GpuRenderPipeline {
        GpuRenderPipeline { inner: call(&self.inner, "createRenderPipeline", &[render_pipeline_desc(&d)]) }
    }

    #[wasm_bindgen(js_name = createCommandEncoder)]
    pub fn create_command_encoder(&self, _d: JsValue) -> GpuCommandEncoder {
        GpuCommandEncoder { inner: call(&self.inner, "createCommandEncoder", &[Object::new().into()]) }
    }

    #[wasm_bindgen(js_name = createRenderBundleEncoder)]
    pub fn create_render_bundle_encoder(&self, d: JsValue) -> GpuRenderBundleEncoder {
        GpuRenderBundleEncoder { inner: call(&self.inner, "createRenderBundleEncoder", &[lower(&d)]) }
    }

    #[wasm_bindgen(js_name = createQuerySet)]
    pub fn create_query_set(&self, d: JsValue) -> GpuQuerySet {
        GpuQuerySet { inner: call(&self.inner, "createQuerySet", &[lower(&d)]) }
    }
}

#[wasm_bindgen]
pub struct GpuQueue {
    inner: JsValue,
}
#[wasm_bindgen]
impl GpuQueue {
    pub fn submit(&self, command_buffers: JsValue) {
        let mapped = Array::new();
        Array::from(&command_buffers).for_each(&mut |cb, _, _| {
            mapped.push(&handle(&cb));
        });
        call(&self.inner, "submit", &[mapped.into()]);
    }

    #[wasm_bindgen(js_name = writeBuffer)]
    pub fn write_buffer(&self, buffer: &GpuBuffer, buffer_offset: JsValue, data: JsValue, data_offset: JsValue, size: JsValue) {
        call(&self.inner, "writeBuffer", &[
            buffer.inner.clone(),
            f64v(num(&buffer_offset)),
            data,
            f64v(num(&data_offset)),
            f64v(num(&size)),
        ]);
    }

    #[wasm_bindgen(js_name = writeTexture)]
    pub fn write_texture(&self, destination: JsValue, data: JsValue, data_layout: JsValue, size: JsValue) {
        call(&self.inner, "writeTexture", &[
            lower(&destination),
            data, // buffer-source: pass the typed array through unchanged
            lower(&data_layout),
            lower(&size),
        ]);
    }
}

#[wasm_bindgen]
pub struct GpuBuffer {
    inner: JsValue,
}
#[wasm_bindgen]
impl GpuBuffer {
    #[wasm_bindgen(js_name = __h)]
    pub fn h(&self) -> JsValue { self.inner.clone() }
    pub fn size(&self) -> f64 { num(&get(&self.inner, "size")) }
    pub fn usage(&self) -> JsValue { unbits(num(&get(&self.inner, "usage")) as u32, BUFFER_USAGE) }
    #[wasm_bindgen(js_name = mapState)]
    pub fn map_state(&self) -> JsValue { get(&self.inner, "mapState") }
    #[wasm_bindgen(js_name = map)]
    pub fn map(&self, mode: JsValue, offset: JsValue, size: JsValue) {
        // mapAsync is async in the browser; the sync WIT can't await it. Best-effort.
        call(&self.inner, "mapAsync", &[f64v(map_mode(&mode) as f64), f64v(num(&offset)), f64v(num(&size))]);
    }
    #[wasm_bindgen(js_name = getMappedRange)]
    pub fn get_mapped_range(&self, offset: JsValue, size: JsValue) -> Vec<u8> {
        let range = call(&self.inner, "getMappedRange", &[f64v(num(&offset)), f64v(num(&size))]);
        Uint8Array::new(&range).to_vec()
    }
    pub fn unmap(&self) { call(&self.inner, "unmap", &[]); }
    pub fn destroy(&self) { call(&self.inner, "destroy", &[]); }
}

#[wasm_bindgen]
pub struct GpuTexture {
    inner: JsValue,
    is_surface: bool,
}
#[wasm_bindgen]
impl GpuTexture {
    pub fn width(&self) -> u32 { num(&get(&self.inner, "width")) as u32 }
    pub fn height(&self) -> u32 { num(&get(&self.inner, "height")) as u32 }
    #[wasm_bindgen(js_name = depthOrArrayLayers)]
    pub fn depth_or_array_layers(&self) -> u32 { num(&get(&self.inner, "depthOrArrayLayers")) as u32 }
    #[wasm_bindgen(js_name = mipLevelCount)]
    pub fn mip_level_count(&self) -> u32 { num(&get(&self.inner, "mipLevelCount")) as u32 }
    #[wasm_bindgen(js_name = sampleCount)]
    pub fn sample_count(&self) -> u32 { num(&get(&self.inner, "sampleCount")) as u32 }
    pub fn dimension(&self) -> String { rev_enum(&get(&self.inner, "dimension").as_string().unwrap_or_default()) }
    pub fn format(&self) -> String { rev_enum(&get(&self.inner, "format").as_string().unwrap_or_default()) }
    pub fn usage(&self) -> JsValue { unbits(num(&get(&self.inner, "usage")) as u32, TEXTURE_USAGE) }
    #[wasm_bindgen(js_name = createView)]
    pub fn create_view(&self) -> GpuTextureView {
        let view = if self.is_surface {
            let opts = Object::new();
            set(&opts, "format", JsValue::from_str(&surface_view_format()));
            call(&self.inner, "createView", &[opts.into()])
        } else {
            call(&self.inner, "createView", &[])
        };
        GpuTextureView { inner: view }
    }
    pub fn destroy(&self) { call(&self.inner, "destroy", &[]); }
}

macro_rules! handle_class {
    ($name:ident) => {
        #[wasm_bindgen]
        pub struct $name {
            inner: JsValue,
        }
        #[wasm_bindgen]
        impl $name {
            #[wasm_bindgen(js_name = __h)]
            pub fn h(&self) -> JsValue {
                self.inner.clone()
            }
        }
    };
}

handle_class!(GpuTextureView);
handle_class!(GpuBindGroupLayout);
handle_class!(GpuBindGroup);
handle_class!(GpuPipelineLayout);
handle_class!(GpuSampler);
handle_class!(GpuCommandBuffer);
handle_class!(GpuRenderBundle);

#[wasm_bindgen]
pub struct GpuShaderModule {
    inner: JsValue,
}
#[wasm_bindgen]
impl GpuShaderModule {
    #[wasm_bindgen(js_name = __h)]
    pub fn h(&self) -> JsValue { self.inner.clone() }
    #[wasm_bindgen(js_name = getCompilationInfo)]
    pub fn get_compilation_info(&self) -> JsValue {
        // Returns the compilation info object as-is (messages array).
        call(&self.inner, "getCompilationInfo", &[])
    }
}

#[wasm_bindgen]
pub struct GpuComputePipeline {
    inner: JsValue,
}
#[wasm_bindgen]
impl GpuComputePipeline {
    #[wasm_bindgen(js_name = __h)]
    pub fn h(&self) -> JsValue { self.inner.clone() }
    #[wasm_bindgen(js_name = getBindGroupLayout)]
    pub fn get_bind_group_layout(&self, index: u32) -> GpuBindGroupLayout {
        GpuBindGroupLayout { inner: call(&self.inner, "getBindGroupLayout", &[f64v(index as f64)]) }
    }
}

#[wasm_bindgen]
pub struct GpuRenderPipeline {
    inner: JsValue,
}
#[wasm_bindgen]
impl GpuRenderPipeline {
    #[wasm_bindgen(js_name = __h)]
    pub fn h(&self) -> JsValue { self.inner.clone() }
    #[wasm_bindgen(js_name = getBindGroupLayout)]
    pub fn get_bind_group_layout(&self, index: u32) -> GpuBindGroupLayout {
        GpuBindGroupLayout { inner: call(&self.inner, "getBindGroupLayout", &[f64v(index as f64)]) }
    }
}

#[wasm_bindgen]
pub struct GpuQuerySet {
    inner: JsValue,
}
#[wasm_bindgen]
impl GpuQuerySet {
    #[wasm_bindgen(js_name = __h)]
    pub fn h(&self) -> JsValue { self.inner.clone() }
    #[wasm_bindgen(js_name = type)]
    pub fn type_(&self) -> JsValue { get(&self.inner, "type") }
    pub fn count(&self) -> u32 { num(&get(&self.inner, "count")) as u32 }
    pub fn destroy(&self) { call(&self.inner, "destroy", &[]); }
}

#[wasm_bindgen]
pub struct GpuCommandEncoder {
    inner: JsValue,
}
#[wasm_bindgen]
impl GpuCommandEncoder {
    #[wasm_bindgen(js_name = beginRenderPass)]
    pub fn begin_render_pass(&self, d: JsValue) -> GpuRenderPassEncoder {
        GpuRenderPassEncoder { inner: call(&self.inner, "beginRenderPass", &[render_pass_desc(&d)]) }
    }
    #[wasm_bindgen(js_name = beginComputePass)]
    pub fn begin_compute_pass(&self, d: JsValue) -> GpuComputePassEncoder {
        let desc = if is_some(&d) { lower(&d) } else { Object::new().into() };
        GpuComputePassEncoder { inner: call(&self.inner, "beginComputePass", &[desc]) }
    }
    #[wasm_bindgen(js_name = copyBufferToBuffer)]
    pub fn copy_buffer_to_buffer(&self, source: &GpuBuffer, source_offset: JsValue, destination: &GpuBuffer, destination_offset: JsValue, size: JsValue) {
        call(&self.inner, "copyBufferToBuffer", &[
            source.inner.clone(), f64v(num(&source_offset)),
            destination.inner.clone(), f64v(num(&destination_offset)), f64v(num(&size)),
        ]);
    }
    #[wasm_bindgen(js_name = copyBufferToTexture)]
    pub fn copy_buffer_to_texture(&self, source: JsValue, destination: JsValue, copy_size: JsValue) {
        call(&self.inner, "copyBufferToTexture", &[lower(&source), lower(&destination), lower(&copy_size)]);
    }
    #[wasm_bindgen(js_name = copyTextureToBuffer)]
    pub fn copy_texture_to_buffer(&self, source: JsValue, destination: JsValue, copy_size: JsValue) {
        call(&self.inner, "copyTextureToBuffer", &[lower(&source), lower(&destination), lower(&copy_size)]);
    }
    #[wasm_bindgen(js_name = copyTextureToTexture)]
    pub fn copy_texture_to_texture(&self, source: JsValue, destination: JsValue, copy_size: JsValue) {
        call(&self.inner, "copyTextureToTexture", &[lower(&source), lower(&destination), lower(&copy_size)]);
    }
    #[wasm_bindgen(js_name = clearBuffer)]
    pub fn clear_buffer(&self, buffer: &GpuBuffer, offset: JsValue, size: JsValue) {
        let off = if is_some(&offset) { f64v(num(&offset)) } else { JsValue::UNDEFINED };
        let sz = if is_some(&size) { f64v(num(&size)) } else { JsValue::UNDEFINED };
        call(&self.inner, "clearBuffer", &[buffer.inner.clone(), off, sz]);
    }
    #[wasm_bindgen(js_name = writeTimestamp)]
    pub fn write_timestamp(&self, query_set: &GpuQuerySet, query_index: u32) {
        call(&self.inner, "writeTimestamp", &[query_set.inner.clone(), f64v(query_index as f64)]);
    }
    #[wasm_bindgen(js_name = resolveQuerySet)]
    pub fn resolve_query_set(&self, query_set: &GpuQuerySet, first_query: u32, query_count: u32, destination: &GpuBuffer, destination_offset: JsValue) {
        call(&self.inner, "resolveQuerySet", &[
            query_set.inner.clone(), f64v(first_query as f64), f64v(query_count as f64),
            destination.inner.clone(), f64v(num(&destination_offset)),
        ]);
    }
    pub fn finish(&self) -> GpuCommandBuffer {
        GpuCommandBuffer { inner: call(&self.inner, "finish", &[]) }
    }
}

#[wasm_bindgen]
pub struct GpuComputePassEncoder {
    inner: JsValue,
}
#[wasm_bindgen]
impl GpuComputePassEncoder {
    #[wasm_bindgen(js_name = setPipeline)]
    pub fn set_pipeline(&self, pipeline: &GpuComputePipeline) {
        call(&self.inner, "setPipeline", &[pipeline.inner.clone()]);
    }
    #[wasm_bindgen(js_name = dispatchWorkgroups)]
    pub fn dispatch_workgroups(&self, x: u32, y: JsValue, z: JsValue) {
        let yv = if is_some(&y) { f64v(num(&y)) } else { f64v(1.0) };
        let zv = if is_some(&z) { f64v(num(&z)) } else { f64v(1.0) };
        call(&self.inner, "dispatchWorkgroups", &[f64v(x as f64), yv, zv]);
    }
    #[wasm_bindgen(js_name = dispatchWorkgroupsIndirect)]
    pub fn dispatch_workgroups_indirect(&self, indirect_buffer: &GpuBuffer, indirect_offset: JsValue) {
        call(&self.inner, "dispatchWorkgroupsIndirect", &[indirect_buffer.inner.clone(), f64v(num(&indirect_offset))]);
    }
    pub fn end(&self) { call(&self.inner, "end", &[]); }
    #[wasm_bindgen(js_name = setBindGroup)]
    pub fn set_bind_group(&self, index: u32, bind_group: JsValue, dynamic_offsets: JsValue) {
        call(&self.inner, "setBindGroup", &[f64v(index as f64), opt_handle(&bind_group), offsets(&dynamic_offsets)]);
    }
    #[wasm_bindgen(js_name = pushDebugGroup)]
    pub fn push_debug_group(&self, label: String) { call(&self.inner, "pushDebugGroup", &[JsValue::from_str(&label)]); }
    #[wasm_bindgen(js_name = popDebugGroup)]
    pub fn pop_debug_group(&self) { call(&self.inner, "popDebugGroup", &[]); }
    #[wasm_bindgen(js_name = insertDebugMarker)]
    pub fn insert_debug_marker(&self, label: String) { call(&self.inner, "insertDebugMarker", &[JsValue::from_str(&label)]); }
}

#[wasm_bindgen]
pub struct GpuRenderPassEncoder {
    inner: JsValue,
}
#[wasm_bindgen]
impl GpuRenderPassEncoder {
    #[wasm_bindgen(js_name = setPipeline)]
    pub fn set_pipeline(&self, pipeline: &GpuRenderPipeline) {
        call(&self.inner, "setPipeline", &[pipeline.inner.clone()]);
    }
    #[wasm_bindgen(js_name = setIndexBuffer)]
    pub fn set_index_buffer(&self, buffer: &GpuBuffer, index_format: JsValue, offset: JsValue, size: JsValue) {
        let sz = if is_some(&size) { f64v(num(&size)) } else { JsValue::UNDEFINED };
        call(&self.inner, "setIndexBuffer", &[buffer.inner.clone(), index_format, f64v(num(&offset)), sz]);
    }
    #[wasm_bindgen(js_name = setVertexBuffer)]
    pub fn set_vertex_buffer(&self, slot: u32, buffer: &GpuBuffer, offset: JsValue, size: JsValue) {
        let sz = if is_some(&size) { f64v(num(&size)) } else { JsValue::UNDEFINED };
        call(&self.inner, "setVertexBuffer", &[f64v(slot as f64), buffer.inner.clone(), f64v(num(&offset)), sz]);
    }
    pub fn draw(&self, vertex_count: u32, instance_count: u32, first_vertex: u32, first_instance: u32) {
        call(&self.inner, "draw", &[f64v(vertex_count as f64), f64v(instance_count as f64), f64v(first_vertex as f64), f64v(first_instance as f64)]);
    }
    #[wasm_bindgen(js_name = drawIndexed)]
    pub fn draw_indexed(&self, index_count: u32, instance_count: u32, first_index: u32, base_vertex: i32, first_instance: u32) {
        call(&self.inner, "drawIndexed", &[f64v(index_count as f64), f64v(instance_count as f64), f64v(first_index as f64), f64v(base_vertex as f64), f64v(first_instance as f64)]);
    }
    #[wasm_bindgen(js_name = drawIndirect)]
    pub fn draw_indirect(&self, indirect_buffer: &GpuBuffer, indirect_offset: JsValue) {
        call(&self.inner, "drawIndirect", &[indirect_buffer.inner.clone(), f64v(num(&indirect_offset))]);
    }
    #[wasm_bindgen(js_name = drawIndexedIndirect)]
    pub fn draw_indexed_indirect(&self, indirect_buffer: &GpuBuffer, indirect_offset: JsValue) {
        call(&self.inner, "drawIndexedIndirect", &[indirect_buffer.inner.clone(), f64v(num(&indirect_offset))]);
    }
    #[wasm_bindgen(js_name = setViewport)]
    pub fn set_viewport(&self, x: f32, y: f32, width: f32, height: f32, min_depth: f32, max_depth: f32) {
        call(&self.inner, "setViewport", &[f64v(x as f64), f64v(y as f64), f64v(width as f64), f64v(height as f64), f64v(min_depth as f64), f64v(max_depth as f64)]);
    }
    #[wasm_bindgen(js_name = setScissorRect)]
    pub fn set_scissor_rect(&self, x: u32, y: u32, width: u32, height: u32) {
        call(&self.inner, "setScissorRect", &[f64v(x as f64), f64v(y as f64), f64v(width as f64), f64v(height as f64)]);
    }
    #[wasm_bindgen(js_name = setBlendConstant)]
    pub fn set_blend_constant(&self, color: JsValue) {
        call(&self.inner, "setBlendConstant", &[lower(&color)]);
    }
    #[wasm_bindgen(js_name = setStencilReference)]
    pub fn set_stencil_reference(&self, reference: u32) {
        call(&self.inner, "setStencilReference", &[f64v(reference as f64)]);
    }
    #[wasm_bindgen(js_name = beginOcclusionQuery)]
    pub fn begin_occlusion_query(&self, query_index: u32) {
        call(&self.inner, "beginOcclusionQuery", &[f64v(query_index as f64)]);
    }
    #[wasm_bindgen(js_name = endOcclusionQuery)]
    pub fn end_occlusion_query(&self) { call(&self.inner, "endOcclusionQuery", &[]); }
    #[wasm_bindgen(js_name = executeBundles)]
    pub fn execute_bundles(&self, bundles: JsValue) {
        let mapped = Array::new();
        Array::from(&bundles).for_each(&mut |b, _, _| {
            mapped.push(&handle(&b));
        });
        call(&self.inner, "executeBundles", &[mapped.into()]);
    }
    pub fn end(&self) { call(&self.inner, "end", &[]); }
    #[wasm_bindgen(js_name = setBindGroup)]
    pub fn set_bind_group(&self, index: u32, bind_group: JsValue, dynamic_offsets: JsValue) {
        call(&self.inner, "setBindGroup", &[f64v(index as f64), opt_handle(&bind_group), offsets(&dynamic_offsets)]);
    }
    #[wasm_bindgen(js_name = pushDebugGroup)]
    pub fn push_debug_group(&self, label: String) { call(&self.inner, "pushDebugGroup", &[JsValue::from_str(&label)]); }
    #[wasm_bindgen(js_name = popDebugGroup)]
    pub fn pop_debug_group(&self) { call(&self.inner, "popDebugGroup", &[]); }
    #[wasm_bindgen(js_name = insertDebugMarker)]
    pub fn insert_debug_marker(&self, label: String) { call(&self.inner, "insertDebugMarker", &[JsValue::from_str(&label)]); }
}

#[wasm_bindgen]
pub struct GpuRenderBundleEncoder {
    inner: JsValue,
}
#[wasm_bindgen]
impl GpuRenderBundleEncoder {
    pub fn finish(&self, d: JsValue) -> GpuRenderBundle {
        GpuRenderBundle { inner: call(&self.inner, "finish", &[lower(&d)]) }
    }
}

#[wasm_bindgen]
pub struct GpuSurface {}
#[wasm_bindgen]
impl GpuSurface {
    #[wasm_bindgen(js_name = currentTexture)]
    pub fn current_texture(&self) -> GpuTexture {
        GpuTexture { inner: call(&context_handle(), "getCurrentTexture", &[]), is_surface: true }
    }
    #[wasm_bindgen(js_name = getTextureFormat)]
    pub fn get_texture_format(&self) -> String {
        rev_enum(&surface_view_format())
    }
}

// ---- free functions ----

#[wasm_bindgen(js_name = gpuRequestAdapter)]
pub fn request_adapter() -> GpuAdapter {
    GpuAdapter {}
}
#[wasm_bindgen(js_name = gpuSurface)]
pub fn surface() -> GpuSurface {
    GpuSurface {}
}

// ---- shared arg helpers ----

/// `option<borrow<resource>>` -> handle or null.
fn opt_handle(v: &JsValue) -> JsValue {
    if is_some(v) {
        handle(v)
    } else {
        JsValue::NULL
    }
}
/// `option<list<u32>>` dynamic offsets -> array (empty if none).
fn offsets(v: &JsValue) -> JsValue {
    if is_some(v) {
        v.clone()
    } else {
        Array::new().into()
    }
}

// ---- descriptor builders ----

fn binding_resource(r: &JsValue) -> JsValue {
    let tag = get(r, "tag").as_string().unwrap_or_default();
    let val = get(r, "val");
    match tag.as_str() {
        "buffer" => {
            let out = Object::new();
            set(&out, "buffer", handle(&get(&val, "buffer")));
            set(&out, "offset", f64v(num(&get(&val, "offset"))));
            let size = get(&val, "size");
            if is_some(&size) {
                set(&out, "size", f64v(num(&size)));
            }
            out.into()
        }
        "sampler" | "texture-view" => handle(&val),
        other => panic!("unsupported gpu-binding-resource: {other}"),
    }
}

fn blend_component(c: &JsValue) -> JsValue {
    let out = Object::new();
    set(&out, "srcFactor", get(c, "srcFactor"));
    set(&out, "dstFactor", get(c, "dstFactor"));
    set(&out, "operation", get(c, "operation"));
    out.into()
}

fn render_pipeline_desc(d: &JsValue) -> JsValue {
    let out = Object::new();
    set(&out, "layout", lower_layout(&get(d, "layout")));

    // vertex
    let vin = get(d, "vertex");
    let vertex = Object::new();
    set(&vertex, "module", handle(&get(&vin, "module")));
    set(&vertex, "entryPoint", get(&vin, "entryPoint"));
    let buffers_in = get(&vin, "buffers");
    if is_some(&buffers_in) {
        let buffers = Array::new();
        Array::from(&buffers_in).for_each(&mut |b, _, _| {
            let bo = Object::new();
            set(&bo, "arrayStride", f64v(num(&get(&b, "arrayStride"))));
            set(&bo, "stepMode", get(&b, "stepMode"));
            let attrs = Array::new();
            Array::from(&get(&b, "attributes")).for_each(&mut |a, _, _| {
                let ao = Object::new();
                set(&ao, "format", fmt(&get(&a, "format")));
                set(&ao, "offset", f64v(num(&get(&a, "offset"))));
                set(&ao, "shaderLocation", get(&a, "shaderLocation"));
                attrs.push(&ao);
            });
            set(&bo, "attributes", attrs.into());
            buffers.push(&bo);
        });
        set(&vertex, "buffers", buffers.into());
    }
    set(&out, "vertex", vertex.into());

    // fragment (option)
    let fin = get(d, "fragment");
    if is_some(&fin) {
        let fragment = Object::new();
        set(&fragment, "module", handle(&get(&fin, "module")));
        set(&fragment, "entryPoint", get(&fin, "entryPoint"));
        let targets = Array::new();
        Array::from(&get(&fin, "targets")).for_each(&mut |t, _, _| {
            let to = Object::new();
            set(&to, "format", fmt(&get(&t, "format")));
            let blend = get(&t, "blend");
            if is_some(&blend) {
                let bo = Object::new();
                set(&bo, "color", blend_component(&get(&blend, "color")));
                set(&bo, "alpha", blend_component(&get(&blend, "alpha")));
                set(&to, "blend", bo.into());
            }
            let wm = get(&t, "writeMask");
            if is_some(&wm) {
                set(&to, "writeMask", f64v(color_write(&wm) as f64));
            }
            targets.push(&to);
        });
        set(&fragment, "targets", targets.into());
        set(&out, "fragment", fragment.into());
    }

    // primitive (option)
    let pin = get(d, "primitive");
    if is_some(&pin) {
        let prim = Object::new();
        set(&prim, "topology", get(&pin, "topology"));
        let sif = get(&pin, "stripIndexFormat");
        if is_some(&sif) {
            set(&prim, "stripIndexFormat", sif);
        }
        set(&prim, "frontFace", get(&pin, "frontFace"));
        set(&prim, "cullMode", get(&pin, "cullMode"));
        set(&out, "primitive", prim.into());
    }

    // depthStencil (option)
    let din = get(d, "depthStencil");
    if is_some(&din) {
        let ds = Object::new();
        set(&ds, "format", fmt(&get(&din, "format")));
        set(&ds, "depthWriteEnabled", get(&din, "depthWriteEnabled"));
        set(&ds, "depthCompare", get(&din, "depthCompare"));
        set(&out, "depthStencil", ds.into());
    }

    out.into()
}

fn render_pass_desc(d: &JsValue) -> JsValue {
    let out = Object::new();

    let color = Array::new();
    Array::from(&get(d, "colorAttachments")).for_each(&mut |a, _, _| {
        let ao = Object::new();
        set(&ao, "view", handle(&get(&a, "view")));
        let rt = get(&a, "resolveTarget");
        if is_some(&rt) {
            set(&ao, "resolveTarget", handle(&rt));
        }
        set(&ao, "loadOp", get(&a, "loadOp"));
        set(&ao, "storeOp", get(&a, "storeOp"));
        let cv = get(&a, "clearValue");
        if is_some(&cv) {
            set(&ao, "clearValue", cv);
        }
        color.push(&ao);
    });
    set(&out, "colorAttachments", color.into());

    let din = get(d, "depthStencilAttachment");
    if is_some(&din) {
        let ds = Object::new();
        set(&ds, "view", handle(&get(&din, "view")));
        set(&ds, "depthClearValue", get(&din, "depthClearValue"));
        set(&ds, "depthLoadOp", get(&din, "depthLoadOp"));
        set(&ds, "depthStoreOp", get(&din, "depthStoreOp"));
        set(&ds, "depthReadOnly", get(&din, "depthReadOnly"));
        // Stencil aspect omitted (depth-only formats reject it).
        set(&out, "depthStencilAttachment", ds.into());
    }

    out.into()
}
