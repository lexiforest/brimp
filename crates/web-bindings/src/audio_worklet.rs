use std::{
    cell::RefCell,
    collections::HashMap,
    rc::Rc,
    sync::atomic::{AtomicU64, Ordering},
};

use jsc::{JsRuntime, NativeCall, NativeError, NativeValue, ProtectedJsObject};
use serde::{Deserialize, Serialize};
use web_audio_api::{
    AudioParamDescriptor, AutomationRate,
    context::BaseAudioContext,
    node::{AudioNodeOptions, ChannelCountMode, ChannelInterpretation},
    worklet::{
        AudioParamValues, AudioWorkletGlobalScope, AudioWorkletNode, AudioWorkletNodeOptions,
        AudioWorkletProcessor,
    },
};

const RENDER_QUANTUM_SIZE: usize = 128;
static NEXT_REALM_KEY: AtomicU64 = AtomicU64::new(1);

const WORKLET_BOOTSTRAP: &str = r#"
(() => {
    "use strict";
    const registrations = new Map();
    const instances = new Map();
    let nextInstance = 0;
    const sampleRateValue = Number(globalThis.__brimpWorkletSampleRate);
    delete globalThis.__brimpWorkletSampleRate;

    const DOMExceptionValue = globalThis.DOMException ?? class DOMException extends Error {
        constructor(message = "", name = "Error") {
            super(String(message));
            this.name = String(name);
        }
    };

    class WorkletMessagePort {
        constructor(instance = 0) {
            Object.defineProperties(this, {
                __instance: { value: instance, writable: true },
                onmessage: { value: null, writable: true, enumerable: true },
                onmessageerror: { value: null, writable: true, enumerable: true },
            });
        }
        postMessage(value) {
            const encoded = JSON.stringify(value);
            if (encoded === undefined) throw new DOMException("The value cannot be cloned", "DataCloneError");
            globalThis.__brimpWorkletHost?.("postMessage", this.__instance, encoded);
        }
        start() {}
        close() {}
        addEventListener(type, callback) {
            if (type === "message") this.onmessage = callback;
            else if (type === "messageerror") this.onmessageerror = callback;
        }
        removeEventListener(type, callback) {
            if (type === "message" && this.onmessage === callback) this.onmessage = null;
            else if (type === "messageerror" && this.onmessageerror === callback) this.onmessageerror = null;
        }
    }

    class AudioWorkletProcessor {
        constructor() {
            Object.defineProperty(this, "port", {
                value: new WorkletMessagePort(), enumerable: true,
            });
        }
    }

    function normalizedDescriptors(ctor) {
        const raw = ctor.parameterDescriptors ?? [];
        const descriptors = Array.from(raw, entry => {
            const name = String(entry.name);
            const defaultValue = entry.defaultValue === undefined ? 0 : Number(entry.defaultValue);
            const minValue = entry.minValue === undefined ? -3.4028234663852886e38 : Number(entry.minValue);
            const maxValue = entry.maxValue === undefined ? 3.4028234663852886e38 : Number(entry.maxValue);
            const automationRate = entry.automationRate === undefined ? "a-rate" : String(entry.automationRate);
            if (!name || !Number.isFinite(defaultValue) || !Number.isFinite(minValue) || !Number.isFinite(maxValue)
                || minValue > maxValue || defaultValue < minValue || defaultValue > maxValue
                || (automationRate !== "a-rate" && automationRate !== "k-rate")) {
                throw new TypeError("Invalid AudioParam descriptor");
            }
            return { name, defaultValue, minValue, maxValue, automationRate };
        });
        if (new Set(descriptors.map(value => value.name)).size !== descriptors.length) {
            throw new DOMException("AudioParam descriptor names must be unique", "NotSupportedError");
        }
        return descriptors;
    }

    function registerProcessor(name, ctor) {
        name = String(name);
        if (!name || typeof ctor !== "function" || typeof ctor.prototype?.process !== "function") {
            throw new TypeError("registerProcessor requires a name and processor constructor");
        }
        if (registrations.has(name)) {
            throw new DOMException(`Processor '${name}' is already registered`, "NotSupportedError");
        }
        registrations.set(name, { ctor, descriptors: normalizedDescriptors(ctor) });
    }

    function constructProcessor(name, encodedOptions) {
        const registration = registrations.get(name);
        if (!registration) throw new DOMException(`Unknown AudioWorklet processor '${name}'`, "InvalidStateError");
        const instance = new registration.ctor(JSON.parse(encodedOptions));
        if (!instance || typeof instance.process !== "function") {
            throw new TypeError("AudioWorklet processor did not construct a process() method");
        }
        const id = ++nextInstance;
        instance.port.__instance = id;
        instances.set(id, instance);
        return id;
    }

    function processProcessor(id) {
        const host = globalThis.__brimpWorkletHost;
        const instance = instances.get(id);
        if (!host || !instance) return 0;
        const inputs = [];
        const outputs = [];
        const parameters = Object.create(null);
        for (let input = 0; input < host("inputCount"); ++input) {
            const channels = [];
            for (let channel = 0; channel < host("inputChannels", input); ++channel) {
                channels.push(host("input", input, channel));
            }
            inputs.push(channels);
        }
        for (let output = 0; output < host("outputCount"); ++output) {
            const channels = [];
            for (let channel = 0; channel < host("outputChannels", output); ++channel) {
                channels.push(new Float32Array(128));
            }
            outputs.push(channels);
        }
        for (const name of host("parameterNames").split("\n")) {
            if (name) parameters[name] = host("parameter", name);
        }
        const active = Boolean(instance.process(inputs, outputs, parameters));
        for (let output = 0; output < outputs.length; ++output) {
            for (let channel = 0; channel < outputs[output].length; ++channel) {
                const samples = outputs[output][channel];
                if (!(samples instanceof Float32Array) || samples.length !== 128) {
                    throw new TypeError("AudioWorklet output channels must remain 128-sample Float32Arrays");
                }
                host("output", output, channel, samples);
            }
        }
        return active ? 1 : 0;
    }

    function deliverMessage(id, encoded) {
        const instance = instances.get(id);
        if (!instance) return;
        const event = Object.freeze({ data: JSON.parse(encoded), type: "message" });
        if (typeof instance.port.onmessage === "function") instance.port.onmessage.call(instance.port, event);
    }

    Object.defineProperties(globalThis, {
        AudioWorkletProcessor: { value: AudioWorkletProcessor, writable: true, configurable: true },
        DOMException: { value: DOMExceptionValue, writable: true, configurable: true },
        registerProcessor: { value: registerProcessor, writable: true, configurable: true },
        sampleRate: { value: sampleRateValue, enumerable: true },
        currentFrame: { get: () => globalThis.__brimpWorkletHost?.("currentFrame") ?? 0, enumerable: true },
        currentTime: { get: () => globalThis.__brimpWorkletHost?.("currentTime") ?? 0, enumerable: true },
        __brimpWorkletRegistrations: { value: registrations },
        __brimpWorkletConstruct: { value: constructProcessor },
        __brimpWorkletProcess: { value: processProcessor },
        __brimpWorkletDeliverMessage: { value: deliverMessage },
    });
})();
"#;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct WorkletParamDescriptor {
    pub name: String,
    pub default_value: f32,
    pub min_value: f32,
    pub max_value: f32,
    pub automation_rate: String,
}

impl WorkletParamDescriptor {
    fn native(&self) -> AudioParamDescriptor {
        AudioParamDescriptor {
            name: self.name.clone(),
            automation_rate: if self.automation_rate == "k-rate" {
                AutomationRate::K
            } else {
                AutomationRate::A
            },
            default_value: self.default_value,
            min_value: self.min_value,
            max_value: self.max_value,
        }
    }
}

#[derive(Clone, Debug)]
struct WorkletDefinition {
    descriptors: Vec<WorkletParamDescriptor>,
}

#[derive(Debug)]
pub(super) struct AudioWorkletState {
    realm_key: u64,
    sources: Vec<String>,
    definitions: HashMap<String, WorkletDefinition>,
}

impl Default for AudioWorkletState {
    fn default() -> Self {
        Self {
            realm_key: NEXT_REALM_KEY.fetch_add(1, Ordering::Relaxed),
            sources: Vec::new(),
            definitions: HashMap::new(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct WorkletNodeRequest {
    pub number_of_inputs: usize,
    pub number_of_outputs: usize,
    pub output_channel_count: Vec<usize>,
    pub parameter_data: HashMap<String, f64>,
    pub processor_options: serde_json::Value,
    pub channel_count: Option<usize>,
    pub channel_count_mode: Option<String>,
    pub channel_interpretation: Option<String>,
}

pub(super) struct CreatedWorkletNode {
    pub node: AudioWorkletNode,
    pub descriptors: Vec<WorkletParamDescriptor>,
}

pub(super) enum WorkletRenderEvent {
    Message(String),
    ProcessorError(String),
}

impl AudioWorkletState {
    pub fn add_module(&mut self, source: String, sample_rate: f32) -> Result<(), String> {
        let mut sources = self.sources.clone();
        sources.push(source);
        let definitions = validate_sources(&sources, sample_rate)?;
        self.sources = sources;
        self.definitions = definitions;
        Ok(())
    }

    pub fn create_node(
        &self,
        context: &impl BaseAudioContext,
        name: &str,
        request: WorkletNodeRequest,
    ) -> Result<CreatedWorkletNode, String> {
        let definition = self
            .definitions
            .get(name)
            .ok_or_else(|| format!("AudioWorklet processor '{name}' is not registered"))?;
        validate_node_request(&request)?;

        let mut audio_node_options = AudioNodeOptions::default();
        if let Some(count) = request.channel_count {
            if !(1..=32).contains(&count) {
                return Err("channelCount must be between 1 and 32".to_owned());
            }
            audio_node_options.channel_count = count;
        }
        if let Some(mode) = request.channel_count_mode.as_deref() {
            audio_node_options.channel_count_mode = match mode {
                "max" => ChannelCountMode::Max,
                "clamped-max" => ChannelCountMode::ClampedMax,
                "explicit" => ChannelCountMode::Explicit,
                _ => return Err("invalid channelCountMode".to_owned()),
            };
        }
        if let Some(interpretation) = request.channel_interpretation.as_deref() {
            audio_node_options.channel_interpretation = match interpretation {
                "speakers" => ChannelInterpretation::Speakers,
                "discrete" => ChannelInterpretation::Discrete,
                _ => return Err("invalid channelInterpretation".to_owned()),
            };
        }

        let descriptor_values = definition.descriptors.clone();
        let descriptors = descriptor_values
            .iter()
            .map(WorkletParamDescriptor::native)
            .collect();
        let _descriptor_scope = DescriptorScope::new(descriptors);
        let constructor_options = serde_json::to_string(&request)
            .map_err(|error| format!("could not encode AudioWorklet options: {error}"))?;
        let node = AudioWorkletNode::new::<JsAudioWorkletProcessor>(
            context,
            AudioWorkletNodeOptions {
                number_of_inputs: request.number_of_inputs,
                number_of_outputs: request.number_of_outputs,
                output_channel_count: request.output_channel_count,
                parameter_data: request.parameter_data,
                processor_options: JsProcessorOptions {
                    realm_key: self.realm_key,
                    sources: self.sources.clone(),
                    sample_rate: context.sample_rate(),
                    processor_name: name.to_owned(),
                    constructor_options,
                    parameter_names: descriptor_values
                        .iter()
                        .map(|descriptor| descriptor.name.clone())
                        .collect(),
                },
                audio_node_options,
            },
        );
        Ok(CreatedWorkletNode {
            node,
            descriptors: descriptor_values,
        })
    }
}

fn validate_node_request(request: &WorkletNodeRequest) -> Result<(), String> {
    if request.number_of_inputs > 32 || request.number_of_outputs > 32 {
        return Err("numberOfInputs and numberOfOutputs must not exceed 32".to_owned());
    }
    if request.number_of_inputs == 0 && request.number_of_outputs == 0 {
        return Err("numberOfInputs and numberOfOutputs cannot both be zero".to_owned());
    }
    if !request.output_channel_count.is_empty()
        && request.output_channel_count.len() != request.number_of_outputs
    {
        return Err("outputChannelCount length must equal numberOfOutputs".to_owned());
    }
    if request
        .output_channel_count
        .iter()
        .any(|count| !(1..=32).contains(count))
    {
        return Err("outputChannelCount entries must be between 1 and 32".to_owned());
    }
    if request
        .parameter_data
        .values()
        .any(|value| !value.is_finite())
    {
        return Err("parameterData values must be finite".to_owned());
    }
    Ok(())
}

fn validate_sources(
    sources: &[String],
    sample_rate: f32,
) -> Result<HashMap<String, WorkletDefinition>, String> {
    let runtime = JsRuntime::new().map_err(|error| error.to_string())?;
    runtime
        .eval(&format!(
            "globalThis.__brimpWorkletSampleRate={sample_rate};"
        ))
        .map_err(|error| error.to_string())?;
    runtime
        .eval(WORKLET_BOOTSTRAP)
        .map_err(|error| format!("could not initialize AudioWorkletGlobalScope: {error}"))?;
    for source in sources {
        runtime.eval(source).map_err(|error| {
            format!(
                "AudioWorklet module evaluation failed (ES module import/export syntax is not supported by the current JSC boundary): {error}"
            )
        })?;
    }
    let encoded = runtime
        .eval(
            r#"JSON.stringify(Object.fromEntries([...__brimpWorkletRegistrations].map(([name, value]) => [name, value.descriptors])))"#,
        )
        .and_then(|value| value.to_string())
        .map_err(|error| error.to_string())?;
    let decoded: HashMap<String, Vec<WorkletParamDescriptor>> =
        serde_json::from_str(&encoded).map_err(|error| error.to_string())?;
    Ok(decoded
        .into_iter()
        .map(|(name, descriptors)| (name, WorkletDefinition { descriptors }))
        .collect())
}

thread_local! {
    static NEXT_DESCRIPTORS: RefCell<Option<Vec<AudioParamDescriptor>>> = const { RefCell::new(None) };
    static RENDER_REALMS: RefCell<HashMap<u64, WorkletRealm>> = RefCell::new(HashMap::new());
}

struct DescriptorScope;

impl DescriptorScope {
    fn new(descriptors: Vec<AudioParamDescriptor>) -> Self {
        NEXT_DESCRIPTORS.with(|slot| {
            assert!(
                slot.borrow().is_none(),
                "nested AudioWorkletNode construction"
            );
            *slot.borrow_mut() = Some(descriptors);
        });
        Self
    }
}

impl Drop for DescriptorScope {
    fn drop(&mut self) {
        NEXT_DESCRIPTORS.with(|slot| *slot.borrow_mut() = None);
    }
}

#[derive(Clone)]
struct JsProcessorOptions {
    realm_key: u64,
    sources: Vec<String>,
    sample_rate: f32,
    processor_name: String,
    constructor_options: String,
    parameter_names: Vec<String>,
}

struct JsAudioWorkletProcessor {
    realm_key: u64,
    instance: Option<u64>,
    initialization_error: Option<String>,
    parameter_names: Vec<String>,
}

impl Drop for JsAudioWorkletProcessor {
    fn drop(&mut self) {
        RENDER_REALMS.with(|realms| {
            let mut realms = realms.borrow_mut();
            let remove = realms.get_mut(&self.realm_key).is_some_and(|realm| {
                realm.users = realm.users.saturating_sub(1);
                realm.users == 0
            });
            if remove {
                realms.remove(&self.realm_key);
            }
        });
    }
}

impl AudioWorkletProcessor for JsAudioWorkletProcessor {
    type ProcessorOptions = JsProcessorOptions;

    fn constructor(options: Self::ProcessorOptions) -> Self {
        let initialized = RENDER_REALMS.with(|realms| -> Result<u64, String> {
            let mut realms = realms.borrow_mut();
            if let std::collections::hash_map::Entry::Vacant(entry) =
                realms.entry(options.realm_key)
            {
                let realm = WorkletRealm::new(&options.sources, options.sample_rate)?;
                entry.insert(realm);
            }
            let realm = realms
                .get_mut(&options.realm_key)
                .ok_or_else(|| "AudioWorklet render realm disappeared".to_owned())?;
            realm.ensure_sources(&options.sources)?;
            let instance =
                realm.construct(&options.processor_name, &options.constructor_options)?;
            realm.users += 1;
            Ok(instance)
        });
        Self {
            realm_key: options.realm_key,
            instance: initialized.as_ref().ok().copied(),
            initialization_error: initialized.err(),
            parameter_names: options.parameter_names,
        }
    }

    fn parameter_descriptors() -> Vec<AudioParamDescriptor> {
        NEXT_DESCRIPTORS.with(|slot| slot.borrow().clone().unwrap_or_default())
    }

    fn process<'a, 'b>(
        &mut self,
        inputs: &'b [&'a [&'a [f32]]],
        outputs: &'b mut [&'a mut [&'a mut [f32]]],
        params: AudioParamValues<'b>,
        scope: &'b AudioWorkletGlobalScope,
    ) -> bool {
        if let Some(error) = self.initialization_error.take() {
            scope.post_message(Box::new(WorkletRenderEvent::ProcessorError(error)));
            return false;
        }
        let Some(instance) = self.instance else {
            return false;
        };
        RENDER_REALMS.with(|realms| {
            let mut realms = realms.borrow_mut();
            let Some(realm) = realms.get_mut(&self.realm_key) else {
                return false;
            };
            match realm.process(
                instance,
                &self.parameter_names,
                inputs,
                outputs,
                params,
                scope,
            ) {
                Ok(active) => active,
                Err(error) => {
                    scope.post_message(Box::new(WorkletRenderEvent::ProcessorError(error)));
                    false
                }
            }
        })
    }

    fn onmessage(&mut self, message: &mut dyn std::any::Any) {
        let Some(instance) = self.instance else {
            return;
        };
        let Some(encoded) = message.downcast_ref::<String>() else {
            return;
        };
        RENDER_REALMS.with(|realms| {
            if let Some(realm) = realms.borrow_mut().get_mut(&self.realm_key) {
                let _ = realm.deliver_message(instance, encoded);
            }
        });
    }
}

#[derive(Default)]
struct RenderIo {
    inputs: Vec<Vec<Vec<f32>>>,
    output_channels: Vec<usize>,
    outputs: Vec<Vec<Vec<f32>>>,
    parameters: HashMap<String, Vec<f32>>,
    parameter_names: String,
    current_frame: u64,
    current_time: f64,
    outgoing_messages: Vec<String>,
}

struct WorkletRealm {
    construct: ProtectedJsObject,
    process: ProtectedJsObject,
    deliver_message: ProtectedJsObject,
    io: Rc<RefCell<RenderIo>>,
    loaded_sources: Vec<String>,
    users: usize,
    // Protected JSC objects must be released before their owning runtime.
    runtime: JsRuntime,
}

impl WorkletRealm {
    fn new(sources: &[String], sample_rate: f32) -> Result<Self, String> {
        let runtime = JsRuntime::new().map_err(|error| error.to_string())?;
        let io = Rc::new(RefCell::new(RenderIo::default()));
        let callback_io = Rc::clone(&io);
        runtime
            .set_global_function("__brimpWorkletHost", move |call| {
                worklet_host(&callback_io, call)
            })
            .map_err(|error| error.to_string())?;
        runtime
            .eval(&format!(
                "globalThis.__brimpWorkletSampleRate={sample_rate};"
            ))
            .map_err(|error| error.to_string())?;
        runtime
            .eval(WORKLET_BOOTSTRAP)
            .map_err(|error| error.to_string())?;
        let construct = runtime
            .eval("value => { const [name, options] = JSON.parse(value); return __brimpWorkletConstruct(name, options); }")
            .and_then(|value| value.to_object())
            .map_err(|error| error.to_string())?;
        let process = runtime
            .eval("value => __brimpWorkletProcess(Number(value))")
            .and_then(|value| value.to_object())
            .map_err(|error| error.to_string())?;
        let deliver_message = runtime
            .eval("value => { const [id, message] = JSON.parse(value); __brimpWorkletDeliverMessage(id, message); }")
            .and_then(|value| value.to_object())
            .map_err(|error| error.to_string())?;
        let mut realm = Self {
            construct,
            process,
            deliver_message,
            io,
            loaded_sources: Vec::new(),
            users: 0,
            runtime,
        };
        realm.ensure_sources(sources)?;
        Ok(realm)
    }

    fn ensure_sources(&mut self, sources: &[String]) -> Result<(), String> {
        if !sources.starts_with(&self.loaded_sources) {
            return Err(
                "AudioWorklet module history changed after render initialization".to_owned(),
            );
        }
        for source in &sources[self.loaded_sources.len()..] {
            self.runtime
                .eval(source)
                .map_err(|error| error.to_string())?;
            self.loaded_sources.push(source.clone());
        }
        Ok(())
    }

    fn construct(&self, name: &str, options: &str) -> Result<u64, String> {
        let encoded = serde_json::to_string(&(name, options)).map_err(|error| error.to_string())?;
        self.runtime
            .call_function_with_string(&self.construct, &encoded)
            .and_then(|value| value.to_number())
            .map(|value| value as u64)
            .map_err(|error| error.to_string())
    }

    fn deliver_message(&self, instance: u64, encoded: &str) -> Result<(), String> {
        let argument =
            serde_json::to_string(&(instance, encoded)).map_err(|error| error.to_string())?;
        self.runtime
            .call_function_with_string(&self.deliver_message, &argument)
            .map(|_| ())
            .map_err(|error| error.to_string())
    }

    fn process<'a, 'b>(
        &self,
        instance: u64,
        parameter_names: &[String],
        inputs: &'b [&'a [&'a [f32]]],
        outputs: &'b mut [&'a mut [&'a mut [f32]]],
        params: AudioParamValues<'b>,
        scope: &'b AudioWorkletGlobalScope,
    ) -> Result<bool, String> {
        {
            let mut io = self.io.borrow_mut();
            io.inputs = inputs
                .iter()
                .map(|input| input.iter().map(|channel| channel.to_vec()).collect())
                .collect();
            io.output_channels = outputs.iter().map(|output| output.len()).collect();
            io.outputs = io
                .output_channels
                .iter()
                .map(|channels| vec![vec![0.; RENDER_QUANTUM_SIZE]; *channels])
                .collect();
            io.parameters.clear();
            for name in parameter_names {
                io.parameters
                    .insert(name.clone(), params.get(name).to_vec());
            }
            io.parameter_names = parameter_names.join("\n");
            io.current_frame = scope.current_frame;
            io.current_time = scope.current_time;
            io.outgoing_messages.clear();
        }
        let argument = instance.to_string();
        let active = self
            .runtime
            .call_function_with_string(&self.process, &argument)
            .and_then(|value| value.to_number())
            .map(|value| value != 0.)
            .map_err(|error| error.to_string());
        let mut io = self.io.borrow_mut();
        for (output, rendered) in outputs.iter_mut().zip(&io.outputs) {
            for (channel, samples) in output.iter_mut().zip(rendered) {
                channel.copy_from_slice(samples);
            }
        }
        for message in io.outgoing_messages.drain(..) {
            scope.post_message(Box::new(WorkletRenderEvent::Message(message)));
        }
        active
    }
}

fn worklet_host(
    io: &Rc<RefCell<RenderIo>>,
    call: NativeCall<'_>,
) -> Result<NativeValue, NativeError> {
    let operation = call
        .argument(0)
        .ok_or_else(|| NativeError::new("missing AudioWorklet host operation"))?
        .to_string()?;
    let index = |position: usize, name: &str| -> Result<usize, NativeError> {
        let value = call
            .argument(position)
            .ok_or_else(|| NativeError::new(format!("missing {name}")))?
            .to_number()?;
        if !value.is_finite() || value < 0. || value.fract() != 0. {
            return Err(NativeError::new(format!("invalid {name}")));
        }
        Ok(value as usize)
    };
    match operation.as_str() {
        "inputCount" => Ok(NativeValue::Number(io.borrow().inputs.len() as f64)),
        "inputChannels" => {
            let input = index(1, "input index")?;
            Ok(NativeValue::Number(
                io.borrow().inputs.get(input).map_or(0, Vec::len) as f64,
            ))
        }
        "input" => {
            let input = index(1, "input index")?;
            let channel = index(2, "input channel")?;
            let samples = io
                .borrow()
                .inputs
                .get(input)
                .and_then(|channels| channels.get(channel))
                .cloned()
                .ok_or_else(|| NativeError::new("unknown AudioWorklet input channel"))?;
            Ok(NativeValue::Float32Array(samples))
        }
        "outputCount" => Ok(NativeValue::Number(io.borrow().output_channels.len() as f64)),
        "outputChannels" => {
            let output = index(1, "output index")?;
            Ok(NativeValue::Number(
                io.borrow()
                    .output_channels
                    .get(output)
                    .copied()
                    .unwrap_or(0) as f64,
            ))
        }
        "output" => {
            let output = index(1, "output index")?;
            let channel = index(2, "output channel")?;
            let bytes = call
                .argument(3)
                .ok_or_else(|| NativeError::new("missing AudioWorklet output samples"))?
                .to_bytes()?;
            if bytes.len() != RENDER_QUANTUM_SIZE * size_of::<f32>() {
                return Err(NativeError::new(
                    "AudioWorklet output has an invalid length",
                ));
            }
            let samples = bytes
                .chunks_exact(size_of::<f32>())
                .map(|chunk| f32::from_ne_bytes(chunk.try_into().expect("four-byte chunk")))
                .collect::<Vec<_>>();
            let mut io = io.borrow_mut();
            let destination = io
                .outputs
                .get_mut(output)
                .and_then(|channels| channels.get_mut(channel))
                .ok_or_else(|| NativeError::new("unknown AudioWorklet output channel"))?;
            *destination = samples;
            Ok(NativeValue::Undefined)
        }
        "parameterNames" => Ok(NativeValue::String(io.borrow().parameter_names.clone())),
        "parameter" => {
            let name = call
                .argument(1)
                .ok_or_else(|| NativeError::new("missing AudioWorklet parameter name"))?
                .to_string()?;
            let values = io
                .borrow()
                .parameters
                .get(&name)
                .cloned()
                .ok_or_else(|| NativeError::new("unknown AudioWorklet parameter"))?;
            Ok(NativeValue::Float32Array(values))
        }
        "currentFrame" => Ok(NativeValue::Number(io.borrow().current_frame as f64)),
        "currentTime" => Ok(NativeValue::Number(io.borrow().current_time)),
        "postMessage" => {
            let encoded = call
                .argument(2)
                .ok_or_else(|| NativeError::new("missing AudioWorklet message"))?
                .to_string()?;
            io.borrow_mut().outgoing_messages.push(encoded);
            Ok(NativeValue::Undefined)
        }
        _ => Err(NativeError::new("unknown AudioWorklet host operation")),
    }
}
