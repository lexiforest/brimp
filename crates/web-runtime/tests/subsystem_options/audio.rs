use std::{sync::Arc, time::Duration};

use http::StatusCode;
use network::{HeaderList, NetworkError, ResourceLoader, ResourceRequest, ResourceResponse};
use web_runtime::{Browser, PageOptions};

use super::support::UnusedLoader;

struct MediaLoader;

struct AudioWorkletLoader;

#[async_trait::async_trait]
impl ResourceLoader for AudioWorkletLoader {
    async fn fetch(&self, request: ResourceRequest) -> Result<ResourceResponse, NetworkError> {
        let body = if request.url.ends_with("processor.js") {
            br#"
            class BrimpTestProcessor extends AudioWorkletProcessor {
                static get parameterDescriptors() {
                    return [{ name: "gain", defaultValue: 2, minValue: 0, maxValue: 4, automationRate: "a-rate" }];
                }
                constructor(options) {
                    super();
                    this.bias = options.processorOptions.bias;
                    this.port.onmessage = event => { this.bias = event.data.bias; };
                }
                process(inputs, outputs, parameters) {
                    const input = inputs[0][0];
                    const output = outputs[0][0];
                    for (let index = 0; index < output.length; ++index) {
                        const gain = parameters.gain.length === 1 ? parameters.gain[0] : parameters.gain[index];
                        output[index] = (input?.[index] ?? 0) * gain + this.bias;
                    }
                    if (currentFrame === 0) this.port.postMessage({ sampleRate, currentFrame, bias: this.bias });
                    return true;
                }
            }
            registerProcessor("brimp-test", BrimpTestProcessor);
            registerProcessor("brimp-error", class extends AudioWorkletProcessor {
                process() { throw new Error("processor failure"); }
            });
            "#
            .to_vec()
        } else if request.url.ends_with("module-syntax.js") {
            b"export const unsupported = true;".to_vec()
        } else {
            b"<!doctype html><title>audio worklet</title>".to_vec()
        };
        Ok(ResourceResponse {
            status: StatusCode::OK,
            headers: HeaderList::new(),
            body,
            effective_url: request.url,
        })
    }
}

fn media_test_wav() -> Vec<u8> {
    let frames = 8_000_u32;
    let mut wav = vec![0_u8; 44 + frames as usize * 2];
    wav[0..4].copy_from_slice(b"RIFF");
    wav[4..8].copy_from_slice(&(36 + frames * 2).to_le_bytes());
    wav[8..12].copy_from_slice(b"WAVE");
    wav[12..16].copy_from_slice(b"fmt ");
    wav[16..20].copy_from_slice(&16_u32.to_le_bytes());
    wav[20..22].copy_from_slice(&1_u16.to_le_bytes());
    wav[22..24].copy_from_slice(&1_u16.to_le_bytes());
    wav[24..28].copy_from_slice(&8_000_u32.to_le_bytes());
    wav[28..32].copy_from_slice(&16_000_u32.to_le_bytes());
    wav[32..34].copy_from_slice(&2_u16.to_le_bytes());
    wav[34..36].copy_from_slice(&16_u16.to_le_bytes());
    wav[36..40].copy_from_slice(b"data");
    wav[40..44].copy_from_slice(&(frames * 2).to_le_bytes());
    for frame in 0..frames as usize {
        let sample =
            ((frame as f32 * std::f32::consts::TAU * 440.0 / 8_000.0).sin() * 16_000.0) as i16;
        wav[44 + frame * 2..46 + frame * 2].copy_from_slice(&sample.to_le_bytes());
    }
    wav
}

#[async_trait::async_trait]
impl ResourceLoader for MediaLoader {
    async fn fetch(&self, request: ResourceRequest) -> Result<ResourceResponse, NetworkError> {
        let mut headers = HeaderList::new();
        let body = if request.url.ends_with(".wav") {
            headers.insert(
                http::header::CONTENT_TYPE,
                http::HeaderValue::from_static("audio/wav"),
            );
            if request.url.contains("cors-tone.wav") {
                headers.insert(
                    http::header::ACCESS_CONTROL_ALLOW_ORIGIN,
                    http::HeaderValue::from_static("https://media.test"),
                );
            }
            media_test_wav()
        } else {
            b"<!doctype html><title>media</title>".to_vec()
        };
        Ok(ResourceResponse {
            status: StatusCode::OK,
            headers,
            body,
            effective_url: request.url,
        })
    }
}

#[test]
fn webaudio_option_renders_a_native_offline_graph() {
    let browser = Browser::with_resource_loader(Arc::new(UnusedLoader));
    let page = browser
        .new_page(PageOptions::builder().webaudio(true).build())
        .unwrap();

    page.eval(
        r#"
        globalThis.audioResult = null;
        const context = new OfflineAudioContext(1, 5000, 44100);
        const oscillator = context.createOscillator();
        const filter = context.createBiquadFilter();
        const compressor = context.createDynamicsCompressor();
        const panner = context.createStereoPanner();
        oscillator.type = "triangle";
        const periodicWave = context.createPeriodicWave([0, 0, 0], [0, 1, 0.5]);
        oscillator.setPeriodicWave(periodicWave);
        oscillator.frequency.setValueAtTime(200, 0).linearRampToValueAtTime(1000, 0.05);
        compressor.threshold.value = -50;
        compressor.knee.value = 40;
        compressor.ratio.value = 12;
        filter.type = "highpass";
        filter.frequency.value = 80;
        panner.pan.value = -0.25;
        const responseFrequencies = new Float32Array([100, 1000]);
        const responseMagnitude = new Float32Array(2);
        const responsePhase = new Float32Array(2);
        filter.getFrequencyResponse(responseFrequencies, responseMagnitude, responsePhase);
        oscillator.connect(filter).connect(compressor);
        compressor.connect(context.destination);
        oscillator.start(0);
        oscillator.stop(0.08);
        context.startRendering().then(buffer => {
            const samples = buffer.getChannelData(0);
            audioResult = JSON.stringify({
                constructor: samples.constructor.name,
                channels: buffer.numberOfChannels,
                length: samples.length,
                energy: samples.reduce((sum, value) => sum + Math.abs(value), 0) > 1,
                native: OfflineAudioContext.prototype.startRendering.toString(),
                automation: AudioParam.prototype.linearRampToValueAtTime.toString(),
                filter: filter instanceof BiquadFilterNode && filter.type === "highpass",
                panner: panner instanceof StereoPannerNode && panner.pan.value === -0.25,
                scheduled: oscillator instanceof AudioScheduledSourceNode && oscillator instanceof EventTarget,
                periodic: periodicWave instanceof PeriodicWave && oscillator.type === "custom",
                response: responseMagnitude.every(Number.isFinite) && responsePhase.every(Number.isFinite),
            });
        });
        "scheduled";
        "#,
    )
    .unwrap();

    assert_eq!(
        page.eval("audioResult").unwrap().to_string().unwrap(),
        r#"{"constructor":"Float32Array","channels":1,"length":5000,"energy":true,"native":"function startRendering() { [native code] }","automation":"function linearRampToValueAtTime() { [native code] }","filter":true,"panner":true,"scheduled":true,"periodic":true,"response":true}"#,
    );
}

#[test]
fn webaudio_worklet_runs_javascript_on_the_render_thread() {
    let browser = Browser::with_resource_loader(Arc::new(AudioWorkletLoader));
    let mut page = browser
        .new_page(PageOptions::builder().webaudio(true).build())
        .unwrap();

    page.eval(
        r#"
        globalThis.audioWorkletResult = "pending";
        const workletContext = new OfflineAudioContext(1, 256, 48000);
        workletContext.audioWorklet.addModule("https://audio.test/processor.js").then(async () => {
            let moduleSyntaxRejected = false;
            try {
                await workletContext.audioWorklet.addModule("https://audio.test/module-syntax.js");
            } catch (error) {
                moduleSyntaxRejected = String(error).includes("import/export");
            }
            const source = workletContext.createConstantSource();
            source.offset.value = 0.25;
            const processor = new AudioWorkletNode(workletContext, "brimp-test", {
                parameterData: { gain: 3 },
                processorOptions: { bias: 0.05 },
                outputChannelCount: [1],
            });
            let message = null;
            processor.port.onmessage = event => { message = event.data; };
            processor.port.postMessage({ bias: 0.1 });
            const failing = new AudioWorkletNode(workletContext, "brimp-error", {
                numberOfInputs: 0,
                outputChannelCount: [1],
            });
            let processorError = false;
            failing.onprocessorerror = event => {
                processorError = event.isTrusted && event.target === failing;
            };
            source.connect(processor).connect(workletContext.destination);
            failing.connect(workletContext.destination);
            source.start();
            return workletContext.startRendering().then(buffer => {
                const samples = buffer.getChannelData(0);
                audioWorkletResult = JSON.stringify({
                    shape: workletContext.audioWorklet instanceof AudioWorklet
                        && processor instanceof AudioWorkletNode
                        && processor instanceof AudioNode
                        && processor.numberOfInputs === 1
                        && processor.numberOfOutputs === 1
                        && processor.parameters instanceof Map,
                    parameter: processor.parameters.get("gain") instanceof AudioParam
                        && processor.parameters.get("gain").value === 3
                        && processor.parameters.get("gain").automationRate === "a-rate",
                    signal: samples.every(value => Math.abs(value - 0.85) < 0.0001),
                    message,
                    moduleSyntaxRejected,
                    processorError,
                    native: [
                        AudioWorkletNode.toString(),
                        AudioWorklet.prototype.addModule.toString(),
                        AudioWorkletNode.prototype.constructor.toString(),
                    ],
                });
            });
        }).catch(error => audioWorkletResult = `error:${error?.stack ?? error}`);
        "scheduled";
        "#,
    )
    .unwrap();

    let mut result = "pending".to_owned();
    for _ in 0..200 {
        page.run_pending_tasks().unwrap();
        result = page
            .eval("audioWorkletResult")
            .unwrap()
            .to_string()
            .unwrap();
        if result != "pending" {
            break;
        }
        std::thread::sleep(Duration::from_millis(2));
    }
    assert!(!result.starts_with("error:"), "{result}");
    let result: serde_json::Value = serde_json::from_str(&result).unwrap();
    assert_eq!(result["shape"], true);
    assert_eq!(result["parameter"], true);
    assert_eq!(result["signal"], true, "{result}");
    assert_eq!(result["message"]["sampleRate"], 48_000);
    assert_eq!(result["message"]["currentFrame"], 0);
    assert_eq!(result["message"]["bias"], 0.1);
    assert_eq!(result["moduleSyntaxRejected"], true);
    assert_eq!(result["processorError"], true);
    assert_eq!(
        result["native"],
        serde_json::json!([
            "function AudioWorkletNode() { [native code] }",
            "function addModule() { [native code] }",
            "function AudioWorkletNode() { [native code] }",
        ])
    );
}

#[test]
fn webaudio_value_curves_drive_native_audio_param_automation() {
    let browser = Browser::with_resource_loader(Arc::new(UnusedLoader));
    let page = browser
        .new_page(PageOptions::builder().webaudio(true).build())
        .unwrap();

    page.eval(
        r#"
        globalThis.audioCurveResult = "pending";
        const context = new OfflineAudioContext(1, 4800, 48000);
        const source = context.createConstantSource();
        const gain = context.createGain();
        const curve = new Float32Array([0, 1, 0]);
        const returned = gain.gain.setValueCurveAtTime(curve, 0, 0.1) === gain.gain;
        curve.fill(0.25);
        let shortCurve;
        try {
            gain.gain.setValueCurveAtTime([0], 0.2, 0.1);
            shortCurve = "none";
        } catch (error) {
            shortCurve = error.name;
        }
        let invalidDuration;
        try {
            gain.gain.setValueCurveAtTime([0, 1], 0.2, 0);
            invalidDuration = "none";
        } catch (error) {
            invalidDuration = error.name;
        }
        let invalidValue;
        try {
            gain.gain.setValueCurveAtTime([0, NaN], 0.2, 0.1);
            invalidValue = "none";
        } catch (error) {
            invalidValue = error.name;
        }
        source.connect(gain).connect(context.destination);
        source.start();
        context.startRendering().then(buffer => {
            const samples = buffer.getChannelData(0);
            const rounded = [0, 1200, 2400, 3600, 4799]
                .map(index => Math.round(samples[index] * 100) / 100);
            audioCurveResult = JSON.stringify({
                returned,
                copied: rounded[0] === 0 && rounded[2] === 1 && rounded[4] === 0,
                rounded,
                shortCurve,
                invalidDuration,
                invalidValue,
                native: AudioParam.prototype.setValueCurveAtTime.toString(),
            });
        }).catch(error => audioCurveResult = `error:${error}`);
        "scheduled";
        "#,
    )
    .unwrap();

    assert_eq!(
        page.eval("audioCurveResult").unwrap().to_string().unwrap(),
        r#"{"returned":true,"copied":true,"rounded":[0,0.5,1,0.5,0],"shortCurve":"InvalidStateError","invalidDuration":"RangeError","invalidValue":"TypeError","native":"function setValueCurveAtTime() { [native code] }"}"#,
    );
}

#[test]
fn webaudio_channel_configuration_and_automation_rates_follow_native_graph_state() {
    let browser = Browser::with_resource_loader(Arc::new(UnusedLoader));
    let page = browser
        .new_page(PageOptions::builder().webaudio(true).build())
        .unwrap();

    page.eval(
        r#"
        globalThis.audioChannelConfigResult = "pending";
        const context = new OfflineAudioContext(2, 128, 48000);
        const gain = context.createGain();
        const initialGain = [gain.channelCount, gain.channelCountMode, gain.channelInterpretation, gain.gain.automationRate];
        gain.channelCount = 1;
        gain.channelCountMode = "explicit";
        gain.channelInterpretation = "discrete";
        gain.gain.automationRate = "k-rate";
        gain.gain.value = 0.5;
        const splitter = context.createChannelSplitter(2);
        const merger = context.createChannelMerger(2);
        const bufferSource = context.createBufferSource();
        const compressor = context.createDynamicsCompressor();
        let constrainedRate = false;
        let constrainedMode = false;
        try { bufferSource.playbackRate.automationRate = "a-rate"; } catch (_) { constrainedRate = true; }
        try { compressor.channelCountMode = "max"; } catch (_) { constrainedMode = true; }
        const source = context.createConstantSource();
        source.offset.value = 0.25;
        source.connect(gain).connect(context.destination);
        source.start();
        context.startRendering().then(buffer => {
            audioChannelConfigResult = JSON.stringify({
                initialGain,
                configuredGain: [gain.channelCount, gain.channelCountMode, gain.channelInterpretation, gain.gain.automationRate],
                splitter: [splitter.channelCount, splitter.channelCountMode, splitter.channelInterpretation],
                merger: [merger.channelCount, merger.channelCountMode, merger.channelInterpretation],
                destination: [context.destination.channelCount, context.destination.channelCountMode, context.destination.channelInterpretation],
                sourceRates: [bufferSource.playbackRate.automationRate, bufferSource.detune.automationRate],
                constrainedRate,
                constrainedMode,
                rendered: [buffer.getChannelData(0)[0], buffer.getChannelData(1)[0]],
                native: [
                    Object.getOwnPropertyDescriptor(AudioNode.prototype, "channelCount").set.toString(),
                    Object.getOwnPropertyDescriptor(AudioParam.prototype, "automationRate").set.toString(),
                ],
            });
        }).catch(error => audioChannelConfigResult = `error:${error.name}:${error.message}`);
        "scheduled";
        "#,
    )
    .unwrap();

    let result = page
        .eval("audioChannelConfigResult")
        .unwrap()
        .to_string()
        .unwrap();
    assert_eq!(
        result,
        r#"{"initialGain":[2,"max","speakers","a-rate"],"configuredGain":[1,"explicit","discrete","k-rate"],"splitter":[2,"explicit","discrete"],"merger":[1,"explicit","speakers"],"destination":[2,"explicit","speakers"],"sourceRates":["k-rate","k-rate"],"constrainedRate":true,"constrainedMode":true,"rendered":[0.125,0.125],"native":["function set channelCount() { [native code] }","function set automationRate() { [native code] }"]}"#,
    );
}

#[test]
fn webaudio_decodes_wav_and_exposes_mutable_audio_buffers() {
    let browser = Browser::with_resource_loader(Arc::new(UnusedLoader));
    let page = browser
        .new_page(PageOptions::builder().webaudio(true).build())
        .unwrap();

    page.eval(
        r#"
        globalThis.audioDecodeResult = "pending";
        const context = new OfflineAudioContext(1, 1, 44100);
        const manual = context.createBuffer(1, 4, 44100);
        manual.copyToChannel(new Float32Array([0.25, -0.5, 0.75, -1]), 0);
        const wav = new Uint8Array(60);
        const view = new DataView(wav.buffer);
        const text = (offset, value) => { for (let index = 0; index < value.length; index++) wav[offset + index] = value.charCodeAt(index); };
        text(0, "RIFF"); view.setUint32(4, 52, true); text(8, "WAVE");
        text(12, "fmt "); view.setUint32(16, 16, true); view.setUint16(20, 1, true);
        view.setUint16(22, 1, true); view.setUint32(24, 8000, true); view.setUint32(28, 16000, true);
        view.setUint16(32, 2, true); view.setUint16(34, 16, true); text(36, "data"); view.setUint32(40, 16, true);
        for (let index = 0; index < 8; index++) view.setInt16(44 + index * 2, index % 2 ? -16000 : 16000, true);
        let callbackCalled = false;
        context.decodeAudioData(wav.buffer, () => callbackCalled = true).then(buffer => {
            const decoded = buffer.getChannelData(0);
            audioDecodeResult = JSON.stringify({
                manual: [...manual.getChannelData(0)],
                callbackCalled,
                buffer: buffer instanceof AudioBuffer,
                sampleRate: buffer.sampleRate,
                length: buffer.length > 8,
                energy: decoded.some(value => Math.abs(value) > 0.1),
                native: context.decodeAudioData.toString(),
            });
        }).catch(error => audioDecodeResult = `error:${error}`);
        "scheduled";
        "#,
    )
    .unwrap();

    assert_eq!(
        page.eval("audioDecodeResult").unwrap().to_string().unwrap(),
        r#"{"manual":[0.25,-0.5,0.75,-1],"callbackCalled":true,"buffer":true,"sampleRate":44100,"length":true,"energy":true,"native":"function decodeAudioData() { [native code] }"}"#,
    );
}

#[test]
fn webaudio_scheduled_sources_dispatch_native_ended_events() {
    let browser = Browser::with_resource_loader(Arc::new(UnusedLoader));
    let page = browser
        .new_page(PageOptions::builder().webaudio(true).build())
        .unwrap();

    page.eval(
        r#"
        globalThis.audioEndedResult = "pending";
        const context = new OfflineAudioContext(1, 1024, 8000);
        const oscillator = context.createOscillator();
        const constant = context.createConstantSource();
        const bufferSource = context.createBufferSource();
        bufferSource.buffer = context.createBuffer(1, 16, 8000);
        oscillator.connect(context.destination);
        constant.connect(context.destination);
        bufferSource.connect(context.destination);
        const ended = { oscillator: 0, listener: 0, constant: 0, buffer: 0, trusted: true, target: true };
        let rendered = false;
        const finish = () => {
            if (!rendered || ended.oscillator !== 1 || ended.listener !== 1 || ended.constant !== 1 || ended.buffer !== 1) return;
            audioEndedResult = JSON.stringify({
                ...ended,
                hierarchy: oscillator instanceof AudioScheduledSourceNode && oscillator instanceof EventTarget,
                native: oscillator.addEventListener.toString(),
            });
        };
        oscillator.onended = event => {
            ended.oscillator++;
            ended.trusted &&= event.isTrusted;
            ended.target &&= event.target === oscillator;
            finish();
        };
        oscillator.addEventListener("ended", () => { ended.listener++; finish(); });
        constant.onended = () => { ended.constant++; finish(); };
        bufferSource.onended = () => { ended.buffer++; finish(); };
        oscillator.start(0);
        oscillator.stop(0.01);
        constant.start(0);
        constant.stop(0.02);
        bufferSource.start(0);
        context.startRendering().then(() => {
            rendered = true;
            finish();
        }).catch(error => audioEndedResult = `error:${error.name}:${error.message}`);
        "scheduled";
        "#,
    )
    .unwrap();

    let result = page.eval("audioEndedResult").unwrap().to_string().unwrap();
    assert_eq!(
        result,
        r#"{"oscillator":1,"listener":1,"constant":1,"buffer":1,"trusted":true,"target":true,"hierarchy":true,"native":"function addEventListener() { [native code] }"}"#,
    );
}

#[test]
fn webaudio_offline_lifecycle_dispatches_browser_shaped_events() {
    let browser = Browser::with_resource_loader(Arc::new(UnusedLoader));
    let page = browser
        .new_page(PageOptions::builder().webaudio(true).build())
        .unwrap();

    page.eval(
        r#"
        globalThis.audioLifecycleResult = "pending";
        const lifecycleContext = new OfflineAudioContext(1, 128, 8000);
        let renderedBuffer = null;
        let stateProperty = 0;
        let stateListener = 0;
        let completeProperty = 0;
        let completeListener = 0;
        let completionEvent = null;
        const finishLifecycle = () => {
            if (!renderedBuffer || stateProperty !== 2 || stateListener !== 2 || completeProperty !== 1 || completeListener !== 1) return;
            audioLifecycleResult = JSON.stringify({
                event: completionEvent instanceof OfflineAudioCompletionEvent,
                baseEvent: completionEvent instanceof Event,
                buffer: completionEvent.renderedBuffer === renderedBuffer,
                target: completionEvent.target === lifecycleContext,
                trusted: completionEvent.isTrusted,
                state: lifecycleContext.state,
                time: lifecycleContext.currentTime === lifecycleContext.length / lifecycleContext.sampleRate,
                native: OfflineAudioCompletionEvent.toString(),
            });
        };
        lifecycleContext.onstatechange = event => {
            stateProperty++;
            if (event.target !== lifecycleContext || !event.isTrusted) audioLifecycleResult = "invalid-state-event";
            finishLifecycle();
        };
        lifecycleContext.addEventListener("statechange", () => { stateListener++; finishLifecycle(); });
        lifecycleContext.oncomplete = event => {
            completeProperty++;
            completionEvent = event;
            finishLifecycle();
        };
        lifecycleContext.addEventListener("complete", () => { completeListener++; finishLifecycle(); });
        lifecycleContext.startRendering().then(buffer => { renderedBuffer = buffer; finishLifecycle(); });
        "scheduled";
        "#,
    )
    .unwrap();

    assert_eq!(
        page.eval("audioLifecycleResult")
            .unwrap()
            .to_string()
            .unwrap(),
        r#"{"event":true,"baseEvent":true,"buffer":true,"target":true,"trusted":true,"state":"closed","time":true,"native":"function OfflineAudioCompletionEvent() { [native code] }"}"#,
    );
}

#[test]
fn webaudio_offline_suspension_yields_for_graph_mutation_and_resumes_rendering() {
    let browser = Browser::with_resource_loader(Arc::new(UnusedLoader));
    let mut page = browser
        .new_page(PageOptions::builder().webaudio(true).build())
        .unwrap();

    page.eval(
        r#"
        globalThis.audioSuspendResult = "pending";
        const context = new OfflineAudioContext(1, 512, 48000);
        const suspendTime = 128 / context.sampleRate;
        let stateChanges = 0;
        context.onstatechange = () => stateChanges++;
        let earlyResume;
        context.resume().catch(error => earlyResume = error.name);
        const rendered = context.startRendering();
        const suspended = context.suspend(suspendTime).then(async () => {
            const suspendedState = context.state;
            const suspendedTime = context.currentTime;
            let lateSuspend;
            await context.suspend(suspendTime * 2).catch(error => lateSuspend = error.name);
            const source = context.createConstantSource();
            source.offset.value = 1;
            source.connect(context.destination);
            source.start();
            await context.resume();
            return { suspendedState, suspendedTime, lateSuspend };
        });
        Promise.all([suspended, rendered]).then(([lifecycle, buffer]) => {
            const samples = buffer.getChannelData(0);
            audioSuspendResult = JSON.stringify({
                ...lifecycle,
                earlyResume,
                silentBefore: samples.slice(0, 128).every(value => value === 0),
                signalAfter: samples.slice(128).every(value => Math.abs(value - 1) < 0.0001),
                closed: context.state === "closed",
                finalTime: context.currentTime === context.length / context.sampleRate,
                stateChanges,
                native: [
                    OfflineAudioContext.prototype.suspend.toString(),
                    OfflineAudioContext.prototype.resume.toString(),
                ],
            });
        }).catch(error => audioSuspendResult = `error:${error.name}:${error.message}`);
        "scheduled";
        "#,
    )
    .unwrap();
    page.run_pending_tasks().unwrap();

    assert_eq!(
        page.eval("audioSuspendResult")
            .unwrap()
            .to_string()
            .unwrap(),
        r#"{"suspendedState":"suspended","suspendedTime":0.0026666666666666666,"lateSuspend":"InvalidStateError","earlyResume":"InvalidStateError","silentBefore":true,"signalAfter":true,"closed":true,"finalTime":true,"stateChanges":4,"native":["function suspend() { [native code] }","function resume() { [native code] }"]}"#,
    );
}

#[test]
fn webaudio_realtime_context_uses_the_device_free_sink() {
    let browser = Browser::with_resource_loader(Arc::new(UnusedLoader));
    let page = browser
        .new_page(PageOptions::builder().webaudio(true).build())
        .unwrap();

    page.eval(
        r#"
        globalThis.realtimeAudioResult = "pending";
        const context = new AudioContext({ sampleRate: 44100 });
        const initial = context.state;
        const sink = context.sinkId;
        const playbackStats = context.playbackStats;
        const initialTimestamp = context.getOutputTimestamp();
        const statsSnapshot = playbackStats.toJSON();
        const resetResult = playbackStats.resetLatency();
        const resetSnapshot = playbackStats.toJSON();
        let statsConstructorRejected = false;
        try { new AudioPlaybackStats(); }
        catch (error) { statsConstructorRejected = error instanceof TypeError; }
        let hardwareRejected = false;
        try { new AudioContext({ sinkId: "" }); }
        catch { hardwareRejected = true; }
        let silentStringRejected = false;
        try { new AudioContext({ sinkId: "none" }); }
        catch { silentStringRejected = true; }
        const oscillator = context.createOscillator();
        oscillator.connect(context.destination);
        oscillator.start();
        context.suspend().then(() => {
            const suspended = context.state;
            const suspendedTimestamp = context.getOutputTimestamp();
            const repeatedSuspendedTimestamp = context.getOutputTimestamp();
            return context.resume().then(() => {
                const resumed = context.state;
                return context.close().then(() => {
                    realtimeAudioResult = JSON.stringify({
                        context: context instanceof AudioContext,
                        base: context instanceof BaseAudioContext,
                        sampleRate: context.sampleRate,
                        sink: sink instanceof AudioSinkInfo && sink.type === "none"
                            && context.sinkId === sink,
                        playbackStats: playbackStats instanceof AudioPlaybackStats
                            && context.playbackStats === playbackStats
                            && statsConstructorRejected
                            && resetResult === undefined
                            && Object.keys(statsSnapshot).sort().join(",") === "averageLatency,maximumLatency,minimumLatency,totalDuration,underrunDuration,underrunEvents"
                            && Object.values(statsSnapshot).every(value => Number.isFinite(value) && value >= 0)
                            && resetSnapshot.averageLatency >= 0
                            && resetSnapshot.minimumLatency >= 0
                            && resetSnapshot.maximumLatency >= 0,
                        timestamp: Object.keys(initialTimestamp).sort().join(",") === "contextTime,performanceTime"
                            && initialTimestamp.contextTime >= 0
                            && initialTimestamp.contextTime <= context.currentTime
                            && initialTimestamp.performanceTime >= 0
                            && suspendedTimestamp.contextTime <= context.currentTime
                            && suspendedTimestamp.performanceTime >= 0
                            && repeatedSuspendedTimestamp.contextTime === suspendedTimestamp.contextTime
                            && repeatedSuspendedTimestamp.performanceTime === suspendedTimestamp.performanceTime,
                        hardwareRejected,
                        silentStringRejected,
                        time: context.currentTime >= 0,
                        initial,
                        suspended,
                        resumed,
                        closed: context.state,
                        native: [
                            AudioContext.prototype.resume.toString(),
                            AudioSinkInfo.toString(),
                            AudioPlaybackStats.toString(),
                            AudioPlaybackStats.prototype.resetLatency.toString(),
                            AudioContext.prototype.getOutputTimestamp.toString(),
                            Object.getOwnPropertyDescriptor(AudioPlaybackStats.prototype, "totalDuration").get.toString(),
                        ],
                    });
                });
            });
        }).catch(error => realtimeAudioResult = `error:${error}`);
        "scheduled";
        "#,
    )
    .unwrap();

    assert_eq!(
        page.eval("realtimeAudioResult")
            .unwrap()
            .to_string()
            .unwrap(),
        r#"{"context":true,"base":true,"sampleRate":44100,"sink":true,"playbackStats":true,"timestamp":true,"hardwareRejected":true,"silentStringRejected":true,"time":true,"initial":"running","suspended":"suspended","resumed":"running","closed":"closed","native":["function resume() { [native code] }","function AudioSinkInfo() { [native code] }","function AudioPlaybackStats() { [native code] }","function resetLatency() { [native code] }","function getOutputTimestamp() { [native code] }","function get totalDuration() { [native code] }"]}"#,
    );
}

#[test]
fn webaudio_media_stream_destination_exposes_a_native_live_audio_track() {
    let browser = Browser::with_resource_loader(Arc::new(UnusedLoader));
    let page = browser
        .new_page(PageOptions::builder().webaudio(true).build())
        .unwrap();

    let result = page
        .eval(
            r#"JSON.stringify((() => {
                const context = new AudioContext({ sampleRate: 44100 });
                const destination = context.createMediaStreamDestination();
                const track = destination.stream.getAudioTracks()[0];
                const sameTrack = destination.stream.getTracks()[0] === track
                    && destination.stream.getTrackById(track.id) === track;
                const copy = new MediaStream(destination.stream);
                copy.removeTrack(track);
                const removed = copy.getTracks().length === 0;
                copy.addTrack(track);
                const restored = copy.getTracks().length === 1;
                const constructed = new MediaStreamAudioDestinationNode(context, {
                    channelCount: 1,
                    channelCountMode: "explicit",
                    channelInterpretation: "discrete",
                });
                const oscillator = context.createOscillator();
                oscillator.connect(destination);
                oscillator.start();
                const consumer = new AudioContext({ sampleRate: 48000 });
                const streamSource = consumer.createMediaStreamSource(destination.stream);
                const trackSource = consumer.createMediaStreamTrackSource(track);
                const constructedStreamSource = new MediaStreamAudioSourceNode(consumer, {
                    mediaStream: destination.stream,
                });
                const constructedTrackSource = new MediaStreamTrackAudioSourceNode(consumer, {
                    mediaStreamTrack: track,
                });
                let sameContextRejected = false;
                try { context.createMediaStreamSource(destination.stream); }
                catch { sameContextRejected = true; }
                const beforeStop = {
                    active: destination.stream.active,
                    state: track.readyState,
                    settings: track.getSettings(),
                };
                track.stop();
                const afterStop = {
                    active: destination.stream.active,
                    state: track.readyState,
                };
                consumer.close();
                context.close();
                return {
                    destination: destination instanceof MediaStreamAudioDestinationNode
                        && destination instanceof AudioNode
                        && destination.context === context
                        && destination.numberOfInputs === 1
                        && destination.numberOfOutputs === 0
                        && destination.channelCount === 2
                        && destination.channelCountMode === "explicit",
                    stream: destination.stream instanceof MediaStream
                        && destination.stream.getVideoTracks().length === 0
                        && sameTrack,
                    track: track instanceof MediaStreamTrack
                        && track.kind === "audio"
                        && track.label === "Web Audio destination"
                        && track.enabled
                        && !track.muted
                        && track.id.length > 0,
                    beforeStop,
                    afterStop,
                    mutableStream: removed && restored,
                    constructed: constructed.channelCount === 1
                        && constructed.channelCountMode === "explicit"
                        && constructed.channelInterpretation === "discrete"
                        && constructed.stream.getAudioTracks().length === 1,
                    sources: streamSource instanceof MediaStreamAudioSourceNode
                        && streamSource.mediaStream === destination.stream
                        && streamSource.context === consumer
                        && streamSource.numberOfInputs === 0
                        && streamSource.numberOfOutputs === 1
                        && trackSource instanceof MediaStreamTrackAudioSourceNode
                        && trackSource.mediaStreamTrack === track
                        && constructedStreamSource.mediaStream === destination.stream
                        && constructedTrackSource.mediaStreamTrack === track
                        && sameContextRejected,
                    offlineAbsent: OfflineAudioContext.prototype.createMediaStreamDestination
                        === undefined,
                    native: [
                        AudioContext.prototype.createMediaStreamDestination.toString(),
                        MediaStreamAudioDestinationNode.toString(),
                        AudioContext.prototype.createMediaStreamSource.toString(),
                        MediaStreamAudioSourceNode.toString(),
                        AudioContext.prototype.createMediaStreamTrackSource.toString(),
                        MediaStreamTrackAudioSourceNode.toString(),
                        MediaStream.prototype.getAudioTracks.toString(),
                        MediaStreamTrack.prototype.stop.toString(),
                        Object.getOwnPropertyDescriptor(MediaStreamTrack.prototype, "readyState")
                            .get.toString(),
                    ],
                };
            })())"#,
        )
        .unwrap()
        .to_string()
        .unwrap();

    assert_eq!(
        result,
        r#"{"destination":true,"stream":true,"track":true,"beforeStop":{"active":true,"state":"live","settings":{"channelCount":2,"sampleRate":44100}},"afterStop":{"active":false,"state":"ended"},"mutableStream":true,"constructed":true,"sources":true,"offlineAbsent":true,"native":["function createMediaStreamDestination() { [native code] }","function MediaStreamAudioDestinationNode() { [native code] }","function createMediaStreamSource() { [native code] }","function MediaStreamAudioSourceNode() { [native code] }","function createMediaStreamTrackSource() { [native code] }","function MediaStreamTrackAudioSourceNode() { [native code] }","function getAudioTracks() { [native code] }","function stop() { [native code] }","function get readyState() { [native code] }"]}"#,
    );
}

#[test]
fn webaudio_media_stream_source_routes_audio_between_contexts() {
    let browser = Browser::with_resource_loader(Arc::new(UnusedLoader));
    let mut page = browser
        .new_page(PageOptions::builder().webaudio(true).build())
        .unwrap();

    page.eval(
        r#"
        globalThis.mediaStreamRoutingResult = "pending";
        const mediaProducer = new AudioContext({ sampleRate: 44100 });
        const mediaDestination = mediaProducer.createMediaStreamDestination();
        const mediaOscillator = mediaProducer.createOscillator();
        mediaOscillator.frequency.value = 880;
        mediaOscillator.connect(mediaDestination);
        mediaOscillator.start();

        const mediaConsumer = new AudioContext({ sampleRate: 48000 });
        const mediaSource = mediaConsumer.createMediaStreamSource(mediaDestination.stream);
        const mediaAnalyser = mediaConsumer.createAnalyser();
        mediaAnalyser.fftSize = 256;
        mediaSource.connect(mediaAnalyser).connect(mediaConsumer.destination);
        setTimeout(() => {
            const samples = new Float32Array(mediaAnalyser.fftSize);
            mediaAnalyser.getFloatTimeDomainData(samples);
            mediaStreamRoutingResult = JSON.stringify({
                signal: samples.some(sample => Math.abs(sample) > 0.01),
                producerRate: mediaDestination.stream.getAudioTracks()[0].getSettings().sampleRate,
                consumerRate: mediaConsumer.sampleRate,
                source: mediaSource instanceof MediaStreamAudioSourceNode,
            });
            mediaConsumer.close();
            mediaProducer.close();
        }, 75);
        "scheduled";
        "#,
    )
    .unwrap();

    let mut result = "pending".to_owned();
    for _ in 0..100 {
        std::thread::sleep(Duration::from_millis(5));
        page.run_pending_tasks().unwrap();
        result = page
            .eval("mediaStreamRoutingResult")
            .unwrap()
            .to_string()
            .unwrap();
        if result != "pending" {
            break;
        }
    }
    assert_eq!(
        result,
        r#"{"signal":true,"producerRate":44100,"consumerRate":48000,"source":true}"#,
    );
}

#[test]
fn webaudio_media_element_source_fetches_decodes_and_enforces_cors_silence() {
    let browser = Browser::with_resource_loader(Arc::new(MediaLoader));
    let mut page = browser
        .new_page(PageOptions::builder().webaudio(true).build())
        .unwrap();
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    runtime.block_on(page.goto("https://media.test/")).unwrap();

    page.eval(
        r#"
        globalThis.mediaElementResult = "pending";
        globalThis.mediaElementStages = [];
        const context = new AudioContext({ sampleRate: 48000 });
        const sameOrigin = document.createElement("audio");
        sameOrigin.src = "https://media.test/tone.wav";
        sameOrigin.volume = 0.5;
        const sameSource = context.createMediaElementSource(sameOrigin);
        const analyser = context.createAnalyser();
        analyser.fftSize = 256;
        sameSource.connect(analyser).connect(context.destination);

        const crossOrigin = document.createElement("audio");
        crossOrigin.src = "https://cross.test/tone.wav";
        const crossSource = context.createMediaElementSource(crossOrigin);
        const crossAnalyser = context.createAnalyser();
        crossAnalyser.fftSize = 256;
        crossSource.connect(crossAnalyser).connect(context.destination);

        const corsApproved = document.createElement("audio");
        corsApproved.crossOrigin = "anonymous";
        corsApproved.src = "https://cross.test/cors-tone.wav";
        const corsApprovedSource = context.createMediaElementSource(corsApproved);
        const corsApprovedAnalyser = context.createAnalyser();
        corsApprovedAnalyser.fftSize = 256;
        corsApprovedSource.connect(corsApprovedAnalyser).connect(context.destination);

        let duplicateRejected = false;
        try { context.createMediaElementSource(sameOrigin); }
        catch (error) { duplicateRejected = error.name === "InvalidStateError"; }
        const samePlay = sameOrigin.play().then(
            () => mediaElementStages.push("same-playing"),
            error => { mediaElementStages.push(`same-error:${String(error)}`); throw error; },
        );
        const crossPlay = crossOrigin.play().then(
            () => mediaElementStages.push("cross-playing"),
            error => { mediaElementStages.push(`cross-error:${String(error)}`); throw error; },
        );
        const corsApprovedPlay = corsApproved.play().then(
            () => mediaElementStages.push("cors-approved-playing"),
            error => { mediaElementStages.push(`cors-approved-error:${String(error)}`); throw error; },
        );
        Promise.all([samePlay, crossPlay, corsApprovedPlay]).then(async () => {
            await new Promise(resolve => setTimeout(resolve, 100));
            const samples = new Float32Array(256);
            analyser.getFloatTimeDomainData(samples);

            sameOrigin.muted = true;
            await new Promise(resolve => setTimeout(resolve, 40));
            const mutedSamples = new Float32Array(256);
            analyser.getFloatTimeDomainData(mutedSamples);

            sameOrigin.muted = false;
            const rateStart = sameOrigin.currentTime;
            sameOrigin.playbackRate = 2;
            await new Promise(resolve => setTimeout(resolve, 50));
            const rateDelta = sameOrigin.currentTime - rateStart;
            const unmutedSamples = new Float32Array(256);
            analyser.getFloatTimeDomainData(unmutedSamples);

            sameOrigin.currentTime = 0.2;
            await new Promise(resolve => setTimeout(resolve, 30));
            const timeBeforePause = sameOrigin.currentTime;
            sameOrigin.pause();

            const crossSamples = new Float32Array(256);
            crossAnalyser.getFloatTimeDomainData(crossSamples);
            const corsApprovedSamples = new Float32Array(256);
            corsApprovedAnalyser.getFloatTimeDomainData(corsApprovedSamples);
            mediaElementResult = JSON.stringify({
                node: sameSource instanceof MediaElementAudioSourceNode
                    && sameSource.mediaElement === sameOrigin
                    && sameSource.numberOfInputs === 0
                    && sameSource.numberOfOutputs === 1,
                media: sameOrigin instanceof HTMLMediaElement
                    && sameOrigin.duration > 0.9
                    && sameOrigin.readyState === sameOrigin.HAVE_ENOUGH_DATA
                    && sameOrigin.canPlayType("audio/wav") === "probably"
                    && sameOrigin.paused,
                signal: samples.some(sample => Math.abs(sample) > 0.01),
                mutedSilence: mutedSamples.every(sample => Math.abs(sample) <= 0.000001),
                unmutedSignal: unmutedSamples.some(sample => Math.abs(sample) > 0.01),
                rateAdvanced: rateDelta > 0.07,
                seeked: timeBeforePause >= 0.2 && timeBeforePause < 0.4,
                corsSilence: crossSamples.every(sample => Math.abs(sample) <= 0.000001),
                corsApprovedSignal: corsApprovedSamples.some(sample => Math.abs(sample) > 0.01),
                duplicateRejected,
                native: [
                    AudioContext.prototype.createMediaElementSource.toString(),
                    MediaElementAudioSourceNode.toString(),
                    HTMLMediaElement.prototype.play.toString(),
                    Object.getOwnPropertyDescriptor(HTMLMediaElement.prototype, "currentTime")
                        .get.toString(),
                ],
            });
            crossOrigin.pause();
            corsApproved.pause();
            context.close();
        }).catch(error => mediaElementResult = `error:${String(error)}:${error?.stack ?? ""}`);
        "scheduled";
        "#,
    )
    .unwrap();

    let mut result = "pending".to_owned();
    for _ in 0..400 {
        std::thread::sleep(Duration::from_millis(5));
        page.run_pending_tasks().unwrap();
        result = page
            .eval("mediaElementResult")
            .unwrap()
            .to_string()
            .unwrap();
        if result != "pending" {
            break;
        }
    }
    let diagnostic = page
        .eval(
            r#"JSON.stringify({
                result: mediaElementResult,
                stages: mediaElementStages,
                sameReady: sameOrigin.readyState,
                crossReady: crossOrigin.readyState,
                corsApprovedReady: corsApproved.readyState,
                samePaused: sameOrigin.paused,
                crossPaused: crossOrigin.paused,
            })"#,
        )
        .unwrap()
        .to_string()
        .unwrap();
    assert_eq!(
        result,
        r#"{"node":true,"media":true,"signal":true,"mutedSilence":true,"unmutedSignal":true,"rateAdvanced":true,"seeked":true,"corsSilence":true,"corsApprovedSignal":true,"duplicateRejected":true,"native":["function createMediaElementSource() { [native code] }","function MediaElementAudioSourceNode() { [native code] }","function play() { [native code] }","function get currentTime() { [native code] }"]}"#,
        "{diagnostic}",
    );
}

#[test]
fn webaudio_hardware_output_is_separately_page_scoped() {
    let browser = Browser::with_resource_loader(Arc::new(UnusedLoader));
    let page = browser
        .new_page(PageOptions::builder().webaudio_output(true).build())
        .unwrap();

    let result = page
        .eval(
            r#"JSON.stringify((() => {
                const option = typeof AudioContext === "function" && typeof AudioSinkInfo === "function";
                try {
                    const context = new AudioContext();
                    return {
                        option,
                        available: true,
                        defaultSink: context.sinkId === "",
                        latency: Number.isFinite(context.baseLatency) && Number.isFinite(context.outputLatency),
                    };
                } catch (error) {
                    return { option, available: false, error: String(error).length > 0 };
                }
            })())"#,
        )
        .unwrap()
        .to_string()
        .unwrap();
    let result: serde_json::Value = serde_json::from_str(&result).unwrap();
    assert_eq!(result["option"], true);
    if result["available"] == true {
        assert_eq!(result["defaultSink"], true);
        assert_eq!(result["latency"], true);
    } else {
        assert_eq!(result["error"], true);
    }
}

#[test]
fn webaudio_realtime_sources_dispatch_ended_events() {
    let browser = Browser::with_resource_loader(Arc::new(UnusedLoader));
    let mut page = browser
        .new_page(PageOptions::builder().webaudio(true).build())
        .unwrap();

    page.eval(
        r#"
        globalThis.realtimeEndedResult = "pending";
        const realtimeEndedContext = new AudioContext({ sampleRate: 8000 });
        const realtimeEndedOscillator = realtimeEndedContext.createOscillator();
        realtimeEndedOscillator.connect(realtimeEndedContext.destination);
        realtimeEndedOscillator.onended = event => {
            realtimeEndedResult = JSON.stringify({
                event: event.type,
                trusted: event.isTrusted,
                target: event.target === realtimeEndedOscillator,
            });
            realtimeEndedContext.close();
        };
        realtimeEndedOscillator.start();
        realtimeEndedOscillator.stop(realtimeEndedContext.currentTime + 0.01);
        "scheduled";
        "#,
    )
    .unwrap();

    let mut result = "pending".to_owned();
    for _ in 0..100 {
        std::thread::sleep(Duration::from_millis(5));
        page.run_pending_tasks().unwrap();
        result = page
            .eval("realtimeEndedResult")
            .unwrap()
            .to_string()
            .unwrap();
        if result != "pending" {
            break;
        }
    }
    assert_eq!(result, r#"{"event":"ended","trusted":true,"target":true}"#,);
}

#[test]
fn webaudio_buffer_sources_feed_native_analyser_data() {
    let browser = Browser::with_resource_loader(Arc::new(UnusedLoader));
    let page = browser
        .new_page(PageOptions::builder().webaudio(true).build())
        .unwrap();

    page.eval(
        r#"
        globalThis.analyserResult = "pending";
        const context = new OfflineAudioContext(1, 1024, 44100);
        const buffer = context.createBuffer(1, 1024, 44100);
        const samples = new Float32Array(1024);
        for (let index = 0; index < samples.length; index++) samples[index] = Math.sin(index * Math.PI / 8);
        buffer.copyToChannel(samples, 0);
        const source = context.createBufferSource();
        source.buffer = buffer;
        const analyser = context.createAnalyser();
        analyser.fftSize = 256;
        source.connect(analyser).connect(context.destination);
        source.start();
        context.startRendering().then(() => {
            const time = new Float32Array(analyser.fftSize);
            const frequency = new Uint8Array(analyser.frequencyBinCount);
            analyser.getFloatTimeDomainData(time);
            analyser.getByteFrequencyData(frequency);
            analyserResult = JSON.stringify({
                source: source instanceof AudioBufferSourceNode,
                analyser: analyser instanceof AnalyserNode,
                bins: analyser.frequencyBinCount,
                timeEnergy: time.some(value => Math.abs(value) > 0.1),
                frequencyEnergy: frequency.some(value => value > 0),
                native: analyser.getFloatTimeDomainData.toString(),
            });
        }).catch(error => analyserResult = `error:${error}`);
        "scheduled";
        "#,
    )
    .unwrap();

    assert_eq!(
        page.eval("analyserResult").unwrap().to_string().unwrap(),
        r#"{"source":true,"analyser":true,"bins":128,"timeEnergy":true,"frequencyEnergy":true,"native":"function getFloatTimeDomainData() { [native code] }"}"#,
    );
}

#[test]
fn webaudio_script_processor_dispatches_and_returns_realtime_audio_buffers() {
    let browser = Browser::with_resource_loader(Arc::new(UnusedLoader));
    let mut page = browser
        .new_page(PageOptions::builder().webaudio(true).build())
        .unwrap();

    page.eval(
        r#"
        globalThis.scriptProcessorResult = "pending";
        const scriptContext = new AudioContext({ sampleRate: 48000 });
        const processor = scriptContext.createScriptProcessor(256, 1, 1);
        const automatic = scriptContext.createScriptProcessor(0, 0, 1);
        const source = scriptContext.createConstantSource();
        source.offset.value = 0.25;
        const analyser = scriptContext.createAnalyser();
        analyser.fftSize = 256;
        source.connect(processor).connect(analyser).connect(scriptContext.destination);

        let events = 0;
        let listenerEvents = 0;
        let eventShape = false;
        let inputSample = 0;
        let lastPlaybackTime = -1;
        let orderedPlaybackTime = true;
        let maximumSample = 0;
        processor.onaudioprocess = event => {
            events++;
            eventShape ||= event instanceof AudioProcessingEvent
                && event instanceof Event
                && event.target === processor
                && event.isTrusted
                && event.bubbles
                && event.inputBuffer instanceof AudioBuffer
                && event.outputBuffer instanceof AudioBuffer
                && !Object.hasOwn(event, "playbackTime")
                && !Object.hasOwn(event, "inputBuffer")
                && !Object.hasOwn(event, "outputBuffer")
                && event.inputBuffer.length === 256
                && event.outputBuffer.length === 256
                && event.inputBuffer.numberOfChannels === 1
                && event.outputBuffer.numberOfChannels === 1;
            orderedPlaybackTime &&= event.playbackTime > lastPlaybackTime;
            lastPlaybackTime = event.playbackTime;
            const input = event.inputBuffer.getChannelData(0);
            const output = event.outputBuffer.getChannelData(0);
            inputSample = input[0];
            for (let index = 0; index < output.length; index++) {
                output[index] = input[index] * 2;
            }
        };
        processor.addEventListener("audioprocess", () => listenerEvents++);

        const observeSignal = () => {
            const samples = new Float32Array(analyser.fftSize);
            analyser.getFloatTimeDomainData(samples);
            maximumSample = Math.max(maximumSample, ...samples);
            if (scriptProcessorResult === "pending") setTimeout(observeSignal, 2);
        };
        observeSignal();

        let invalidBufferSize = false;
        let invalidChannels = false;
        let offlineRejected = false;
        let constructorRejected = false;
        let eventConstructorRejected = false;
        try { scriptContext.createScriptProcessor(128, 1, 1); }
        catch { invalidBufferSize = true; }
        try { scriptContext.createScriptProcessor(256, 0, 0); }
        catch { invalidChannels = true; }
        try { new OfflineAudioContext(1, 512, 48000).createScriptProcessor(256, 1, 1); }
        catch (error) { offlineRejected = error.name === "NotSupportedError"; }
        try { new ScriptProcessorNode(); }
        catch (error) { constructorRejected = error instanceof TypeError; }

        const eventInput = scriptContext.createBuffer(1, 8, 48000);
        const eventOutput = scriptContext.createBuffer(1, 8, 48000);
        const constructedEvent = new AudioProcessingEvent("audioprocess", {
            playbackTime: 1.5,
            inputBuffer: eventInput,
            outputBuffer: eventOutput,
        });
        try {
            new AudioProcessingEvent("audioprocess", {
                inputBuffer: eventInput,
                outputBuffer: eventOutput,
            });
        } catch (error) { eventConstructorRejected = error instanceof TypeError; }
        source.start();
        setTimeout(() => {
            scriptProcessorResult = JSON.stringify({
                node: processor instanceof ScriptProcessorNode
                    && processor instanceof AudioNode
                    && processor.context === scriptContext
                    && processor.bufferSize === 256
                    && processor.numberOfInputs === 1
                    && processor.numberOfOutputs === 1
                    && processor.channelCount === 1
                    && processor.channelCountMode === "explicit"
                    && automatic.bufferSize === 256,
                events,
                listenersMatch: listenerEvents === events,
                eventShape,
                inputSample,
                orderedPlaybackTime,
                signal: maximumSample > 0.45,
                maximumSample,
                validation: invalidBufferSize && invalidChannels
                    && offlineRejected && constructorRejected
                    && eventConstructorRejected,
                constructedEvent: constructedEvent.playbackTime === 1.5
                    && constructedEvent.inputBuffer === eventInput
                    && constructedEvent.outputBuffer === eventOutput,
                native: [
                    scriptContext.createScriptProcessor.toString(),
                    ScriptProcessorNode.toString(),
                    AudioProcessingEvent.toString(),
                    Object.getOwnPropertyDescriptor(
                        ScriptProcessorNode.prototype, "bufferSize").get.toString(),
                    Object.getOwnPropertyDescriptor(
                        AudioProcessingEvent.prototype, "inputBuffer").get.toString(),
                ],
            });
            scriptContext.close();
        }, 120);
        "scheduled";
        "#,
    )
    .unwrap();

    let mut result = "pending".to_owned();
    for _ in 0..100 {
        std::thread::sleep(Duration::from_millis(5));
        page.run_pending_tasks().unwrap();
        result = page
            .eval("scriptProcessorResult")
            .unwrap()
            .to_string()
            .unwrap();
        if result != "pending" {
            break;
        }
    }
    let result: serde_json::Value = serde_json::from_str(&result).unwrap();
    assert_eq!(result["node"], true);
    assert!(result["events"].as_u64().unwrap() > 0);
    assert_eq!(result["listenersMatch"], true);
    assert_eq!(result["eventShape"], true);
    assert!((result["inputSample"].as_f64().unwrap() - 0.25).abs() < 0.001);
    assert_eq!(result["orderedPlaybackTime"], true);
    assert_eq!(result["signal"], true, "{result}");
    assert_eq!(result["validation"], true);
    assert_eq!(result["constructedEvent"], true);
    assert_eq!(
        result["native"],
        serde_json::json!([
            "function createScriptProcessor() { [native code] }",
            "function ScriptProcessorNode() { [native code] }",
            "function AudioProcessingEvent() { [native code] }",
            "function get bufferSize() { [native code] }",
            "function get inputBuffer() { [native code] }",
        ])
    );
}

#[test]
fn webaudio_routes_common_native_processing_nodes() {
    let browser = Browser::with_resource_loader(Arc::new(UnusedLoader));
    let page = browser
        .new_page(PageOptions::builder().webaudio(true).build())
        .unwrap();

    page.eval(
        r#"
        globalThis.audioNodesResult = "pending";
        const context = new OfflineAudioContext(2, 2048, 44100);
        const left = context.createConstantSource();
        const right = context.createConstantSource();
        left.offset.value = 0.25;
        right.offset.value = -0.5;
        const merger = context.createChannelMerger(2);
        const splitter = context.createChannelSplitter(2);
        const output = context.createChannelMerger(2);
        left.connect(merger, 0, 0);
        right.connect(merger, 0, 1);
        merger.connect(splitter);
        splitter.connect(output, 0, 1);
        splitter.connect(output, 1, 0);
        output.connect(context.destination);

        const processingContext = new OfflineAudioContext(1, 2048, 44100);
        const processorSource = processingContext.createConstantSource();
        const shaper = processingContext.createWaveShaper();
        shaper.curve = new Float32Array([-1, -0.5, 0, 0.5, 1]);
        shaper.oversample = "2x";
        const delay = processingContext.createDelay(0.1);
        delay.delayTime.value = 0.005;
        processorSource.connect(shaper).connect(delay).connect(processingContext.destination);

        const apiContext = new OfflineAudioContext(1, 1, 44100);
        const convolver = apiContext.createConvolver();
        convolver.normalize = false;
        const impulse = apiContext.createBuffer(1, 2, 44100);
        impulse.copyToChannel(new Float32Array([1, 0]), 0);
        convolver.buffer = impulse;

        const iir = apiContext.createIIRFilter([0.5, 0.5], [1]);
        const frequencies = new Float32Array([0, 1000, 22050]);
        const magnitude = new Float32Array(3);
        const phase = new Float32Array(3);
        iir.getFrequencyResponse(frequencies, magnitude, phase);

        const spatialContext = new OfflineAudioContext(2, 1024, 44100);
        const spatialSource = spatialContext.createConstantSource();
        const panner = spatialContext.createPanner();
        spatialContext.listener.positionZ.value = 1;
        spatialContext.listener.forwardZ.value = -1;
        panner.positionX.value = 1;
        panner.positionZ.value = -1;
        panner.distanceModel = "linear";
        panner.refDistance = 0.5;
        panner.maxDistance = 10;
        panner.rolloffFactor = 0.5;
        spatialSource.connect(panner).connect(spatialContext.destination);

        left.start();
        right.start();
        processorSource.start();
        spatialSource.start();
        Promise.all([context.startRendering(), processingContext.startRendering(), spatialContext.startRendering()]).then(([buffer, processed, spatial]) => {
            const channel0 = buffer.getChannelData(0);
            const channel1 = buffer.getChannelData(1);
            audioNodesResult = JSON.stringify({
                constant: left instanceof ConstantSourceNode && left.offset.value === 0.25,
                delay: delay instanceof DelayNode && Math.abs(delay.delayTime.value - 0.005) < 0.000001,
                shaper: shaper instanceof WaveShaperNode && shaper.curve.length === 5 && shaper.oversample === "2x",
                routing: splitter instanceof ChannelSplitterNode && splitter.numberOfOutputs === 2 && merger.numberOfInputs === 2,
                convolver: convolver instanceof ConvolverNode && convolver.buffer === impulse && !convolver.normalize,
                iir: iir instanceof IIRFilterNode && Math.abs(magnitude[0] - 1) < 0.0001 && phase.every(Number.isFinite),
                hierarchy: left instanceof AudioScheduledSourceNode && left instanceof EventTarget && context instanceof EventTarget,
                listener: spatialContext.listener instanceof AudioListener && spatialContext.listener.positionZ.value === 1,
                panner: panner instanceof PannerNode && panner.distanceModel === "linear" && panner.refDistance === 0.5,
                channel0: channel0.some(value => Math.abs(value) > 0.4),
                channel1: channel1.some(value => Math.abs(value) > 0.2),
                processed: processed.getChannelData(0).some(value => Math.abs(value) > 0.5),
                spatial: spatial.getChannelData(0).some(value => Math.abs(value) > 0.01) || spatial.getChannelData(1).some(value => Math.abs(value) > 0.01),
                native: IIRFilterNode.prototype.getFrequencyResponse.toString(),
            });
        }).catch(error => audioNodesResult = `error:${error}`);
        "scheduled";
        "#,
    )
    .unwrap();

    assert_eq!(
        page.eval("audioNodesResult").unwrap().to_string().unwrap(),
        r#"{"constant":true,"delay":true,"shaper":true,"routing":true,"convolver":true,"iir":true,"hierarchy":true,"listener":true,"panner":true,"channel0":true,"channel1":true,"processed":true,"spatial":true,"native":"function getFrequencyResponse() { [native code] }"}"#,
    );
}
