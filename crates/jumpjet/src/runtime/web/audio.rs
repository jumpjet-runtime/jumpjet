//! `jumpjet:runtime/audio` host import as Rust `#[wasm_bindgen]` classes wrapping the
//! browser's WebAudio objects. The WIT API mirrors WebAudio, so methods forward
//! directly; the adaptations are a few enum-string remaps and the `audio-node`
//! `connect` variant (`{tag,val}` -> the borrowed node's handle).
//!
//! Known gap: `decode-audio-data` is async in the browser but sync in WIT, so it
//! returns a silent placeholder buffer (matching the `buffer.map` GPU gap).

use js_sys::{Array, Float32Array, Function, Object, Reflect, Uint8Array};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

fn get(o: &JsValue, k: &str) -> JsValue {
    Reflect::get(o, &JsValue::from_str(k)).unwrap_or(JsValue::UNDEFINED)
}
fn set_prop(o: &JsValue, k: &str, v: JsValue) {
    let _ = Reflect::set(o, &JsValue::from_str(k), &v);
}
fn obj_set(o: &Object, k: &str, v: JsValue) {
    let _ = Reflect::set(o, &JsValue::from_str(k), &v);
}
fn call(obj: &JsValue, method: &str, args: &[JsValue]) -> JsValue {
    let f: Function = get(obj, method).unchecked_into();
    let arr = Array::new();
    for a in args {
        arr.push(a);
    }
    Reflect::apply(&f, obj, &arr).unwrap_or(JsValue::UNDEFINED)
}
fn handle(v: &JsValue) -> JsValue {
    call(v, "__h", &[])
}
fn f32v(x: f32) -> JsValue {
    JsValue::from_f64(x as f64)
}
fn getf(o: &JsValue, k: &str) -> f32 {
    get(o, k).as_f64().unwrap_or(0.0) as f32
}

/// WIT audio enum -> WebAudio string (where they differ).
fn enum_to_web(v: &JsValue) -> JsValue {
    match v.as_string().as_deref() {
        Some("a") => JsValue::from_str("a-rate"),
        Some("k") => JsValue::from_str("k-rate"),
        Some("x2") => JsValue::from_str("2x"),
        Some("x4") => JsValue::from_str("4x"),
        Some("equal-power") => JsValue::from_str("equalpower"),
        Some("hrtf") => JsValue::from_str("HRTF"),
        _ => v.clone(),
    }
}
/// WebAudio string -> WIT audio enum (for getters).
fn enum_from_web(s: &str) -> String {
    match s {
        "a-rate" => "a",
        "k-rate" => "k",
        "2x" => "x2",
        "4x" => "x4",
        "equalpower" => "equal-power",
        "HRTF" => "hrtf",
        other => other,
    }
    .to_owned()
}

/// `audio-node` variant `{tag, val: borrow<node>}` -> the node's browser handle.
fn node_handle(variant: &JsValue) -> JsValue {
    handle(&get(variant, "val"))
}

// ---- audio-param ----

#[wasm_bindgen]
pub struct AudioParam {
    inner: JsValue,
}
#[wasm_bindgen]
impl AudioParam {
    #[wasm_bindgen(js_name = __h)]
    pub fn h(&self) -> JsValue { self.inner.clone() }
    #[wasm_bindgen(js_name = automationRate)]
    pub fn automation_rate(&self) -> String { enum_from_web(&get(&self.inner, "automationRate").as_string().unwrap_or_default()) }
    #[wasm_bindgen(js_name = setAutomationRate)]
    pub fn set_automation_rate(&self, rate: JsValue) { set_prop(&self.inner, "automationRate", enum_to_web(&rate)); }
    #[wasm_bindgen(js_name = defaultValue)]
    pub fn default_value(&self) -> f32 { getf(&self.inner, "defaultValue") }
    #[wasm_bindgen(js_name = minValue)]
    pub fn min_value(&self) -> f32 { getf(&self.inner, "minValue") }
    #[wasm_bindgen(js_name = maxValue)]
    pub fn max_value(&self) -> f32 { getf(&self.inner, "maxValue") }
    pub fn value(&self) -> f32 { getf(&self.inner, "value") }
    #[wasm_bindgen(js_name = setValue)]
    pub fn set_value(&self, value: f32) { set_prop(&self.inner, "value", f32v(value)); }
    #[wasm_bindgen(js_name = setValueAtTime)]
    pub fn set_value_at_time(&self, value: f32, start_time: f32) { call(&self.inner, "setValueAtTime", &[f32v(value), f32v(start_time)]); }
    #[wasm_bindgen(js_name = setValueCurveAtTime)]
    pub fn set_value_curve_at_time(&self, values: JsValue, start_time: f32, duration: f32) { call(&self.inner, "setValueCurveAtTime", &[values, f32v(start_time), f32v(duration)]); }
    #[wasm_bindgen(js_name = linearRampToValueAtTime)]
    pub fn linear_ramp_to_value_at_time(&self, value: f32, end_time: f32) { call(&self.inner, "linearRampToValueAtTime", &[f32v(value), f32v(end_time)]); }
    #[wasm_bindgen(js_name = exponentialRampToValueAtTime)]
    pub fn exponential_ramp_to_value_at_time(&self, value: f32, end_time: f32) { call(&self.inner, "exponentialRampToValueAtTime", &[f32v(value), f32v(end_time)]); }
    #[wasm_bindgen(js_name = setTargetAtTime)]
    pub fn set_target_at_time(&self, value: f32, start_time: f32, time_constant: f32) { call(&self.inner, "setTargetAtTime", &[f32v(value), f32v(start_time), f32v(time_constant)]); }
    #[wasm_bindgen(js_name = cancelScheduledValues)]
    pub fn cancel_scheduled_values(&self, cancel_time: f32) { call(&self.inner, "cancelScheduledValues", &[f32v(cancel_time)]); }
    #[wasm_bindgen(js_name = cancelAndHoldAtTime)]
    pub fn cancel_and_hold_at_time(&self, cancel_time: f32) { call(&self.inner, "cancelAndHoldAtTime", &[f32v(cancel_time)]); }
}
fn param(node: &JsValue, prop: &str) -> AudioParam {
    AudioParam { inner: get(node, prop) }
}

// ---- handle-only resources ----

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
handle_class!(AudioRenderCapacity);
handle_class!(PeriodicWave);

// ---- node macro: __h + connect ----

macro_rules! audio_node {
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
            pub fn connect(&self, destination: JsValue) {
                call(&self.inner, "connect", &[node_handle(&destination)]);
            }
        }
    };
}

audio_node!(AnalyzerNode);
audio_node!(BiquadFilterNode);
audio_node!(AudioBufferSourceNode);
audio_node!(ConstantSourceNode);
audio_node!(ConvolverNode);
audio_node!(ChannelMergerNode);
audio_node!(ChannelSplitterNode);
audio_node!(DelayNode);
audio_node!(DynamicsCompressorNode);
audio_node!(GainNode);
audio_node!(IirFilterNode);
audio_node!(OscillatorNode);
audio_node!(PannerNode);
audio_node!(StereoPannerNode);
audio_node!(WaveShaperNode);

// node-specific params/methods (second impl blocks)

#[wasm_bindgen]
impl GainNode {
    pub fn gain(&self) -> AudioParam { param(&self.inner, "gain") }
}
#[wasm_bindgen]
impl DelayNode {
    #[wasm_bindgen(js_name = delayTime)]
    pub fn delay_time(&self) -> AudioParam { param(&self.inner, "delayTime") }
}
#[wasm_bindgen]
impl StereoPannerNode {
    pub fn pan(&self) -> AudioParam { param(&self.inner, "pan") }
}
#[wasm_bindgen]
impl ConstantSourceNode {
    pub fn offset(&self) -> AudioParam { param(&self.inner, "offset") }
}
#[wasm_bindgen]
impl OscillatorNode {
    pub fn detune(&self) -> AudioParam { param(&self.inner, "detune") }
    pub fn frequency(&self) -> AudioParam { param(&self.inner, "frequency") }
    #[wasm_bindgen(js_name = type)]
    pub fn type_(&self) -> JsValue { get(&self.inner, "type") }
    #[wasm_bindgen(js_name = setType)]
    pub fn set_type(&self, t: JsValue) { set_prop(&self.inner, "type", t); }
    #[wasm_bindgen(js_name = setPeriodicWave)]
    pub fn set_periodic_wave(&self, wave: &PeriodicWave) { call(&self.inner, "setPeriodicWave", &[wave.inner.clone()]); }
}
#[wasm_bindgen]
impl BiquadFilterNode {
    pub fn gain(&self) -> AudioParam { param(&self.inner, "gain") }
    pub fn frequency(&self) -> AudioParam { param(&self.inner, "frequency") }
    pub fn detune(&self) -> AudioParam { param(&self.inner, "detune") }
    #[wasm_bindgen(js_name = q)]
    pub fn q(&self) -> AudioParam { param(&self.inner, "Q") }
    #[wasm_bindgen(js_name = type)]
    pub fn type_(&self) -> JsValue { get(&self.inner, "type") }
    #[wasm_bindgen(js_name = setType)]
    pub fn set_type(&self, t: JsValue) { set_prop(&self.inner, "type", t); }
}
#[wasm_bindgen]
impl DynamicsCompressorNode {
    pub fn attack(&self) -> AudioParam { param(&self.inner, "attack") }
    pub fn knee(&self) -> AudioParam { param(&self.inner, "knee") }
    pub fn ratio(&self) -> AudioParam { param(&self.inner, "ratio") }
    pub fn release(&self) -> AudioParam { param(&self.inner, "release") }
    pub fn threshold(&self) -> AudioParam { param(&self.inner, "threshold") }
    pub fn reduction(&self) -> f32 { getf(&self.inner, "reduction") }
}
#[wasm_bindgen]
impl ConvolverNode {
    pub fn buffer(&self) -> Option<AudioBuffer> {
        let b = get(&self.inner, "buffer");
        if b.is_null() || b.is_undefined() { None } else { Some(AudioBuffer { inner: b }) }
    }
    #[wasm_bindgen(js_name = setBuffer)]
    pub fn set_buffer(&self, buffer: &AudioBuffer) { set_prop(&self.inner, "buffer", buffer.inner.clone()); }
    pub fn normalize(&self) -> bool { get(&self.inner, "normalize").as_bool().unwrap_or(false) }
    #[wasm_bindgen(js_name = setNormalize)]
    pub fn set_normalize(&self, value: bool) { set_prop(&self.inner, "normalize", JsValue::from_bool(value)); }
}
#[wasm_bindgen]
impl WaveShaperNode {
    pub fn curve(&self) -> Option<Vec<f32>> {
        let c = get(&self.inner, "curve");
        if c.is_null() || c.is_undefined() { None } else { Some(Float32Array::new(&c).to_vec()) }
    }
    #[wasm_bindgen(js_name = setCurve)]
    pub fn set_curve(&self, curve: JsValue) { set_prop(&self.inner, "curve", curve); }
    pub fn oversample(&self) -> String { enum_from_web(&get(&self.inner, "oversample").as_string().unwrap_or_default()) }
    #[wasm_bindgen(js_name = setOversample)]
    pub fn set_oversample(&self, oversample: JsValue) { set_prop(&self.inner, "oversample", enum_to_web(&oversample)); }
}
#[wasm_bindgen]
impl PannerNode {
    #[wasm_bindgen(js_name = positionX)] pub fn position_x(&self) -> AudioParam { param(&self.inner, "positionX") }
    #[wasm_bindgen(js_name = positionY)] pub fn position_y(&self) -> AudioParam { param(&self.inner, "positionY") }
    #[wasm_bindgen(js_name = positionZ)] pub fn position_z(&self) -> AudioParam { param(&self.inner, "positionZ") }
    #[wasm_bindgen(js_name = setPosition)] pub fn set_position(&self, x: f32, y: f32, z: f32) { call(&self.inner, "setPosition", &[f32v(x), f32v(y), f32v(z)]); }
    #[wasm_bindgen(js_name = orientationX)] pub fn orientation_x(&self) -> AudioParam { param(&self.inner, "orientationX") }
    #[wasm_bindgen(js_name = orientationY)] pub fn orientation_y(&self) -> AudioParam { param(&self.inner, "orientationY") }
    #[wasm_bindgen(js_name = orientationZ)] pub fn orientation_z(&self) -> AudioParam { param(&self.inner, "orientationZ") }
    #[wasm_bindgen(js_name = setOrientation)] pub fn set_orientation(&self, x: f32, y: f32, z: f32) { call(&self.inner, "setOrientation", &[f32v(x), f32v(y), f32v(z)]); }
    #[wasm_bindgen(js_name = distanceModel)] pub fn distance_model(&self) -> String { enum_from_web(&get(&self.inner, "distanceModel").as_string().unwrap_or_default()) }
    #[wasm_bindgen(js_name = setDistanceModel)] pub fn set_distance_model(&self, value: JsValue) { set_prop(&self.inner, "distanceModel", enum_to_web(&value)); }
    #[wasm_bindgen(js_name = refDistance)] pub fn ref_distance(&self) -> f32 { getf(&self.inner, "refDistance") }
    #[wasm_bindgen(js_name = setRefDistance)] pub fn set_ref_distance(&self, value: f32) { set_prop(&self.inner, "refDistance", f32v(value)); }
    #[wasm_bindgen(js_name = maxDistance)] pub fn max_distance(&self) -> f32 { getf(&self.inner, "maxDistance") }
    #[wasm_bindgen(js_name = setMaxDistance)] pub fn set_max_distance(&self, value: f32) { set_prop(&self.inner, "maxDistance", f32v(value)); }
    #[wasm_bindgen(js_name = rolloffFactor)] pub fn rolloff_factor(&self) -> f32 { getf(&self.inner, "rolloffFactor") }
    #[wasm_bindgen(js_name = setRolloffFactor)] pub fn set_rolloff_factor(&self, value: f32) { set_prop(&self.inner, "rolloffFactor", f32v(value)); }
    #[wasm_bindgen(js_name = coneInnerAngle)] pub fn cone_inner_angle(&self) -> f32 { getf(&self.inner, "coneInnerAngle") }
    #[wasm_bindgen(js_name = setConeInnerAngle)] pub fn set_cone_inner_angle(&self, value: f32) { set_prop(&self.inner, "coneInnerAngle", f32v(value)); }
    #[wasm_bindgen(js_name = coneOuterAngle)] pub fn cone_outer_angle(&self) -> f32 { getf(&self.inner, "coneOuterAngle") }
    #[wasm_bindgen(js_name = setConeOuterAngle)] pub fn set_cone_outer_angle(&self, value: f32) { set_prop(&self.inner, "coneOuterAngle", f32v(value)); }
    #[wasm_bindgen(js_name = coneOuterGain)] pub fn cone_outer_gain(&self) -> f32 { getf(&self.inner, "coneOuterGain") }
    #[wasm_bindgen(js_name = setConeOuterGain)] pub fn set_cone_outer_gain(&self, value: f32) { set_prop(&self.inner, "coneOuterGain", f32v(value)); }
    #[wasm_bindgen(js_name = panningModel)] pub fn panning_model(&self) -> String { enum_from_web(&get(&self.inner, "panningModel").as_string().unwrap_or_default()) }
    #[wasm_bindgen(js_name = setPanningModel)] pub fn set_panning_model(&self, value: JsValue) { set_prop(&self.inner, "panningModel", enum_to_web(&value)); }
}
#[wasm_bindgen]
impl AnalyzerNode {
    #[wasm_bindgen(js_name = fftSize)] pub fn fft_size(&self) -> u32 { get(&self.inner, "fftSize").as_f64().unwrap_or(0.0) as u32 }
    #[wasm_bindgen(js_name = setFftSize)] pub fn set_fft_size(&self, v: u32) { set_prop(&self.inner, "fftSize", JsValue::from_f64(v as f64)); }
    #[wasm_bindgen(js_name = smoothingTimeConstant)] pub fn smoothing_time_constant(&self) -> f32 { getf(&self.inner, "smoothingTimeConstant") }
    #[wasm_bindgen(js_name = setSmoothingTimeConstant)] pub fn set_smoothing_time_constant(&self, v: f32) { set_prop(&self.inner, "smoothingTimeConstant", f32v(v)); }
    #[wasm_bindgen(js_name = minDecibels)] pub fn min_decibels(&self) -> f32 { getf(&self.inner, "minDecibels") }
    #[wasm_bindgen(js_name = setMinDecibels)] pub fn set_min_decibels(&self, v: f32) { set_prop(&self.inner, "minDecibels", f32v(v)); }
    #[wasm_bindgen(js_name = maxDecibels)] pub fn max_decibels(&self) -> f32 { getf(&self.inner, "maxDecibels") }
    #[wasm_bindgen(js_name = setMaxDecibels)] pub fn set_max_decibels(&self, v: f32) { set_prop(&self.inner, "maxDecibels", f32v(v)); }
    #[wasm_bindgen(js_name = frequencyBinCount)] pub fn frequency_bin_count(&self) -> u32 { get(&self.inner, "frequencyBinCount").as_f64().unwrap_or(0.0) as u32 }
    #[wasm_bindgen(js_name = getFloatTimeDomainData)] pub fn get_float_time_domain_data(&self) -> Vec<f32> {
        let n = self.fft_size(); let arr = Float32Array::new_with_length(n); call(&self.inner, "getFloatTimeDomainData", &[arr.clone().into()]); arr.to_vec()
    }
    #[wasm_bindgen(js_name = getByteTimeDomainData)] pub fn get_byte_time_domain_data(&self) -> Vec<u8> {
        let n = self.fft_size(); let arr = Uint8Array::new_with_length(n); call(&self.inner, "getByteTimeDomainData", &[arr.clone().into()]); arr.to_vec()
    }
    #[wasm_bindgen(js_name = getFloatFrequencyData)] pub fn get_float_frequency_data(&self) -> Vec<f32> {
        let n = self.frequency_bin_count(); let arr = Float32Array::new_with_length(n); call(&self.inner, "getFloatFrequencyData", &[arr.clone().into()]); arr.to_vec()
    }
    #[wasm_bindgen(js_name = getByteFrequencyData)] pub fn get_byte_frequency_data(&self) -> Vec<u8> {
        let n = self.frequency_bin_count(); let arr = Uint8Array::new_with_length(n); call(&self.inner, "getByteFrequencyData", &[arr.clone().into()]); arr.to_vec()
    }
}
#[wasm_bindgen]
impl AudioBufferSourceNode {
    #[wasm_bindgen(js_name = startAtWithOffset)] pub fn start_at_with_offset(&self, start: f32, offset: f32) { call(&self.inner, "start", &[f32v(start), f32v(offset)]); }
    #[wasm_bindgen(js_name = startAtWithOffsetAndDuration)] pub fn start_at_with_offset_and_duration(&self, start: f32, offset: f32, duration: f32) { call(&self.inner, "start", &[f32v(start), f32v(offset), f32v(duration)]); }
    pub fn buffer(&self) -> Option<AudioBuffer> {
        let b = get(&self.inner, "buffer");
        if b.is_null() || b.is_undefined() { None } else { Some(AudioBuffer { inner: b }) }
    }
    #[wasm_bindgen(js_name = setBuffer)] pub fn set_buffer(&self, audio_buffer: &AudioBuffer) { set_prop(&self.inner, "buffer", audio_buffer.inner.clone()); }
    #[wasm_bindgen(js_name = playbackRate)] pub fn playback_rate(&self) -> AudioParam { param(&self.inner, "playbackRate") }
    pub fn position(&self) -> f32 { getf(&self.inner, "position") }
    pub fn detune(&self) -> AudioParam { param(&self.inner, "detune") }
    #[wasm_bindgen(js_name = loop)] pub fn loop_(&self) -> bool { get(&self.inner, "loop").as_bool().unwrap_or(false) }
    #[wasm_bindgen(js_name = setLoop)] pub fn set_loop(&self, value: bool) { set_prop(&self.inner, "loop", JsValue::from_bool(value)); }
    #[wasm_bindgen(js_name = loopStart)] pub fn loop_start(&self) -> f32 { getf(&self.inner, "loopStart") }
    #[wasm_bindgen(js_name = setLoopStart)] pub fn set_loop_start(&self, value: f32) { set_prop(&self.inner, "loopStart", f32v(value)); }
    #[wasm_bindgen(js_name = loopEnd)] pub fn loop_end(&self) -> f32 { getf(&self.inner, "loopEnd") }
    #[wasm_bindgen(js_name = setLoopEnd)] pub fn set_loop_end(&self, value: f32) { set_prop(&self.inner, "loopEnd", f32v(value)); }
    pub fn start(&self) { call(&self.inner, "start", &[]); }
}

// ---- audio-buffer ----

#[wasm_bindgen]
pub struct AudioBuffer {
    inner: JsValue,
}
#[wasm_bindgen]
impl AudioBuffer {
    /// WIT `constructor(samples: list<list<f32>>, sample-rate: f32)`. jco passes
    /// `samples` as a JS array of `Float32Array` (one per channel). Built via the
    /// global `AudioBuffer` constructor (context-free), then filled per channel.
    #[wasm_bindgen(constructor)]
    pub fn new(samples: JsValue, sample_rate: f32) -> AudioBuffer {
        let channels = Array::from(&samples);
        let number_of_channels = channels.length().max(1);
        let length = channels
            .get(0)
            .dyn_into::<Float32Array>()
            .map(|a| a.length())
            .unwrap_or(1)
            .max(1);

        let options = Object::new();
        obj_set(&options, "numberOfChannels", JsValue::from_f64(number_of_channels as f64));
        obj_set(&options, "length", JsValue::from_f64(length as f64));
        obj_set(&options, "sampleRate", f32v(sample_rate));

        let ctor = Reflect::get(&js_sys::global(), &JsValue::from_str("AudioBuffer"))
            .ok()
            .and_then(|c| c.dyn_into::<Function>().ok());
        let inner = match ctor {
            Some(ctor) => {
                let args = Array::new();
                args.push(&options.into());
                Reflect::construct(&ctor, &args).unwrap_or(JsValue::UNDEFINED)
            }
            None => JsValue::UNDEFINED,
        };

        for ch in 0..number_of_channels {
            if let Ok(arr) = channels.get(ch).dyn_into::<Float32Array>() {
                call(&inner, "copyToChannel", &[arr.into(), JsValue::from_f64(ch as f64)]);
            }
        }

        AudioBuffer { inner }
    }

    #[wasm_bindgen(js_name = __h)]
    pub fn h(&self) -> JsValue { self.inner.clone() }
    #[wasm_bindgen(js_name = numberOfChannels)] pub fn number_of_channels(&self) -> u32 { get(&self.inner, "numberOfChannels").as_f64().unwrap_or(0.0) as u32 }
    pub fn length(&self) -> u32 { get(&self.inner, "length").as_f64().unwrap_or(0.0) as u32 }
    #[wasm_bindgen(js_name = sampleRate)] pub fn sample_rate(&self) -> f32 { getf(&self.inner, "sampleRate") }
    pub fn duration(&self) -> f32 { getf(&self.inner, "duration") }
    #[wasm_bindgen(js_name = getChannelData)] pub fn get_channel_data(&self, channel_number: u32) -> Vec<f32> {
        Float32Array::new(&call(&self.inner, "getChannelData", &[JsValue::from_f64(channel_number as f64)])).to_vec()
    }
}

// ---- WIT resource constructors ----
//
// Each audio node resource declares `constructor(context: borrow<audio-context>, ...)`
// in the WIT. wasm-bindgen classes have no constructor unless one is declared, so
// without these jco's `new XNode(context, ...)` would yield a pointer-less object
// that traps ("index out of bounds") on first use. Each delegates to the matching
// `AudioContext::create*` factory.

#[wasm_bindgen]
impl AudioBufferSourceNode {
    #[wasm_bindgen(constructor)]
    pub fn new(context: &AudioContext) -> AudioBufferSourceNode { context.create_buffer_source() }
}
#[wasm_bindgen]
impl AnalyzerNode {
    #[wasm_bindgen(constructor)]
    pub fn new(context: &AudioContext) -> AnalyzerNode { context.create_analyzer() }
}
#[wasm_bindgen]
impl BiquadFilterNode {
    #[wasm_bindgen(constructor)]
    pub fn new(context: &AudioContext) -> BiquadFilterNode { context.create_biquad_filter() }
}
#[wasm_bindgen]
impl ConstantSourceNode {
    #[wasm_bindgen(constructor)]
    pub fn new(context: &AudioContext) -> ConstantSourceNode { context.create_constant_source() }
}
#[wasm_bindgen]
impl ConvolverNode {
    #[wasm_bindgen(constructor)]
    pub fn new(context: &AudioContext) -> ConvolverNode { context.create_convolver() }
}
#[wasm_bindgen]
impl ChannelMergerNode {
    #[wasm_bindgen(constructor)]
    pub fn new(context: &AudioContext, number_of_inputs: u32) -> ChannelMergerNode { context.create_channel_merger(number_of_inputs) }
}
#[wasm_bindgen]
impl ChannelSplitterNode {
    #[wasm_bindgen(constructor)]
    pub fn new(context: &AudioContext, number_of_outputs: u32) -> ChannelSplitterNode { context.create_channel_splitter(number_of_outputs) }
}
#[wasm_bindgen]
impl DelayNode {
    #[wasm_bindgen(constructor)]
    pub fn new(context: &AudioContext, max_delay_time: f32) -> DelayNode { context.create_delay(max_delay_time) }
}
#[wasm_bindgen]
impl DynamicsCompressorNode {
    #[wasm_bindgen(constructor)]
    pub fn new(context: &AudioContext) -> DynamicsCompressorNode { context.create_dynamics_compressor() }
}
#[wasm_bindgen]
impl GainNode {
    #[wasm_bindgen(constructor)]
    pub fn new(context: &AudioContext) -> GainNode { context.create_gain() }
}
#[wasm_bindgen]
impl IirFilterNode {
    #[wasm_bindgen(constructor)]
    pub fn new(context: &AudioContext, feedforward: JsValue, feedback: JsValue) -> IirFilterNode { context.create_iir_filter(feedforward, feedback) }
}
#[wasm_bindgen]
impl OscillatorNode {
    #[wasm_bindgen(constructor)]
    pub fn new(context: &AudioContext) -> OscillatorNode { context.create_oscillator() }
}
#[wasm_bindgen]
impl PannerNode {
    #[wasm_bindgen(constructor)]
    pub fn new(context: &AudioContext) -> PannerNode { context.create_panner() }
}

// ---- audio-destination-node / audio-listener ----

#[wasm_bindgen]
pub struct AudioDestinationNode {
    inner: JsValue,
}
#[wasm_bindgen]
impl AudioDestinationNode {
    #[wasm_bindgen(js_name = __h)]
    pub fn h(&self) -> JsValue { self.inner.clone() }
    #[wasm_bindgen(js_name = maxChannelCount)] pub fn max_channel_count(&self) -> u32 { get(&self.inner, "maxChannelCount").as_f64().unwrap_or(0.0) as u32 }
}

#[wasm_bindgen]
pub struct AudioListener {
    inner: JsValue,
}
#[wasm_bindgen]
impl AudioListener {
    #[wasm_bindgen(js_name = positionX)] pub fn position_x(&self) -> AudioParam { param(&self.inner, "positionX") }
    #[wasm_bindgen(js_name = positionY)] pub fn position_y(&self) -> AudioParam { param(&self.inner, "positionY") }
    #[wasm_bindgen(js_name = positionZ)] pub fn position_z(&self) -> AudioParam { param(&self.inner, "positionZ") }
    #[wasm_bindgen(js_name = forwardX)] pub fn forward_x(&self) -> AudioParam { param(&self.inner, "forwardX") }
    #[wasm_bindgen(js_name = forwardY)] pub fn forward_y(&self) -> AudioParam { param(&self.inner, "forwardY") }
    #[wasm_bindgen(js_name = forwardZ)] pub fn forward_z(&self) -> AudioParam { param(&self.inner, "forwardZ") }
    #[wasm_bindgen(js_name = upX)] pub fn up_x(&self) -> AudioParam { param(&self.inner, "upX") }
    #[wasm_bindgen(js_name = upY)] pub fn up_y(&self) -> AudioParam { param(&self.inner, "upY") }
    #[wasm_bindgen(js_name = upZ)] pub fn up_z(&self) -> AudioParam { param(&self.inner, "upZ") }
}

// ---- audio-context ----

#[wasm_bindgen]
pub struct AudioContext {
    inner: JsValue,
}
#[wasm_bindgen]
impl AudioContext {
    #[wasm_bindgen(js_name = __h)]
    pub fn h(&self) -> JsValue { self.inner.clone() }
    #[wasm_bindgen(js_name = baseLatency)] pub fn base_latency(&self) -> f32 { getf(&self.inner, "baseLatency") }
    #[wasm_bindgen(js_name = outputLatency)] pub fn output_latency(&self) -> f32 { getf(&self.inner, "outputLatency") }
    #[wasm_bindgen(js_name = sinkId)] pub fn sink_id(&self) -> String { get(&self.inner, "sinkId").as_string().unwrap_or_default() }
    #[wasm_bindgen(js_name = setSinkId)] pub fn set_sink_id(&self, sink_id: String) { call(&self.inner, "setSinkId", &[JsValue::from_str(&sink_id)]); }
    #[wasm_bindgen(js_name = renderCapacity)] pub fn render_capacity(&self) -> AudioRenderCapacity { AudioRenderCapacity { inner: call(&self.inner, "renderCapacity", &[]) } }
    pub fn suspend(&self) { call(&self.inner, "suspend", &[]); }
    pub fn resume(&self) { call(&self.inner, "resume", &[]); }
    pub fn close(&self) { call(&self.inner, "close", &[]); }
    #[wasm_bindgen(js_name = decodeAudioData)]
    pub fn decode_audio_data(&self, _data: Vec<u8>) -> AudioBuffer {
        // decodeAudioData is async; the sync WIT can't await it. Return a silent
        // placeholder so the guest gets a valid buffer (known limitation).
        web_sys::console::warn_1(&JsValue::from_str("jumpjet: audio decode-audio-data is not supported on web (async); returning a silent buffer"));
        let sr = getf(&self.inner, "sampleRate");
        AudioBuffer { inner: call(&self.inner, "createBuffer", &[JsValue::from_f64(2.0), JsValue::from_f64(1.0), f32v(sr)]) }
    }
    #[wasm_bindgen(js_name = createBuffer)]
    pub fn create_buffer(&self, number_of_channels: u32, length: u32, sample_rate: f32) -> AudioBuffer {
        AudioBuffer { inner: call(&self.inner, "createBuffer", &[JsValue::from_f64(number_of_channels as f64), JsValue::from_f64(length as f64), f32v(sample_rate)]) }
    }
    #[wasm_bindgen(js_name = createAnalyzer)] pub fn create_analyzer(&self) -> AnalyzerNode { AnalyzerNode { inner: call(&self.inner, "createAnalyser", &[]) } }
    #[wasm_bindgen(js_name = createBiquadFilter)] pub fn create_biquad_filter(&self) -> BiquadFilterNode { BiquadFilterNode { inner: call(&self.inner, "createBiquadFilter", &[]) } }
    #[wasm_bindgen(js_name = createBufferSource)] pub fn create_buffer_source(&self) -> AudioBufferSourceNode { AudioBufferSourceNode { inner: call(&self.inner, "createBufferSource", &[]) } }
    #[wasm_bindgen(js_name = createChannelMerger)] pub fn create_channel_merger(&self, number_of_inputs: u32) -> ChannelMergerNode { ChannelMergerNode { inner: call(&self.inner, "createChannelMerger", &[JsValue::from_f64(number_of_inputs as f64)]) } }
    #[wasm_bindgen(js_name = createChannelSplitter)] pub fn create_channel_splitter(&self, number_of_outputs: u32) -> ChannelSplitterNode { ChannelSplitterNode { inner: call(&self.inner, "createChannelSplitter", &[JsValue::from_f64(number_of_outputs as f64)]) } }
    #[wasm_bindgen(js_name = createConstantSource)] pub fn create_constant_source(&self) -> ConstantSourceNode { ConstantSourceNode { inner: call(&self.inner, "createConstantSource", &[]) } }
    #[wasm_bindgen(js_name = createConvolver)] pub fn create_convolver(&self) -> ConvolverNode { ConvolverNode { inner: call(&self.inner, "createConvolver", &[]) } }
    #[wasm_bindgen(js_name = createDelay)] pub fn create_delay(&self, max_delay_time: f32) -> DelayNode { DelayNode { inner: call(&self.inner, "createDelay", &[f32v(max_delay_time)]) } }
    #[wasm_bindgen(js_name = createDynamicsCompressor)] pub fn create_dynamics_compressor(&self) -> DynamicsCompressorNode { DynamicsCompressorNode { inner: call(&self.inner, "createDynamicsCompressor", &[]) } }
    #[wasm_bindgen(js_name = createGain)] pub fn create_gain(&self) -> GainNode { GainNode { inner: call(&self.inner, "createGain", &[]) } }
    #[wasm_bindgen(js_name = createIirFilter)] pub fn create_iir_filter(&self, feedforward: JsValue, feedback: JsValue) -> IirFilterNode { IirFilterNode { inner: call(&self.inner, "createIIRFilter", &[feedforward, feedback]) } }
    #[wasm_bindgen(js_name = createOscillator)] pub fn create_oscillator(&self) -> OscillatorNode { OscillatorNode { inner: call(&self.inner, "createOscillator", &[]) } }
    #[wasm_bindgen(js_name = createPanner)] pub fn create_panner(&self) -> PannerNode { PannerNode { inner: call(&self.inner, "createPanner", &[]) } }
    #[wasm_bindgen(js_name = createPeriodicWave)]
    pub fn create_periodic_wave(&self, options: JsValue) -> PeriodicWave {
        let real = get(&options, "real");
        let imag = get(&options, "imag");
        let constraints = Object::new();
        obj_set(&constraints, "disableNormalization", get(&options, "disableNormalization"));
        PeriodicWave { inner: call(&self.inner, "createPeriodicWave", &[real, imag, constraints.into()]) }
    }
    #[wasm_bindgen(js_name = createStereoPanner)] pub fn create_stereo_panner(&self) -> StereoPannerNode { StereoPannerNode { inner: call(&self.inner, "createStereoPanner", &[]) } }
    #[wasm_bindgen(js_name = createWaveShaper)] pub fn create_wave_shaper(&self) -> WaveShaperNode { WaveShaperNode { inner: call(&self.inner, "createWaveShaper", &[]) } }
    pub fn destination(&self) -> AudioDestinationNode { AudioDestinationNode { inner: get(&self.inner, "destination") } }
    pub fn listener(&self) -> AudioListener { AudioListener { inner: get(&self.inner, "listener") } }
    #[wasm_bindgen(js_name = sampleRate)] pub fn sample_rate(&self) -> f32 { getf(&self.inner, "sampleRate") }
    pub fn state(&self) -> JsValue { get(&self.inner, "state") }
    #[wasm_bindgen(js_name = currentTime)] pub fn current_time(&self) -> f32 { getf(&self.inner, "currentTime") }
}

// ---- audio-device + entry ----

#[wasm_bindgen]
pub struct AudioDevice {}
#[wasm_bindgen]
impl AudioDevice {
    pub fn name(&self) -> String {
        "default".to_owned()
    }
    #[wasm_bindgen(js_name = createContext)]
    pub fn create_context(&self) -> AudioContext {
        let ctx = js_sys::Reflect::get(&js_sys::global(), &JsValue::from_str("AudioContext"))
            .ok()
            .and_then(|c| c.dyn_into::<Function>().ok())
            .and_then(|ctor| js_sys::Reflect::construct(&ctor, &Array::new()).ok())
            .unwrap_or(JsValue::UNDEFINED);
        AudioContext { inner: ctx }
    }
}

#[wasm_bindgen(js_name = audioOutput)]
pub fn output() -> Option<AudioDevice> {
    Some(AudioDevice {})
}
