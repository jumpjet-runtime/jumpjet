export class AnalyzerNode {
    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(AnalyzerNode.prototype);
        obj.__wbg_ptr = ptr;
        AnalyzerNodeFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        AnalyzerNodeFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_analyzernode_free(ptr, 0);
    }
    /**
     * @returns {any}
     */
    __h() {
        const ret = wasm.analyzernode___h(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {any} destination
     */
    connect(destination) {
        wasm.analyzernode_connect(this.__wbg_ptr, destination);
    }
    /**
     * @returns {number}
     */
    fftSize() {
        const ret = wasm.analyzernode_fftSize(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    frequencyBinCount() {
        const ret = wasm.analyzernode_frequencyBinCount(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {Uint8Array}
     */
    getByteFrequencyData() {
        const ret = wasm.analyzernode_getByteFrequencyData(this.__wbg_ptr);
        var v1 = getArrayU8FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        return v1;
    }
    /**
     * @returns {Uint8Array}
     */
    getByteTimeDomainData() {
        const ret = wasm.analyzernode_getByteTimeDomainData(this.__wbg_ptr);
        var v1 = getArrayU8FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        return v1;
    }
    /**
     * @returns {Float32Array}
     */
    getFloatFrequencyData() {
        const ret = wasm.analyzernode_getFloatFrequencyData(this.__wbg_ptr);
        var v1 = getArrayF32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @returns {Float32Array}
     */
    getFloatTimeDomainData() {
        const ret = wasm.analyzernode_getFloatTimeDomainData(this.__wbg_ptr);
        var v1 = getArrayF32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @returns {number}
     */
    maxDecibels() {
        const ret = wasm.analyzernode_maxDecibels(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    minDecibels() {
        const ret = wasm.analyzernode_minDecibels(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {AudioContext} context
     */
    constructor(context) {
        _assertClass(context, AudioContext);
        const ret = wasm.analyzernode_new(context.__wbg_ptr);
        this.__wbg_ptr = ret >>> 0;
        AnalyzerNodeFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * @param {number} v
     */
    setFftSize(v) {
        wasm.analyzernode_setFftSize(this.__wbg_ptr, v);
    }
    /**
     * @param {number} v
     */
    setMaxDecibels(v) {
        wasm.analyzernode_setMaxDecibels(this.__wbg_ptr, v);
    }
    /**
     * @param {number} v
     */
    setMinDecibels(v) {
        wasm.analyzernode_setMinDecibels(this.__wbg_ptr, v);
    }
    /**
     * @param {number} v
     */
    setSmoothingTimeConstant(v) {
        wasm.analyzernode_setSmoothingTimeConstant(this.__wbg_ptr, v);
    }
    /**
     * @returns {number}
     */
    smoothingTimeConstant() {
        const ret = wasm.analyzernode_smoothingTimeConstant(this.__wbg_ptr);
        return ret;
    }
}
if (Symbol.dispose) AnalyzerNode.prototype[Symbol.dispose] = AnalyzerNode.prototype.free;

export class AudioBuffer {
    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(AudioBuffer.prototype);
        obj.__wbg_ptr = ptr;
        AudioBufferFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        AudioBufferFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_audiobuffer_free(ptr, 0);
    }
    /**
     * @returns {any}
     */
    __h() {
        const ret = wasm.audiobuffer___h(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    duration() {
        const ret = wasm.audiobuffer_duration(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {number} channel_number
     * @returns {Float32Array}
     */
    getChannelData(channel_number) {
        const ret = wasm.audiobuffer_getChannelData(this.__wbg_ptr, channel_number);
        var v1 = getArrayF32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @returns {number}
     */
    length() {
        const ret = wasm.audiobuffer_length(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * WIT `constructor(samples: list<list<f32>>, sample-rate: f32)`. jco passes
     * `samples` as a JS array of `Float32Array` (one per channel). Built via the
     * global `AudioBuffer` constructor (context-free), then filled per channel.
     * @param {any} samples
     * @param {number} sample_rate
     */
    constructor(samples, sample_rate) {
        const ret = wasm.audiobuffer_new(samples, sample_rate);
        this.__wbg_ptr = ret >>> 0;
        AudioBufferFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * @returns {number}
     */
    numberOfChannels() {
        const ret = wasm.audiobuffer_numberOfChannels(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    sampleRate() {
        const ret = wasm.audiobuffer_sampleRate(this.__wbg_ptr);
        return ret;
    }
}
if (Symbol.dispose) AudioBuffer.prototype[Symbol.dispose] = AudioBuffer.prototype.free;

export class AudioBufferSourceNode {
    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(AudioBufferSourceNode.prototype);
        obj.__wbg_ptr = ptr;
        AudioBufferSourceNodeFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        AudioBufferSourceNodeFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_audiobuffersourcenode_free(ptr, 0);
    }
    /**
     * @returns {any}
     */
    __h() {
        const ret = wasm.audiobuffersourcenode___h(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {AudioBuffer | undefined}
     */
    buffer() {
        const ret = wasm.audiobuffersourcenode_buffer(this.__wbg_ptr);
        return ret === 0 ? undefined : AudioBuffer.__wrap(ret);
    }
    /**
     * @param {any} destination
     */
    connect(destination) {
        wasm.audiobuffersourcenode_connect(this.__wbg_ptr, destination);
    }
    /**
     * @returns {AudioParam}
     */
    detune() {
        const ret = wasm.audiobuffersourcenode_detune(this.__wbg_ptr);
        return AudioParam.__wrap(ret);
    }
    /**
     * @returns {boolean}
     */
    loop() {
        const ret = wasm.audiobuffersourcenode_loop(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @returns {number}
     */
    loopEnd() {
        const ret = wasm.audiobuffersourcenode_loopEnd(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    loopStart() {
        const ret = wasm.audiobuffersourcenode_loopStart(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {AudioContext} context
     */
    constructor(context) {
        _assertClass(context, AudioContext);
        const ret = wasm.audiobuffersourcenode_new(context.__wbg_ptr);
        this.__wbg_ptr = ret >>> 0;
        AudioBufferSourceNodeFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * @returns {AudioParam}
     */
    playbackRate() {
        const ret = wasm.audiobuffersourcenode_playbackRate(this.__wbg_ptr);
        return AudioParam.__wrap(ret);
    }
    /**
     * @returns {number}
     */
    position() {
        const ret = wasm.audiobuffersourcenode_position(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {AudioBuffer} audio_buffer
     */
    setBuffer(audio_buffer) {
        _assertClass(audio_buffer, AudioBuffer);
        wasm.audiobuffersourcenode_setBuffer(this.__wbg_ptr, audio_buffer.__wbg_ptr);
    }
    /**
     * @param {boolean} value
     */
    setLoop(value) {
        wasm.audiobuffersourcenode_setLoop(this.__wbg_ptr, value);
    }
    /**
     * @param {number} value
     */
    setLoopEnd(value) {
        wasm.audiobuffersourcenode_setLoopEnd(this.__wbg_ptr, value);
    }
    /**
     * @param {number} value
     */
    setLoopStart(value) {
        wasm.audiobuffersourcenode_setLoopStart(this.__wbg_ptr, value);
    }
    start() {
        wasm.audiobuffersourcenode_start(this.__wbg_ptr);
    }
    /**
     * @param {number} start
     * @param {number} offset
     */
    startAtWithOffset(start, offset) {
        wasm.audiobuffersourcenode_startAtWithOffset(this.__wbg_ptr, start, offset);
    }
    /**
     * @param {number} start
     * @param {number} offset
     * @param {number} duration
     */
    startAtWithOffsetAndDuration(start, offset, duration) {
        wasm.audiobuffersourcenode_startAtWithOffsetAndDuration(this.__wbg_ptr, start, offset, duration);
    }
}
if (Symbol.dispose) AudioBufferSourceNode.prototype[Symbol.dispose] = AudioBufferSourceNode.prototype.free;

export class AudioContext {
    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(AudioContext.prototype);
        obj.__wbg_ptr = ptr;
        AudioContextFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        AudioContextFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_audiocontext_free(ptr, 0);
    }
    /**
     * @returns {any}
     */
    __h() {
        const ret = wasm.audiocontext___h(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    baseLatency() {
        const ret = wasm.audiocontext_baseLatency(this.__wbg_ptr);
        return ret;
    }
    close() {
        wasm.audiocontext_close(this.__wbg_ptr);
    }
    /**
     * @returns {AnalyzerNode}
     */
    createAnalyzer() {
        const ret = wasm.analyzernode_new(this.__wbg_ptr);
        return AnalyzerNode.__wrap(ret);
    }
    /**
     * @returns {BiquadFilterNode}
     */
    createBiquadFilter() {
        const ret = wasm.audiocontext_createBiquadFilter(this.__wbg_ptr);
        return BiquadFilterNode.__wrap(ret);
    }
    /**
     * @param {number} number_of_channels
     * @param {number} length
     * @param {number} sample_rate
     * @returns {AudioBuffer}
     */
    createBuffer(number_of_channels, length, sample_rate) {
        const ret = wasm.audiocontext_createBuffer(this.__wbg_ptr, number_of_channels, length, sample_rate);
        return AudioBuffer.__wrap(ret);
    }
    /**
     * @returns {AudioBufferSourceNode}
     */
    createBufferSource() {
        const ret = wasm.audiobuffersourcenode_new(this.__wbg_ptr);
        return AudioBufferSourceNode.__wrap(ret);
    }
    /**
     * @param {number} number_of_inputs
     * @returns {ChannelMergerNode}
     */
    createChannelMerger(number_of_inputs) {
        const ret = wasm.audiocontext_createChannelMerger(this.__wbg_ptr, number_of_inputs);
        return ChannelMergerNode.__wrap(ret);
    }
    /**
     * @param {number} number_of_outputs
     * @returns {ChannelSplitterNode}
     */
    createChannelSplitter(number_of_outputs) {
        const ret = wasm.audiocontext_createChannelSplitter(this.__wbg_ptr, number_of_outputs);
        return ChannelSplitterNode.__wrap(ret);
    }
    /**
     * @returns {ConstantSourceNode}
     */
    createConstantSource() {
        const ret = wasm.audiocontext_createConstantSource(this.__wbg_ptr);
        return ConstantSourceNode.__wrap(ret);
    }
    /**
     * @returns {ConvolverNode}
     */
    createConvolver() {
        const ret = wasm.audiocontext_createConvolver(this.__wbg_ptr);
        return ConvolverNode.__wrap(ret);
    }
    /**
     * @param {number} max_delay_time
     * @returns {DelayNode}
     */
    createDelay(max_delay_time) {
        const ret = wasm.audiocontext_createDelay(this.__wbg_ptr, max_delay_time);
        return DelayNode.__wrap(ret);
    }
    /**
     * @returns {DynamicsCompressorNode}
     */
    createDynamicsCompressor() {
        const ret = wasm.audiocontext_createDynamicsCompressor(this.__wbg_ptr);
        return DynamicsCompressorNode.__wrap(ret);
    }
    /**
     * @returns {GainNode}
     */
    createGain() {
        const ret = wasm.audiocontext_createGain(this.__wbg_ptr);
        return GainNode.__wrap(ret);
    }
    /**
     * @param {any} feedforward
     * @param {any} feedback
     * @returns {IirFilterNode}
     */
    createIirFilter(feedforward, feedback) {
        const ret = wasm.audiocontext_createIirFilter(this.__wbg_ptr, feedforward, feedback);
        return IirFilterNode.__wrap(ret);
    }
    /**
     * @returns {OscillatorNode}
     */
    createOscillator() {
        const ret = wasm.audiocontext_createOscillator(this.__wbg_ptr);
        return OscillatorNode.__wrap(ret);
    }
    /**
     * @returns {PannerNode}
     */
    createPanner() {
        const ret = wasm.audiocontext_createPanner(this.__wbg_ptr);
        return PannerNode.__wrap(ret);
    }
    /**
     * @param {any} options
     * @returns {PeriodicWave}
     */
    createPeriodicWave(options) {
        const ret = wasm.audiocontext_createPeriodicWave(this.__wbg_ptr, options);
        return PeriodicWave.__wrap(ret);
    }
    /**
     * @returns {StereoPannerNode}
     */
    createStereoPanner() {
        const ret = wasm.audiocontext_createStereoPanner(this.__wbg_ptr);
        return StereoPannerNode.__wrap(ret);
    }
    /**
     * @returns {WaveShaperNode}
     */
    createWaveShaper() {
        const ret = wasm.audiocontext_createWaveShaper(this.__wbg_ptr);
        return WaveShaperNode.__wrap(ret);
    }
    /**
     * @returns {number}
     */
    currentTime() {
        const ret = wasm.audiocontext_currentTime(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {Uint8Array} _data
     * @returns {AudioBuffer}
     */
    decodeAudioData(_data) {
        const ptr0 = passArray8ToWasm0(_data, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.audiocontext_decodeAudioData(this.__wbg_ptr, ptr0, len0);
        return AudioBuffer.__wrap(ret);
    }
    /**
     * @returns {AudioDestinationNode}
     */
    destination() {
        const ret = wasm.audiocontext_destination(this.__wbg_ptr);
        return AudioDestinationNode.__wrap(ret);
    }
    /**
     * @returns {AudioListener}
     */
    listener() {
        const ret = wasm.audiocontext_listener(this.__wbg_ptr);
        return AudioListener.__wrap(ret);
    }
    /**
     * @returns {number}
     */
    outputLatency() {
        const ret = wasm.audiocontext_outputLatency(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {AudioRenderCapacity}
     */
    renderCapacity() {
        const ret = wasm.audiocontext_renderCapacity(this.__wbg_ptr);
        return AudioRenderCapacity.__wrap(ret);
    }
    resume() {
        wasm.audiocontext_resume(this.__wbg_ptr);
    }
    /**
     * @returns {number}
     */
    sampleRate() {
        const ret = wasm.audiocontext_sampleRate(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {string} sink_id
     */
    setSinkId(sink_id) {
        const ptr0 = passStringToWasm0(sink_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        wasm.audiocontext_setSinkId(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * @returns {string}
     */
    sinkId() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.audiocontext_sinkId(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @returns {any}
     */
    state() {
        const ret = wasm.audiocontext_state(this.__wbg_ptr);
        return ret;
    }
    suspend() {
        wasm.audiocontext_suspend(this.__wbg_ptr);
    }
}
if (Symbol.dispose) AudioContext.prototype[Symbol.dispose] = AudioContext.prototype.free;

export class AudioDestinationNode {
    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(AudioDestinationNode.prototype);
        obj.__wbg_ptr = ptr;
        AudioDestinationNodeFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        AudioDestinationNodeFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_audiodestinationnode_free(ptr, 0);
    }
    /**
     * @returns {any}
     */
    __h() {
        const ret = wasm.audiodestinationnode___h(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    maxChannelCount() {
        const ret = wasm.audiodestinationnode_maxChannelCount(this.__wbg_ptr);
        return ret >>> 0;
    }
}
if (Symbol.dispose) AudioDestinationNode.prototype[Symbol.dispose] = AudioDestinationNode.prototype.free;

export class AudioDevice {
    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(AudioDevice.prototype);
        obj.__wbg_ptr = ptr;
        AudioDeviceFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        AudioDeviceFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_audiodevice_free(ptr, 0);
    }
    /**
     * @returns {AudioContext}
     */
    createContext() {
        const ret = wasm.audiodevice_createContext(this.__wbg_ptr);
        return AudioContext.__wrap(ret);
    }
    /**
     * @returns {string}
     */
    name() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.audiodevice_name(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
}
if (Symbol.dispose) AudioDevice.prototype[Symbol.dispose] = AudioDevice.prototype.free;

export class AudioListener {
    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(AudioListener.prototype);
        obj.__wbg_ptr = ptr;
        AudioListenerFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        AudioListenerFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_audiolistener_free(ptr, 0);
    }
    /**
     * @returns {AudioParam}
     */
    forwardX() {
        const ret = wasm.audiolistener_forwardX(this.__wbg_ptr);
        return AudioParam.__wrap(ret);
    }
    /**
     * @returns {AudioParam}
     */
    forwardY() {
        const ret = wasm.audiolistener_forwardY(this.__wbg_ptr);
        return AudioParam.__wrap(ret);
    }
    /**
     * @returns {AudioParam}
     */
    forwardZ() {
        const ret = wasm.audiolistener_forwardZ(this.__wbg_ptr);
        return AudioParam.__wrap(ret);
    }
    /**
     * @returns {AudioParam}
     */
    positionX() {
        const ret = wasm.audiolistener_positionX(this.__wbg_ptr);
        return AudioParam.__wrap(ret);
    }
    /**
     * @returns {AudioParam}
     */
    positionY() {
        const ret = wasm.audiolistener_positionY(this.__wbg_ptr);
        return AudioParam.__wrap(ret);
    }
    /**
     * @returns {AudioParam}
     */
    positionZ() {
        const ret = wasm.audiolistener_positionZ(this.__wbg_ptr);
        return AudioParam.__wrap(ret);
    }
    /**
     * @returns {AudioParam}
     */
    upX() {
        const ret = wasm.audiolistener_upX(this.__wbg_ptr);
        return AudioParam.__wrap(ret);
    }
    /**
     * @returns {AudioParam}
     */
    upY() {
        const ret = wasm.audiolistener_upY(this.__wbg_ptr);
        return AudioParam.__wrap(ret);
    }
    /**
     * @returns {AudioParam}
     */
    upZ() {
        const ret = wasm.audiolistener_upZ(this.__wbg_ptr);
        return AudioParam.__wrap(ret);
    }
}
if (Symbol.dispose) AudioListener.prototype[Symbol.dispose] = AudioListener.prototype.free;

export class AudioParam {
    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(AudioParam.prototype);
        obj.__wbg_ptr = ptr;
        AudioParamFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        AudioParamFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_audioparam_free(ptr, 0);
    }
    /**
     * @returns {any}
     */
    __h() {
        const ret = wasm.audioparam___h(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {string}
     */
    automationRate() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.audioparam_automationRate(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @param {number} cancel_time
     */
    cancelAndHoldAtTime(cancel_time) {
        wasm.audioparam_cancelAndHoldAtTime(this.__wbg_ptr, cancel_time);
    }
    /**
     * @param {number} cancel_time
     */
    cancelScheduledValues(cancel_time) {
        wasm.audioparam_cancelScheduledValues(this.__wbg_ptr, cancel_time);
    }
    /**
     * @returns {number}
     */
    defaultValue() {
        const ret = wasm.audioparam_defaultValue(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {number} value
     * @param {number} end_time
     */
    exponentialRampToValueAtTime(value, end_time) {
        wasm.audioparam_exponentialRampToValueAtTime(this.__wbg_ptr, value, end_time);
    }
    /**
     * @param {number} value
     * @param {number} end_time
     */
    linearRampToValueAtTime(value, end_time) {
        wasm.audioparam_linearRampToValueAtTime(this.__wbg_ptr, value, end_time);
    }
    /**
     * @returns {number}
     */
    maxValue() {
        const ret = wasm.audioparam_maxValue(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    minValue() {
        const ret = wasm.audioparam_minValue(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {any} rate
     */
    setAutomationRate(rate) {
        wasm.audioparam_setAutomationRate(this.__wbg_ptr, rate);
    }
    /**
     * @param {number} value
     * @param {number} start_time
     * @param {number} time_constant
     */
    setTargetAtTime(value, start_time, time_constant) {
        wasm.audioparam_setTargetAtTime(this.__wbg_ptr, value, start_time, time_constant);
    }
    /**
     * @param {number} value
     */
    setValue(value) {
        wasm.audioparam_setValue(this.__wbg_ptr, value);
    }
    /**
     * @param {number} value
     * @param {number} start_time
     */
    setValueAtTime(value, start_time) {
        wasm.audioparam_setValueAtTime(this.__wbg_ptr, value, start_time);
    }
    /**
     * @param {any} values
     * @param {number} start_time
     * @param {number} duration
     */
    setValueCurveAtTime(values, start_time, duration) {
        wasm.audioparam_setValueCurveAtTime(this.__wbg_ptr, values, start_time, duration);
    }
    /**
     * @returns {number}
     */
    value() {
        const ret = wasm.audioparam_value(this.__wbg_ptr);
        return ret;
    }
}
if (Symbol.dispose) AudioParam.prototype[Symbol.dispose] = AudioParam.prototype.free;

export class AudioRenderCapacity {
    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(AudioRenderCapacity.prototype);
        obj.__wbg_ptr = ptr;
        AudioRenderCapacityFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        AudioRenderCapacityFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_audiorendercapacity_free(ptr, 0);
    }
    /**
     * @returns {any}
     */
    __h() {
        const ret = wasm.audiorendercapacity___h(this.__wbg_ptr);
        return ret;
    }
}
if (Symbol.dispose) AudioRenderCapacity.prototype[Symbol.dispose] = AudioRenderCapacity.prototype.free;

export class BiquadFilterNode {
    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(BiquadFilterNode.prototype);
        obj.__wbg_ptr = ptr;
        BiquadFilterNodeFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        BiquadFilterNodeFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_biquadfilternode_free(ptr, 0);
    }
    /**
     * @returns {any}
     */
    __h() {
        const ret = wasm.biquadfilternode___h(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {any} destination
     */
    connect(destination) {
        wasm.biquadfilternode_connect(this.__wbg_ptr, destination);
    }
    /**
     * @returns {AudioParam}
     */
    detune() {
        const ret = wasm.biquadfilternode_detune(this.__wbg_ptr);
        return AudioParam.__wrap(ret);
    }
    /**
     * @returns {AudioParam}
     */
    frequency() {
        const ret = wasm.biquadfilternode_frequency(this.__wbg_ptr);
        return AudioParam.__wrap(ret);
    }
    /**
     * @returns {AudioParam}
     */
    gain() {
        const ret = wasm.biquadfilternode_gain(this.__wbg_ptr);
        return AudioParam.__wrap(ret);
    }
    /**
     * @param {AudioContext} context
     */
    constructor(context) {
        _assertClass(context, AudioContext);
        const ret = wasm.audiocontext_createBiquadFilter(context.__wbg_ptr);
        this.__wbg_ptr = ret >>> 0;
        BiquadFilterNodeFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * @returns {AudioParam}
     */
    q() {
        const ret = wasm.biquadfilternode_q(this.__wbg_ptr);
        return AudioParam.__wrap(ret);
    }
    /**
     * @param {any} t
     */
    setType(t) {
        wasm.biquadfilternode_setType(this.__wbg_ptr, t);
    }
    /**
     * @returns {any}
     */
    type() {
        const ret = wasm.biquadfilternode_type(this.__wbg_ptr);
        return ret;
    }
}
if (Symbol.dispose) BiquadFilterNode.prototype[Symbol.dispose] = BiquadFilterNode.prototype.free;

export class ChannelMergerNode {
    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(ChannelMergerNode.prototype);
        obj.__wbg_ptr = ptr;
        ChannelMergerNodeFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        ChannelMergerNodeFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_channelmergernode_free(ptr, 0);
    }
    /**
     * @returns {any}
     */
    __h() {
        const ret = wasm.channelmergernode___h(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {any} destination
     */
    connect(destination) {
        wasm.channelmergernode_connect(this.__wbg_ptr, destination);
    }
    /**
     * @param {AudioContext} context
     * @param {number} number_of_inputs
     */
    constructor(context, number_of_inputs) {
        _assertClass(context, AudioContext);
        const ret = wasm.audiocontext_createChannelMerger(context.__wbg_ptr, number_of_inputs);
        this.__wbg_ptr = ret >>> 0;
        ChannelMergerNodeFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
}
if (Symbol.dispose) ChannelMergerNode.prototype[Symbol.dispose] = ChannelMergerNode.prototype.free;

export class ChannelSplitterNode {
    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(ChannelSplitterNode.prototype);
        obj.__wbg_ptr = ptr;
        ChannelSplitterNodeFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        ChannelSplitterNodeFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_channelsplitternode_free(ptr, 0);
    }
    /**
     * @returns {any}
     */
    __h() {
        const ret = wasm.channelsplitternode___h(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {any} destination
     */
    connect(destination) {
        wasm.channelsplitternode_connect(this.__wbg_ptr, destination);
    }
    /**
     * @param {AudioContext} context
     * @param {number} number_of_outputs
     */
    constructor(context, number_of_outputs) {
        _assertClass(context, AudioContext);
        const ret = wasm.audiocontext_createChannelSplitter(context.__wbg_ptr, number_of_outputs);
        this.__wbg_ptr = ret >>> 0;
        ChannelSplitterNodeFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
}
if (Symbol.dispose) ChannelSplitterNode.prototype[Symbol.dispose] = ChannelSplitterNode.prototype.free;

export class ConstantSourceNode {
    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(ConstantSourceNode.prototype);
        obj.__wbg_ptr = ptr;
        ConstantSourceNodeFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        ConstantSourceNodeFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_constantsourcenode_free(ptr, 0);
    }
    /**
     * @returns {any}
     */
    __h() {
        const ret = wasm.constantsourcenode___h(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {any} destination
     */
    connect(destination) {
        wasm.constantsourcenode_connect(this.__wbg_ptr, destination);
    }
    /**
     * @param {AudioContext} context
     */
    constructor(context) {
        _assertClass(context, AudioContext);
        const ret = wasm.audiocontext_createConstantSource(context.__wbg_ptr);
        this.__wbg_ptr = ret >>> 0;
        ConstantSourceNodeFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * @returns {AudioParam}
     */
    offset() {
        const ret = wasm.constantsourcenode_offset(this.__wbg_ptr);
        return AudioParam.__wrap(ret);
    }
}
if (Symbol.dispose) ConstantSourceNode.prototype[Symbol.dispose] = ConstantSourceNode.prototype.free;

export class ConvolverNode {
    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(ConvolverNode.prototype);
        obj.__wbg_ptr = ptr;
        ConvolverNodeFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        ConvolverNodeFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_convolvernode_free(ptr, 0);
    }
    /**
     * @returns {any}
     */
    __h() {
        const ret = wasm.convolvernode___h(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {AudioBuffer | undefined}
     */
    buffer() {
        const ret = wasm.convolvernode_buffer(this.__wbg_ptr);
        return ret === 0 ? undefined : AudioBuffer.__wrap(ret);
    }
    /**
     * @param {any} destination
     */
    connect(destination) {
        wasm.convolvernode_connect(this.__wbg_ptr, destination);
    }
    /**
     * @param {AudioContext} context
     */
    constructor(context) {
        _assertClass(context, AudioContext);
        const ret = wasm.audiocontext_createConvolver(context.__wbg_ptr);
        this.__wbg_ptr = ret >>> 0;
        ConvolverNodeFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * @returns {boolean}
     */
    normalize() {
        const ret = wasm.convolvernode_normalize(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @param {AudioBuffer} buffer
     */
    setBuffer(buffer) {
        _assertClass(buffer, AudioBuffer);
        wasm.convolvernode_setBuffer(this.__wbg_ptr, buffer.__wbg_ptr);
    }
    /**
     * @param {boolean} value
     */
    setNormalize(value) {
        wasm.convolvernode_setNormalize(this.__wbg_ptr, value);
    }
}
if (Symbol.dispose) ConvolverNode.prototype[Symbol.dispose] = ConvolverNode.prototype.free;

export class DelayNode {
    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(DelayNode.prototype);
        obj.__wbg_ptr = ptr;
        DelayNodeFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        DelayNodeFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_delaynode_free(ptr, 0);
    }
    /**
     * @returns {any}
     */
    __h() {
        const ret = wasm.delaynode___h(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {any} destination
     */
    connect(destination) {
        wasm.delaynode_connect(this.__wbg_ptr, destination);
    }
    /**
     * @returns {AudioParam}
     */
    delayTime() {
        const ret = wasm.delaynode_delayTime(this.__wbg_ptr);
        return AudioParam.__wrap(ret);
    }
    /**
     * @param {AudioContext} context
     * @param {number} max_delay_time
     */
    constructor(context, max_delay_time) {
        _assertClass(context, AudioContext);
        const ret = wasm.audiocontext_createDelay(context.__wbg_ptr, max_delay_time);
        this.__wbg_ptr = ret >>> 0;
        DelayNodeFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
}
if (Symbol.dispose) DelayNode.prototype[Symbol.dispose] = DelayNode.prototype.free;

export class DynamicsCompressorNode {
    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(DynamicsCompressorNode.prototype);
        obj.__wbg_ptr = ptr;
        DynamicsCompressorNodeFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        DynamicsCompressorNodeFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_dynamicscompressornode_free(ptr, 0);
    }
    /**
     * @returns {any}
     */
    __h() {
        const ret = wasm.dynamicscompressornode___h(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {AudioParam}
     */
    attack() {
        const ret = wasm.dynamicscompressornode_attack(this.__wbg_ptr);
        return AudioParam.__wrap(ret);
    }
    /**
     * @param {any} destination
     */
    connect(destination) {
        wasm.dynamicscompressornode_connect(this.__wbg_ptr, destination);
    }
    /**
     * @returns {AudioParam}
     */
    knee() {
        const ret = wasm.dynamicscompressornode_knee(this.__wbg_ptr);
        return AudioParam.__wrap(ret);
    }
    /**
     * @param {AudioContext} context
     */
    constructor(context) {
        _assertClass(context, AudioContext);
        const ret = wasm.audiocontext_createDynamicsCompressor(context.__wbg_ptr);
        this.__wbg_ptr = ret >>> 0;
        DynamicsCompressorNodeFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * @returns {AudioParam}
     */
    ratio() {
        const ret = wasm.dynamicscompressornode_ratio(this.__wbg_ptr);
        return AudioParam.__wrap(ret);
    }
    /**
     * @returns {number}
     */
    reduction() {
        const ret = wasm.dynamicscompressornode_reduction(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {AudioParam}
     */
    release() {
        const ret = wasm.dynamicscompressornode_release(this.__wbg_ptr);
        return AudioParam.__wrap(ret);
    }
    /**
     * @returns {AudioParam}
     */
    threshold() {
        const ret = wasm.dynamicscompressornode_threshold(this.__wbg_ptr);
        return AudioParam.__wrap(ret);
    }
}
if (Symbol.dispose) DynamicsCompressorNode.prototype[Symbol.dispose] = DynamicsCompressorNode.prototype.free;

export class GainNode {
    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(GainNode.prototype);
        obj.__wbg_ptr = ptr;
        GainNodeFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        GainNodeFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_gainnode_free(ptr, 0);
    }
    /**
     * @returns {any}
     */
    __h() {
        const ret = wasm.gainnode___h(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {any} destination
     */
    connect(destination) {
        wasm.gainnode_connect(this.__wbg_ptr, destination);
    }
    /**
     * @returns {AudioParam}
     */
    gain() {
        const ret = wasm.gainnode_gain(this.__wbg_ptr);
        return AudioParam.__wrap(ret);
    }
    /**
     * @param {AudioContext} context
     */
    constructor(context) {
        _assertClass(context, AudioContext);
        const ret = wasm.audiocontext_createGain(context.__wbg_ptr);
        this.__wbg_ptr = ret >>> 0;
        GainNodeFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
}
if (Symbol.dispose) GainNode.prototype[Symbol.dispose] = GainNode.prototype.free;

export class GamepadDevice {
    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(GamepadDevice.prototype);
        obj.__wbg_ptr = ptr;
        GamepadDeviceFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        GamepadDeviceFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_gamepaddevice_free(ptr, 0);
    }
    /**
     * @param {any} axis
     * @returns {any}
     */
    axisData(axis) {
        const ret = wasm.gamepaddevice_axisData(this.__wbg_ptr, axis);
        return ret;
    }
    /**
     * @param {any} btn
     * @returns {any}
     */
    buttonData(btn) {
        const ret = wasm.gamepaddevice_buttonData(this.__wbg_ptr, btn);
        return ret;
    }
    /**
     * @param {any} btn
     * @returns {boolean}
     */
    isPressed(btn) {
        const ret = wasm.gamepaddevice_isPressed(this.__wbg_ptr, btn);
        return ret !== 0;
    }
    /**
     * @returns {string}
     */
    name() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.gamepaddevice_name(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @param {any} axis
     * @returns {number}
     */
    value(axis) {
        const ret = wasm.gamepaddevice_value(this.__wbg_ptr, axis);
        return ret;
    }
}
if (Symbol.dispose) GamepadDevice.prototype[Symbol.dispose] = GamepadDevice.prototype.free;

export class GpuAdapter {
    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(GpuAdapter.prototype);
        obj.__wbg_ptr = ptr;
        GpuAdapterFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        GpuAdapterFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_gpuadapter_free(ptr, 0);
    }
    /**
     * @returns {GpuDevice}
     */
    requestDevice() {
        const ret = wasm.gpuadapter_requestDevice(this.__wbg_ptr);
        return GpuDevice.__wrap(ret);
    }
}
if (Symbol.dispose) GpuAdapter.prototype[Symbol.dispose] = GpuAdapter.prototype.free;

export class GpuBindGroup {
    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(GpuBindGroup.prototype);
        obj.__wbg_ptr = ptr;
        GpuBindGroupFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        GpuBindGroupFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_gpubindgroup_free(ptr, 0);
    }
    /**
     * @returns {any}
     */
    __h() {
        const ret = wasm.gpubindgroup___h(this.__wbg_ptr);
        return ret;
    }
}
if (Symbol.dispose) GpuBindGroup.prototype[Symbol.dispose] = GpuBindGroup.prototype.free;

export class GpuBindGroupLayout {
    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(GpuBindGroupLayout.prototype);
        obj.__wbg_ptr = ptr;
        GpuBindGroupLayoutFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        GpuBindGroupLayoutFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_gpubindgrouplayout_free(ptr, 0);
    }
    /**
     * @returns {any}
     */
    __h() {
        const ret = wasm.gpubindgrouplayout___h(this.__wbg_ptr);
        return ret;
    }
}
if (Symbol.dispose) GpuBindGroupLayout.prototype[Symbol.dispose] = GpuBindGroupLayout.prototype.free;

export class GpuBuffer {
    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(GpuBuffer.prototype);
        obj.__wbg_ptr = ptr;
        GpuBufferFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        GpuBufferFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_gpubuffer_free(ptr, 0);
    }
    /**
     * @returns {any}
     */
    __h() {
        const ret = wasm.gpubuffer___h(this.__wbg_ptr);
        return ret;
    }
    destroy() {
        wasm.gpubuffer_destroy(this.__wbg_ptr);
    }
    /**
     * @param {any} offset
     * @param {any} size
     * @returns {Uint8Array}
     */
    getMappedRange(offset, size) {
        const ret = wasm.gpubuffer_getMappedRange(this.__wbg_ptr, offset, size);
        var v1 = getArrayU8FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        return v1;
    }
    /**
     * @param {any} mode
     * @param {any} offset
     * @param {any} size
     */
    map(mode, offset, size) {
        wasm.gpubuffer_map(this.__wbg_ptr, mode, offset, size);
    }
    /**
     * @returns {any}
     */
    mapState() {
        const ret = wasm.gpubuffer_mapState(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    size() {
        const ret = wasm.gpubuffer_size(this.__wbg_ptr);
        return ret;
    }
    unmap() {
        wasm.gpubuffer_unmap(this.__wbg_ptr);
    }
    /**
     * @returns {any}
     */
    usage() {
        const ret = wasm.gpubuffer_usage(this.__wbg_ptr);
        return ret;
    }
}
if (Symbol.dispose) GpuBuffer.prototype[Symbol.dispose] = GpuBuffer.prototype.free;

export class GpuCommandBuffer {
    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(GpuCommandBuffer.prototype);
        obj.__wbg_ptr = ptr;
        GpuCommandBufferFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        GpuCommandBufferFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_gpucommandbuffer_free(ptr, 0);
    }
    /**
     * @returns {any}
     */
    __h() {
        const ret = wasm.gpucommandbuffer___h(this.__wbg_ptr);
        return ret;
    }
}
if (Symbol.dispose) GpuCommandBuffer.prototype[Symbol.dispose] = GpuCommandBuffer.prototype.free;

export class GpuCommandEncoder {
    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(GpuCommandEncoder.prototype);
        obj.__wbg_ptr = ptr;
        GpuCommandEncoderFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        GpuCommandEncoderFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_gpucommandencoder_free(ptr, 0);
    }
    /**
     * @param {any} d
     * @returns {GpuComputePassEncoder}
     */
    beginComputePass(d) {
        const ret = wasm.gpucommandencoder_beginComputePass(this.__wbg_ptr, d);
        return GpuComputePassEncoder.__wrap(ret);
    }
    /**
     * @param {any} d
     * @returns {GpuRenderPassEncoder}
     */
    beginRenderPass(d) {
        const ret = wasm.gpucommandencoder_beginRenderPass(this.__wbg_ptr, d);
        return GpuRenderPassEncoder.__wrap(ret);
    }
    /**
     * @param {GpuBuffer} buffer
     * @param {any} offset
     * @param {any} size
     */
    clearBuffer(buffer, offset, size) {
        _assertClass(buffer, GpuBuffer);
        wasm.gpucommandencoder_clearBuffer(this.__wbg_ptr, buffer.__wbg_ptr, offset, size);
    }
    /**
     * @param {GpuBuffer} source
     * @param {any} source_offset
     * @param {GpuBuffer} destination
     * @param {any} destination_offset
     * @param {any} size
     */
    copyBufferToBuffer(source, source_offset, destination, destination_offset, size) {
        _assertClass(source, GpuBuffer);
        _assertClass(destination, GpuBuffer);
        wasm.gpucommandencoder_copyBufferToBuffer(this.__wbg_ptr, source.__wbg_ptr, source_offset, destination.__wbg_ptr, destination_offset, size);
    }
    /**
     * @param {any} source
     * @param {any} destination
     * @param {any} copy_size
     */
    copyBufferToTexture(source, destination, copy_size) {
        wasm.gpucommandencoder_copyBufferToTexture(this.__wbg_ptr, source, destination, copy_size);
    }
    /**
     * @param {any} source
     * @param {any} destination
     * @param {any} copy_size
     */
    copyTextureToBuffer(source, destination, copy_size) {
        wasm.gpucommandencoder_copyTextureToBuffer(this.__wbg_ptr, source, destination, copy_size);
    }
    /**
     * @param {any} source
     * @param {any} destination
     * @param {any} copy_size
     */
    copyTextureToTexture(source, destination, copy_size) {
        wasm.gpucommandencoder_copyTextureToTexture(this.__wbg_ptr, source, destination, copy_size);
    }
    /**
     * @returns {GpuCommandBuffer}
     */
    finish() {
        const ret = wasm.gpucommandencoder_finish(this.__wbg_ptr);
        return GpuCommandBuffer.__wrap(ret);
    }
    /**
     * @param {GpuQuerySet} query_set
     * @param {number} first_query
     * @param {number} query_count
     * @param {GpuBuffer} destination
     * @param {any} destination_offset
     */
    resolveQuerySet(query_set, first_query, query_count, destination, destination_offset) {
        _assertClass(query_set, GpuQuerySet);
        _assertClass(destination, GpuBuffer);
        wasm.gpucommandencoder_resolveQuerySet(this.__wbg_ptr, query_set.__wbg_ptr, first_query, query_count, destination.__wbg_ptr, destination_offset);
    }
    /**
     * @param {GpuQuerySet} query_set
     * @param {number} query_index
     */
    writeTimestamp(query_set, query_index) {
        _assertClass(query_set, GpuQuerySet);
        wasm.gpucommandencoder_writeTimestamp(this.__wbg_ptr, query_set.__wbg_ptr, query_index);
    }
}
if (Symbol.dispose) GpuCommandEncoder.prototype[Symbol.dispose] = GpuCommandEncoder.prototype.free;

export class GpuComputePassEncoder {
    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(GpuComputePassEncoder.prototype);
        obj.__wbg_ptr = ptr;
        GpuComputePassEncoderFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        GpuComputePassEncoderFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_gpucomputepassencoder_free(ptr, 0);
    }
    /**
     * @param {number} x
     * @param {any} y
     * @param {any} z
     */
    dispatchWorkgroups(x, y, z) {
        wasm.gpucomputepassencoder_dispatchWorkgroups(this.__wbg_ptr, x, y, z);
    }
    /**
     * @param {GpuBuffer} indirect_buffer
     * @param {any} indirect_offset
     */
    dispatchWorkgroupsIndirect(indirect_buffer, indirect_offset) {
        _assertClass(indirect_buffer, GpuBuffer);
        wasm.gpucomputepassencoder_dispatchWorkgroupsIndirect(this.__wbg_ptr, indirect_buffer.__wbg_ptr, indirect_offset);
    }
    end() {
        wasm.gpucomputepassencoder_end(this.__wbg_ptr);
    }
    /**
     * @param {string} label
     */
    insertDebugMarker(label) {
        const ptr0 = passStringToWasm0(label, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        wasm.gpucomputepassencoder_insertDebugMarker(this.__wbg_ptr, ptr0, len0);
    }
    popDebugGroup() {
        wasm.gpucomputepassencoder_popDebugGroup(this.__wbg_ptr);
    }
    /**
     * @param {string} label
     */
    pushDebugGroup(label) {
        const ptr0 = passStringToWasm0(label, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        wasm.gpucomputepassencoder_pushDebugGroup(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * @param {number} index
     * @param {any} bind_group
     * @param {any} dynamic_offsets
     */
    setBindGroup(index, bind_group, dynamic_offsets) {
        wasm.gpucomputepassencoder_setBindGroup(this.__wbg_ptr, index, bind_group, dynamic_offsets);
    }
    /**
     * @param {GpuComputePipeline} pipeline
     */
    setPipeline(pipeline) {
        _assertClass(pipeline, GpuComputePipeline);
        wasm.gpucomputepassencoder_setPipeline(this.__wbg_ptr, pipeline.__wbg_ptr);
    }
}
if (Symbol.dispose) GpuComputePassEncoder.prototype[Symbol.dispose] = GpuComputePassEncoder.prototype.free;

export class GpuComputePipeline {
    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(GpuComputePipeline.prototype);
        obj.__wbg_ptr = ptr;
        GpuComputePipelineFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        GpuComputePipelineFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_gpucomputepipeline_free(ptr, 0);
    }
    /**
     * @returns {any}
     */
    __h() {
        const ret = wasm.gpucomputepipeline___h(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {number} index
     * @returns {GpuBindGroupLayout}
     */
    getBindGroupLayout(index) {
        const ret = wasm.gpucomputepipeline_getBindGroupLayout(this.__wbg_ptr, index);
        return GpuBindGroupLayout.__wrap(ret);
    }
}
if (Symbol.dispose) GpuComputePipeline.prototype[Symbol.dispose] = GpuComputePipeline.prototype.free;

export class GpuDevice {
    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(GpuDevice.prototype);
        obj.__wbg_ptr = ptr;
        GpuDeviceFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        GpuDeviceFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_gpudevice_free(ptr, 0);
    }
    /**
     * @param {any} d
     * @returns {GpuBindGroup}
     */
    createBindGroup(d) {
        const ret = wasm.gpudevice_createBindGroup(this.__wbg_ptr, d);
        return GpuBindGroup.__wrap(ret);
    }
    /**
     * @param {any} d
     * @returns {GpuBindGroupLayout}
     */
    createBindGroupLayout(d) {
        const ret = wasm.gpudevice_createBindGroupLayout(this.__wbg_ptr, d);
        return GpuBindGroupLayout.__wrap(ret);
    }
    /**
     * @param {any} d
     * @returns {GpuBuffer}
     */
    createBuffer(d) {
        const ret = wasm.gpudevice_createBuffer(this.__wbg_ptr, d);
        return GpuBuffer.__wrap(ret);
    }
    /**
     * @param {any} _d
     * @returns {GpuCommandEncoder}
     */
    createCommandEncoder(_d) {
        const ret = wasm.gpudevice_createCommandEncoder(this.__wbg_ptr, _d);
        return GpuCommandEncoder.__wrap(ret);
    }
    /**
     * @param {any} d
     * @returns {GpuComputePipeline}
     */
    createComputePipeline(d) {
        const ret = wasm.gpudevice_createComputePipeline(this.__wbg_ptr, d);
        return GpuComputePipeline.__wrap(ret);
    }
    /**
     * @param {any} d
     * @returns {GpuPipelineLayout}
     */
    createPipelineLayout(d) {
        const ret = wasm.gpudevice_createPipelineLayout(this.__wbg_ptr, d);
        return GpuPipelineLayout.__wrap(ret);
    }
    /**
     * @param {any} d
     * @returns {GpuQuerySet}
     */
    createQuerySet(d) {
        const ret = wasm.gpudevice_createQuerySet(this.__wbg_ptr, d);
        return GpuQuerySet.__wrap(ret);
    }
    /**
     * @param {any} d
     * @returns {GpuRenderBundleEncoder}
     */
    createRenderBundleEncoder(d) {
        const ret = wasm.gpudevice_createRenderBundleEncoder(this.__wbg_ptr, d);
        return GpuRenderBundleEncoder.__wrap(ret);
    }
    /**
     * @param {any} d
     * @returns {GpuRenderPipeline}
     */
    createRenderPipeline(d) {
        const ret = wasm.gpudevice_createRenderPipeline(this.__wbg_ptr, d);
        return GpuRenderPipeline.__wrap(ret);
    }
    /**
     * @param {any} d
     * @returns {GpuSampler}
     */
    createSampler(d) {
        const ret = wasm.gpudevice_createSampler(this.__wbg_ptr, d);
        return GpuSampler.__wrap(ret);
    }
    /**
     * @param {any} d
     * @returns {GpuShaderModule}
     */
    createShaderModule(d) {
        const ret = wasm.gpudevice_createShaderModule(this.__wbg_ptr, d);
        return GpuShaderModule.__wrap(ret);
    }
    /**
     * @param {any} d
     * @returns {GpuTexture}
     */
    createTexture(d) {
        const ret = wasm.gpudevice_createTexture(this.__wbg_ptr, d);
        return GpuTexture.__wrap(ret);
    }
    /**
     * @returns {GpuQueue}
     */
    queue() {
        const ret = wasm.gpudevice_queue(this.__wbg_ptr);
        return GpuQueue.__wrap(ret);
    }
}
if (Symbol.dispose) GpuDevice.prototype[Symbol.dispose] = GpuDevice.prototype.free;

export class GpuPipelineLayout {
    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(GpuPipelineLayout.prototype);
        obj.__wbg_ptr = ptr;
        GpuPipelineLayoutFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        GpuPipelineLayoutFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_gpupipelinelayout_free(ptr, 0);
    }
    /**
     * @returns {any}
     */
    __h() {
        const ret = wasm.gpupipelinelayout___h(this.__wbg_ptr);
        return ret;
    }
}
if (Symbol.dispose) GpuPipelineLayout.prototype[Symbol.dispose] = GpuPipelineLayout.prototype.free;

export class GpuQuerySet {
    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(GpuQuerySet.prototype);
        obj.__wbg_ptr = ptr;
        GpuQuerySetFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        GpuQuerySetFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_gpuqueryset_free(ptr, 0);
    }
    /**
     * @returns {any}
     */
    __h() {
        const ret = wasm.gpuqueryset___h(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    count() {
        const ret = wasm.gpuqueryset_count(this.__wbg_ptr);
        return ret >>> 0;
    }
    destroy() {
        wasm.gpuqueryset_destroy(this.__wbg_ptr);
    }
    /**
     * @returns {any}
     */
    type() {
        const ret = wasm.gpuqueryset_type(this.__wbg_ptr);
        return ret;
    }
}
if (Symbol.dispose) GpuQuerySet.prototype[Symbol.dispose] = GpuQuerySet.prototype.free;

export class GpuQueue {
    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(GpuQueue.prototype);
        obj.__wbg_ptr = ptr;
        GpuQueueFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        GpuQueueFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_gpuqueue_free(ptr, 0);
    }
    /**
     * @param {any} command_buffers
     */
    submit(command_buffers) {
        wasm.gpuqueue_submit(this.__wbg_ptr, command_buffers);
    }
    /**
     * @param {GpuBuffer} buffer
     * @param {any} buffer_offset
     * @param {any} data
     * @param {any} data_offset
     * @param {any} size
     */
    writeBuffer(buffer, buffer_offset, data, data_offset, size) {
        _assertClass(buffer, GpuBuffer);
        wasm.gpuqueue_writeBuffer(this.__wbg_ptr, buffer.__wbg_ptr, buffer_offset, data, data_offset, size);
    }
    /**
     * @param {any} destination
     * @param {any} data
     * @param {any} data_layout
     * @param {any} size
     */
    writeTexture(destination, data, data_layout, size) {
        wasm.gpuqueue_writeTexture(this.__wbg_ptr, destination, data, data_layout, size);
    }
}
if (Symbol.dispose) GpuQueue.prototype[Symbol.dispose] = GpuQueue.prototype.free;

export class GpuRenderBundle {
    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(GpuRenderBundle.prototype);
        obj.__wbg_ptr = ptr;
        GpuRenderBundleFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        GpuRenderBundleFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_gpurenderbundle_free(ptr, 0);
    }
    /**
     * @returns {any}
     */
    __h() {
        const ret = wasm.gpurenderbundle___h(this.__wbg_ptr);
        return ret;
    }
}
if (Symbol.dispose) GpuRenderBundle.prototype[Symbol.dispose] = GpuRenderBundle.prototype.free;

export class GpuRenderBundleEncoder {
    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(GpuRenderBundleEncoder.prototype);
        obj.__wbg_ptr = ptr;
        GpuRenderBundleEncoderFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        GpuRenderBundleEncoderFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_gpurenderbundleencoder_free(ptr, 0);
    }
    /**
     * @param {any} d
     * @returns {GpuRenderBundle}
     */
    finish(d) {
        const ret = wasm.gpurenderbundleencoder_finish(this.__wbg_ptr, d);
        return GpuRenderBundle.__wrap(ret);
    }
}
if (Symbol.dispose) GpuRenderBundleEncoder.prototype[Symbol.dispose] = GpuRenderBundleEncoder.prototype.free;

export class GpuRenderPassEncoder {
    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(GpuRenderPassEncoder.prototype);
        obj.__wbg_ptr = ptr;
        GpuRenderPassEncoderFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        GpuRenderPassEncoderFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_gpurenderpassencoder_free(ptr, 0);
    }
    /**
     * @param {number} query_index
     */
    beginOcclusionQuery(query_index) {
        wasm.gpurenderpassencoder_beginOcclusionQuery(this.__wbg_ptr, query_index);
    }
    /**
     * @param {number} vertex_count
     * @param {number} instance_count
     * @param {number} first_vertex
     * @param {number} first_instance
     */
    draw(vertex_count, instance_count, first_vertex, first_instance) {
        wasm.gpurenderpassencoder_draw(this.__wbg_ptr, vertex_count, instance_count, first_vertex, first_instance);
    }
    /**
     * @param {number} index_count
     * @param {number} instance_count
     * @param {number} first_index
     * @param {number} base_vertex
     * @param {number} first_instance
     */
    drawIndexed(index_count, instance_count, first_index, base_vertex, first_instance) {
        wasm.gpurenderpassencoder_drawIndexed(this.__wbg_ptr, index_count, instance_count, first_index, base_vertex, first_instance);
    }
    /**
     * @param {GpuBuffer} indirect_buffer
     * @param {any} indirect_offset
     */
    drawIndexedIndirect(indirect_buffer, indirect_offset) {
        _assertClass(indirect_buffer, GpuBuffer);
        wasm.gpurenderpassencoder_drawIndexedIndirect(this.__wbg_ptr, indirect_buffer.__wbg_ptr, indirect_offset);
    }
    /**
     * @param {GpuBuffer} indirect_buffer
     * @param {any} indirect_offset
     */
    drawIndirect(indirect_buffer, indirect_offset) {
        _assertClass(indirect_buffer, GpuBuffer);
        wasm.gpurenderpassencoder_drawIndirect(this.__wbg_ptr, indirect_buffer.__wbg_ptr, indirect_offset);
    }
    end() {
        wasm.gpurenderpassencoder_end(this.__wbg_ptr);
    }
    endOcclusionQuery() {
        wasm.gpurenderpassencoder_endOcclusionQuery(this.__wbg_ptr);
    }
    /**
     * @param {any} bundles
     */
    executeBundles(bundles) {
        wasm.gpurenderpassencoder_executeBundles(this.__wbg_ptr, bundles);
    }
    /**
     * @param {string} label
     */
    insertDebugMarker(label) {
        const ptr0 = passStringToWasm0(label, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        wasm.gpurenderpassencoder_insertDebugMarker(this.__wbg_ptr, ptr0, len0);
    }
    popDebugGroup() {
        wasm.gpurenderpassencoder_popDebugGroup(this.__wbg_ptr);
    }
    /**
     * @param {string} label
     */
    pushDebugGroup(label) {
        const ptr0 = passStringToWasm0(label, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        wasm.gpurenderpassencoder_pushDebugGroup(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * @param {number} index
     * @param {any} bind_group
     * @param {any} dynamic_offsets
     */
    setBindGroup(index, bind_group, dynamic_offsets) {
        wasm.gpurenderpassencoder_setBindGroup(this.__wbg_ptr, index, bind_group, dynamic_offsets);
    }
    /**
     * @param {any} color
     */
    setBlendConstant(color) {
        wasm.gpurenderpassencoder_setBlendConstant(this.__wbg_ptr, color);
    }
    /**
     * @param {GpuBuffer} buffer
     * @param {any} index_format
     * @param {any} offset
     * @param {any} size
     */
    setIndexBuffer(buffer, index_format, offset, size) {
        _assertClass(buffer, GpuBuffer);
        wasm.gpurenderpassencoder_setIndexBuffer(this.__wbg_ptr, buffer.__wbg_ptr, index_format, offset, size);
    }
    /**
     * @param {GpuRenderPipeline} pipeline
     */
    setPipeline(pipeline) {
        _assertClass(pipeline, GpuRenderPipeline);
        wasm.gpurenderpassencoder_setPipeline(this.__wbg_ptr, pipeline.__wbg_ptr);
    }
    /**
     * @param {number} x
     * @param {number} y
     * @param {number} width
     * @param {number} height
     */
    setScissorRect(x, y, width, height) {
        wasm.gpurenderpassencoder_setScissorRect(this.__wbg_ptr, x, y, width, height);
    }
    /**
     * @param {number} reference
     */
    setStencilReference(reference) {
        wasm.gpurenderpassencoder_setStencilReference(this.__wbg_ptr, reference);
    }
    /**
     * @param {number} slot
     * @param {GpuBuffer} buffer
     * @param {any} offset
     * @param {any} size
     */
    setVertexBuffer(slot, buffer, offset, size) {
        _assertClass(buffer, GpuBuffer);
        wasm.gpurenderpassencoder_setVertexBuffer(this.__wbg_ptr, slot, buffer.__wbg_ptr, offset, size);
    }
    /**
     * @param {number} x
     * @param {number} y
     * @param {number} width
     * @param {number} height
     * @param {number} min_depth
     * @param {number} max_depth
     */
    setViewport(x, y, width, height, min_depth, max_depth) {
        wasm.gpurenderpassencoder_setViewport(this.__wbg_ptr, x, y, width, height, min_depth, max_depth);
    }
}
if (Symbol.dispose) GpuRenderPassEncoder.prototype[Symbol.dispose] = GpuRenderPassEncoder.prototype.free;

export class GpuRenderPipeline {
    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(GpuRenderPipeline.prototype);
        obj.__wbg_ptr = ptr;
        GpuRenderPipelineFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        GpuRenderPipelineFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_gpurenderpipeline_free(ptr, 0);
    }
    /**
     * @returns {any}
     */
    __h() {
        const ret = wasm.gpurenderpipeline___h(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {number} index
     * @returns {GpuBindGroupLayout}
     */
    getBindGroupLayout(index) {
        const ret = wasm.gpurenderpipeline_getBindGroupLayout(this.__wbg_ptr, index);
        return GpuBindGroupLayout.__wrap(ret);
    }
}
if (Symbol.dispose) GpuRenderPipeline.prototype[Symbol.dispose] = GpuRenderPipeline.prototype.free;

export class GpuSampler {
    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(GpuSampler.prototype);
        obj.__wbg_ptr = ptr;
        GpuSamplerFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        GpuSamplerFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_gpusampler_free(ptr, 0);
    }
    /**
     * @returns {any}
     */
    __h() {
        const ret = wasm.gpusampler___h(this.__wbg_ptr);
        return ret;
    }
}
if (Symbol.dispose) GpuSampler.prototype[Symbol.dispose] = GpuSampler.prototype.free;

export class GpuShaderModule {
    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(GpuShaderModule.prototype);
        obj.__wbg_ptr = ptr;
        GpuShaderModuleFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        GpuShaderModuleFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_gpushadermodule_free(ptr, 0);
    }
    /**
     * @returns {any}
     */
    __h() {
        const ret = wasm.gpushadermodule___h(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {any}
     */
    getCompilationInfo() {
        const ret = wasm.gpushadermodule_getCompilationInfo(this.__wbg_ptr);
        return ret;
    }
}
if (Symbol.dispose) GpuShaderModule.prototype[Symbol.dispose] = GpuShaderModule.prototype.free;

export class GpuSurface {
    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(GpuSurface.prototype);
        obj.__wbg_ptr = ptr;
        GpuSurfaceFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        GpuSurfaceFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_gpusurface_free(ptr, 0);
    }
    /**
     * @returns {GpuTexture}
     */
    currentTexture() {
        const ret = wasm.gpusurface_currentTexture(this.__wbg_ptr);
        return GpuTexture.__wrap(ret);
    }
    /**
     * @returns {string}
     */
    getTextureFormat() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.gpusurface_getTextureFormat(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
}
if (Symbol.dispose) GpuSurface.prototype[Symbol.dispose] = GpuSurface.prototype.free;

export class GpuTexture {
    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(GpuTexture.prototype);
        obj.__wbg_ptr = ptr;
        GpuTextureFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        GpuTextureFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_gputexture_free(ptr, 0);
    }
    /**
     * @returns {GpuTextureView}
     */
    createView() {
        const ret = wasm.gputexture_createView(this.__wbg_ptr);
        return GpuTextureView.__wrap(ret);
    }
    /**
     * @returns {number}
     */
    depthOrArrayLayers() {
        const ret = wasm.gputexture_depthOrArrayLayers(this.__wbg_ptr);
        return ret >>> 0;
    }
    destroy() {
        wasm.gputexture_destroy(this.__wbg_ptr);
    }
    /**
     * @returns {string}
     */
    dimension() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.gputexture_dimension(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @returns {string}
     */
    format() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.gputexture_format(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @returns {number}
     */
    height() {
        const ret = wasm.gputexture_height(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    mipLevelCount() {
        const ret = wasm.gputexture_mipLevelCount(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    sampleCount() {
        const ret = wasm.gputexture_sampleCount(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {any}
     */
    usage() {
        const ret = wasm.gputexture_usage(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    width() {
        const ret = wasm.gputexture_width(this.__wbg_ptr);
        return ret >>> 0;
    }
}
if (Symbol.dispose) GpuTexture.prototype[Symbol.dispose] = GpuTexture.prototype.free;

export class GpuTextureView {
    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(GpuTextureView.prototype);
        obj.__wbg_ptr = ptr;
        GpuTextureViewFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        GpuTextureViewFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_gputextureview_free(ptr, 0);
    }
    /**
     * @returns {any}
     */
    __h() {
        const ret = wasm.gputextureview___h(this.__wbg_ptr);
        return ret;
    }
}
if (Symbol.dispose) GpuTextureView.prototype[Symbol.dispose] = GpuTextureView.prototype.free;

export class IirFilterNode {
    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(IirFilterNode.prototype);
        obj.__wbg_ptr = ptr;
        IirFilterNodeFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        IirFilterNodeFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_iirfilternode_free(ptr, 0);
    }
    /**
     * @returns {any}
     */
    __h() {
        const ret = wasm.iirfilternode___h(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {any} destination
     */
    connect(destination) {
        wasm.iirfilternode_connect(this.__wbg_ptr, destination);
    }
    /**
     * @param {AudioContext} context
     * @param {any} feedforward
     * @param {any} feedback
     */
    constructor(context, feedforward, feedback) {
        _assertClass(context, AudioContext);
        const ret = wasm.iirfilternode_new(context.__wbg_ptr, feedforward, feedback);
        this.__wbg_ptr = ret >>> 0;
        IirFilterNodeFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
}
if (Symbol.dispose) IirFilterNode.prototype[Symbol.dispose] = IirFilterNode.prototype.free;

export class KeyboardDevice {
    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(KeyboardDevice.prototype);
        obj.__wbg_ptr = ptr;
        KeyboardDeviceFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        KeyboardDeviceFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_keyboarddevice_free(ptr, 0);
    }
    /**
     * @returns {any}
     */
    activeKeys() {
        const ret = wasm.keyboarddevice_activeKeys(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {any} key
     * @returns {boolean}
     */
    isPressed(key) {
        const ret = wasm.keyboarddevice_isPressed(this.__wbg_ptr, key);
        return ret !== 0;
    }
    /**
     * @param {any} key
     * @returns {boolean}
     */
    justPressed(key) {
        const ret = wasm.keyboarddevice_justPressed(this.__wbg_ptr, key);
        return ret !== 0;
    }
}
if (Symbol.dispose) KeyboardDevice.prototype[Symbol.dispose] = KeyboardDevice.prototype.free;

export class MouseDevice {
    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(MouseDevice.prototype);
        obj.__wbg_ptr = ptr;
        MouseDeviceFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        MouseDeviceFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_mousedevice_free(ptr, 0);
    }
    /**
     * @param {any} _btn
     * @returns {boolean}
     */
    isPressed(_btn) {
        const ret = wasm.mousedevice_isPressed(this.__wbg_ptr, _btn);
        return ret !== 0;
    }
}
if (Symbol.dispose) MouseDevice.prototype[Symbol.dispose] = MouseDevice.prototype.free;

export class OscillatorNode {
    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(OscillatorNode.prototype);
        obj.__wbg_ptr = ptr;
        OscillatorNodeFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        OscillatorNodeFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_oscillatornode_free(ptr, 0);
    }
    /**
     * @returns {any}
     */
    __h() {
        const ret = wasm.oscillatornode___h(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {any} destination
     */
    connect(destination) {
        wasm.oscillatornode_connect(this.__wbg_ptr, destination);
    }
    /**
     * @returns {AudioParam}
     */
    detune() {
        const ret = wasm.oscillatornode_detune(this.__wbg_ptr);
        return AudioParam.__wrap(ret);
    }
    /**
     * @returns {AudioParam}
     */
    frequency() {
        const ret = wasm.oscillatornode_frequency(this.__wbg_ptr);
        return AudioParam.__wrap(ret);
    }
    /**
     * @param {AudioContext} context
     */
    constructor(context) {
        _assertClass(context, AudioContext);
        const ret = wasm.audiocontext_createOscillator(context.__wbg_ptr);
        this.__wbg_ptr = ret >>> 0;
        OscillatorNodeFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * @param {PeriodicWave} wave
     */
    setPeriodicWave(wave) {
        _assertClass(wave, PeriodicWave);
        wasm.oscillatornode_setPeriodicWave(this.__wbg_ptr, wave.__wbg_ptr);
    }
    /**
     * @param {any} t
     */
    setType(t) {
        wasm.oscillatornode_setType(this.__wbg_ptr, t);
    }
    /**
     * @returns {any}
     */
    type() {
        const ret = wasm.oscillatornode_type(this.__wbg_ptr);
        return ret;
    }
}
if (Symbol.dispose) OscillatorNode.prototype[Symbol.dispose] = OscillatorNode.prototype.free;

export class PannerNode {
    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(PannerNode.prototype);
        obj.__wbg_ptr = ptr;
        PannerNodeFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        PannerNodeFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_pannernode_free(ptr, 0);
    }
    /**
     * @returns {any}
     */
    __h() {
        const ret = wasm.pannernode___h(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    coneInnerAngle() {
        const ret = wasm.pannernode_coneInnerAngle(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    coneOuterAngle() {
        const ret = wasm.pannernode_coneOuterAngle(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    coneOuterGain() {
        const ret = wasm.pannernode_coneOuterGain(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {any} destination
     */
    connect(destination) {
        wasm.pannernode_connect(this.__wbg_ptr, destination);
    }
    /**
     * @returns {string}
     */
    distanceModel() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.pannernode_distanceModel(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @returns {number}
     */
    maxDistance() {
        const ret = wasm.pannernode_maxDistance(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {AudioContext} context
     */
    constructor(context) {
        _assertClass(context, AudioContext);
        const ret = wasm.audiocontext_createPanner(context.__wbg_ptr);
        this.__wbg_ptr = ret >>> 0;
        PannerNodeFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * @returns {AudioParam}
     */
    orientationX() {
        const ret = wasm.pannernode_orientationX(this.__wbg_ptr);
        return AudioParam.__wrap(ret);
    }
    /**
     * @returns {AudioParam}
     */
    orientationY() {
        const ret = wasm.pannernode_orientationY(this.__wbg_ptr);
        return AudioParam.__wrap(ret);
    }
    /**
     * @returns {AudioParam}
     */
    orientationZ() {
        const ret = wasm.pannernode_orientationZ(this.__wbg_ptr);
        return AudioParam.__wrap(ret);
    }
    /**
     * @returns {string}
     */
    panningModel() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.pannernode_panningModel(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @returns {AudioParam}
     */
    positionX() {
        const ret = wasm.pannernode_positionX(this.__wbg_ptr);
        return AudioParam.__wrap(ret);
    }
    /**
     * @returns {AudioParam}
     */
    positionY() {
        const ret = wasm.pannernode_positionY(this.__wbg_ptr);
        return AudioParam.__wrap(ret);
    }
    /**
     * @returns {AudioParam}
     */
    positionZ() {
        const ret = wasm.pannernode_positionZ(this.__wbg_ptr);
        return AudioParam.__wrap(ret);
    }
    /**
     * @returns {number}
     */
    refDistance() {
        const ret = wasm.pannernode_refDistance(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    rolloffFactor() {
        const ret = wasm.pannernode_rolloffFactor(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {number} value
     */
    setConeInnerAngle(value) {
        wasm.pannernode_setConeInnerAngle(this.__wbg_ptr, value);
    }
    /**
     * @param {number} value
     */
    setConeOuterAngle(value) {
        wasm.pannernode_setConeOuterAngle(this.__wbg_ptr, value);
    }
    /**
     * @param {number} value
     */
    setConeOuterGain(value) {
        wasm.pannernode_setConeOuterGain(this.__wbg_ptr, value);
    }
    /**
     * @param {any} value
     */
    setDistanceModel(value) {
        wasm.pannernode_setDistanceModel(this.__wbg_ptr, value);
    }
    /**
     * @param {number} value
     */
    setMaxDistance(value) {
        wasm.pannernode_setMaxDistance(this.__wbg_ptr, value);
    }
    /**
     * @param {number} x
     * @param {number} y
     * @param {number} z
     */
    setOrientation(x, y, z) {
        wasm.pannernode_setOrientation(this.__wbg_ptr, x, y, z);
    }
    /**
     * @param {any} value
     */
    setPanningModel(value) {
        wasm.pannernode_setPanningModel(this.__wbg_ptr, value);
    }
    /**
     * @param {number} x
     * @param {number} y
     * @param {number} z
     */
    setPosition(x, y, z) {
        wasm.pannernode_setPosition(this.__wbg_ptr, x, y, z);
    }
    /**
     * @param {number} value
     */
    setRefDistance(value) {
        wasm.pannernode_setRefDistance(this.__wbg_ptr, value);
    }
    /**
     * @param {number} value
     */
    setRolloffFactor(value) {
        wasm.pannernode_setRolloffFactor(this.__wbg_ptr, value);
    }
}
if (Symbol.dispose) PannerNode.prototype[Symbol.dispose] = PannerNode.prototype.free;

export class Path {
    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(Path.prototype);
        obj.__wbg_ptr = ptr;
        PathFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        PathFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_path_free(ptr, 0);
    }
    /**
     * @returns {string | undefined}
     */
    extension() {
        const ret = wasm.path_extension(this.__wbg_ptr);
        let v1;
        if (ret[0] !== 0) {
            v1 = getStringFromWasm0(ret[0], ret[1]).slice();
            wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        }
        return v1;
    }
    /**
     * @returns {string | undefined}
     */
    filename() {
        const ret = wasm.path_filename(this.__wbg_ptr);
        let v1;
        if (ret[0] !== 0) {
            v1 = getStringFromWasm0(ret[0], ret[1]).slice();
            wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        }
        return v1;
    }
    /**
     * @returns {boolean}
     */
    isDir() {
        const ret = wasm.path_isDir(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @returns {boolean}
     */
    isFile() {
        const ret = wasm.path_isFile(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @returns {boolean}
     */
    isRoot() {
        const ret = wasm.path_isRoot(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @param {string} path
     * @returns {Path}
     */
    join(path) {
        const ptr0 = passStringToWasm0(path, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.path_join(this.__wbg_ptr, ptr0, len0);
        return Path.__wrap(ret);
    }
    /**
     * @param {StorageDevice} storage
     * @param {string} path
     */
    constructor(storage, path) {
        _assertClass(storage, StorageDevice);
        const ptr0 = passStringToWasm0(path, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.path_new(storage.__wbg_ptr, ptr0, len0);
        this.__wbg_ptr = ret >>> 0;
        PathFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * @returns {Path}
     */
    parent() {
        const ret = wasm.path_parent(this.__wbg_ptr);
        return Path.__wrap(ret);
    }
    /**
     * @returns {string}
     */
    toString() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.path_toString(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
}
if (Symbol.dispose) Path.prototype[Symbol.dispose] = Path.prototype.free;

export class PeriodicWave {
    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(PeriodicWave.prototype);
        obj.__wbg_ptr = ptr;
        PeriodicWaveFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        PeriodicWaveFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_periodicwave_free(ptr, 0);
    }
    /**
     * @returns {any}
     */
    __h() {
        const ret = wasm.periodicwave___h(this.__wbg_ptr);
        return ret;
    }
}
if (Symbol.dispose) PeriodicWave.prototype[Symbol.dispose] = PeriodicWave.prototype.free;

export class StereoPannerNode {
    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(StereoPannerNode.prototype);
        obj.__wbg_ptr = ptr;
        StereoPannerNodeFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        StereoPannerNodeFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_stereopannernode_free(ptr, 0);
    }
    /**
     * @returns {any}
     */
    __h() {
        const ret = wasm.stereopannernode___h(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {any} destination
     */
    connect(destination) {
        wasm.stereopannernode_connect(this.__wbg_ptr, destination);
    }
    /**
     * @returns {AudioParam}
     */
    pan() {
        const ret = wasm.stereopannernode_pan(this.__wbg_ptr);
        return AudioParam.__wrap(ret);
    }
}
if (Symbol.dispose) StereoPannerNode.prototype[Symbol.dispose] = StereoPannerNode.prototype.free;

export class StorageDevice {
    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(StorageDevice.prototype);
        obj.__wbg_ptr = ptr;
        StorageDeviceFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        StorageDeviceFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_storagedevice_free(ptr, 0);
    }
    /**
     * @param {any} path
     */
    createDir(path) {
        wasm.storagedevice_createDir(this.__wbg_ptr, path);
    }
    /**
     * @param {any} path
     * @returns {boolean}
     */
    exists(path) {
        const ret = wasm.storagedevice_exists(this.__wbg_ptr, path);
        return ret !== 0;
    }
    /**
     * @param {any} path
     * @returns {any}
     */
    listDir(path) {
        const ret = wasm.storagedevice_listDir(this.__wbg_ptr, path);
        return ret;
    }
    /**
     * @param {any} path
     * @returns {Uint8Array | undefined}
     */
    read(path) {
        const ret = wasm.storagedevice_read(this.__wbg_ptr, path);
        let v1;
        if (ret[0] !== 0) {
            v1 = getArrayU8FromWasm0(ret[0], ret[1]).slice();
            wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        }
        return v1;
    }
    /**
     * @param {any} path
     * @returns {string | undefined}
     */
    readString(path) {
        const ret = wasm.storagedevice_readString(this.__wbg_ptr, path);
        let v1;
        if (ret[0] !== 0) {
            v1 = getStringFromWasm0(ret[0], ret[1]).slice();
            wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        }
        return v1;
    }
    /**
     * @param {any} path
     * @returns {boolean | undefined}
     */
    remove(path) {
        const ret = wasm.storagedevice_remove(this.__wbg_ptr, path);
        return ret === 0xFFFFFF ? undefined : ret !== 0;
    }
    /**
     * @param {any} path
     * @param {any} content
     */
    write(path, content) {
        wasm.storagedevice_write(this.__wbg_ptr, path, content);
    }
}
if (Symbol.dispose) StorageDevice.prototype[Symbol.dispose] = StorageDevice.prototype.free;

export class TouchDevice {
    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(TouchDevice.prototype);
        obj.__wbg_ptr = ptr;
        TouchDeviceFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        TouchDeviceFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_touchdevice_free(ptr, 0);
    }
}
if (Symbol.dispose) TouchDevice.prototype[Symbol.dispose] = TouchDevice.prototype.free;

export class WaveShaperNode {
    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(WaveShaperNode.prototype);
        obj.__wbg_ptr = ptr;
        WaveShaperNodeFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        WaveShaperNodeFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_waveshapernode_free(ptr, 0);
    }
    /**
     * @returns {any}
     */
    __h() {
        const ret = wasm.waveshapernode___h(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {any} destination
     */
    connect(destination) {
        wasm.waveshapernode_connect(this.__wbg_ptr, destination);
    }
    /**
     * @returns {Float32Array | undefined}
     */
    curve() {
        const ret = wasm.waveshapernode_curve(this.__wbg_ptr);
        let v1;
        if (ret[0] !== 0) {
            v1 = getArrayF32FromWasm0(ret[0], ret[1]).slice();
            wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        }
        return v1;
    }
    /**
     * @returns {string}
     */
    oversample() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.waveshapernode_oversample(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @param {any} curve
     */
    setCurve(curve) {
        wasm.waveshapernode_setCurve(this.__wbg_ptr, curve);
    }
    /**
     * @param {any} oversample
     */
    setOversample(oversample) {
        wasm.waveshapernode_setOversample(this.__wbg_ptr, oversample);
    }
}
if (Symbol.dispose) WaveShaperNode.prototype[Symbol.dispose] = WaveShaperNode.prototype.free;

/**
 * @returns {AudioDevice | undefined}
 */
export function audioOutput() {
    const ret = wasm.audioOutput();
    return ret === 0 ? undefined : AudioDevice.__wrap(ret);
}

/**
 * @returns {GpuAdapter}
 */
export function gpuRequestAdapter() {
    const ret = wasm.gpuRequestAdapter();
    return GpuAdapter.__wrap(ret);
}

/**
 * Called by the harness after `await`ing the adapter/device and configuring the
 * canvas context, before instantiating the guest.
 * @param {any} device
 * @param {any} context
 * @param {string} surface_view_format
 */
export function gpuSetContext(device, context, surface_view_format) {
    const ptr0 = passStringToWasm0(surface_view_format, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    wasm.gpuSetContext(device, context, ptr0, len0);
}

/**
 * @returns {GpuSurface}
 */
export function gpuSurface() {
    const ret = wasm.gpuRequestAdapter();
    return GpuSurface.__wrap(ret);
}

/**
 * Clears the per-frame "just pressed" set. Called by the run loop each frame.
 */
export function inputEndFrame() {
    wasm.inputEndFrame();
}

/**
 * @param {number} id
 * @returns {GamepadDevice | undefined}
 */
export function inputGamepad(id) {
    const ret = wasm.inputGamepad(id);
    return ret === 0 ? undefined : GamepadDevice.__wrap(ret);
}

/**
 * Installs the DOM event listeners that track keyboard/mouse state. Idempotent.
 */
export function inputInstall() {
    wasm.inputInstall();
}

/**
 * @returns {KeyboardDevice | undefined}
 */
export function inputKeyboard() {
    const ret = wasm.inputKeyboard();
    return ret === 0 ? undefined : KeyboardDevice.__wrap(ret);
}

/**
 * @returns {MouseDevice | undefined}
 */
export function inputMouse() {
    const ret = wasm.inputKeyboard();
    return ret === 0 ? undefined : MouseDevice.__wrap(ret);
}

/**
 * Drains pending gilrs events to refresh gamepad state. Called once per frame.
 */
export function inputPoll() {
    wasm.inputPoll();
}

/**
 * @returns {TouchDevice | undefined}
 */
export function inputTouch() {
    const ret = wasm.inputTouch();
    return ret === 0 ? undefined : TouchDevice.__wrap(ret);
}

/**
 * Web entry point, called by the HTML harness after it has installed
 * `window.jco`. The guest is transpiled at build time (it is loaded by the
 * harness, not fetched here), so this just starts the web runtime, which
 * instantiates the guest via jco and drives the frame loop.
 */
export function run() {
    wasm.run();
}

/**
 * @returns {StorageDevice | undefined}
 */
export function storageCloud() {
    const ret = wasm.storageCloud();
    return ret === 0 ? undefined : StorageDevice.__wrap(ret);
}

/**
 * @returns {StorageDevice}
 */
export function storageLocal() {
    const ret = wasm.storageLocal();
    return StorageDevice.__wrap(ret);
}

function __wbg_get_imports() {
    const import0 = {
        __proto__: null,
        __wbg___wbindgen_boolean_get_bbbb1c18aa2f5e25: function(arg0) {
            const v = arg0;
            const ret = typeof(v) === 'boolean' ? v : undefined;
            return isLikeNone(ret) ? 0xFFFFFF : ret ? 1 : 0;
        },
        __wbg___wbindgen_debug_string_0bc8482c6e3508ae: function(arg0, arg1) {
            const ret = debugString(arg1);
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg___wbindgen_is_function_0095a73b8b156f76: function(arg0) {
            const ret = typeof(arg0) === 'function';
            return ret;
        },
        __wbg___wbindgen_is_null_ac34f5003991759a: function(arg0) {
            const ret = arg0 === null;
            return ret;
        },
        __wbg___wbindgen_is_undefined_9e4d92534c42d778: function(arg0) {
            const ret = arg0 === undefined;
            return ret;
        },
        __wbg___wbindgen_number_get_8ff4255516ccad3e: function(arg0, arg1) {
            const obj = arg1;
            const ret = typeof(obj) === 'number' ? obj : undefined;
            getDataViewMemory0().setFloat64(arg0 + 8 * 1, isLikeNone(ret) ? 0 : ret, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, !isLikeNone(ret), true);
        },
        __wbg___wbindgen_string_get_72fb696202c56729: function(arg0, arg1) {
            const obj = arg1;
            const ret = typeof(obj) === 'string' ? obj : undefined;
            var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            var len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg___wbindgen_throw_be289d5034ed271b: function(arg0, arg1) {
            throw new Error(getStringFromWasm0(arg0, arg1));
        },
        __wbg___wbindgen_typeof_8dbc59353f59e72a: function(arg0) {
            const ret = typeof arg0;
            return ret;
        },
        __wbg__wbg_cb_unref_d9b87ff7982e3b21: function(arg0) {
            arg0._wbg_cb_unref();
        },
        __wbg_addEventListener_3acb0aad4483804c: function() { return handleError(function (arg0, arg1, arg2, arg3) {
            arg0.addEventListener(getStringFromWasm0(arg1, arg2), arg3);
        }, arguments); },
        __wbg_appendChild_dea38765a26d346d: function() { return handleError(function (arg0, arg1) {
            const ret = arg0.appendChild(arg1);
            return ret;
        }, arguments); },
        __wbg_apply_2e22c45cb4f12415: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = Reflect.apply(arg0, arg1, arg2);
            return ret;
        }, arguments); },
        __wbg_axes_6c53f544c314fe19: function(arg0) {
            const ret = arg0.axes;
            return ret;
        },
        __wbg_body_f67922363a220026: function(arg0) {
            const ret = arg0.body;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_buttons_172c77c2ce62b90c: function(arg0) {
            const ret = arg0.buttons;
            return ret;
        },
        __wbg_call_389efe28435a9388: function() { return handleError(function (arg0, arg1) {
            const ret = arg0.call(arg1);
            return ret;
        }, arguments); },
        __wbg_call_4708e0c13bdc8e95: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = arg0.call(arg1, arg2);
            return ret;
        }, arguments); },
        __wbg_call_812d25f1510c13c8: function() { return handleError(function (arg0, arg1, arg2, arg3) {
            const ret = arg0.call(arg1, arg2, arg3);
            return ret;
        }, arguments); },
        __wbg_connected_6d06e0c95c0d644a: function(arg0) {
            const ret = arg0.connected;
            return ret;
        },
        __wbg_construct_86626e847de3b629: function() { return handleError(function (arg0, arg1) {
            const ret = Reflect.construct(arg0, arg1);
            return ret;
        }, arguments); },
        __wbg_createElement_49f60fdcaae809c8: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = arg0.createElement(getStringFromWasm0(arg1, arg2));
            return ret;
        }, arguments); },
        __wbg_debug_a4099fa12db6cd61: function(arg0) {
            console.debug(arg0);
        },
        __wbg_devicePixelRatio_5c458affc89fc209: function(arg0) {
            const ret = arg0.devicePixelRatio;
            return ret;
        },
        __wbg_document_ee35a3d3ae34ef6c: function(arg0) {
            const ret = arg0.document;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_error_7534b8e9a36f1ab4: function(arg0, arg1) {
            let deferred0_0;
            let deferred0_1;
            try {
                deferred0_0 = arg0;
                deferred0_1 = arg1;
                console.error(getStringFromWasm0(arg0, arg1));
            } finally {
                wasm.__wbindgen_free(deferred0_0, deferred0_1, 1);
            }
        },
        __wbg_error_9a7fe3f932034cde: function(arg0) {
            console.error(arg0);
        },
        __wbg_exec_48e0e0ad953102ac: function(arg0, arg1, arg2) {
            const ret = arg0.exec(getStringFromWasm0(arg1, arg2));
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_forEach_a2bfcdf179e573de: function(arg0, arg1, arg2) {
            try {
                var state0 = {a: arg1, b: arg2};
                var cb0 = (arg0, arg1, arg2) => {
                    const a = state0.a;
                    state0.a = 0;
                    try {
                        return wasm_bindgen_942aa1b67ce6ba0a___convert__closures_____invoke___wasm_bindgen_942aa1b67ce6ba0a___JsValue__wasm_bindgen_942aa1b67ce6ba0a___JsValue__js_sys_a05094b4bd4c8726___Set_____(a, state0.b, arg0, arg1, arg2);
                    } finally {
                        state0.a = a;
                    }
                };
                arg0.forEach(cb0);
            } finally {
                state0.a = state0.b = 0;
            }
        },
        __wbg_from_bddd64e7d5ff6941: function(arg0) {
            const ret = Array.from(arg0);
            return ret;
        },
        __wbg_getGamepads_5bcfb8576d477d73: function() { return handleError(function (arg0) {
            const ret = arg0.getGamepads();
            return ret;
        }, arguments); },
        __wbg_getItem_0c792d344808dcf5: function() { return handleError(function (arg0, arg1, arg2, arg3) {
            const ret = arg1.getItem(getStringFromWasm0(arg2, arg3));
            var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            var len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        }, arguments); },
        __wbg_get_9b94d73e6221f75c: function(arg0, arg1) {
            const ret = arg0[arg1 >>> 0];
            return ret;
        },
        __wbg_get_b3ed3ad4be2bc8ac: function() { return handleError(function (arg0, arg1) {
            const ret = Reflect.get(arg0, arg1);
            return ret;
        }, arguments); },
        __wbg_id_e4749bded4876c88: function(arg0, arg1) {
            const ret = arg1.id;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_index_254eff7dac36f09f: function(arg0) {
            const ret = arg0.index;
            return ret;
        },
        __wbg_info_148d043840582012: function(arg0) {
            console.info(arg0);
        },
        __wbg_innerHeight_54aa104da08becd2: function() { return handleError(function (arg0) {
            const ret = arg0.innerHeight;
            return ret;
        }, arguments); },
        __wbg_innerWidth_fa95c57321f4f033: function() { return handleError(function (arg0) {
            const ret = arg0.innerWidth;
            return ret;
        }, arguments); },
        __wbg_instanceof_DomException_99c177193e554b75: function(arg0) {
            let result;
            try {
                result = arg0 instanceof DOMException;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_Float32Array_c882a172bf41d92a: function(arg0) {
            let result;
            try {
                result = arg0 instanceof Float32Array;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_HtmlCanvasElement_3f2f6e1edb1c9792: function(arg0) {
            let result;
            try {
                result = arg0 instanceof HTMLCanvasElement;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_Object_1c6af87502b733ed: function(arg0) {
            let result;
            try {
                result = arg0 instanceof Object;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_Promise_0094681e3519d6ec: function(arg0) {
            let result;
            try {
                result = arg0 instanceof Promise;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_Window_ed49b2db8df90359: function(arg0) {
            let result;
            try {
                result = arg0 instanceof Window;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_isArray_d314bb98fcf08331: function(arg0) {
            const ret = Array.isArray(arg0);
            return ret;
        },
        __wbg_isSecureContext_1e186b850f07cfb3: function(arg0) {
            const ret = arg0.isSecureContext;
            return ret;
        },
        __wbg_key_0167bc764945979a: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = arg1.key(arg2 >>> 0);
            var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            var len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        }, arguments); },
        __wbg_keys_b50a709a76add04e: function(arg0) {
            const ret = Object.keys(arg0);
            return ret;
        },
        __wbg_length_32ed9a279acd054c: function(arg0) {
            const ret = arg0.length;
            return ret;
        },
        __wbg_length_35a7bace40f36eac: function(arg0) {
            const ret = arg0.length;
            return ret;
        },
        __wbg_length_7724867d8e59c610: function() { return handleError(function (arg0) {
            const ret = arg0.length;
            return ret;
        }, arguments); },
        __wbg_length_9a7876c9728a0979: function(arg0) {
            const ret = arg0.length;
            return ret;
        },
        __wbg_localStorage_a22d31b9eacc4594: function() { return handleError(function (arg0) {
            const ret = arg0.localStorage;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        }, arguments); },
        __wbg_log_6b5ca2e6124b2808: function(arg0) {
            console.log(arg0);
        },
        __wbg_mapping_1b290f968173c826: function(arg0) {
            const ret = arg0.mapping;
            return (__wbindgen_enum_GamepadMappingType.indexOf(ret) + 1 || 3) - 1;
        },
        __wbg_message_0b2b0298a231b0d4: function(arg0, arg1) {
            const ret = arg1.message;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_navigator_43be698ba96fc088: function(arg0) {
            const ret = arg0.navigator;
            return ret;
        },
        __wbg_new_361308b2356cecd0: function() {
            const ret = new Object();
            return ret;
        },
        __wbg_new_3eb36ae241fe6f44: function() {
            const ret = new Array();
            return ret;
        },
        __wbg_new_5dc3c2adaffe3280: function(arg0) {
            const ret = new Number(arg0);
            return ret;
        },
        __wbg_new_8a6f238a6ece86ea: function() {
            const ret = new Error();
            return ret;
        },
        __wbg_new_9ccae8c3d6099588: function(arg0) {
            const ret = new Float32Array(arg0);
            return ret;
        },
        __wbg_new_dd2b680c8bf6ae29: function(arg0) {
            const ret = new Uint8Array(arg0);
            return ret;
        },
        __wbg_new_de07934a2f24c2ec: function(arg0, arg1, arg2, arg3) {
            const ret = new RegExp(getStringFromWasm0(arg0, arg1), getStringFromWasm0(arg2, arg3));
            return ret;
        },
        __wbg_new_no_args_1c7c842f08d00ebb: function(arg0, arg1) {
            const ret = new Function(getStringFromWasm0(arg0, arg1));
            return ret;
        },
        __wbg_new_with_length_63f2683cc2521026: function(arg0) {
            const ret = new Float32Array(arg0 >>> 0);
            return ret;
        },
        __wbg_new_with_length_a2c39cbe88fd8ff1: function(arg0) {
            const ret = new Uint8Array(arg0 >>> 0);
            return ret;
        },
        __wbg_now_2c95c9de01293173: function(arg0) {
            const ret = arg0.now();
            return ret;
        },
        __wbg_now_a3af9a2f4bbaa4d1: function() {
            const ret = Date.now();
            return ret;
        },
        __wbg_path_new: function(arg0) {
            const ret = Path.__wrap(arg0);
            return ret;
        },
        __wbg_performance_7a3ffd0b17f663ad: function(arg0) {
            const ret = arg0.performance;
            return ret;
        },
        __wbg_pressed_95e62758ea924dc3: function(arg0) {
            const ret = arg0.pressed;
            return ret;
        },
        __wbg_prototypesetcall_bdcdcc5842e4d77d: function(arg0, arg1, arg2) {
            Uint8Array.prototype.set.call(getArrayU8FromWasm0(arg0, arg1), arg2);
        },
        __wbg_prototypesetcall_c7e6a26aeade796d: function(arg0, arg1, arg2) {
            Float32Array.prototype.set.call(getArrayF32FromWasm0(arg0, arg1), arg2);
        },
        __wbg_push_8ffdcb2063340ba5: function(arg0, arg1) {
            const ret = arg0.push(arg1);
            return ret;
        },
        __wbg_queueMicrotask_0aa0a927f78f5d98: function(arg0) {
            const ret = arg0.queueMicrotask;
            return ret;
        },
        __wbg_queueMicrotask_5bb536982f78a56f: function(arg0) {
            queueMicrotask(arg0);
        },
        __wbg_removeItem_f6369b1a6fa39850: function() { return handleError(function (arg0, arg1, arg2) {
            arg0.removeItem(getStringFromWasm0(arg1, arg2));
        }, arguments); },
        __wbg_requestAnimationFrame_43682f8e1c5e5348: function() { return handleError(function (arg0, arg1) {
            const ret = arg0.requestAnimationFrame(arg1);
            return ret;
        }, arguments); },
        __wbg_resolve_002c4b7d9d8f6b64: function(arg0) {
            const ret = Promise.resolve(arg0);
            return ret;
        },
        __wbg_setItem_cf340bb2edbd3089: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
            arg0.setItem(getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
        }, arguments); },
        __wbg_set_25cf9deff6bf0ea8: function(arg0, arg1, arg2) {
            arg0.set(arg1, arg2 >>> 0);
        },
        __wbg_set_6cb8631f80447a67: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = Reflect.set(arg0, arg1, arg2);
            return ret;
        }, arguments); },
        __wbg_set_height_f21f985387070100: function(arg0, arg1) {
            arg0.height = arg1 >>> 0;
        },
        __wbg_set_id_9b8330f661385753: function(arg0, arg1, arg2) {
            arg0.id = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_width_d60bc4f2f20c56a4: function(arg0, arg1) {
            arg0.width = arg1 >>> 0;
        },
        __wbg_stack_0ed75d68575b0f3c: function(arg0, arg1) {
            const ret = arg1.stack;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_static_accessor_GLOBAL_12837167ad935116: function() {
            const ret = typeof global === 'undefined' ? null : global;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_static_accessor_GLOBAL_THIS_e628e89ab3b1c95f: function() {
            const ret = typeof globalThis === 'undefined' ? null : globalThis;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_static_accessor_SELF_a621d3dfbb60d0ce: function() {
            const ret = typeof self === 'undefined' ? null : self;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_static_accessor_WINDOW_f8727f0cf888e0bd: function() {
            const ret = typeof window === 'undefined' ? null : window;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_then_0d9fe2c7b1857d32: function(arg0, arg1, arg2) {
            const ret = arg0.then(arg1, arg2);
            return ret;
        },
        __wbg_then_b9e7b3b5f1a9e1b5: function(arg0, arg1) {
            const ret = arg0.then(arg1);
            return ret;
        },
        __wbg_valueOf_670bef5709ccda33: function(arg0) {
            const ret = arg0.valueOf();
            return ret;
        },
        __wbg_value_6cd76ff54b88ed85: function(arg0) {
            const ret = arg0.value;
            return ret;
        },
        __wbg_warn_f7ae1b2e66ccb930: function(arg0) {
            console.warn(arg0);
        },
        __wbindgen_cast_0000000000000001: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { dtor_idx: 1, function: Function { arguments: [String], shim_idx: 2, ret: Unit, inner_ret: Some(Unit) }, mutable: false }) -> Externref`.
            const ret = makeClosure(arg0, arg1, wasm.wasm_bindgen_942aa1b67ce6ba0a___closure__destroy___dyn_core_4777b9c83c0d5d57___ops__function__Fn__alloc_ba1363f49e95c7f2___string__String____Output_______, wasm_bindgen_942aa1b67ce6ba0a___convert__closures_____invoke___alloc_ba1363f49e95c7f2___string__String_____);
            return ret;
        },
        __wbindgen_cast_0000000000000002: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { dtor_idx: 1, function: Function { arguments: [], shim_idx: 4, ret: NamedExternref("Array<any>"), inner_ret: Some(NamedExternref("Array<any>")) }, mutable: false }) -> Externref`.
            const ret = makeClosure(arg0, arg1, wasm.wasm_bindgen_942aa1b67ce6ba0a___closure__destroy___dyn_core_4777b9c83c0d5d57___ops__function__Fn__alloc_ba1363f49e95c7f2___string__String____Output_______, wasm_bindgen_942aa1b67ce6ba0a___convert__closures_____invoke___js_sys_a05094b4bd4c8726___Array_);
            return ret;
        },
        __wbindgen_cast_0000000000000003: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { dtor_idx: 103, function: Function { arguments: [Externref], shim_idx: 104, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, wasm.wasm_bindgen_942aa1b67ce6ba0a___closure__destroy___dyn_core_4777b9c83c0d5d57___ops__function__FnMut__wasm_bindgen_942aa1b67ce6ba0a___JsValue____Output_______, wasm_bindgen_942aa1b67ce6ba0a___convert__closures_____invoke___wasm_bindgen_942aa1b67ce6ba0a___JsValue_____);
            return ret;
        },
        __wbindgen_cast_0000000000000004: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { dtor_idx: 70, function: Function { arguments: [], shim_idx: 71, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, wasm.wasm_bindgen_942aa1b67ce6ba0a___closure__destroy___dyn_core_4777b9c83c0d5d57___ops__function__FnMut__core_4777b9c83c0d5d57___option__Option_web_sys_98350932e927b18f___features__gen_Blob__Blob_____Output_______, wasm_bindgen_942aa1b67ce6ba0a___convert__closures_____invoke______);
            return ret;
        },
        __wbindgen_cast_0000000000000005: function(arg0) {
            // Cast intrinsic for `F64 -> Externref`.
            const ret = arg0;
            return ret;
        },
        __wbindgen_cast_0000000000000006: function(arg0, arg1) {
            // Cast intrinsic for `Ref(String) -> Externref`.
            const ret = getStringFromWasm0(arg0, arg1);
            return ret;
        },
        __wbindgen_init_externref_table: function() {
            const table = wasm.__wbindgen_externrefs;
            const offset = table.grow(4);
            table.set(0, undefined);
            table.set(offset + 0, undefined);
            table.set(offset + 1, null);
            table.set(offset + 2, true);
            table.set(offset + 3, false);
        },
    };
    return {
        __proto__: null,
        "./web_bg.js": import0,
    };
}

function wasm_bindgen_942aa1b67ce6ba0a___convert__closures_____invoke______(arg0, arg1) {
    wasm.wasm_bindgen_942aa1b67ce6ba0a___convert__closures_____invoke______(arg0, arg1);
}

function wasm_bindgen_942aa1b67ce6ba0a___convert__closures_____invoke___js_sys_a05094b4bd4c8726___Array_(arg0, arg1) {
    const ret = wasm.wasm_bindgen_942aa1b67ce6ba0a___convert__closures_____invoke___js_sys_a05094b4bd4c8726___Array_(arg0, arg1);
    return ret;
}

function wasm_bindgen_942aa1b67ce6ba0a___convert__closures_____invoke___wasm_bindgen_942aa1b67ce6ba0a___JsValue_____(arg0, arg1, arg2) {
    wasm.wasm_bindgen_942aa1b67ce6ba0a___convert__closures_____invoke___wasm_bindgen_942aa1b67ce6ba0a___JsValue_____(arg0, arg1, arg2);
}

function wasm_bindgen_942aa1b67ce6ba0a___convert__closures_____invoke___wasm_bindgen_942aa1b67ce6ba0a___JsValue__wasm_bindgen_942aa1b67ce6ba0a___JsValue__js_sys_a05094b4bd4c8726___Set_____(arg0, arg1, arg2, arg3, arg4) {
    wasm.wasm_bindgen_942aa1b67ce6ba0a___convert__closures_____invoke___wasm_bindgen_942aa1b67ce6ba0a___JsValue__wasm_bindgen_942aa1b67ce6ba0a___JsValue__js_sys_a05094b4bd4c8726___Set_____(arg0, arg1, arg2, arg3, arg4);
}

function wasm_bindgen_942aa1b67ce6ba0a___convert__closures_____invoke___alloc_ba1363f49e95c7f2___string__String_____(arg0, arg1, arg2) {
    const ptr0 = passStringToWasm0(arg2, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    wasm.wasm_bindgen_942aa1b67ce6ba0a___convert__closures_____invoke___alloc_ba1363f49e95c7f2___string__String_____(arg0, arg1, ptr0, len0);
}


const __wbindgen_enum_GamepadMappingType = ["", "standard"];
const AnalyzerNodeFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_analyzernode_free(ptr >>> 0, 1));
const AudioBufferFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_audiobuffer_free(ptr >>> 0, 1));
const AudioBufferSourceNodeFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_audiobuffersourcenode_free(ptr >>> 0, 1));
const AudioContextFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_audiocontext_free(ptr >>> 0, 1));
const AudioDestinationNodeFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_audiodestinationnode_free(ptr >>> 0, 1));
const AudioDeviceFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_audiodevice_free(ptr >>> 0, 1));
const AudioListenerFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_audiolistener_free(ptr >>> 0, 1));
const AudioParamFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_audioparam_free(ptr >>> 0, 1));
const AudioRenderCapacityFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_audiorendercapacity_free(ptr >>> 0, 1));
const BiquadFilterNodeFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_biquadfilternode_free(ptr >>> 0, 1));
const ChannelMergerNodeFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_channelmergernode_free(ptr >>> 0, 1));
const ChannelSplitterNodeFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_channelsplitternode_free(ptr >>> 0, 1));
const ConstantSourceNodeFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_constantsourcenode_free(ptr >>> 0, 1));
const ConvolverNodeFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_convolvernode_free(ptr >>> 0, 1));
const DelayNodeFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_delaynode_free(ptr >>> 0, 1));
const DynamicsCompressorNodeFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_dynamicscompressornode_free(ptr >>> 0, 1));
const GainNodeFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_gainnode_free(ptr >>> 0, 1));
const GamepadDeviceFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_gamepaddevice_free(ptr >>> 0, 1));
const GpuAdapterFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_gpuadapter_free(ptr >>> 0, 1));
const GpuBindGroupFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_gpubindgroup_free(ptr >>> 0, 1));
const GpuBindGroupLayoutFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_gpubindgrouplayout_free(ptr >>> 0, 1));
const GpuBufferFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_gpubuffer_free(ptr >>> 0, 1));
const GpuCommandBufferFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_gpucommandbuffer_free(ptr >>> 0, 1));
const GpuCommandEncoderFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_gpucommandencoder_free(ptr >>> 0, 1));
const GpuComputePassEncoderFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_gpucomputepassencoder_free(ptr >>> 0, 1));
const GpuComputePipelineFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_gpucomputepipeline_free(ptr >>> 0, 1));
const GpuDeviceFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_gpudevice_free(ptr >>> 0, 1));
const GpuPipelineLayoutFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_gpupipelinelayout_free(ptr >>> 0, 1));
const GpuQuerySetFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_gpuqueryset_free(ptr >>> 0, 1));
const GpuQueueFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_gpuqueue_free(ptr >>> 0, 1));
const GpuRenderBundleFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_gpurenderbundle_free(ptr >>> 0, 1));
const GpuRenderBundleEncoderFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_gpurenderbundleencoder_free(ptr >>> 0, 1));
const GpuRenderPassEncoderFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_gpurenderpassencoder_free(ptr >>> 0, 1));
const GpuRenderPipelineFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_gpurenderpipeline_free(ptr >>> 0, 1));
const GpuSamplerFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_gpusampler_free(ptr >>> 0, 1));
const GpuShaderModuleFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_gpushadermodule_free(ptr >>> 0, 1));
const GpuSurfaceFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_gpusurface_free(ptr >>> 0, 1));
const GpuTextureFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_gputexture_free(ptr >>> 0, 1));
const GpuTextureViewFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_gputextureview_free(ptr >>> 0, 1));
const IirFilterNodeFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_iirfilternode_free(ptr >>> 0, 1));
const KeyboardDeviceFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_keyboarddevice_free(ptr >>> 0, 1));
const MouseDeviceFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_mousedevice_free(ptr >>> 0, 1));
const OscillatorNodeFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_oscillatornode_free(ptr >>> 0, 1));
const PannerNodeFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_pannernode_free(ptr >>> 0, 1));
const PathFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_path_free(ptr >>> 0, 1));
const PeriodicWaveFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_periodicwave_free(ptr >>> 0, 1));
const StereoPannerNodeFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_stereopannernode_free(ptr >>> 0, 1));
const StorageDeviceFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_storagedevice_free(ptr >>> 0, 1));
const TouchDeviceFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_touchdevice_free(ptr >>> 0, 1));
const WaveShaperNodeFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_waveshapernode_free(ptr >>> 0, 1));

function addToExternrefTable0(obj) {
    const idx = wasm.__externref_table_alloc();
    wasm.__wbindgen_externrefs.set(idx, obj);
    return idx;
}

function _assertClass(instance, klass) {
    if (!(instance instanceof klass)) {
        throw new Error(`expected instance of ${klass.name}`);
    }
}

const CLOSURE_DTORS = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(state => state.dtor(state.a, state.b));

function debugString(val) {
    // primitive types
    const type = typeof val;
    if (type == 'number' || type == 'boolean' || val == null) {
        return  `${val}`;
    }
    if (type == 'string') {
        return `"${val}"`;
    }
    if (type == 'symbol') {
        const description = val.description;
        if (description == null) {
            return 'Symbol';
        } else {
            return `Symbol(${description})`;
        }
    }
    if (type == 'function') {
        const name = val.name;
        if (typeof name == 'string' && name.length > 0) {
            return `Function(${name})`;
        } else {
            return 'Function';
        }
    }
    // objects
    if (Array.isArray(val)) {
        const length = val.length;
        let debug = '[';
        if (length > 0) {
            debug += debugString(val[0]);
        }
        for(let i = 1; i < length; i++) {
            debug += ', ' + debugString(val[i]);
        }
        debug += ']';
        return debug;
    }
    // Test for built-in
    const builtInMatches = /\[object ([^\]]+)\]/.exec(toString.call(val));
    let className;
    if (builtInMatches && builtInMatches.length > 1) {
        className = builtInMatches[1];
    } else {
        // Failed to match the standard '[object ClassName]'
        return toString.call(val);
    }
    if (className == 'Object') {
        // we're a user defined class or Object
        // JSON.stringify avoids problems with cycles, and is generally much
        // easier than looping through ownProperties of `val`.
        try {
            return 'Object(' + JSON.stringify(val) + ')';
        } catch (_) {
            return 'Object';
        }
    }
    // errors
    if (val instanceof Error) {
        return `${val.name}: ${val.message}\n${val.stack}`;
    }
    // TODO we could test for more things here, like `Set`s and `Map`s.
    return className;
}

function getArrayF32FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getFloat32ArrayMemory0().subarray(ptr / 4, ptr / 4 + len);
}

function getArrayU8FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getUint8ArrayMemory0().subarray(ptr / 1, ptr / 1 + len);
}

let cachedDataViewMemory0 = null;
function getDataViewMemory0() {
    if (cachedDataViewMemory0 === null || cachedDataViewMemory0.buffer.detached === true || (cachedDataViewMemory0.buffer.detached === undefined && cachedDataViewMemory0.buffer !== wasm.memory.buffer)) {
        cachedDataViewMemory0 = new DataView(wasm.memory.buffer);
    }
    return cachedDataViewMemory0;
}

let cachedFloat32ArrayMemory0 = null;
function getFloat32ArrayMemory0() {
    if (cachedFloat32ArrayMemory0 === null || cachedFloat32ArrayMemory0.byteLength === 0) {
        cachedFloat32ArrayMemory0 = new Float32Array(wasm.memory.buffer);
    }
    return cachedFloat32ArrayMemory0;
}

function getStringFromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return decodeText(ptr, len);
}

let cachedUint8ArrayMemory0 = null;
function getUint8ArrayMemory0() {
    if (cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.byteLength === 0) {
        cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8ArrayMemory0;
}

function handleError(f, args) {
    try {
        return f.apply(this, args);
    } catch (e) {
        const idx = addToExternrefTable0(e);
        wasm.__wbindgen_exn_store(idx);
    }
}

function isLikeNone(x) {
    return x === undefined || x === null;
}

function makeClosure(arg0, arg1, dtor, f) {
    const state = { a: arg0, b: arg1, cnt: 1, dtor };
    const real = (...args) => {

        // First up with a closure we increment the internal reference
        // count. This ensures that the Rust closure environment won't
        // be deallocated while we're invoking it.
        state.cnt++;
        try {
            return f(state.a, state.b, ...args);
        } finally {
            real._wbg_cb_unref();
        }
    };
    real._wbg_cb_unref = () => {
        if (--state.cnt === 0) {
            state.dtor(state.a, state.b);
            state.a = 0;
            CLOSURE_DTORS.unregister(state);
        }
    };
    CLOSURE_DTORS.register(real, state, state);
    return real;
}

function makeMutClosure(arg0, arg1, dtor, f) {
    const state = { a: arg0, b: arg1, cnt: 1, dtor };
    const real = (...args) => {

        // First up with a closure we increment the internal reference
        // count. This ensures that the Rust closure environment won't
        // be deallocated while we're invoking it.
        state.cnt++;
        const a = state.a;
        state.a = 0;
        try {
            return f(a, state.b, ...args);
        } finally {
            state.a = a;
            real._wbg_cb_unref();
        }
    };
    real._wbg_cb_unref = () => {
        if (--state.cnt === 0) {
            state.dtor(state.a, state.b);
            state.a = 0;
            CLOSURE_DTORS.unregister(state);
        }
    };
    CLOSURE_DTORS.register(real, state, state);
    return real;
}

function passArray8ToWasm0(arg, malloc) {
    const ptr = malloc(arg.length * 1, 1) >>> 0;
    getUint8ArrayMemory0().set(arg, ptr / 1);
    WASM_VECTOR_LEN = arg.length;
    return ptr;
}

function passStringToWasm0(arg, malloc, realloc) {
    if (realloc === undefined) {
        const buf = cachedTextEncoder.encode(arg);
        const ptr = malloc(buf.length, 1) >>> 0;
        getUint8ArrayMemory0().subarray(ptr, ptr + buf.length).set(buf);
        WASM_VECTOR_LEN = buf.length;
        return ptr;
    }

    let len = arg.length;
    let ptr = malloc(len, 1) >>> 0;

    const mem = getUint8ArrayMemory0();

    let offset = 0;

    for (; offset < len; offset++) {
        const code = arg.charCodeAt(offset);
        if (code > 0x7F) break;
        mem[ptr + offset] = code;
    }
    if (offset !== len) {
        if (offset !== 0) {
            arg = arg.slice(offset);
        }
        ptr = realloc(ptr, len, len = offset + arg.length * 3, 1) >>> 0;
        const view = getUint8ArrayMemory0().subarray(ptr + offset, ptr + len);
        const ret = cachedTextEncoder.encodeInto(arg, view);

        offset += ret.written;
        ptr = realloc(ptr, len, offset, 1) >>> 0;
    }

    WASM_VECTOR_LEN = offset;
    return ptr;
}

let cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
cachedTextDecoder.decode();
const MAX_SAFARI_DECODE_BYTES = 2146435072;
let numBytesDecoded = 0;
function decodeText(ptr, len) {
    numBytesDecoded += len;
    if (numBytesDecoded >= MAX_SAFARI_DECODE_BYTES) {
        cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
        cachedTextDecoder.decode();
        numBytesDecoded = len;
    }
    return cachedTextDecoder.decode(getUint8ArrayMemory0().subarray(ptr, ptr + len));
}

const cachedTextEncoder = new TextEncoder();

if (!('encodeInto' in cachedTextEncoder)) {
    cachedTextEncoder.encodeInto = function (arg, view) {
        const buf = cachedTextEncoder.encode(arg);
        view.set(buf);
        return {
            read: arg.length,
            written: buf.length
        };
    };
}

let WASM_VECTOR_LEN = 0;

let wasmModule, wasm;
function __wbg_finalize_init(instance, module) {
    wasm = instance.exports;
    wasmModule = module;
    cachedDataViewMemory0 = null;
    cachedFloat32ArrayMemory0 = null;
    cachedUint8ArrayMemory0 = null;
    wasm.__wbindgen_start();
    return wasm;
}

async function __wbg_load(module, imports) {
    if (typeof Response === 'function' && module instanceof Response) {
        if (typeof WebAssembly.instantiateStreaming === 'function') {
            try {
                return await WebAssembly.instantiateStreaming(module, imports);
            } catch (e) {
                const validResponse = module.ok && expectedResponseType(module.type);

                if (validResponse && module.headers.get('Content-Type') !== 'application/wasm') {
                    console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve Wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);

                } else { throw e; }
            }
        }

        const bytes = await module.arrayBuffer();
        return await WebAssembly.instantiate(bytes, imports);
    } else {
        const instance = await WebAssembly.instantiate(module, imports);

        if (instance instanceof WebAssembly.Instance) {
            return { instance, module };
        } else {
            return instance;
        }
    }

    function expectedResponseType(type) {
        switch (type) {
            case 'basic': case 'cors': case 'default': return true;
        }
        return false;
    }
}

function initSync(module) {
    if (wasm !== undefined) return wasm;


    if (module !== undefined) {
        if (Object.getPrototypeOf(module) === Object.prototype) {
            ({module} = module)
        } else {
            console.warn('using deprecated parameters for `initSync()`; pass a single object instead')
        }
    }

    const imports = __wbg_get_imports();
    if (!(module instanceof WebAssembly.Module)) {
        module = new WebAssembly.Module(module);
    }
    const instance = new WebAssembly.Instance(module, imports);
    return __wbg_finalize_init(instance, module);
}

async function __wbg_init(module_or_path) {
    if (wasm !== undefined) return wasm;


    if (module_or_path !== undefined) {
        if (Object.getPrototypeOf(module_or_path) === Object.prototype) {
            ({module_or_path} = module_or_path)
        } else {
            console.warn('using deprecated parameters for the initialization function; pass a single object instead')
        }
    }

    if (module_or_path === undefined) {
        module_or_path = new URL('web_bg.wasm', import.meta.url);
    }
    const imports = __wbg_get_imports();

    if (typeof module_or_path === 'string' || (typeof Request === 'function' && module_or_path instanceof Request) || (typeof URL === 'function' && module_or_path instanceof URL)) {
        module_or_path = fetch(module_or_path);
    }

    const { instance, module } = await __wbg_load(await module_or_path, imports);

    return __wbg_finalize_init(instance, module);
}

export { initSync, __wbg_init as default };
