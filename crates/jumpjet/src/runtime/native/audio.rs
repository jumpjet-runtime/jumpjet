use std::io::Cursor;

use cpal::traits::DeviceTrait;
use wasmtime::Result;
use wasmtime::component::Resource;
use wasmtime_wasi::ResourceTable;
use web_audio_api::context::{AudioContext, AudioContextOptions, BaseAudioContext};
use web_audio_api::node::{AudioNode, AudioScheduledSourceNode, IIRFilterNode};

use super::state::JumpjetRuntimeState;
use crate::jumpjet::runtime::audio::*;

impl Host for JumpjetRuntimeState {
    async fn output(&mut self) -> Option<Resource<AudioDevice>> {
        Some(Resource::new_own(0))
    }
}

impl HostAudioDevice for JumpjetRuntimeState {
    async fn name(&mut self, _audio_device: Resource<AudioDevice>) -> String {
        match self.audio_state.device.name() {
            Ok(name) => name,
            Err(_) => "Unknown".to_owned(),
        }
    }

    async fn create_context(
        &mut self,
        _audio_device: Resource<AudioDevice>,
    ) -> Resource<AudioContext> {
        let audio_context = AudioContext::new(AudioContextOptions {
            sample_rate: Some(44100.),
            ..AudioContextOptions::default()
        });
        self.table.push(audio_context).unwrap()
    }

    async fn drop(&mut self, _rep: Resource<AudioDevice>) -> Result<()> {
        Ok(())
    }
}

impl HostAudioContext for JumpjetRuntimeState {
    async fn base_latency(&mut self, audio_context: Resource<AudioContext>) -> f32 {
        let audio_context = self.table.get(&audio_context).unwrap();
        audio_context.base_latency() as f32
    }

    async fn output_latency(&mut self, audio_context: Resource<AudioContext>) -> f32 {
        let audio_context = self.table.get(&audio_context).unwrap();
        audio_context.output_latency() as f32
    }

    async fn sink_id(&mut self, audio_context: Resource<AudioContext>) -> String {
        let audio_context = self.table.get(&audio_context).unwrap();
        audio_context.sink_id()
    }

    async fn set_sink_id(&mut self, audio_context: Resource<AudioContext>, sink_id: String) {
        let audio_context = self.table.get(&audio_context).unwrap();
        audio_context.set_sink_id_sync(sink_id).unwrap()
    }

    async fn render_capacity(
        &mut self,
        audio_context: Resource<AudioContext>,
    ) -> Resource<AudioRenderCapacity> {
        let audio_context = self.table.get(&audio_context).unwrap();
        self.table.push(audio_context.render_capacity()).unwrap()
    }

    async fn suspend(&mut self, audio_context: Resource<AudioContext>) {
        let audio_context = self.table.get(&audio_context).unwrap();
        audio_context.suspend_sync()
    }

    async fn resume(&mut self, audio_context: Resource<AudioContext>) {
        let audio_context = self.table.get(&audio_context).unwrap();
        audio_context.resume_sync()
    }

    async fn close(&mut self, audio_context: Resource<AudioContext>) {
        let audio_context = self.table.get(&audio_context).unwrap();
        audio_context.close_sync()
    }

    async fn decode_audio_data(
        &mut self,
        audio_context: Resource<AudioContext>,
        data: Vec<u8>,
    ) -> Resource<AudioBuffer> {
        let audio_context = self.table.get(&audio_context).unwrap();
        self.table
            .push(
                audio_context
                    .decode_audio_data_sync(Cursor::new(data))
                    .unwrap(),
            )
            .unwrap()
    }

    async fn create_buffer(
        &mut self,
        audio_context: Resource<AudioContext>,
        number_of_channels: u32,
        length: u32,
        sample_rate: f32,
    ) -> Resource<AudioBuffer> {
        let audio_context = self.table.get(&audio_context).unwrap();
        self.table
            .push(audio_context.create_buffer(
                number_of_channels as usize,
                length as usize,
                sample_rate,
            ))
            .unwrap()
    }

    async fn create_biquad_filter(
        &mut self,
        audio_context: Resource<AudioContext>,
    ) -> Resource<BiquadFilterNode> {
        let audio_context = self.table.get(&audio_context).unwrap();
        self.table
            .push(audio_context.create_biquad_filter())
            .unwrap()
    }

    async fn create_buffer_source(
        &mut self,
        audio_context: Resource<AudioContext>,
    ) -> Resource<AudioBufferSourceNode> {
        let audio_context = self.table.get(&audio_context).unwrap();
        self.table
            .push(audio_context.create_buffer_source())
            .unwrap()
    }

    async fn create_analyzer(
        &mut self,
        audio_context: Resource<AudioContext>,
    ) -> Resource<AnalyzerNode> {
        let audio_context = self.table.get(&audio_context).unwrap();
        self.table.push(audio_context.create_analyser()).unwrap()
    }

    async fn create_constant_source(
        &mut self,
        audio_context: Resource<AudioContext>,
    ) -> Resource<ConstantSourceNode> {
        let audio_context = self.table.get(&audio_context).unwrap();
        self.table
            .push(audio_context.create_constant_source())
            .unwrap()
    }

    async fn create_convolver(
        &mut self,
        audio_context: Resource<AudioContext>,
    ) -> Resource<ConvolverNode> {
        let audio_context = self.table.get(&audio_context).unwrap();
        self.table.push(audio_context.create_convolver()).unwrap()
    }

    async fn create_channel_merger(
        &mut self,
        audio_context: Resource<AudioContext>,
        number_of_inputs: u32,
    ) -> Resource<ChannelMergerNode> {
        let audio_context = self.table.get(&audio_context).unwrap();
        self.table
            .push(audio_context.create_channel_merger(number_of_inputs as usize))
            .unwrap()
    }

    async fn create_channel_splitter(
        &mut self,
        audio_context: Resource<AudioContext>,
        number_of_outputs: u32,
    ) -> Resource<ChannelSplitterNode> {
        let audio_context = self.table.get(&audio_context).unwrap();
        self.table
            .push(audio_context.create_channel_splitter(number_of_outputs as usize))
            .unwrap()
    }

    async fn create_delay(
        &mut self,
        audio_context: Resource<AudioContext>,
        max_delay_time: f32,
    ) -> Resource<DelayNode> {
        let audio_context = self.table.get(&audio_context).unwrap();
        self.table
            .push(audio_context.create_delay(max_delay_time as f64))
            .unwrap()
    }

    async fn create_dynamics_compressor(
        &mut self,
        audio_context: Resource<AudioContext>,
    ) -> Resource<DynamicsCompressorNode> {
        let audio_context = self.table.get(&audio_context).unwrap();
        self.table
            .push(audio_context.create_dynamics_compressor())
            .unwrap()
    }

    async fn create_gain(&mut self, audio_context: Resource<AudioContext>) -> Resource<GainNode> {
        let audio_context = self.table.get(&audio_context).unwrap();
        self.table.push(audio_context.create_gain()).unwrap()
    }

    async fn create_iir_filter(
        &mut self,
        audio_context: Resource<AudioContext>,
        feedforward: Vec<f32>,
        feedback: Vec<f32>,
    ) -> Resource<IIRFilterNode> {
        let audio_context = self.table.get(&audio_context).unwrap();
        self.table
            .push(audio_context.create_iir_filter(
                feedforward.iter().map(|&x| x as f64).collect(),
                feedback.iter().map(|&x| x as f64).collect(),
            ))
            .unwrap()
    }

    async fn create_oscillator(
        &mut self,
        audio_context: Resource<AudioContext>,
    ) -> Resource<OscillatorNode> {
        let audio_context = self.table.get(&audio_context).unwrap();
        self.table.push(audio_context.create_oscillator()).unwrap()
    }

    async fn create_panner(
        &mut self,
        audio_context: Resource<AudioContext>,
    ) -> Resource<PannerNode> {
        let audio_context = self.table.get(&audio_context).unwrap();
        self.table.push(audio_context.create_panner()).unwrap()
    }

    async fn create_periodic_wave(
        &mut self,
        _audio_context: Resource<AudioContext>,
        _options: PeriodicWaveOptions,
    ) -> PeriodicWave {
        todo!()
        // let audio_context = self.table.get(&self_).unwrap();
        // let periodic_wave = audio_context.create_periodic_wave(options.into());
        // periodic_wave.into()
    }

    async fn create_stereo_panner(
        &mut self,
        audio_context: Resource<AudioContext>,
    ) -> Resource<StereoPannerNode> {
        let audio_context = self.table.get(&audio_context).unwrap();
        self.table
            .push(audio_context.create_stereo_panner())
            .unwrap()
    }

    async fn create_wave_shaper(
        &mut self,
        audio_context: Resource<AudioContext>,
    ) -> Resource<WaveShaperNode> {
        let audio_context = self.table.get(&audio_context).unwrap();
        self.table.push(audio_context.create_wave_shaper()).unwrap()
    }

    async fn destination(
        &mut self,
        audio_context: Resource<AudioContext>,
    ) -> Resource<AudioDestinationNode> {
        let audio_context = self.table.get(&audio_context).unwrap();
        self.table.push(audio_context.destination()).unwrap()
    }

    async fn listener(&mut self, audio_context: Resource<AudioContext>) -> Resource<AudioListener> {
        let audio_context = self.table.get(&audio_context).unwrap();
        self.table.push(audio_context.listener()).unwrap()
    }

    async fn sample_rate(&mut self, audio_context: Resource<AudioContext>) -> f32 {
        let audio_context = self.table.get(&audio_context).unwrap();
        audio_context.sample_rate() as f32
    }

    async fn state(&mut self, audio_context: Resource<AudioContext>) -> AudioContextState {
        let audio_context = self.table.get(&audio_context).unwrap();
        audio_context.state().into()
    }

    async fn current_time(&mut self, audio_context: Resource<AudioContext>) -> f32 {
        let audio_context = self.table.get(&audio_context).unwrap();
        audio_context.current_time() as f32
    }

    async fn drop(&mut self, _rep: Resource<AudioContext>) -> Result<()> {
        Ok(())
    }
}

impl HostAudioBuffer for JumpjetRuntimeState {
    async fn new(&mut self, samples: Vec<Vec<f32>>, sample_rate: f32) -> Resource<AudioBuffer> {
        self.table
            .push(web_audio_api::AudioBuffer::from(samples, sample_rate))
            .unwrap()
    }

    async fn number_of_channels(&mut self, audio_buffer: Resource<AudioBuffer>) -> u32 {
        let audio_buffer = self.table.get(&audio_buffer).unwrap();
        audio_buffer.number_of_channels() as u32
    }

    async fn length(&mut self, audio_buffer: Resource<AudioBuffer>) -> u32 {
        let audio_buffer = self.table.get(&audio_buffer).unwrap();
        audio_buffer.length() as u32
    }

    async fn sample_rate(&mut self, audio_buffer: Resource<AudioBuffer>) -> f32 {
        let audio_buffer = self.table.get(&audio_buffer).unwrap();
        audio_buffer.sample_rate()
    }

    async fn duration(&mut self, audio_buffer: Resource<AudioBuffer>) -> f32 {
        let audio_buffer = self.table.get(&audio_buffer).unwrap();
        audio_buffer.duration() as f32
    }

    async fn get_channel_data(
        &mut self,
        audio_buffer: Resource<AudioBuffer>,
        channel_number: u32,
    ) -> Vec<f32> {
        let audio_buffer = self.table.get(&audio_buffer).unwrap();
        audio_buffer
            .get_channel_data(channel_number as usize)
            .to_vec()
    }

    async fn drop(&mut self, _rep: Resource<AudioBuffer>) -> Result<()> {
        Ok(())
    }
}

impl HostAudioParam for JumpjetRuntimeState {
    async fn automation_rate(&mut self, audio_param: Resource<AudioParam>) -> AutomationRate {
        let audio_param = self.table.get(&audio_param).unwrap();
        audio_param.automation_rate().into()
    }

    async fn set_automation_rate(
        &mut self,
        audio_param: Resource<AudioParam>,
        automation_rate: AutomationRate,
    ) {
        let audio_param = self.table.get_mut(&audio_param).unwrap();
        audio_param.set_automation_rate(automation_rate.into());
    }

    async fn default_value(&mut self, audio_param: Resource<AudioParam>) -> f32 {
        let audio_param = self.table.get(&audio_param).unwrap();
        audio_param.default_value()
    }

    async fn min_value(&mut self, audio_param: Resource<AudioParam>) -> f32 {
        let audio_param = self.table.get(&audio_param).unwrap();
        audio_param.min_value()
    }

    async fn max_value(&mut self, audio_param: Resource<AudioParam>) -> f32 {
        let audio_param = self.table.get(&audio_param).unwrap();
        audio_param.max_value()
    }

    async fn value(&mut self, audio_param: Resource<AudioParam>) -> f32 {
        let audio_param = self.table.get(&audio_param).unwrap();
        audio_param.value()
    }

    async fn set_value(&mut self, audio_param: Resource<AudioParam>, value: f32) {
        let audio_param = self.table.get(&audio_param).unwrap();
        audio_param.set_value(value);
    }

    async fn set_value_at_time(
        &mut self,
        audio_param: Resource<AudioParam>,
        value: f32,
        end_time: f32,
    ) {
        let audio_param = self.table.get_mut(&audio_param).unwrap();
        audio_param.set_value_at_time(value, end_time as f64);
    }

    async fn set_value_curve_at_time(
        &mut self,
        audio_param: Resource<AudioParam>,
        value: Vec<f32>,
        start_time: f32,
        duration: f32,
    ) {
        let audio_param = self.table.get_mut(&audio_param).unwrap();
        audio_param.set_value_curve_at_time(&value, start_time as f64, duration as f64);
    }

    async fn linear_ramp_to_value_at_time(
        &mut self,
        audio_param: Resource<AudioParam>,
        value: f32,
        end_time: f32,
    ) {
        let audio_param = self.table.get(&audio_param).unwrap();
        audio_param.linear_ramp_to_value_at_time(value, end_time as f64);
    }

    async fn exponential_ramp_to_value_at_time(
        &mut self,
        audio_param: Resource<AudioParam>,
        value: f32,
        end_time: f32,
    ) {
        let audio_param = self.table.get(&audio_param).unwrap();
        audio_param.exponential_ramp_to_value_at_time(value, end_time as f64);
    }

    async fn set_target_at_time(
        &mut self,
        audio_param: Resource<AudioParam>,
        value: f32,
        start_time: f32,
        time_constant: f32,
    ) {
        let audio_param = self.table.get_mut(&audio_param).unwrap();
        audio_param.set_target_at_time(value, start_time as f64, time_constant as f64);
    }

    async fn cancel_scheduled_values(
        &mut self,
        audio_param: Resource<AudioParam>,
        cancel_time: f32,
    ) {
        let audio_param = self.table.get(&audio_param).unwrap();
        audio_param.cancel_scheduled_values(cancel_time as f64);
    }

    async fn cancel_and_hold_at_time(
        &mut self,
        audio_param: Resource<AudioParam>,
        cancel_time: f32,
    ) {
        let audio_param = self.table.get(&audio_param).unwrap();
        audio_param.cancel_and_hold_at_time(cancel_time as f64);
    }

    async fn drop(&mut self, _rep: Resource<AudioParam>) -> Result<()> {
        Ok(())
    }
}

impl HostAudioRenderCapacity for JumpjetRuntimeState {
    async fn drop(&mut self, _rep: Resource<AudioRenderCapacity>) -> Result<()> {
        Ok(())
    }
}

impl HostAnalyzerNode for JumpjetRuntimeState {
    async fn new(&mut self, audio_context: Resource<AudioContext>) -> Resource<AnalyzerNode> {
        let audio_context = self.table.get(&audio_context).unwrap();
        self.table.push(audio_context.create_analyser()).unwrap()
    }
    async fn fft_size(&mut self, node: Resource<AnalyzerNode>) -> u32 {
        let audio_param = self.table.get(&node).unwrap();
        audio_param.fft_size() as u32
    }

    async fn set_fft_size(&mut self, node: Resource<AnalyzerNode>, fft_size: u32) {
        let audio_param = self.table.get_mut(&node).unwrap();
        audio_param.set_fft_size(fft_size as usize);
    }

    async fn smoothing_time_constant(&mut self, node: Resource<AnalyzerNode>) -> f32 {
        let audio_param = self.table.get(&node).unwrap();
        audio_param.smoothing_time_constant() as f32
    }

    async fn set_smoothing_time_constant(
        &mut self,
        node: Resource<AnalyzerNode>,
        smoothing_time_constant: f32,
    ) {
        let audio_param = self.table.get_mut(&node).unwrap();
        audio_param.set_smoothing_time_constant(smoothing_time_constant as f64)
    }

    async fn min_decibels(&mut self, node: Resource<AnalyzerNode>) -> f32 {
        let audio_param = self.table.get(&node).unwrap();
        audio_param.min_decibels() as f32
    }

    async fn set_min_decibels(&mut self, node: Resource<AnalyzerNode>, min_decibels: f32) {
        let audio_param = self.table.get_mut(&node).unwrap();
        audio_param.set_min_decibels(min_decibels as f64);
    }

    async fn max_decibels(&mut self, node: Resource<AnalyzerNode>) -> f32 {
        let audio_param = self.table.get(&node).unwrap();
        audio_param.max_decibels() as f32
    }

    async fn set_max_decibels(&mut self, node: Resource<AnalyzerNode>, max_decibels: f32) {
        let audio_param = self.table.get_mut(&node).unwrap();
        audio_param.set_max_decibels(max_decibels as f64);
    }

    async fn frequency_bin_count(&mut self, node: Resource<AnalyzerNode>) -> u32 {
        let audio_param = self.table.get(&node).unwrap();
        audio_param.frequency_bin_count() as u32
    }

    async fn get_float_time_domain_data(&mut self, node: Resource<AnalyzerNode>) -> Vec<f32> {
        let _audio_param = self.table.get(&node).unwrap();
        // audio_param.get_float_time_domain_data()
        Vec::new()
    }

    async fn get_byte_time_domain_data(&mut self, node: Resource<AnalyzerNode>) -> Vec<u8> {
        let _audio_param = self.table.get(&node).unwrap();
        // audio_param.get_byte_time_domain_data()
        Vec::new()
    }

    async fn get_float_frequency_data(&mut self, node: Resource<AnalyzerNode>) -> Vec<f32> {
        let _audio_param = self.table.get(&node).unwrap();
        // audio_param.get_float_frequency_data()
        Vec::new()
    }

    async fn get_byte_frequency_data(&mut self, node: Resource<AnalyzerNode>) -> Vec<u8> {
        let _audio_param = self.table.get(&node).unwrap();
        // audio_param.get_byte_frequency_data()
        Vec::new()
    }

    async fn connect(
        &mut self,
        node: Resource<AnalyzerNode>,
        destination: crate::jumpjet::runtime::audio::AudioNode,
    ) {
        let source: &dyn web_audio_api::node::AudioNode = { self.table.get(&node).unwrap() };
        audio_node_connect(&self.table, source, destination);
    }

    async fn drop(&mut self, _rep: Resource<AnalyzerNode>) -> Result<()> {
        Ok(())
    }
}

impl HostBiquadFilterNode for JumpjetRuntimeState {
    async fn new(&mut self, audio_context: Resource<AudioContext>) -> Resource<BiquadFilterNode> {
        let audio_context = self.table.get(&audio_context).unwrap();
        self.table
            .push(audio_context.create_biquad_filter())
            .unwrap()
    }
    async fn gain(&mut self, node: Resource<BiquadFilterNode>) -> Resource<AudioParam> {
        let node = self.table.get(&node).unwrap();
        self.table.push(node.gain().clone()).unwrap()
    }

    async fn frequency(&mut self, node: Resource<BiquadFilterNode>) -> Resource<AudioParam> {
        let node = self.table.get(&node).unwrap();
        self.table.push(node.frequency().clone()).unwrap()
    }

    async fn detune(&mut self, node: Resource<BiquadFilterNode>) -> Resource<AudioParam> {
        let node = self.table.get(&node).unwrap();
        self.table.push(node.detune().clone()).unwrap()
    }

    async fn q(&mut self, node: Resource<BiquadFilterNode>) -> Resource<AudioParam> {
        let node = self.table.get(&node).unwrap();
        self.table.push(node.q().clone()).unwrap()
    }

    async fn type_(&mut self, node: Resource<BiquadFilterNode>) -> BiquadFilterType {
        let node = self.table.get(&node).unwrap();
        node.type_().into()
    }

    async fn set_type(&mut self, node: Resource<BiquadFilterNode>, type_: BiquadFilterType) {
        let node = self.table.get_mut(&node).unwrap();
        node.set_type(type_.into())
    }

    async fn connect(
        &mut self,
        node: Resource<BiquadFilterNode>,
        destination: crate::jumpjet::runtime::audio::AudioNode,
    ) {
        let source: &dyn web_audio_api::node::AudioNode = { self.table.get(&node).unwrap() };
        audio_node_connect(&self.table, source, destination);
    }

    async fn drop(&mut self, _rep: Resource<BiquadFilterNode>) -> Result<()> {
        Ok(())
    }
}

impl HostAudioDestinationNode for JumpjetRuntimeState {
    async fn max_channel_count(&mut self, node: Resource<AudioDestinationNode>) -> u32 {
        let node = self.table.get(&node).unwrap();
        node.max_channel_count() as u32
    }

    async fn drop(&mut self, _rep: Resource<AudioDestinationNode>) -> Result<()> {
        Ok(())
    }
}

impl HostAudioBufferSourceNode for JumpjetRuntimeState {
    async fn new(
        &mut self,
        audio_context: Resource<AudioContext>,
    ) -> Resource<AudioBufferSourceNode> {
        let audio_context = self.table.get(&audio_context).unwrap();
        self.table
            .push(audio_context.create_buffer_source())
            .unwrap()
    }
    async fn start_at_with_offset(
        &mut self,
        node: Resource<AudioBufferSourceNode>,
        start: f32,
        offset: f32,
    ) {
        let node = self.table.get_mut(&node).unwrap();
        node.start_at_with_offset(start as f64, offset as f64);
    }

    async fn start_at_with_offset_and_duration(
        &mut self,
        node: Resource<AudioBufferSourceNode>,
        start: f32,
        offset: f32,
        duration: f32,
    ) {
        let node = self.table.get_mut(&node).unwrap();
        node.start_at_with_offset_and_duration(start as f64, offset as f64, duration as f64);
    }

    async fn buffer(
        &mut self,
        node: Resource<AudioBufferSourceNode>,
    ) -> Option<Resource<AudioBuffer>> {
        let node = self.table.get(&node).unwrap();
        match node.buffer() {
            Some(buffer) => Some(self.table.push(buffer.clone()).unwrap()),
            None => None,
        }
    }

    async fn set_buffer(
        &mut self,
        node: Resource<AudioBufferSourceNode>,
        audio_buffer: Resource<AudioBuffer>,
    ) {
        let audio_buffer = { self.table.get(&audio_buffer).unwrap().clone() };
        let node = self.table.get_mut(&node).unwrap();
        node.set_buffer(audio_buffer);
    }

    async fn playback_rate(
        &mut self,
        node: Resource<AudioBufferSourceNode>,
    ) -> Resource<AudioParam> {
        let node = self.table.get(&node).unwrap();
        self.table.push(node.playback_rate().clone()).unwrap()
    }

    async fn position(&mut self, node: Resource<AudioBufferSourceNode>) -> f32 {
        let node = self.table.get(&node).unwrap();
        node.position() as f32
    }

    async fn detune(&mut self, node: Resource<AudioBufferSourceNode>) -> Resource<AudioParam> {
        let node = self.table.get(&node).unwrap();
        self.table.push(node.detune().clone()).unwrap()
    }

    async fn loop_(&mut self, node: Resource<AudioBufferSourceNode>) -> bool {
        let node = self.table.get(&node).unwrap();
        node.loop_()
    }

    async fn set_loop(&mut self, node: Resource<AudioBufferSourceNode>, value: bool) {
        let node = self.table.get_mut(&node).unwrap();
        node.set_loop(value);
    }

    async fn loop_start(&mut self, node: Resource<AudioBufferSourceNode>) -> f32 {
        let node = self.table.get(&node).unwrap();
        node.loop_start() as f32
    }

    async fn set_loop_start(&mut self, node: Resource<AudioBufferSourceNode>, value: f32) {
        let node = self.table.get_mut(&node).unwrap();
        node.set_loop_start(value as f64);
    }

    async fn loop_end(&mut self, node: Resource<AudioBufferSourceNode>) -> f32 {
        let node = self.table.get(&node).unwrap();
        node.loop_end() as f32
    }

    async fn set_loop_end(&mut self, node: Resource<AudioBufferSourceNode>, value: f32) {
        let node = self.table.get_mut(&node).unwrap();
        node.set_loop_end(value as f64);
    }

    async fn connect(
        &mut self,
        node: Resource<AudioBufferSourceNode>,
        destination: crate::jumpjet::runtime::audio::AudioNode,
    ) {
        let source: &dyn web_audio_api::node::AudioNode = { self.table.get(&node).unwrap() };
        audio_node_connect(&self.table, source, destination);
    }

    async fn start(&mut self, node: Resource<AudioBufferSourceNode>) {
        let node = self.table.get_mut(&node).unwrap();
        node.start();
    }

    async fn drop(&mut self, _rep: Resource<AudioBufferSourceNode>) -> Result<()> {
        Ok(())
    }
}

impl HostConstantSourceNode for JumpjetRuntimeState {
    async fn new(&mut self, audio_context: Resource<AudioContext>) -> Resource<ConstantSourceNode> {
        let audio_context = self.table.get(&audio_context).unwrap();
        self.table
            .push(audio_context.create_constant_source())
            .unwrap()
    }
    async fn offset(&mut self, node: Resource<ConstantSourceNode>) -> Resource<AudioParam> {
        let node = self.table.get(&node).unwrap();
        self.table.push(node.offset().clone()).unwrap()
    }

    async fn connect(
        &mut self,
        node: Resource<ConstantSourceNode>,
        destination: crate::jumpjet::runtime::audio::AudioNode,
    ) {
        let source: &dyn web_audio_api::node::AudioNode = { self.table.get(&node).unwrap() };
        audio_node_connect(&self.table, source, destination);
    }

    async fn drop(&mut self, _rep: Resource<ConstantSourceNode>) -> Result<()> {
        Ok(())
    }
}

impl HostConvolverNode for JumpjetRuntimeState {
    async fn new(&mut self, audio_context: Resource<AudioContext>) -> Resource<ConvolverNode> {
        let audio_context = self.table.get(&audio_context).unwrap();
        self.table.push(audio_context.create_convolver()).unwrap()
    }
    async fn buffer(&mut self, node: Resource<ConvolverNode>) -> Option<Resource<AudioBuffer>> {
        let node = self.table.get(&node).unwrap();
        match node.buffer() {
            Some(buffer) => Some(self.table.push(buffer.clone()).unwrap()),
            None => None,
        }
    }

    async fn set_buffer(&mut self, node: Resource<ConvolverNode>, buffer: Resource<AudioBuffer>) {
        let buffer = { self.table.get(&buffer).unwrap().clone() };
        let node = self.table.get_mut(&node).unwrap();
        node.set_buffer(buffer);
    }

    async fn normalize(&mut self, node: Resource<ConvolverNode>) -> bool {
        let node = self.table.get(&node).unwrap();
        node.normalize()
    }

    async fn set_normalize(&mut self, node: Resource<ConvolverNode>, value: bool) {
        let node = self.table.get_mut(&node).unwrap();
        node.set_normalize(value);
    }

    async fn connect(
        &mut self,
        node: Resource<ConvolverNode>,
        destination: crate::jumpjet::runtime::audio::AudioNode,
    ) {
        let source: &dyn web_audio_api::node::AudioNode = { self.table.get(&node).unwrap() };
        audio_node_connect(&self.table, source, destination);
    }

    async fn drop(&mut self, _rep: Resource<ConvolverNode>) -> Result<()> {
        Ok(())
    }
}

impl HostChannelMergerNode for JumpjetRuntimeState {
    async fn new(
        &mut self,
        audio_context: Resource<AudioContext>,
        number_of_inputs: u32,
    ) -> Resource<ChannelMergerNode> {
        let audio_context = self.table.get(&audio_context).unwrap();
        self.table
            .push(audio_context.create_channel_merger(number_of_inputs as usize))
            .unwrap()
    }
    async fn connect(
        &mut self,
        node: Resource<ChannelMergerNode>,
        destination: crate::jumpjet::runtime::audio::AudioNode,
    ) {
        let source: &dyn web_audio_api::node::AudioNode = { self.table.get(&node).unwrap() };
        audio_node_connect(&self.table, source, destination);
    }

    async fn drop(&mut self, _rep: Resource<ChannelMergerNode>) -> Result<()> {
        Ok(())
    }
}

impl HostChannelSplitterNode for JumpjetRuntimeState {
    async fn new(
        &mut self,
        audio_context: Resource<AudioContext>,
        number_of_outputs: u32,
    ) -> Resource<ChannelSplitterNode> {
        let audio_context = self.table.get(&audio_context).unwrap();
        self.table
            .push(audio_context.create_channel_splitter(number_of_outputs as usize))
            .unwrap()
    }
    async fn connect(
        &mut self,
        node: Resource<ChannelSplitterNode>,
        destination: crate::jumpjet::runtime::audio::AudioNode,
    ) {
        let source: &dyn web_audio_api::node::AudioNode = { self.table.get(&node).unwrap() };
        audio_node_connect(&self.table, source, destination);
    }

    async fn drop(&mut self, _rep: Resource<ChannelSplitterNode>) -> Result<()> {
        Ok(())
    }
}

impl HostDelayNode for JumpjetRuntimeState {
    async fn new(
        &mut self,
        audio_context: Resource<AudioContext>,
        max_delay_time: f32,
    ) -> Resource<DelayNode> {
        let audio_context = self.table.get(&audio_context).unwrap();
        self.table
            .push(audio_context.create_delay(max_delay_time as f64))
            .unwrap()
    }
    async fn delay_time(&mut self, node: Resource<DelayNode>) -> Resource<AudioParam> {
        let node = self.table.get(&node).unwrap();
        self.table.push(node.delay_time().clone()).unwrap()
    }

    async fn connect(
        &mut self,
        node: Resource<DelayNode>,
        destination: crate::jumpjet::runtime::audio::AudioNode,
    ) {
        let source: &dyn web_audio_api::node::AudioNode = { self.table.get(&node).unwrap() };
        audio_node_connect(&self.table, source, destination);
    }

    async fn drop(&mut self, _rep: Resource<DelayNode>) -> Result<()> {
        Ok(())
    }
}

impl HostDynamicsCompressorNode for JumpjetRuntimeState {
    async fn new(
        &mut self,
        audio_context: Resource<AudioContext>,
    ) -> Resource<DynamicsCompressorNode> {
        let audio_context = self.table.get(&audio_context).unwrap();
        self.table
            .push(audio_context.create_dynamics_compressor())
            .unwrap()
    }
    async fn attack(&mut self, node: Resource<DynamicsCompressorNode>) -> Resource<AudioParam> {
        let node = self.table.get(&node).unwrap();
        self.table.push(node.attack().clone()).unwrap()
    }

    async fn knee(&mut self, node: Resource<DynamicsCompressorNode>) -> Resource<AudioParam> {
        let node = self.table.get(&node).unwrap();
        self.table.push(node.knee().clone()).unwrap()
    }

    async fn ratio(&mut self, node: Resource<DynamicsCompressorNode>) -> Resource<AudioParam> {
        let node = self.table.get(&node).unwrap();
        self.table.push(node.ratio().clone()).unwrap()
    }

    async fn release(&mut self, node: Resource<DynamicsCompressorNode>) -> Resource<AudioParam> {
        let node = self.table.get(&node).unwrap();
        self.table.push(node.release().clone()).unwrap()
    }

    async fn threshold(&mut self, node: Resource<DynamicsCompressorNode>) -> Resource<AudioParam> {
        let node = self.table.get(&node).unwrap();
        self.table.push(node.threshold().clone()).unwrap()
    }

    async fn reduction(&mut self, node: Resource<DynamicsCompressorNode>) -> f32 {
        let node = self.table.get(&node).unwrap();
        node.reduction()
    }

    async fn connect(
        &mut self,
        node: Resource<DynamicsCompressorNode>,
        destination: crate::jumpjet::runtime::audio::AudioNode,
    ) {
        let source: &dyn web_audio_api::node::AudioNode = { self.table.get(&node).unwrap() };
        audio_node_connect(&self.table, source, destination);
    }

    async fn drop(&mut self, _rep: Resource<DynamicsCompressorNode>) -> Result<()> {
        Ok(())
    }
}

impl HostGainNode for JumpjetRuntimeState {
    async fn new(&mut self, audio_context: Resource<AudioContext>) -> Resource<GainNode> {
        let audio_context = self.table.get(&audio_context).unwrap();
        self.table.push(audio_context.create_gain()).unwrap()
    }
    async fn gain(&mut self, node: Resource<GainNode>) -> Resource<AudioParam> {
        let node = self.table.get(&node).unwrap();
        self.table.push(node.gain().clone()).unwrap()
    }

    async fn connect(
        &mut self,
        node: Resource<GainNode>,
        destination: crate::jumpjet::runtime::audio::AudioNode,
    ) {
        let source: &dyn web_audio_api::node::AudioNode = { self.table.get(&node).unwrap() };
        audio_node_connect(&self.table, source, destination);
    }

    async fn drop(&mut self, _rep: Resource<GainNode>) -> Result<()> {
        Ok(())
    }
}

impl HostIirFilterNode for JumpjetRuntimeState {
    async fn new(
        &mut self,
        audio_context: Resource<AudioContext>,
        feedforward: Vec<f32>,
        feedback: Vec<f32>,
    ) -> Resource<IIRFilterNode> {
        let audio_context = self.table.get(&audio_context).unwrap();
        self.table
            .push(audio_context.create_iir_filter(
                feedforward.iter().map(|&x| x as f64).collect(),
                feedback.iter().map(|&x| x as f64).collect(),
            ))
            .unwrap()
    }
    async fn connect(
        &mut self,
        node: Resource<IIRFilterNode>,
        destination: crate::jumpjet::runtime::audio::AudioNode,
    ) {
        let source: &dyn web_audio_api::node::AudioNode = { self.table.get(&node).unwrap() };
        audio_node_connect(&self.table, source, destination);
    }

    async fn drop(&mut self, _rep: Resource<IIRFilterNode>) -> Result<()> {
        Ok(())
    }
}

impl HostOscillatorNode for JumpjetRuntimeState {
    async fn new(&mut self, audio_context: Resource<AudioContext>) -> Resource<OscillatorNode> {
        let audio_context = self.table.get(&audio_context).unwrap();
        self.table.push(audio_context.create_oscillator()).unwrap()
    }
    async fn detune(&mut self, node: Resource<OscillatorNode>) -> Resource<AudioParam> {
        let node = self.table.get(&node).unwrap();
        self.table.push(node.detune().clone()).unwrap()
    }

    async fn frequency(&mut self, node: Resource<OscillatorNode>) -> Resource<AudioParam> {
        let node = self.table.get(&node).unwrap();
        self.table.push(node.frequency().clone()).unwrap()
    }

    async fn type_(&mut self, node: Resource<OscillatorNode>) -> OscillatorType {
        let node = self.table.get(&node).unwrap();
        node.type_().into()
    }

    async fn set_type(&mut self, node: Resource<OscillatorNode>, _type: OscillatorType) {
        let node = self.table.get_mut(&node).unwrap();
        node.set_type(_type.into());
    }

    async fn set_periodic_wave(
        &mut self,
        _node: Resource<OscillatorNode>,
        _periodic_wave: PeriodicWave,
    ) {
        // let node = self.table.get_mut(&self_).unwrap();
        // node.set_periodic_wave(periodic_wave.into());
        todo!()
    }

    async fn connect(
        &mut self,
        node: Resource<OscillatorNode>,
        destination: crate::jumpjet::runtime::audio::AudioNode,
    ) {
        let source: &dyn web_audio_api::node::AudioNode = { self.table.get(&node).unwrap() };
        audio_node_connect(&self.table, source, destination);
    }

    async fn drop(&mut self, _rep: Resource<OscillatorNode>) -> Result<()> {
        Ok(())
    }
}

impl HostPannerNode for JumpjetRuntimeState {
    async fn new(&mut self, audio_context: Resource<AudioContext>) -> Resource<PannerNode> {
        let audio_context = self.table.get(&audio_context).unwrap();
        self.table.push(audio_context.create_panner()).unwrap()
    }
    async fn position_x(&mut self, node: Resource<PannerNode>) -> Resource<AudioParam> {
        let node = self.table.get(&node).unwrap();
        self.table.push(node.position_x().clone()).unwrap()
    }

    async fn position_y(&mut self, node: Resource<PannerNode>) -> Resource<AudioParam> {
        let node = self.table.get(&node).unwrap();
        self.table.push(node.position_y().clone()).unwrap()
    }

    async fn position_z(&mut self, node: Resource<PannerNode>) -> Resource<AudioParam> {
        let node = self.table.get(&node).unwrap();
        self.table.push(node.position_z().clone()).unwrap()
    }

    async fn set_position(&mut self, node: Resource<PannerNode>, x: f32, y: f32, z: f32) {
        let node = self.table.get_mut(&node).unwrap();
        node.set_position(x, y, z);
    }

    async fn orientation_x(&mut self, node: Resource<PannerNode>) -> Resource<AudioParam> {
        let node = self.table.get(&node).unwrap();
        self.table.push(node.orientation_x().clone()).unwrap()
    }

    async fn orientation_y(&mut self, node: Resource<PannerNode>) -> Resource<AudioParam> {
        let node = self.table.get(&node).unwrap();
        self.table.push(node.orientation_y().clone()).unwrap()
    }

    async fn orientation_z(&mut self, node: Resource<PannerNode>) -> Resource<AudioParam> {
        let node = self.table.get(&node).unwrap();
        self.table.push(node.orientation_z().clone()).unwrap()
    }

    async fn set_orientation(&mut self, node: Resource<PannerNode>, x: f32, y: f32, z: f32) {
        let node = self.table.get_mut(&node).unwrap();
        node.set_orientation(x, y, z);
    }

    async fn distance_model(&mut self, node: Resource<PannerNode>) -> DistanceModelType {
        let node = self.table.get(&node).unwrap();
        node.distance_model().into()
    }

    async fn set_distance_model(&mut self, node: Resource<PannerNode>, value: DistanceModelType) {
        let node = self.table.get_mut(&node).unwrap();
        node.set_distance_model(value.into());
    }

    async fn ref_distance(&mut self, node: Resource<PannerNode>) -> f32 {
        let node = self.table.get(&node).unwrap();
        node.ref_distance() as f32
    }

    async fn set_ref_distance(&mut self, node: Resource<PannerNode>, value: f32) {
        let node = self.table.get_mut(&node).unwrap();
        node.set_ref_distance(value as f64);
    }

    async fn max_distance(&mut self, node: Resource<PannerNode>) -> f32 {
        let node = self.table.get(&node).unwrap();
        node.max_distance() as f32
    }

    async fn set_max_distance(&mut self, node: Resource<PannerNode>, value: f32) {
        let node = self.table.get_mut(&node).unwrap();
        node.set_max_distance(value as f64);
    }

    async fn rolloff_factor(&mut self, node: Resource<PannerNode>) -> f32 {
        let node = self.table.get(&node).unwrap();
        node.rolloff_factor() as f32
    }

    async fn set_rolloff_factor(&mut self, node: Resource<PannerNode>, value: f32) {
        let node = self.table.get_mut(&node).unwrap();
        node.set_rolloff_factor(value as f64);
    }

    async fn cone_inner_angle(&mut self, node: Resource<PannerNode>) -> f32 {
        let node = self.table.get(&node).unwrap();
        node.cone_inner_angle() as f32
    }

    async fn set_cone_inner_angle(&mut self, node: Resource<PannerNode>, value: f32) {
        let node = self.table.get_mut(&node).unwrap();
        node.set_cone_inner_angle(value as f64);
    }

    async fn cone_outer_angle(&mut self, node: Resource<PannerNode>) -> f32 {
        let node = self.table.get(&node).unwrap();
        node.cone_outer_angle() as f32
    }

    async fn set_cone_outer_angle(&mut self, node: Resource<PannerNode>, value: f32) {
        let node = self.table.get_mut(&node).unwrap();
        node.set_cone_outer_angle(value as f64);
    }

    async fn cone_outer_gain(&mut self, node: Resource<PannerNode>) -> f32 {
        let node = self.table.get(&node).unwrap();
        node.cone_outer_gain() as f32
    }

    async fn set_cone_outer_gain(&mut self, node: Resource<PannerNode>, value: f32) {
        let node = self.table.get_mut(&node).unwrap();
        node.set_cone_outer_gain(value as f64);
    }

    async fn panning_model(&mut self, node: Resource<PannerNode>) -> PanningModelType {
        let node = self.table.get(&node).unwrap();
        node.panning_model().into()
    }

    async fn set_panning_model(&mut self, node: Resource<PannerNode>, value: PanningModelType) {
        let node = self.table.get_mut(&node).unwrap();
        node.set_panning_model(value.into());
    }

    async fn connect(
        &mut self,
        node: Resource<PannerNode>,
        destination: crate::jumpjet::runtime::audio::AudioNode,
    ) {
        let source: &dyn web_audio_api::node::AudioNode = { self.table.get(&node).unwrap() };
        audio_node_connect(&self.table, source, destination);
    }

    async fn drop(&mut self, _rep: Resource<PannerNode>) -> Result<()> {
        Ok(())
    }
}

impl HostStereoPannerNode for JumpjetRuntimeState {
    async fn pan(&mut self, node: Resource<StereoPannerNode>) -> Resource<AudioParam> {
        let node = self.table.get(&node).unwrap();
        self.table.push(node.pan().clone()).unwrap()
    }

    async fn connect(
        &mut self,
        node: Resource<StereoPannerNode>,
        destination: crate::jumpjet::runtime::audio::AudioNode,
    ) {
        let source: &dyn web_audio_api::node::AudioNode = { self.table.get(&node).unwrap() };
        audio_node_connect(&self.table, source, destination);
    }

    async fn drop(&mut self, _rep: Resource<StereoPannerNode>) -> Result<()> {
        Ok(())
    }
}

impl HostWaveShaperNode for JumpjetRuntimeState {
    async fn curve(&mut self, node: Resource<WaveShaperNode>) -> Option<Vec<f32>> {
        let node = self.table.get_mut(&node).unwrap();
        node.curve().map(|c| c.to_vec())
    }

    async fn set_curve(&mut self, node: Resource<WaveShaperNode>, curve: Vec<f32>) {
        let node = self.table.get_mut(&node).unwrap();
        node.set_curve(curve);
    }

    async fn oversample(&mut self, node: Resource<WaveShaperNode>) -> OverSampleType {
        let node = self.table.get_mut(&node).unwrap();
        node.oversample().into()
    }

    async fn set_oversample(&mut self, node: Resource<WaveShaperNode>, oversample: OverSampleType) {
        let node = self.table.get_mut(&node).unwrap();
        node.set_oversample(oversample.into());
    }

    async fn connect(
        &mut self,
        node: Resource<WaveShaperNode>,
        destination: crate::jumpjet::runtime::audio::AudioNode,
    ) {
        let source: &dyn web_audio_api::node::AudioNode = { self.table.get(&node).unwrap() };
        audio_node_connect(&self.table, source, destination);
    }

    async fn drop(&mut self, _rep: Resource<WaveShaperNode>) -> Result<()> {
        Ok(())
    }
}

impl HostAudioListener for JumpjetRuntimeState {
    async fn position_x(&mut self, listener: Resource<AudioListener>) -> Resource<AudioParam> {
        let node = self.table.get(&listener).unwrap();
        self.table.push(node.position_x().clone()).unwrap()
    }

    async fn position_y(&mut self, listener: Resource<AudioListener>) -> Resource<AudioParam> {
        let node = self.table.get(&listener).unwrap();
        self.table.push(node.position_y().clone()).unwrap()
    }

    async fn position_z(&mut self, listener: Resource<AudioListener>) -> Resource<AudioParam> {
        let node = self.table.get(&listener).unwrap();
        self.table.push(node.position_z().clone()).unwrap()
    }

    async fn forward_x(&mut self, listener: Resource<AudioListener>) -> Resource<AudioParam> {
        let node = self.table.get(&listener).unwrap();
        self.table.push(node.forward_x().clone()).unwrap()
    }

    async fn forward_y(&mut self, listener: Resource<AudioListener>) -> Resource<AudioParam> {
        let node = self.table.get(&listener).unwrap();
        self.table.push(node.forward_y().clone()).unwrap()
    }

    async fn forward_z(&mut self, listener: Resource<AudioListener>) -> Resource<AudioParam> {
        let node = self.table.get(&listener).unwrap();
        self.table.push(node.forward_z().clone()).unwrap()
    }

    async fn up_x(&mut self, listener: Resource<AudioListener>) -> Resource<AudioParam> {
        let node = self.table.get(&listener).unwrap();
        self.table.push(node.up_x().clone()).unwrap()
    }

    async fn up_y(&mut self, listener: Resource<AudioListener>) -> Resource<AudioParam> {
        let node = self.table.get(&listener).unwrap();
        self.table.push(node.up_y().clone()).unwrap()
    }

    async fn up_z(&mut self, listener: Resource<AudioListener>) -> Resource<AudioParam> {
        let node = self.table.get(&listener).unwrap();
        self.table.push(node.up_z().clone()).unwrap()
    }

    async fn drop(&mut self, _rep: Resource<AudioListener>) -> Result<()> {
        Ok(())
    }
}

fn audio_node_connect(
    table: &ResourceTable,
    source: &dyn AudioNode,
    destination: crate::jumpjet::runtime::audio::AudioNode,
) {
    let audio_node: &dyn web_audio_api::node::AudioNode = {
        match destination {
            crate::jumpjet::runtime::audio::AudioNode::Analyzer(d) => table.get(&d).unwrap(),
            crate::jumpjet::runtime::audio::AudioNode::BiquadFilter(d) => table.get(&d).unwrap(),
            crate::jumpjet::runtime::audio::AudioNode::BufferSource(d) => table.get(&d).unwrap(),
            crate::jumpjet::runtime::audio::AudioNode::Destination(d) => table.get(&d).unwrap(),
            crate::jumpjet::runtime::audio::AudioNode::ConstantSource(d) => table.get(&d).unwrap(),
            crate::jumpjet::runtime::audio::AudioNode::Convolver(d) => table.get(&d).unwrap(),
            crate::jumpjet::runtime::audio::AudioNode::ChannelMerger(d) => table.get(&d).unwrap(),
            crate::jumpjet::runtime::audio::AudioNode::ChannelSplitter(d) => table.get(&d).unwrap(),
            crate::jumpjet::runtime::audio::AudioNode::Delay(d) => table.get(&d).unwrap(),
            crate::jumpjet::runtime::audio::AudioNode::DynamicsCompressor(d) => {
                table.get(&d).unwrap()
            }
            crate::jumpjet::runtime::audio::AudioNode::Gain(d) => table.get(&d).unwrap(),
            crate::jumpjet::runtime::audio::AudioNode::Oscillator(d) => table.get(&d).unwrap(),
            crate::jumpjet::runtime::audio::AudioNode::Panner(d) => table.get(&d).unwrap(),
            crate::jumpjet::runtime::audio::AudioNode::StereoPanner(d) => table.get(&d).unwrap(),
            crate::jumpjet::runtime::audio::AudioNode::WaveShaper(d) => table.get(&d).unwrap(),
        }
    };
    source.connect(audio_node);
}
