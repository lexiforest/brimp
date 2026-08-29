use std::{
    collections::{HashMap, HashSet, VecDeque},
    future::Future,
    io::Cursor,
    pin::Pin,
    sync::{
        Arc, Mutex,
        mpsc::{SyncSender, sync_channel},
    },
    task::{Context as TaskContext, Poll, Waker},
    time::Duration,
};

use web_audio_api::{
    AudioBuffer, AudioListener, AudioParam, AudioProcessingEvent, AutomationRate, PeriodicWave,
    PeriodicWaveOptions,
    context::{AudioContext, AudioContextOptions, BaseAudioContext, OfflineAudioContext},
    node::{
        AnalyserNode, AudioBufferSourceNode, AudioNode, AudioScheduledSourceNode, BiquadFilterNode,
        BiquadFilterType, ChannelCountMode, ChannelInterpretation, ChannelMergerNode,
        ChannelSplitterNode, ConstantSourceNode, ConvolverNode, DelayNode, DistanceModelType,
        DynamicsCompressorNode, GainNode, IIRFilterNode, MediaStreamAudioDestinationNode,
        MediaStreamAudioSourceNode, MediaStreamTrackAudioSourceNode, OscillatorNode,
        OscillatorType, OverSampleType, PannerNode, PanningModelType, ScriptProcessorNode,
        StereoPannerNode, WaveShaperNode,
    },
    worklet::AudioWorkletNode,
};

use crate::audio_worklet::{
    AudioWorkletState, WorkletNodeRequest, WorkletParamDescriptor, WorkletRenderEvent,
};

const LISTENER_NODE_ID: u64 = 9_007_199_254_740_991;
const AUDIO_RENDER_QUANTUM_SIZE: usize = 128;

enum Node {
    Oscillator(OscillatorNode),
    Compressor(DynamicsCompressorNode),
    Gain(GainNode),
    Biquad(BiquadFilterNode),
    StereoPanner(StereoPannerNode),
    BufferSource(AudioBufferSourceNode),
    Analyser(AnalyserNode),
    ConstantSource(ConstantSourceNode),
    Delay(DelayNode),
    WaveShaper(WaveShaperNode),
    ChannelSplitter(ChannelSplitterNode),
    ChannelMerger(ChannelMergerNode),
    Convolver(ConvolverNode),
    IirFilter(IIRFilterNode),
    Panner(PannerNode),
    MediaStreamDestination(MediaStreamAudioDestinationNode),
    MediaStreamSource(MediaStreamAudioSourceNode),
    MediaStreamTrackSource(MediaStreamTrackAudioSourceNode),
    ScriptProcessor(ScriptProcessorNode),
    Worklet(AudioWorkletNode),
}

impl Node {
    fn audio_node(&self) -> &dyn AudioNode {
        match self {
            Self::Oscillator(node) => node,
            Self::Compressor(node) => node,
            Self::Gain(node) => node,
            Self::Biquad(node) => node,
            Self::StereoPanner(node) => node,
            Self::BufferSource(node) => node,
            Self::Analyser(node) => node,
            Self::ConstantSource(node) => node,
            Self::Delay(node) => node,
            Self::WaveShaper(node) => node,
            Self::ChannelSplitter(node) => node,
            Self::ChannelMerger(node) => node,
            Self::Convolver(node) => node,
            Self::IirFilter(node) => node,
            Self::Panner(node) => node,
            Self::MediaStreamDestination(node) => node,
            Self::MediaStreamSource(node) => node,
            Self::MediaStreamTrackSource(node) => node,
            Self::ScriptProcessor(node) => node,
            Self::Worklet(node) => node,
        }
    }
}

struct ProcessingRequest {
    node: u64,
    input: AudioBuffer,
    output: AudioBuffer,
    playback_time: f64,
    response: SyncSender<Vec<Vec<f32>>>,
}

#[derive(serde::Serialize)]
struct WorkletMessage {
    node: u64,
    kind: &'static str,
    data: String,
}

#[derive(Default)]
struct ProcessingQueue {
    requests: VecDeque<ProcessingRequest>,
    closed: bool,
}

struct Context {
    inner: ContextInner,
    nodes: HashMap<u64, Node>,
    next_node: u64,
    buffers: HashMap<u64, AudioBuffer>,
    next_buffer: u64,
    periodic_waves: HashMap<u64, PeriodicWave>,
    next_periodic_wave: u64,
    rendered: bool,
    offline_render_polled: bool,
    offline_render: Option<Pin<Box<dyn Future<Output = AudioBuffer>>>>,
    offline_resume: Option<Pin<Box<dyn Future<Output = ()>>>>,
    offline_suspensions: HashMap<u64, Pin<Box<dyn Future<Output = ()>>>>,
    offline_suspend_quanta: HashSet<usize>,
    next_offline_suspension: u64,
    listener: AudioListener,
    ended_nodes: Arc<Mutex<Vec<u64>>>,
    processing_requests: Arc<Mutex<ProcessingQueue>>,
    processing_responses: HashMap<u64, SyncSender<Vec<Vec<f32>>>>,
    next_processing_event: u64,
    stopped_media_stream_tracks: HashSet<u64>,
    worklet: AudioWorkletState,
    worklet_messages: Arc<Mutex<VecDeque<WorkletMessage>>>,
}

enum ContextInner {
    Offline(Arc<OfflineAudioContext>),
    Online(AudioContext),
}

impl Context {
    fn create_oscillator(&self) -> OscillatorNode {
        match &self.inner {
            ContextInner::Offline(inner) => inner.create_oscillator(),
            ContextInner::Online(inner) => inner.create_oscillator(),
        }
    }
    fn create_compressor(&self) -> DynamicsCompressorNode {
        match &self.inner {
            ContextInner::Offline(inner) => inner.create_dynamics_compressor(),
            ContextInner::Online(inner) => inner.create_dynamics_compressor(),
        }
    }
    fn create_gain(&self) -> GainNode {
        match &self.inner {
            ContextInner::Offline(inner) => inner.create_gain(),
            ContextInner::Online(inner) => inner.create_gain(),
        }
    }
    fn create_biquad(&self) -> BiquadFilterNode {
        match &self.inner {
            ContextInner::Offline(inner) => inner.create_biquad_filter(),
            ContextInner::Online(inner) => inner.create_biquad_filter(),
        }
    }
    fn create_stereo_panner(&self) -> StereoPannerNode {
        match &self.inner {
            ContextInner::Offline(inner) => inner.create_stereo_panner(),
            ContextInner::Online(inner) => inner.create_stereo_panner(),
        }
    }
    fn create_buffer_source(&self) -> AudioBufferSourceNode {
        match &self.inner {
            ContextInner::Offline(inner) => inner.create_buffer_source(),
            ContextInner::Online(inner) => inner.create_buffer_source(),
        }
    }
    fn create_analyser(&self) -> AnalyserNode {
        match &self.inner {
            ContextInner::Offline(inner) => inner.create_analyser(),
            ContextInner::Online(inner) => inner.create_analyser(),
        }
    }
    fn create_constant_source(&self) -> ConstantSourceNode {
        match &self.inner {
            ContextInner::Offline(inner) => inner.create_constant_source(),
            ContextInner::Online(inner) => inner.create_constant_source(),
        }
    }
    fn create_delay(&self, max_delay_time: f64) -> DelayNode {
        match &self.inner {
            ContextInner::Offline(inner) => inner.create_delay(max_delay_time),
            ContextInner::Online(inner) => inner.create_delay(max_delay_time),
        }
    }
    fn create_wave_shaper(&self) -> WaveShaperNode {
        match &self.inner {
            ContextInner::Offline(inner) => inner.create_wave_shaper(),
            ContextInner::Online(inner) => inner.create_wave_shaper(),
        }
    }
    fn create_channel_splitter(&self, outputs: usize) -> ChannelSplitterNode {
        match &self.inner {
            ContextInner::Offline(inner) => inner.create_channel_splitter(outputs),
            ContextInner::Online(inner) => inner.create_channel_splitter(outputs),
        }
    }
    fn create_channel_merger(&self, inputs: usize) -> ChannelMergerNode {
        match &self.inner {
            ContextInner::Offline(inner) => inner.create_channel_merger(inputs),
            ContextInner::Online(inner) => inner.create_channel_merger(inputs),
        }
    }
    fn create_convolver(&self) -> ConvolverNode {
        match &self.inner {
            ContextInner::Offline(inner) => inner.create_convolver(),
            ContextInner::Online(inner) => inner.create_convolver(),
        }
    }
    fn create_iir_filter(&self, feedforward: Vec<f64>, feedback: Vec<f64>) -> IIRFilterNode {
        match &self.inner {
            ContextInner::Offline(inner) => inner.create_iir_filter(feedforward, feedback),
            ContextInner::Online(inner) => inner.create_iir_filter(feedforward, feedback),
        }
    }
    fn create_panner(&self) -> PannerNode {
        match &self.inner {
            ContextInner::Offline(inner) => inner.create_panner(),
            ContextInner::Online(inner) => inner.create_panner(),
        }
    }
    fn create_periodic_wave(&self, options: PeriodicWaveOptions) -> PeriodicWave {
        match &self.inner {
            ContextInner::Offline(inner) => inner.create_periodic_wave(options),
            ContextInner::Online(inner) => inner.create_periodic_wave(options),
        }
    }
    fn create_media_stream_destination(&self) -> Result<MediaStreamAudioDestinationNode, String> {
        let ContextInner::Online(inner) = &self.inner else {
            return Err("MediaStreamAudioDestinationNode requires an AudioContext".to_owned());
        };
        let node = inner.create_media_stream_destination();
        node.set_channel_count(2);
        node.set_channel_count_mode(ChannelCountMode::Explicit);
        Ok(node)
    }
    fn create_script_processor(
        &self,
        buffer_size: usize,
        input_channels: usize,
        output_channels: usize,
    ) -> Result<ScriptProcessorNode, String> {
        let ContextInner::Online(inner) = &self.inner else {
            return Err("ScriptProcessorNode requires an AudioContext".to_owned());
        };
        Ok(inner.create_script_processor(buffer_size, input_channels, output_channels))
    }
    fn destination(&self) -> web_audio_api::node::AudioDestinationNode {
        match &self.inner {
            ContextInner::Offline(inner) => inner.destination(),
            ContextInner::Online(inner) => inner.destination(),
        }
    }
    fn decode_audio_data(
        &self,
        bytes: Cursor<Vec<u8>>,
    ) -> Result<AudioBuffer, Box<dyn std::error::Error + Send + Sync>> {
        match &self.inner {
            ContextInner::Offline(inner) => inner.decode_audio_data_sync(bytes),
            ContextInner::Online(inner) => inner.decode_audio_data_sync(bytes),
        }
    }
}

#[derive(Default)]
pub(crate) struct AudioStore {
    contexts: HashMap<u64, Context>,
    next_context: u64,
}

impl AudioStore {
    pub(crate) fn clear(&mut self) {
        self.contexts.clear();
    }

    pub(crate) fn create_offline(
        &mut self,
        channels: usize,
        length: usize,
        sample_rate: f32,
    ) -> Result<u64, String> {
        if !(1..=32).contains(&channels) {
            return Err("numberOfChannels must be between 1 and 32".to_owned());
        }
        if length == 0 {
            return Err("length must be greater than zero".to_owned());
        }
        if !sample_rate.is_finite() || !(3_000.0..=768_000.0).contains(&sample_rate) {
            return Err("sampleRate is outside the supported range".to_owned());
        }
        self.next_context = self
            .next_context
            .checked_add(1)
            .ok_or("audio context id overflow")?;
        let id = self.next_context;
        let inner = OfflineAudioContext::new(channels, length, sample_rate);
        let listener = inner.listener();
        self.contexts.insert(
            id,
            Context {
                inner: ContextInner::Offline(Arc::new(inner)),
                nodes: HashMap::new(),
                next_node: 0,
                buffers: HashMap::new(),
                next_buffer: 0,
                periodic_waves: HashMap::new(),
                next_periodic_wave: 0,
                rendered: false,
                offline_render_polled: false,
                offline_render: None,
                offline_resume: None,
                offline_suspensions: HashMap::new(),
                offline_suspend_quanta: HashSet::new(),
                next_offline_suspension: 0,
                listener,
                ended_nodes: Arc::new(Mutex::new(Vec::new())),
                processing_requests: Arc::new(Mutex::new(ProcessingQueue::default())),
                processing_responses: HashMap::new(),
                next_processing_event: 0,
                stopped_media_stream_tracks: HashSet::new(),
                worklet: AudioWorkletState::default(),
                worklet_messages: Arc::new(Mutex::new(VecDeque::new())),
            },
        );
        Ok(id)
    }

    pub(crate) fn create_realtime(
        &mut self,
        sample_rate: Option<f32>,
        sink_id: String,
    ) -> Result<(u64, f32, f64, f64, String), String> {
        if sample_rate
            .is_some_and(|rate| !rate.is_finite() || !(3_000.0..=768_000.0).contains(&rate))
        {
            return Err("sampleRate is outside the supported range".to_owned());
        }
        let device_free = sink_id == "none";
        let inner = AudioContext::try_new(AudioContextOptions {
            sample_rate,
            sink_id,
            ..AudioContextOptions::default()
        })
        .map_err(|error| format!("could not create headless AudioContext: {error}"))?;
        if device_free {
            inner.resume_sync();
        }
        let actual_sample_rate = inner.sample_rate();
        let base_latency = inner.base_latency();
        let output_latency = inner.output_latency();
        let sink_id = inner.sink_id();
        let listener = inner.listener();
        self.next_context = self
            .next_context
            .checked_add(1)
            .ok_or("audio context id overflow")?;
        let id = self.next_context;
        self.contexts.insert(
            id,
            Context {
                inner: ContextInner::Online(inner),
                nodes: HashMap::new(),
                next_node: 0,
                buffers: HashMap::new(),
                next_buffer: 0,
                periodic_waves: HashMap::new(),
                next_periodic_wave: 0,
                rendered: false,
                offline_render_polled: false,
                offline_render: None,
                offline_resume: None,
                offline_suspensions: HashMap::new(),
                offline_suspend_quanta: HashSet::new(),
                next_offline_suspension: 0,
                listener,
                ended_nodes: Arc::new(Mutex::new(Vec::new())),
                processing_requests: Arc::new(Mutex::new(ProcessingQueue::default())),
                processing_responses: HashMap::new(),
                next_processing_event: 0,
                stopped_media_stream_tracks: HashSet::new(),
                worklet: AudioWorkletState::default(),
                worklet_messages: Arc::new(Mutex::new(VecDeque::new())),
            },
        );
        Ok((
            id,
            actual_sample_rate,
            base_latency,
            output_latency,
            sink_id,
        ))
    }

    pub(crate) fn create_node(
        &mut self,
        context: u64,
        kind: &str,
        option: f64,
    ) -> Result<u64, String> {
        if !option.is_finite() {
            return Err("audio node option must be finite".to_owned());
        }
        let context = self.context_mut(context)?;
        let node = match kind {
            "oscillator" => Node::Oscillator(context.create_oscillator()),
            "compressor" => Node::Compressor(context.create_compressor()),
            "gain" => Node::Gain(context.create_gain()),
            "biquad" => Node::Biquad(context.create_biquad()),
            "stereo-panner" => Node::StereoPanner(context.create_stereo_panner()),
            "buffer-source" => Node::BufferSource(context.create_buffer_source()),
            "analyser" => Node::Analyser(context.create_analyser()),
            "constant-source" => Node::ConstantSource(context.create_constant_source()),
            "delay" if option > 0.0 => Node::Delay(context.create_delay(option)),
            "delay" => return Err("maxDelayTime must be greater than zero".to_owned()),
            "wave-shaper" => Node::WaveShaper(context.create_wave_shaper()),
            "channel-splitter" if (1.0..=32.0).contains(&option) && option.fract() == 0.0 => {
                Node::ChannelSplitter(context.create_channel_splitter(option as usize))
            }
            "channel-splitter" => {
                return Err("numberOfOutputs must be an integer between 1 and 32".to_owned());
            }
            "channel-merger" if (1.0..=32.0).contains(&option) && option.fract() == 0.0 => {
                Node::ChannelMerger(context.create_channel_merger(option as usize))
            }
            "channel-merger" => {
                return Err("numberOfInputs must be an integer between 1 and 32".to_owned());
            }
            "convolver" => Node::Convolver(context.create_convolver()),
            "panner" => Node::Panner(context.create_panner()),
            "media-stream-destination" => {
                Node::MediaStreamDestination(context.create_media_stream_destination()?)
            }
            _ => return Err("unsupported audio node type".to_owned()),
        };
        context.next_node = context
            .next_node
            .checked_add(1)
            .ok_or("audio node id overflow")?;
        let id = context.next_node;
        let ended_nodes = Arc::clone(&context.ended_nodes);
        let notify_ended = move |_| {
            if let Ok(mut nodes) = ended_nodes.lock() {
                nodes.push(id);
            }
        };
        match &node {
            Node::Oscillator(node) => node.set_onended(notify_ended),
            Node::BufferSource(node) => node.set_onended(notify_ended),
            Node::ConstantSource(node) => node.set_onended(notify_ended),
            _ => {}
        }
        context.nodes.insert(id, node);
        Ok(id)
    }

    pub(crate) fn create_script_processor(
        &mut self,
        context: u64,
        buffer_size: usize,
        input_channels: usize,
        output_channels: usize,
    ) -> Result<(u64, usize), String> {
        if !matches!(
            buffer_size,
            0 | 256 | 512 | 1024 | 2048 | 4096 | 8192 | 16384
        ) {
            return Err(
                "bufferSize must be 0 or one of 256, 512, 1024, 2048, 4096, 8192, or 16384"
                    .to_owned(),
            );
        }
        if input_channels > 32 || output_channels > 32 || input_channels + output_channels == 0 {
            return Err(
                "input and output channels must be between 0 and 32 and cannot both be zero"
                    .to_owned(),
            );
        }
        let context = self.context_mut(context)?;
        let id = context
            .next_node
            .checked_add(1)
            .ok_or("audio node id overflow")?;
        let node = context.create_script_processor(buffer_size, input_channels, output_channels)?;
        let selected_buffer_size = node.buffer_size();
        let requests = Arc::clone(&context.processing_requests);
        node.set_onaudioprocess(move |mut event: AudioProcessingEvent| {
            let (response, receiver) = sync_channel(1);
            let request = ProcessingRequest {
                node: id,
                input: event.input_buffer.clone(),
                output: event.output_buffer.clone(),
                playback_time: event.playback_time,
                response,
            };
            let queued = requests
                .lock()
                .map(|mut queue| {
                    if queue.closed {
                        false
                    } else {
                        queue.requests.push_back(request);
                        true
                    }
                })
                .unwrap_or(false);
            if !queued {
                return;
            }
            let Ok(channels) = receiver.recv_timeout(Duration::from_millis(250)) else {
                return;
            };
            for (index, samples) in channels.into_iter().enumerate() {
                if index >= event.output_buffer.number_of_channels() {
                    break;
                }
                let output = event.output_buffer.get_channel_data_mut(index);
                let length = output.len().min(samples.len());
                output[..length].copy_from_slice(&samples[..length]);
            }
        });
        context.next_node = id;
        context.nodes.insert(id, Node::ScriptProcessor(node));
        Ok((id, selected_buffer_size))
    }

    pub(crate) fn register_worklet_module(
        &mut self,
        context: u64,
        source: String,
    ) -> Result<(), String> {
        let context = self.context_mut(context)?;
        let sample_rate = match &context.inner {
            ContextInner::Offline(inner) => inner.sample_rate(),
            ContextInner::Online(inner) => inner.sample_rate(),
        };
        context.worklet.add_module(source, sample_rate)
    }

    pub(crate) fn create_worklet_node(
        &mut self,
        context: u64,
        name: &str,
        request: WorkletNodeRequest,
    ) -> Result<(u64, Vec<WorkletParamDescriptor>), String> {
        let context = self.context_mut(context)?;
        let created = match &context.inner {
            ContextInner::Offline(inner) => {
                context.worklet.create_node(inner.as_ref(), name, request)?
            }
            ContextInner::Online(inner) => context.worklet.create_node(inner, name, request)?,
        };
        context.next_node = context
            .next_node
            .checked_add(1)
            .ok_or("audio node id overflow")?;
        let id = context.next_node;
        let messages = Arc::clone(&context.worklet_messages);
        created.node.port().set_onmessage(move |message| {
            let Ok(event) = message.downcast::<WorkletRenderEvent>() else {
                return;
            };
            let (kind, data) = match *event {
                WorkletRenderEvent::Message(data) => ("message", data),
                WorkletRenderEvent::ProcessorError(data) => ("processorerror", data),
            };
            if let Ok(mut messages) = messages.lock() {
                messages.push_back(WorkletMessage {
                    node: id,
                    kind,
                    data,
                });
            }
        });
        context.nodes.insert(id, Node::Worklet(created.node));
        Ok((id, created.descriptors))
    }

    pub(crate) fn post_worklet_message(
        &self,
        context: u64,
        node: u64,
        data: String,
    ) -> Result<(), String> {
        let node = self
            .context(context)?
            .nodes
            .get(&node)
            .ok_or("unknown audio node")?;
        let Node::Worklet(node) = node else {
            return Err("audio node is not an AudioWorkletNode".to_owned());
        };
        node.port().post_message(data);
        Ok(())
    }

    pub(crate) fn take_worklet_messages(&self, context: u64) -> Result<String, String> {
        let messages = &self.context(context)?.worklet_messages;
        let mut messages = messages
            .lock()
            .map_err(|_| "AudioWorklet message queue is unavailable")?;
        serde_json::to_string(&messages.drain(..).collect::<Vec<_>>())
            .map_err(|error| error.to_string())
    }

    pub(crate) fn take_processing_events(&mut self, context: u64) -> Result<String, String> {
        let context = self.context_mut(context)?;
        let requests = {
            let mut requests = context
                .processing_requests
                .lock()
                .map_err(|_| "audio processing-event queue is unavailable")?;
            requests.requests.drain(..).collect::<Vec<_>>()
        };
        let mut encoded = Vec::with_capacity(requests.len());
        for request in requests {
            context.next_processing_event = context
                .next_processing_event
                .checked_add(1)
                .ok_or("audio processing-event id overflow")?;
            let event = context.next_processing_event;
            let input = insert_buffer(context, request.input)?;
            let output = insert_buffer(context, request.output)?;
            let input_buffer = context
                .buffers
                .get(&input)
                .expect("processing input buffer was inserted");
            let output_buffer = context
                .buffers
                .get(&output)
                .expect("processing output buffer was inserted");
            encoded.push(serde_json::json!({
                "event": event,
                "node": request.node,
                "playbackTime": request.playback_time,
                "input": {
                    "id": input,
                    "channels": input_buffer.number_of_channels(),
                    "length": input_buffer.length(),
                    "sampleRate": input_buffer.sample_rate(),
                },
                "output": {
                    "id": output,
                    "channels": output_buffer.number_of_channels(),
                    "length": output_buffer.length(),
                    "sampleRate": output_buffer.sample_rate(),
                },
            }));
            context.processing_responses.insert(event, request.response);
        }
        serde_json::to_string(&encoded).map_err(|error| error.to_string())
    }

    pub(crate) fn complete_processing_event(
        &mut self,
        context: u64,
        event: u64,
        output: u64,
    ) -> Result<(), String> {
        let context = self.context_mut(context)?;
        let response = context
            .processing_responses
            .remove(&event)
            .ok_or("unknown audio processing event")?;
        let output = context.buffers.get(&output).ok_or("unknown AudioBuffer")?;
        let channels = (0..output.number_of_channels())
            .map(|channel| output.get_channel_data(channel).to_vec())
            .collect();
        let _ = response.send(channels);
        Ok(())
    }

    pub(crate) fn media_stream_track_state(
        &self,
        context: u64,
        node: u64,
    ) -> Result<&'static str, String> {
        let context = self.context(context)?;
        if !matches!(
            context.nodes.get(&node),
            Some(Node::MediaStreamDestination(_))
        ) {
            return Err("unknown MediaStream audio track".to_owned());
        }
        Ok(if context.stopped_media_stream_tracks.contains(&node) {
            "ended"
        } else {
            "live"
        })
    }

    pub(crate) fn create_media_stream_source(
        &mut self,
        context: u64,
        source_context: u64,
        source_node: u64,
        track_only: bool,
    ) -> Result<u64, String> {
        if context == source_context {
            return Err(
                "media streams must use a distinct AudioContext to avoid blocking its render thread"
                    .to_owned(),
            );
        }
        let stream = {
            let source = self.context(source_context)?;
            let Some(Node::MediaStreamDestination(destination)) = source.nodes.get(&source_node)
            else {
                return Err("unknown MediaStream audio track".to_owned());
            };
            destination.stream().clone()
        };
        let context = self.context_mut(context)?;
        let ContextInner::Online(inner) = &context.inner else {
            return Err("media stream source nodes require an AudioContext".to_owned());
        };
        let node = if track_only {
            Node::MediaStreamTrackSource(
                inner.create_media_stream_track_source(&stream.get_tracks()[0]),
            )
        } else {
            Node::MediaStreamSource(inner.create_media_stream_source(&stream))
        };
        context.next_node = context
            .next_node
            .checked_add(1)
            .ok_or("audio node id overflow")?;
        let id = context.next_node;
        context.nodes.insert(id, node);
        Ok(id)
    }

    pub(crate) fn stop_media_stream_track(
        &mut self,
        context: u64,
        node: u64,
    ) -> Result<(), String> {
        let context = self.context_mut(context)?;
        let Some(Node::MediaStreamDestination(destination)) = context.nodes.get(&node) else {
            return Err("unknown MediaStream audio track".to_owned());
        };
        destination.stream().get_tracks()[0].close();
        context.stopped_media_stream_tracks.insert(node);
        Ok(())
    }

    pub(crate) fn take_ended_nodes(&self, context: u64) -> Result<Vec<u64>, String> {
        let context = self.context(context)?;
        let mut nodes = context
            .ended_nodes
            .lock()
            .map_err(|_| "audio ended-event queue is unavailable")?;
        Ok(std::mem::take(&mut *nodes))
    }

    pub(crate) fn create_iir_filter(
        &mut self,
        context: u64,
        feedforward: Vec<f64>,
        feedback: Vec<f64>,
    ) -> Result<u64, String> {
        if feedforward.is_empty() || feedforward.len() > 20 {
            return Err("feedforward must contain between 1 and 20 coefficients".to_owned());
        }
        if feedback.is_empty() || feedback.len() > 20 {
            return Err("feedback must contain between 1 and 20 coefficients".to_owned());
        }
        if !feedforward
            .iter()
            .chain(&feedback)
            .all(|value| value.is_finite())
        {
            return Err("IIR filter coefficients must be finite".to_owned());
        }
        if feedforward.iter().all(|value| *value == 0.0) {
            return Err("feedforward coefficients cannot all be zero".to_owned());
        }
        if feedback[0] == 0.0 {
            return Err("the first feedback coefficient cannot be zero".to_owned());
        }
        let context = self.context_mut(context)?;
        let node = Node::IirFilter(context.create_iir_filter(feedforward, feedback));
        context.next_node = context
            .next_node
            .checked_add(1)
            .ok_or("audio node id overflow")?;
        let id = context.next_node;
        context.nodes.insert(id, node);
        Ok(id)
    }

    pub(crate) fn connect(
        &mut self,
        context: u64,
        source: u64,
        destination: u64,
        output: usize,
        input: usize,
    ) -> Result<(), String> {
        let context = self.context_mut(context)?;
        let source = context
            .nodes
            .get(&source)
            .ok_or("unknown audio source node")?;
        if destination == 0 {
            source
                .audio_node()
                .connect_from_output_to_input(&context.destination(), output, input);
        } else {
            let destination = context
                .nodes
                .get(&destination)
                .ok_or("unknown audio destination node")?;
            source.audio_node().connect_from_output_to_input(
                destination.audio_node(),
                output,
                input,
            );
        }
        Ok(())
    }

    pub(crate) fn disconnect(
        &mut self,
        context: u64,
        source: u64,
        destination: Option<u64>,
    ) -> Result<(), String> {
        let context = self.context_mut(context)?;
        let source = context
            .nodes
            .get(&source)
            .ok_or("unknown audio source node")?;
        if let Some(destination) = destination {
            if destination == 0 {
                source.audio_node().disconnect_dest(&context.destination());
            } else {
                let destination = context
                    .nodes
                    .get(&destination)
                    .ok_or("unknown audio destination node")?;
                source
                    .audio_node()
                    .disconnect_dest(destination.audio_node());
            }
        } else {
            source.audio_node().disconnect();
        }
        Ok(())
    }

    pub(crate) fn set_param(
        &mut self,
        context: u64,
        node: u64,
        name: &str,
        value: f32,
    ) -> Result<(), String> {
        if !value.is_finite() {
            return Ok(());
        }
        let param = audio_param(self.context(context)?, node, name)?;
        param.set_value(value);
        Ok(())
    }

    pub(crate) fn get_param(&self, context: u64, node: u64, name: &str) -> Result<f32, String> {
        let param = audio_param(self.context(context)?, node, name)?;
        Ok(param.value())
    }

    pub(crate) fn param_automation_rate(
        &self,
        context: u64,
        node: u64,
        name: &str,
    ) -> Result<&'static str, String> {
        let param = audio_param(self.context(context)?, node, name)?;
        Ok(match param.automation_rate() {
            AutomationRate::A => "a-rate",
            AutomationRate::K => "k-rate",
        })
    }

    pub(crate) fn set_param_automation_rate(
        &mut self,
        context: u64,
        node: u64,
        name: &str,
        rate: &str,
    ) -> Result<(), String> {
        let rate = match rate {
            "a-rate" => AutomationRate::A,
            "k-rate" => AutomationRate::K,
            _ => return Err("invalid AudioParam automationRate".to_owned()),
        };
        let context = self.context(context)?;
        if matches!(context.nodes.get(&node), Some(Node::BufferSource(_)))
            && matches!(name, "playbackRate" | "detune")
            && rate != AutomationRate::K
        {
            return Err("AudioBufferSourceNode parameter automationRate is constrained".to_owned());
        }
        audio_param(context, node, name)?.set_automation_rate(rate);
        Ok(())
    }

    pub(crate) fn node_channel_config(
        &self,
        context: u64,
        node: u64,
    ) -> Result<(usize, &'static str, &'static str), String> {
        let context = self.context(context)?;
        if node == 0 {
            let destination = context.destination();
            return Ok(channel_config(&destination));
        }
        Ok(channel_config(
            context
                .nodes
                .get(&node)
                .ok_or("unknown audio node")?
                .audio_node(),
        ))
    }

    pub(crate) fn set_node_channel_count(
        &self,
        context: u64,
        node: u64,
        count: usize,
    ) -> Result<(), String> {
        if !(1..=32).contains(&count) {
            return Err("AudioNode channelCount must be between 1 and 32".to_owned());
        }
        let context = self.context(context)?;
        if node == 0 {
            let destination = context.destination();
            if count > destination.max_channel_count()
                || matches!(context.inner, ContextInner::Offline(_))
                    && count != destination.channel_count()
            {
                return Err("AudioDestinationNode channelCount is constrained".to_owned());
            }
            destination.set_channel_count(count);
            return Ok(());
        }
        let node = context.nodes.get(&node).ok_or("unknown audio node")?;
        let valid = match node {
            Node::ChannelSplitter(_) => count == node.audio_node().channel_count(),
            Node::ChannelMerger(_) => count == 1,
            Node::Compressor(_) | Node::StereoPanner(_) | Node::Convolver(_) | Node::Panner(_) => {
                count <= 2
            }
            _ => true,
        };
        if !valid {
            return Err("AudioNode channelCount is constrained".to_owned());
        }
        node.audio_node().set_channel_count(count);
        Ok(())
    }

    pub(crate) fn set_node_channel_count_mode(
        &self,
        context: u64,
        node: u64,
        mode: &str,
    ) -> Result<(), String> {
        let mode = match mode {
            "max" => ChannelCountMode::Max,
            "clamped-max" => ChannelCountMode::ClampedMax,
            "explicit" => ChannelCountMode::Explicit,
            _ => return Err("invalid AudioNode channelCountMode".to_owned()),
        };
        let context = self.context(context)?;
        if node == 0 {
            let destination = context.destination();
            if matches!(context.inner, ContextInner::Offline(_))
                && mode != ChannelCountMode::Explicit
            {
                return Err(
                    "OfflineAudioContext destination channelCountMode is constrained".to_owned(),
                );
            }
            destination.set_channel_count_mode(mode);
            return Ok(());
        }
        let node = context.nodes.get(&node).ok_or("unknown audio node")?;
        let valid = match node {
            Node::ChannelSplitter(_) | Node::ChannelMerger(_) => mode == ChannelCountMode::Explicit,
            Node::Compressor(_) | Node::StereoPanner(_) | Node::Convolver(_) | Node::Panner(_) => {
                mode != ChannelCountMode::Max
            }
            _ => true,
        };
        if !valid {
            return Err("AudioNode channelCountMode is constrained".to_owned());
        }
        node.audio_node().set_channel_count_mode(mode);
        Ok(())
    }

    pub(crate) fn set_node_channel_interpretation(
        &self,
        context: u64,
        node: u64,
        interpretation: &str,
    ) -> Result<(), String> {
        let interpretation = match interpretation {
            "speakers" => ChannelInterpretation::Speakers,
            "discrete" => ChannelInterpretation::Discrete,
            _ => return Err("invalid AudioNode channelInterpretation".to_owned()),
        };
        let context = self.context(context)?;
        if node == 0 {
            context
                .destination()
                .set_channel_interpretation(interpretation);
            return Ok(());
        }
        let node = context.nodes.get(&node).ok_or("unknown audio node")?;
        if matches!(node, Node::ChannelSplitter(_))
            && interpretation != ChannelInterpretation::Discrete
        {
            return Err("ChannelSplitterNode channelInterpretation is constrained".to_owned());
        }
        node.audio_node().set_channel_interpretation(interpretation);
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn schedule_param(
        &mut self,
        context: u64,
        node: u64,
        name: &str,
        operation: &str,
        value: f32,
        time: f64,
        extra: f64,
    ) -> Result<(), String> {
        if !value.is_finite() || !time.is_finite() || time < 0.0 || !extra.is_finite() {
            return Err(
                "AudioParam automation values must be finite and times non-negative".to_owned(),
            );
        }
        let param = audio_param(self.context(context)?, node, name)?;
        match operation {
            "set" => {
                param.set_value_at_time(value, time);
            }
            "linear" => {
                param.linear_ramp_to_value_at_time(value, time);
            }
            "exponential" if value != 0.0 => {
                param.exponential_ramp_to_value_at_time(value, time);
            }
            "target" if extra >= 0.0 => {
                param.set_target_at_time(value, time, extra);
            }
            "cancel" => {
                param.cancel_scheduled_values(time);
            }
            "hold" => {
                param.cancel_and_hold_at_time(time);
            }
            "exponential" => return Err("exponential ramp value must not be zero".to_owned()),
            "target" => return Err("time constant must be non-negative".to_owned()),
            _ => return Err("unknown AudioParam automation operation".to_owned()),
        }
        Ok(())
    }

    pub(crate) fn schedule_param_curve(
        &mut self,
        context: u64,
        node: u64,
        name: &str,
        values: &[f32],
        start_time: f64,
        duration: f64,
    ) -> Result<(), String> {
        if values.len() < 2 {
            return Err("AudioParam value curves require at least two values".to_owned());
        }
        if values.iter().any(|value| !value.is_finite()) {
            return Err("AudioParam value curves must contain finite values".to_owned());
        }
        if !start_time.is_finite() || start_time < 0.0 {
            return Err("AudioParam curve start time must be finite and non-negative".to_owned());
        }
        if !duration.is_finite() || duration <= 0.0 {
            return Err("AudioParam curve duration must be finite and positive".to_owned());
        }
        audio_param(self.context(context)?, node, name)?
            .set_value_curve_at_time(values, start_time, duration);
        Ok(())
    }

    pub(crate) fn set_oscillator_type(
        &mut self,
        context: u64,
        node: u64,
        oscillator_type: &str,
    ) -> Result<(), String> {
        let node = self
            .context_mut(context)?
            .nodes
            .get_mut(&node)
            .ok_or("unknown audio node")?;
        let Node::Oscillator(node) = node else {
            return Err("node is not an oscillator".to_owned());
        };
        let oscillator_type = match oscillator_type {
            "sine" => OscillatorType::Sine,
            "square" => OscillatorType::Square,
            "sawtooth" => OscillatorType::Sawtooth,
            "triangle" => OscillatorType::Triangle,
            _ => return Err("unsupported oscillator type".to_owned()),
        };
        node.set_type(oscillator_type);
        Ok(())
    }

    pub(crate) fn create_periodic_wave(
        &mut self,
        context: u64,
        real: Vec<f32>,
        imag: Vec<f32>,
        disable_normalization: bool,
    ) -> Result<u64, String> {
        if real.len() != imag.len() || !(2..=8192).contains(&real.len()) {
            return Err(
                "PeriodicWave coefficient arrays must have the same length between 2 and 8192"
                    .to_owned(),
            );
        }
        if !real.iter().chain(&imag).all(|value| value.is_finite()) {
            return Err("PeriodicWave coefficients must be finite".to_owned());
        }
        let context = self.context_mut(context)?;
        let wave = context.create_periodic_wave(PeriodicWaveOptions {
            real: Some(real),
            imag: Some(imag),
            disable_normalization,
        });
        context.next_periodic_wave = context
            .next_periodic_wave
            .checked_add(1)
            .ok_or("periodic wave id overflow")?;
        let id = context.next_periodic_wave;
        context.periodic_waves.insert(id, wave);
        Ok(id)
    }

    pub(crate) fn set_periodic_wave(
        &mut self,
        context: u64,
        node: u64,
        wave: u64,
    ) -> Result<(), String> {
        let context = self.context_mut(context)?;
        let wave = context
            .periodic_waves
            .get(&wave)
            .cloned()
            .ok_or("unknown PeriodicWave")?;
        let node = context.nodes.get_mut(&node).ok_or("unknown audio node")?;
        let Node::Oscillator(node) = node else {
            return Err("node is not an OscillatorNode".to_owned());
        };
        node.set_periodic_wave(wave);
        Ok(())
    }

    pub(crate) fn set_biquad_type(
        &mut self,
        context: u64,
        node: u64,
        filter_type: &str,
    ) -> Result<(), String> {
        let node = self
            .context_mut(context)?
            .nodes
            .get_mut(&node)
            .ok_or("unknown audio node")?;
        let Node::Biquad(node) = node else {
            return Err("node is not a biquad filter".to_owned());
        };
        let filter_type = match filter_type {
            "lowpass" => BiquadFilterType::Lowpass,
            "highpass" => BiquadFilterType::Highpass,
            "bandpass" => BiquadFilterType::Bandpass,
            "lowshelf" => BiquadFilterType::Lowshelf,
            "highshelf" => BiquadFilterType::Highshelf,
            "peaking" => BiquadFilterType::Peaking,
            "notch" => BiquadFilterType::Notch,
            "allpass" => BiquadFilterType::Allpass,
            _ => return Err("unsupported biquad filter type".to_owned()),
        };
        node.set_type(filter_type);
        Ok(())
    }

    pub(crate) fn configure_wave_shaper(
        &mut self,
        context: u64,
        node: u64,
        curve: Option<Vec<f32>>,
        oversample: Option<&str>,
    ) -> Result<(), String> {
        let node = self
            .context_mut(context)?
            .nodes
            .get_mut(&node)
            .ok_or("unknown audio node")?;
        let Node::WaveShaper(node) = node else {
            return Err("node is not a WaveShaperNode".to_owned());
        };
        if let Some(curve) = curve {
            if curve.len() < 2 || !curve.iter().all(|value| value.is_finite()) {
                return Err("WaveShaper curve must contain at least two finite values".to_owned());
            }
            if node.curve().is_some() {
                return Err("WaveShaper curve may only be assigned once".to_owned());
            }
            node.set_curve(curve);
        }
        if let Some(oversample) = oversample {
            let oversample = match oversample {
                "none" => OverSampleType::None,
                "2x" => OverSampleType::X2,
                "4x" => OverSampleType::X4,
                _ => return Err("invalid WaveShaper oversample value".to_owned()),
            };
            node.set_oversample(oversample);
        }
        Ok(())
    }

    pub(crate) fn configure_convolver(
        &mut self,
        context: u64,
        node: u64,
        buffer: Option<u64>,
        normalize: Option<bool>,
    ) -> Result<(), String> {
        let context = self.context_mut(context)?;
        let buffer = buffer
            .map(|id| {
                context
                    .buffers
                    .get(&id)
                    .cloned()
                    .ok_or("unknown AudioBuffer")
            })
            .transpose()?;
        let node = context.nodes.get_mut(&node).ok_or("unknown audio node")?;
        let Node::Convolver(node) = node else {
            return Err("node is not a ConvolverNode".to_owned());
        };
        if let Some(normalize) = normalize {
            node.set_normalize(normalize);
        }
        if let Some(buffer) = buffer {
            if ![1, 2, 4].contains(&buffer.number_of_channels()) {
                return Err("Convolver buffer must have 1, 2, or 4 channels".to_owned());
            }
            if buffer.sample_rate() != node.context().sample_rate() {
                return Err("Convolver buffer sample rate must match its context".to_owned());
            }
            node.set_buffer(buffer);
        }
        Ok(())
    }

    pub(crate) fn set_panner_number(
        &mut self,
        context: u64,
        node: u64,
        property: &str,
        value: f64,
    ) -> Result<(), String> {
        if !value.is_finite() {
            return Err("PannerNode value must be finite".to_owned());
        }
        let node = self
            .context_mut(context)?
            .nodes
            .get_mut(&node)
            .ok_or("unknown audio node")?;
        let Node::Panner(node) = node else {
            return Err("node is not a PannerNode".to_owned());
        };
        match property {
            "refDistance" if value >= 0.0 => node.set_ref_distance(value),
            "maxDistance" if value > 0.0 => node.set_max_distance(value),
            "rolloffFactor" if value >= 0.0 => node.set_rolloff_factor(value),
            "coneInnerAngle" => node.set_cone_inner_angle(value),
            "coneOuterAngle" => node.set_cone_outer_angle(value),
            "coneOuterGain" if (0.0..=1.0).contains(&value) => node.set_cone_outer_gain(value),
            _ => return Err("invalid PannerNode numeric property".to_owned()),
        }
        Ok(())
    }

    pub(crate) fn set_panner_model(
        &mut self,
        context: u64,
        node: u64,
        property: &str,
        value: &str,
    ) -> Result<(), String> {
        let node = self
            .context_mut(context)?
            .nodes
            .get_mut(&node)
            .ok_or("unknown audio node")?;
        let Node::Panner(node) = node else {
            return Err("node is not a PannerNode".to_owned());
        };
        match (property, value) {
            ("panningModel", "equalpower") => node.set_panning_model(PanningModelType::EqualPower),
            ("panningModel", "HRTF") => node.set_panning_model(PanningModelType::HRTF),
            ("distanceModel", "linear") => node.set_distance_model(DistanceModelType::Linear),
            ("distanceModel", "inverse") => node.set_distance_model(DistanceModelType::Inverse),
            ("distanceModel", "exponential") => {
                node.set_distance_model(DistanceModelType::Exponential)
            }
            _ => return Err("invalid PannerNode model".to_owned()),
        }
        Ok(())
    }

    pub(crate) fn iir_frequency_response(
        &self,
        context: u64,
        node: u64,
        frequencies: &[f32],
    ) -> Result<(Vec<f32>, Vec<f32>), String> {
        let node = self
            .context(context)?
            .nodes
            .get(&node)
            .ok_or("unknown audio node")?;
        let Node::IirFilter(node) = node else {
            return Err("node is not an IIRFilterNode".to_owned());
        };
        let mut magnitude = vec![0.0; frequencies.len()];
        let mut phase = vec![0.0; frequencies.len()];
        node.get_frequency_response(frequencies, &mut magnitude, &mut phase);
        Ok((magnitude, phase))
    }

    pub(crate) fn biquad_frequency_response(
        &self,
        context: u64,
        node: u64,
        frequencies: &[f32],
    ) -> Result<(Vec<f32>, Vec<f32>), String> {
        let node = self
            .context(context)?
            .nodes
            .get(&node)
            .ok_or("unknown audio node")?;
        let Node::Biquad(node) = node else {
            return Err("node is not a BiquadFilterNode".to_owned());
        };
        let mut magnitude = vec![0.0; frequencies.len()];
        let mut phase = vec![0.0; frequencies.len()];
        node.get_frequency_response(frequencies, &mut magnitude, &mut phase);
        Ok((magnitude, phase))
    }

    pub(crate) fn set_buffer_source_buffer(
        &mut self,
        context: u64,
        node: u64,
        buffer: Option<u64>,
    ) -> Result<(), String> {
        let context = self.context_mut(context)?;
        let buffer = buffer
            .map(|id| {
                context
                    .buffers
                    .get(&id)
                    .cloned()
                    .ok_or("unknown AudioBuffer")
            })
            .transpose()?;
        let node = context.nodes.get_mut(&node).ok_or("unknown audio node")?;
        let Node::BufferSource(node) = node else {
            return Err("node is not an AudioBufferSourceNode".to_owned());
        };
        if let Some(buffer) = buffer {
            if node.buffer().is_some() {
                return Err("AudioBufferSourceNode buffer may only be assigned once".to_owned());
            }
            node.set_buffer(buffer);
        }
        Ok(())
    }

    pub(crate) fn configure_buffer_source(
        &mut self,
        context: u64,
        node: u64,
        property: &str,
        value: f64,
    ) -> Result<(), String> {
        if !value.is_finite() {
            return Err("AudioBufferSource value must be finite".to_owned());
        }
        let node = self
            .context_mut(context)?
            .nodes
            .get_mut(&node)
            .ok_or("unknown audio node")?;
        let Node::BufferSource(node) = node else {
            return Err("node is not an AudioBufferSourceNode".to_owned());
        };
        match property {
            "loop" => node.set_loop(value != 0.0),
            "loopStart" if value >= 0.0 => node.set_loop_start(value),
            "loopEnd" if value >= 0.0 => node.set_loop_end(value),
            "loopStart" | "loopEnd" => return Err("loop time must be non-negative".to_owned()),
            _ => return Err("unknown AudioBufferSource property".to_owned()),
        }
        Ok(())
    }

    pub(crate) fn start(&mut self, context: u64, node: u64, when: f64) -> Result<(), String> {
        if !when.is_finite() || when < 0.0 {
            return Err("start time must be a finite non-negative number".to_owned());
        }
        let node = self
            .context_mut(context)?
            .nodes
            .get_mut(&node)
            .ok_or("unknown audio node")?;
        match node {
            Node::Oscillator(node) => node.start_at(when),
            Node::BufferSource(node) => node.start_at(when),
            Node::ConstantSource(node) => node.start_at(when),
            _ => return Err("node is not a scheduled source".to_owned()),
        }
        Ok(())
    }

    pub(crate) fn start_buffer_source(
        &mut self,
        context: u64,
        node: u64,
        when: f64,
        offset: f64,
        duration: Option<f64>,
    ) -> Result<(), String> {
        if ![when, offset]
            .iter()
            .all(|value| value.is_finite() && *value >= 0.0)
            || duration.is_some_and(|value| !value.is_finite() || value < 0.0)
        {
            return Err("buffer source times must be finite and non-negative".to_owned());
        }
        let node = self
            .context_mut(context)?
            .nodes
            .get_mut(&node)
            .ok_or("unknown audio node")?;
        let Node::BufferSource(node) = node else {
            return Err("node is not an AudioBufferSourceNode".to_owned());
        };
        if let Some(duration) = duration {
            node.start_at_with_offset_and_duration(when, offset, duration);
        } else {
            node.start_at_with_offset(when, offset);
        }
        Ok(())
    }

    pub(crate) fn stop(&mut self, context: u64, node: u64, when: f64) -> Result<(), String> {
        if !when.is_finite() || when < 0.0 {
            return Err("stop time must be finite and non-negative".to_owned());
        }
        let node = self
            .context_mut(context)?
            .nodes
            .get_mut(&node)
            .ok_or("unknown audio node")?;
        match node {
            Node::Oscillator(node) => node.stop_at(when),
            Node::BufferSource(node) => node.stop_at(when),
            Node::ConstantSource(node) => node.stop_at(when),
            _ => return Err("node is not a scheduled source".to_owned()),
        }
        Ok(())
    }

    pub(crate) fn analyser_settings(&self, context: u64, node: u64) -> Result<[f64; 5], String> {
        let node = self
            .context(context)?
            .nodes
            .get(&node)
            .ok_or("unknown audio node")?;
        let Node::Analyser(node) = node else {
            return Err("node is not an AnalyserNode".to_owned());
        };
        Ok([
            node.fft_size() as f64,
            node.frequency_bin_count() as f64,
            node.min_decibels(),
            node.max_decibels(),
            node.smoothing_time_constant(),
        ])
    }

    pub(crate) fn set_analyser(
        &mut self,
        context: u64,
        node: u64,
        property: &str,
        value: f64,
    ) -> Result<(), String> {
        let node = self
            .context_mut(context)?
            .nodes
            .get_mut(&node)
            .ok_or("unknown audio node")?;
        let Node::Analyser(node) = node else {
            return Err("node is not an AnalyserNode".to_owned());
        };
        match property {
            "fftSize"
                if (32.0..=32768.0).contains(&value) && (value as usize).is_power_of_two() =>
            {
                node.set_fft_size(value as usize)
            }
            "smoothingTimeConstant" if (0.0..=1.0).contains(&value) => {
                node.set_smoothing_time_constant(value)
            }
            "minDecibels" if value.is_finite() && value < node.max_decibels() => {
                node.set_min_decibels(value)
            }
            "maxDecibels" if value.is_finite() && value > node.min_decibels() => {
                node.set_max_decibels(value)
            }
            _ => return Err("invalid AnalyserNode setting".to_owned()),
        }
        Ok(())
    }

    pub(crate) fn analyser_float_data(
        &mut self,
        context: u64,
        node: u64,
        frequency: bool,
        length: usize,
    ) -> Result<Vec<f32>, String> {
        let node = self
            .context_mut(context)?
            .nodes
            .get_mut(&node)
            .ok_or("unknown audio node")?;
        let Node::Analyser(node) = node else {
            return Err("node is not an AnalyserNode".to_owned());
        };
        let mut values = vec![0.0; length];
        if frequency {
            node.get_float_frequency_data(&mut values);
        } else {
            node.get_float_time_domain_data(&mut values);
        }
        Ok(values)
    }

    pub(crate) fn analyser_byte_data(
        &mut self,
        context: u64,
        node: u64,
        frequency: bool,
        length: usize,
    ) -> Result<Vec<u8>, String> {
        let node = self
            .context_mut(context)?
            .nodes
            .get_mut(&node)
            .ok_or("unknown audio node")?;
        let Node::Analyser(node) = node else {
            return Err("node is not an AnalyserNode".to_owned());
        };
        let mut values = vec![0; length];
        if frequency {
            node.get_byte_frequency_data(&mut values);
        } else {
            node.get_byte_time_domain_data(&mut values);
        }
        Ok(values)
    }

    pub(crate) fn begin_offline_render(&mut self, context: u64) -> Result<(), String> {
        let context = self.context_mut(context)?;
        if context.rendered {
            return Err("startRendering may only be called once".to_owned());
        }
        let ContextInner::Offline(inner) = &context.inner else {
            return Err("startRendering is only available on OfflineAudioContext".to_owned());
        };
        let inner = Arc::clone(inner);
        context.offline_render = Some(Box::pin(async move { inner.start_rendering().await }));
        context.offline_render_polled = false;
        context.rendered = true;
        Ok(())
    }

    pub(crate) fn poll_offline_render(
        &mut self,
        context: u64,
    ) -> Result<Option<(u64, usize, usize, f32)>, String> {
        let context = self.context_mut(context)?;
        let mut render = context
            .offline_render
            .take()
            .ok_or("offline rendering is not pending")?;
        context.offline_render_polled = true;
        let mut task_context = TaskContext::from_waker(Waker::noop());
        let Poll::Ready(buffer) = render.as_mut().poll(&mut task_context) else {
            context.offline_render = Some(render);
            return Ok(None);
        };
        let id = insert_buffer(context, buffer)?;
        let buffer = context
            .buffers
            .get(&id)
            .expect("rendered buffer was inserted");
        Ok(Some((
            id,
            buffer.number_of_channels(),
            buffer.length(),
            buffer.sample_rate(),
        )))
    }

    pub(crate) fn schedule_offline_suspend(
        &mut self,
        context: u64,
        suspend_time: f64,
    ) -> Result<u64, String> {
        let context = self.context_mut(context)?;
        let ContextInner::Offline(inner) = &context.inner else {
            return Err("suspend is only available on OfflineAudioContext".to_owned());
        };
        if !suspend_time.is_finite() || suspend_time < 0.0 {
            return Err("suspend time must be non-negative and finite".to_owned());
        }
        let duration = inner.length() as f64 / f64::from(inner.sample_rate());
        if suspend_time >= duration {
            return Err("suspend time must be before the render duration".to_owned());
        }
        if context.offline_render_polled {
            return Err("cannot schedule suspension after offline rendering has begun".to_owned());
        }
        let quantum = (suspend_time * f64::from(inner.sample_rate())
            / AUDIO_RENDER_QUANTUM_SIZE as f64)
            .ceil() as usize;
        if !context.offline_suspend_quanta.insert(quantum) {
            return Err("a suspension is already scheduled for this render quantum".to_owned());
        }
        context.next_offline_suspension = context
            .next_offline_suspension
            .checked_add(1)
            .ok_or("offline suspension id overflow")?;
        let id = context.next_offline_suspension;
        let inner = Arc::clone(inner);
        let mut suspension = Box::pin(async move { inner.suspend(suspend_time).await });
        let mut task_context = TaskContext::from_waker(Waker::noop());
        if suspension.as_mut().poll(&mut task_context).is_ready() {
            return Err("offline suspension resolved before rendering began".to_owned());
        }
        context.offline_suspensions.insert(id, suspension);
        Ok(id)
    }

    pub(crate) fn poll_offline_suspend(
        &mut self,
        context: u64,
        suspension: u64,
    ) -> Result<Option<f64>, String> {
        let context = self.context_mut(context)?;
        let mut future = context
            .offline_suspensions
            .remove(&suspension)
            .ok_or("unknown offline suspension")?;
        let mut task_context = TaskContext::from_waker(Waker::noop());
        if future.as_mut().poll(&mut task_context).is_pending() {
            context.offline_suspensions.insert(suspension, future);
            return Ok(None);
        }
        let ContextInner::Offline(inner) = &context.inner else {
            unreachable!("offline suspension belongs to an offline context")
        };
        Ok(Some(inner.current_time()))
    }

    pub(crate) fn begin_offline_resume(&mut self, context: u64) -> Result<(), String> {
        let context = self.context_mut(context)?;
        let ContextInner::Offline(inner) = &context.inner else {
            return Err("resume is only available on OfflineAudioContext".to_owned());
        };
        if context.offline_resume.is_some() {
            return Err("offline rendering is already resuming".to_owned());
        }
        if !format!("{:?}", inner.state()).eq_ignore_ascii_case("suspended") {
            return Err("offline rendering is not suspended".to_owned());
        }
        let inner = Arc::clone(inner);
        let mut resume = Box::pin(async move { inner.resume().await });
        let mut task_context = TaskContext::from_waker(Waker::noop());
        if resume.as_mut().poll(&mut task_context).is_ready() {
            return Ok(());
        }
        context.offline_resume = Some(resume);
        Ok(())
    }

    pub(crate) fn poll_offline_resume(&mut self, context: u64) -> Result<bool, String> {
        let context = self.context_mut(context)?;
        let Some(mut resume) = context.offline_resume.take() else {
            return Ok(true);
        };
        let mut task_context = TaskContext::from_waker(Waker::noop());
        if resume.as_mut().poll(&mut task_context).is_pending() {
            context.offline_resume = Some(resume);
            return Ok(false);
        }
        Ok(true)
    }

    pub(crate) fn create_buffer(
        &mut self,
        context: u64,
        channels: usize,
        length: usize,
        sample_rate: f32,
    ) -> Result<(u64, usize, usize, f32), String> {
        if !(1..=32).contains(&channels) || length == 0 {
            return Err("invalid AudioBuffer dimensions".to_owned());
        }
        if !sample_rate.is_finite() || !(3_000.0..=768_000.0).contains(&sample_rate) {
            return Err("sampleRate is outside the supported range".to_owned());
        }
        let context = self.context_mut(context)?;
        let id = insert_buffer(
            context,
            AudioBuffer::from(vec![vec![0.0; length]; channels], sample_rate),
        )?;
        Ok((id, channels, length, sample_rate))
    }

    pub(crate) fn decode(
        &mut self,
        context: u64,
        bytes: Vec<u8>,
    ) -> Result<(u64, usize, usize, f32), String> {
        let context = self.context_mut(context)?;
        let buffer = context
            .decode_audio_data(Cursor::new(bytes))
            .map_err(|error| format!("could not decode audio data: {error}"))?;
        let id = insert_buffer(context, buffer)?;
        let buffer = context
            .buffers
            .get(&id)
            .expect("decoded buffer was inserted");
        Ok((
            id,
            buffer.number_of_channels(),
            buffer.length(),
            buffer.sample_rate(),
        ))
    }

    pub(crate) fn channel(
        &self,
        context: u64,
        buffer: u64,
        channel: usize,
    ) -> Result<Vec<f32>, String> {
        let buffer = self
            .context(context)?
            .buffers
            .get(&buffer)
            .ok_or("unknown AudioBuffer")?;
        if channel >= buffer.number_of_channels() {
            return Err("channel index is out of range".to_owned());
        }
        Ok(buffer.get_channel_data(channel).to_vec())
    }

    pub(crate) fn write_channel(
        &mut self,
        context: u64,
        buffer: u64,
        channel: usize,
        offset: usize,
        samples: &[f32],
    ) -> Result<(), String> {
        let buffer = self
            .context_mut(context)?
            .buffers
            .get_mut(&buffer)
            .ok_or("unknown AudioBuffer")?;
        if channel >= buffer.number_of_channels() {
            return Err("channel index is out of range".to_owned());
        }
        buffer.copy_to_channel_with_offset(samples, channel, offset);
        Ok(())
    }

    pub(crate) fn realtime_state(&self, context: u64) -> Result<(String, f64, f64), String> {
        let context = self.context(context)?;
        let ContextInner::Online(inner) = &context.inner else {
            return Err("context is not realtime".to_owned());
        };
        Ok((
            format!("{:?}", inner.state()).to_ascii_lowercase(),
            inner.current_time(),
            inner.output_latency(),
        ))
    }

    pub(crate) fn realtime_playback_stats(
        &self,
        context: u64,
    ) -> Result<(f64, u32, f64, f64, f64, f64), String> {
        let context = self.context(context)?;
        let ContextInner::Online(inner) = &context.inner else {
            return Err("context is not realtime".to_owned());
        };
        let snapshot = inner.playback_stats().to_json();
        Ok((
            snapshot.underrun_duration,
            snapshot.underrun_events.min(u64::from(u32::MAX)) as u32,
            snapshot.total_duration,
            snapshot.average_latency,
            snapshot.minimum_latency,
            snapshot.maximum_latency,
        ))
    }

    pub(crate) fn reset_realtime_playback_latency(&self, context: u64) -> Result<(), String> {
        let context = self.context(context)?;
        let ContextInner::Online(inner) = &context.inner else {
            return Err("context is not realtime".to_owned());
        };
        inner.playback_stats().reset_latency();
        Ok(())
    }

    pub(crate) fn control_realtime(&mut self, context: u64, operation: &str) -> Result<(), String> {
        let context = self.context_mut(context)?;
        if operation == "close" {
            let mut processing = context
                .processing_requests
                .lock()
                .map_err(|_| "audio processing-event queue is unavailable")?;
            processing.closed = true;
            processing.requests.clear();
            context.processing_responses.clear();
        }
        let ContextInner::Online(inner) = &context.inner else {
            return Err("context is not realtime".to_owned());
        };
        match operation {
            "suspend" => inner.suspend_sync(),
            "resume" => inner.resume_sync(),
            "close" => inner.close_sync(),
            _ => return Err("unknown AudioContext lifecycle operation".to_owned()),
        }
        Ok(())
    }

    fn context(&self, id: u64) -> Result<&Context, String> {
        self.contexts
            .get(&id)
            .ok_or_else(|| "unknown audio context".to_owned())
    }

    fn context_mut(&mut self, id: u64) -> Result<&mut Context, String> {
        self.contexts
            .get_mut(&id)
            .ok_or_else(|| "unknown audio context".to_owned())
    }
}

fn insert_buffer(context: &mut Context, buffer: AudioBuffer) -> Result<u64, String> {
    context.next_buffer = context
        .next_buffer
        .checked_add(1)
        .ok_or("audio buffer id overflow")?;
    let id = context.next_buffer;
    context.buffers.insert(id, buffer);
    Ok(id)
}

fn channel_config(node: &dyn AudioNode) -> (usize, &'static str, &'static str) {
    let mode = match node.channel_count_mode() {
        ChannelCountMode::Max => "max",
        ChannelCountMode::ClampedMax => "clamped-max",
        ChannelCountMode::Explicit => "explicit",
    };
    let interpretation = match node.channel_interpretation() {
        ChannelInterpretation::Speakers => "speakers",
        ChannelInterpretation::Discrete => "discrete",
    };
    (node.channel_count(), mode, interpretation)
}

fn audio_param<'a>(
    context: &'a Context,
    node_id: u64,
    name: &str,
) -> Result<&'a AudioParam, String> {
    if node_id == LISTENER_NODE_ID {
        return match name {
            "positionX" => Ok(context.listener.position_x()),
            "positionY" => Ok(context.listener.position_y()),
            "positionZ" => Ok(context.listener.position_z()),
            "forwardX" => Ok(context.listener.forward_x()),
            "forwardY" => Ok(context.listener.forward_y()),
            "forwardZ" => Ok(context.listener.forward_z()),
            "upX" => Ok(context.listener.up_x()),
            "upY" => Ok(context.listener.up_y()),
            "upZ" => Ok(context.listener.up_z()),
            _ => Err("unknown AudioListener parameter".to_owned()),
        };
    }
    let node = context.nodes.get(&node_id).ok_or("unknown audio node")?;
    match (node, name) {
        (Node::Oscillator(node), "frequency") => Ok(node.frequency()),
        (Node::Oscillator(node), "detune") => Ok(node.detune()),
        (Node::Compressor(node), "threshold") => Ok(node.threshold()),
        (Node::Compressor(node), "knee") => Ok(node.knee()),
        (Node::Compressor(node), "ratio") => Ok(node.ratio()),
        (Node::Compressor(node), "attack") => Ok(node.attack()),
        (Node::Compressor(node), "release") => Ok(node.release()),
        (Node::Gain(node), "gain") => Ok(node.gain()),
        (Node::Biquad(node), "frequency") => Ok(node.frequency()),
        (Node::Biquad(node), "detune") => Ok(node.detune()),
        (Node::Biquad(node), "Q") => Ok(node.q()),
        (Node::Biquad(node), "gain") => Ok(node.gain()),
        (Node::StereoPanner(node), "pan") => Ok(node.pan()),
        (Node::BufferSource(node), "playbackRate") => Ok(node.playback_rate()),
        (Node::BufferSource(node), "detune") => Ok(node.detune()),
        (Node::ConstantSource(node), "offset") => Ok(node.offset()),
        (Node::Delay(node), "delayTime") => Ok(node.delay_time()),
        (Node::Panner(node), "positionX") => Ok(node.position_x()),
        (Node::Panner(node), "positionY") => Ok(node.position_y()),
        (Node::Panner(node), "positionZ") => Ok(node.position_z()),
        (Node::Panner(node), "orientationX") => Ok(node.orientation_x()),
        (Node::Panner(node), "orientationY") => Ok(node.orientation_y()),
        (Node::Panner(node), "orientationZ") => Ok(node.orientation_z()),
        (Node::Worklet(node), name) => node
            .parameters()
            .get(name)
            .ok_or_else(|| "unknown AudioWorklet parameter".to_owned()),
        _ => Err("unknown audio parameter".to_owned()),
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc::{TryRecvError, sync_channel};

    use super::{AudioBuffer, AudioStore, Node, ProcessingRequest};

    #[test]
    fn constant_source_renders_through_store() {
        let mut store = AudioStore::default();
        let context = store.create_offline(2, 512, 44_100.0).unwrap();
        let source = store.create_node(context, "constant-source", 0.0).unwrap();
        store.set_param(context, source, "offset", 0.25).unwrap();
        store.connect(context, source, 0, 0, 0).unwrap();
        store.start(context, source, 0.0).unwrap();
        store.begin_offline_render(context).unwrap();
        let (buffer, _, _, _) = store.poll_offline_render(context).unwrap().unwrap();
        assert!(
            store
                .channel(context, buffer, 0)
                .unwrap()
                .iter()
                .any(|value| value.abs() > 0.2)
        );
    }

    #[test]
    fn scheduled_source_ended_events_reach_the_store_queue() {
        let mut store = AudioStore::default();
        let context = store.create_offline(1, 1024, 8_000.0).unwrap();
        let source = store.create_node(context, "oscillator", 0.0).unwrap();
        store.connect(context, source, 0, 0, 0).unwrap();
        store.start(context, source, 0.0).unwrap();
        store.stop(context, source, 0.01).unwrap();
        store.begin_offline_render(context).unwrap();
        store.poll_offline_render(context).unwrap().unwrap();
        assert_eq!(store.take_ended_nodes(context).unwrap(), vec![source]);
    }

    #[test]
    fn media_stream_destination_receives_realtime_graph_samples() {
        let mut store = AudioStore::default();
        let (context, _, _, _, _) = store
            .create_realtime(Some(44_100.0), "none".to_owned())
            .unwrap();
        let destination = store
            .create_node(context, "media-stream-destination", 0.0)
            .unwrap();
        let oscillator = store.create_node(context, "oscillator", 0.0).unwrap();
        store
            .connect(context, oscillator, destination, 0, 0)
            .unwrap();
        store.start(context, oscillator, 0.0).unwrap();

        let track = match store.context(context).unwrap().nodes.get(&destination) {
            Some(Node::MediaStreamDestination(node)) => node.stream().get_tracks()[0].clone(),
            _ => panic!("media stream destination node was not retained"),
        };
        let mut frames = track.iter();
        let received_signal = (0..32).any(|_| {
            frames.next().and_then(Result::ok).is_some_and(|buffer| {
                (0..buffer.number_of_channels()).any(|channel| {
                    buffer
                        .get_channel_data(channel)
                        .iter()
                        .any(|sample| sample.abs() > 0.01)
                })
            })
        });
        assert!(received_signal);

        store.stop_media_stream_track(context, destination).unwrap();
        assert_eq!(
            store
                .media_stream_track_state(context, destination)
                .unwrap(),
            "ended"
        );
        store.control_realtime(context, "close").unwrap();
    }

    #[test]
    fn media_stream_source_routes_samples_between_realtime_contexts() {
        let mut store = AudioStore::default();
        let (producer, _, _, _, _) = store
            .create_realtime(Some(44_100.0), "none".to_owned())
            .unwrap();
        let stream = store
            .create_node(producer, "media-stream-destination", 0.0)
            .unwrap();
        let oscillator = store.create_node(producer, "oscillator", 0.0).unwrap();
        store.connect(producer, oscillator, stream, 0, 0).unwrap();
        store.start(producer, oscillator, 0.0).unwrap();

        let (consumer, _, _, _, _) = store
            .create_realtime(Some(48_000.0), "none".to_owned())
            .unwrap();
        let source = store
            .create_media_stream_source(consumer, producer, stream, false)
            .unwrap();
        let analyser = store.create_node(consumer, "analyser", 0.0).unwrap();
        store.connect(consumer, source, analyser, 0, 0).unwrap();
        store.connect(consumer, analyser, 0, 0, 0).unwrap();

        std::thread::sleep(std::time::Duration::from_millis(50));
        let samples = store
            .analyser_float_data(consumer, analyser, false, 256)
            .unwrap();
        assert!(samples.iter().any(|sample| sample.abs() > 0.01));

        store.control_realtime(consumer, "close").unwrap();
        store.control_realtime(producer, "close").unwrap();
    }

    #[test]
    fn closing_realtime_context_cancels_script_processing_waits() {
        let mut store = AudioStore::default();
        let (context, _, _, _, _) = store
            .create_realtime(Some(48_000.0), "none".to_owned())
            .unwrap();
        let (queued_sender, queued_receiver) = sync_channel(1);
        let (active_sender, active_receiver) = sync_channel(1);
        let buffer = AudioBuffer::from(vec![vec![0.0; 256]], 48_000.0);
        {
            let context = store.context_mut(context).unwrap();
            context
                .processing_requests
                .lock()
                .unwrap()
                .requests
                .push_back(ProcessingRequest {
                    node: 1,
                    input: buffer.clone(),
                    output: buffer,
                    playback_time: 0.0,
                    response: queued_sender,
                });
            context.processing_responses.insert(1, active_sender);
        }

        store.control_realtime(context, "close").unwrap();

        assert_eq!(queued_receiver.try_recv(), Err(TryRecvError::Disconnected));
        assert_eq!(active_receiver.try_recv(), Err(TryRecvError::Disconnected));
        assert!(
            store
                .context(context)
                .unwrap()
                .processing_requests
                .lock()
                .unwrap()
                .closed
        );
    }
}
