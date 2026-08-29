(() => {
"use strict";

const host = globalThis.__brimpAudioHost;
const call = (operation, ...arguments_) => host(operation, globalThis, ...arguments_);
const construct = Symbol("WebAudio construction");
const listenerNode = 9007199254740991;
const hardwareOutputEnabled = call("audioOutputEnabled");
let mediaStreamId = 0;
const mediaElementSources = new WeakMap();

class AudioSinkInfo {
    constructor(token) {
        if (token !== construct) throw new TypeError("Illegal constructor");
        Object.defineProperty(this, "type", { value: "none", enumerable: true });
        Object.freeze(this);
    }
}

class AudioPlaybackStats {
    constructor(token, context) {
        if (token !== construct) throw new TypeError("Illegal constructor");
        Object.defineProperty(this, "__context", { value: context });
    }
    __snapshot() { return JSON.parse(call("audioPlaybackStats", this.__context)); }
    get underrunDuration() { return this.__snapshot().underrunDuration; }
    get underrunEvents() { return this.__snapshot().underrunEvents; }
    get totalDuration() { return this.__snapshot().totalDuration; }
    get averageLatency() { return this.__snapshot().averageLatency; }
    get minimumLatency() { return this.__snapshot().minimumLatency; }
    get maximumLatency() { return this.__snapshot().maximumLatency; }
    resetLatency() { call("audioResetPlaybackLatency", this.__context); }
    toJSON() { return this.__snapshot(); }
}

class MediaStreamTrack extends EventTarget {
    constructor(token, context, node, sampleRate) {
        if (token !== construct) throw new TypeError("Illegal constructor");
        super();
        Object.defineProperties(this, {
            __context: { value: context },
            __node: { value: node },
            __sampleRate: { value: sampleRate },
            __enabled: { value: true, writable: true },
            __contentHint: { value: "", writable: true },
            kind: { value: "audio", enumerable: true },
            id: { value: `brimp-audio-${context}-${node}`, enumerable: true },
            label: { value: "Web Audio destination", enumerable: true },
            muted: { value: false, enumerable: true },
            onended: { value: null, writable: true, enumerable: true },
            onmute: { value: null, writable: true, enumerable: true },
            onunmute: { value: null, writable: true, enumerable: true },
        });
    }
    get enabled() { return this.__enabled; }
    set enabled(value) { this.__enabled = Boolean(value); }
    get contentHint() { return this.__contentHint; }
    set contentHint(value) {
        value = String(value);
        if (!["", "speech", "speech-recognition", "music"].includes(value)) return;
        this.__contentHint = value;
    }
    get readyState() { return call("audioMediaStreamTrackState", this.__context, this.__node); }
    stop() { call("audioStopMediaStreamTrack", this.__context, this.__node); }
    getCapabilities() { return {}; }
    getConstraints() { return {}; }
    getSettings() {
        return {
            channelCount: JSON.parse(call("audioNodeChannelConfig", this.__context, this.__node)).count,
            sampleRate: this.__sampleRate,
        };
    }
    applyConstraints() { return Promise.resolve(); }
}

class MediaStream extends EventTarget {
    constructor(tracks = []) {
        super();
        let id;
        if (tracks === construct) {
            const track = arguments[1];
            id = arguments[2];
            tracks = [track];
        } else if (tracks instanceof MediaStream) {
            tracks = tracks.getTracks();
        } else {
            tracks = Array.from(tracks);
        }
        if (!tracks.every(track => track instanceof MediaStreamTrack)) {
            throw new TypeError("MediaStream tracks must be MediaStreamTrack instances");
        }
        Object.defineProperties(this, {
            __tracks: { value: [...new Set(tracks)] },
            id: { value: id ?? `brimp-stream-${++mediaStreamId}`, enumerable: true },
            onaddtrack: { value: null, writable: true, enumerable: true },
            onremovetrack: { value: null, writable: true, enumerable: true },
        });
    }
    get active() { return this.__tracks.some(track => track.readyState === "live"); }
    getTracks() { return this.__tracks.slice(); }
    getAudioTracks() { return this.__tracks.filter(track => track.kind === "audio"); }
    getVideoTracks() { return this.__tracks.filter(track => track.kind === "video"); }
    getTrackById(id) { return this.__tracks.find(track => track.id === String(id)) ?? null; }
    addTrack(track) {
        if (!(track instanceof MediaStreamTrack)) throw new TypeError("track must be a MediaStreamTrack");
        if (!this.__tracks.includes(track)) this.__tracks.push(track);
    }
    removeTrack(track) {
        const index = this.__tracks.indexOf(track);
        if (index !== -1) this.__tracks.splice(index, 1);
    }
}

function audioSinkRequest(value, useHardwareDefault = false) {
    if (value === undefined) {
        return useHardwareDefault
            ? { native: "", exposed: "" }
            : { native: "none", exposed: new AudioSinkInfo(construct) };
    }
    if (typeof value === "string") {
        if (value === "none") {
            throw new TypeError("the silent sink must be specified as { type: 'none' }");
        }
        return { native: value, exposed: value };
    }
    if (value !== null && typeof value === "object" && String(value.type) === "none") {
        return { native: "none", exposed: new AudioSinkInfo(construct) };
    }
    throw new TypeError("sinkId must be a device identifier or { type: 'none' }");
}

class AudioParam {
    constructor(token, context, node, name, defaultValue, minValue, maxValue) {
        if (token !== construct) throw new TypeError("Illegal constructor");
        const automationRate = call("audioGetParamAutomationRate", context, node, name);
        Object.defineProperties(this, {
            __context: { value: context }, __node: { value: node }, __name: { value: name },
            __automationRate: { value: automationRate, writable: true },
            defaultValue: { value: defaultValue, enumerable: true },
            minValue: { value: minValue, enumerable: true },
            maxValue: { value: maxValue, enumerable: true },
        });
    }
    get automationRate() { return this.__automationRate; }
    set automationRate(value) {
        value = String(value);
        if (value !== "a-rate" && value !== "k-rate") throw new TypeError("Invalid AudioParam automationRate");
        call("audioSetParamAutomationRate", this.__context, this.__node, this.__name, value);
        this.__automationRate = value;
    }
    get value() { return call("audioGetParam", this.__context, this.__node, this.__name); }
    set value(value) { call("audioSetParam", this.__context, this.__node, this.__name, Number(value)); }
    __schedule(operation, value, time, extra = 0) {
        call("audioScheduleParam", this.__context, this.__node, this.__name, operation, Number(value), Number(time), Number(extra));
        return this;
    }
    setValueAtTime(value, startTime) { return this.__schedule("set", value, startTime); }
    linearRampToValueAtTime(value, endTime) { return this.__schedule("linear", value, endTime); }
    exponentialRampToValueAtTime(value, endTime) { return this.__schedule("exponential", value, endTime); }
    setTargetAtTime(value, startTime, timeConstant) { return this.__schedule("target", value, startTime, timeConstant); }
    setValueCurveAtTime(values, startTime, duration) {
        const curve = Float32Array.from(values ?? [], Number);
        startTime = Number(startTime);
        duration = Number(duration);
        if (curve.length < 2) throw new DOMException("The value curve must contain at least two values", "InvalidStateError");
        if (![...curve].every(Number.isFinite)) throw new TypeError("The value curve must contain finite values");
        if (!Number.isFinite(startTime) || startTime < 0) throw new RangeError("startTime must be finite and non-negative");
        if (!Number.isFinite(duration) || duration <= 0) throw new RangeError("duration must be finite and positive");
        call("audioScheduleParamCurve", this.__context, this.__node, this.__name, curve, startTime, duration);
        return this;
    }
    cancelScheduledValues(cancelTime) { return this.__schedule("cancel", 0, cancelTime); }
    cancelAndHoldAtTime(cancelTime) { return this.__schedule("hold", 0, cancelTime); }
}

class AudioNode extends EventTarget {
    constructor(token, context, id) {
        if (token !== construct) throw new TypeError("Illegal constructor");
        super();
        const channelConfig = JSON.parse(call("audioNodeChannelConfig", context.__id, id));
        Object.defineProperties(this, {
            context: { value: context, enumerable: true },
            __id: { value: id },
            __channelCount: { value: channelConfig.count, writable: true },
            __channelCountMode: { value: channelConfig.mode, writable: true },
            __channelInterpretation: { value: channelConfig.interpretation, writable: true },
        });
    }
    get channelCount() { return this.__channelCount; }
    set channelCount(value) {
        value = Number(value) >>> 0;
        call("audioSetNodeChannelCount", this.context.__id, this.__id, value);
        this.__channelCount = value;
    }
    get channelCountMode() { return this.__channelCountMode; }
    set channelCountMode(value) {
        value = String(value);
        if (!['max', 'clamped-max', 'explicit'].includes(value)) throw new TypeError("Invalid channelCountMode");
        call("audioSetNodeChannelCountMode", this.context.__id, this.__id, value);
        this.__channelCountMode = value;
    }
    get channelInterpretation() { return this.__channelInterpretation; }
    set channelInterpretation(value) {
        value = String(value);
        if (value !== 'speakers' && value !== 'discrete') throw new TypeError("Invalid channelInterpretation");
        call("audioSetNodeChannelInterpretation", this.context.__id, this.__id, value);
        this.__channelInterpretation = value;
    }
    connect(destination, output = 0, input = 0) {
        if (!(destination instanceof AudioNode) || destination.context !== this.context) {
            throw new DOMException("Nodes belong to different audio contexts", "InvalidAccessError");
        }
        output = Number(output) >>> 0;
        input = Number(input) >>> 0;
        if (output >= this.numberOfOutputs || input >= destination.numberOfInputs) throw new DOMException("Audio node index is out of range", "IndexSizeError");
        call("audioConnect", this.context.__id, this.__id, destination.__id, output, input);
        return destination;
    }
    disconnect(destination = undefined) {
        if (destination !== undefined && (!(destination instanceof AudioNode) || destination.context !== this.context)) throw new TypeError("Invalid destination AudioNode");
        call("audioDisconnect", this.context.__id, this.__id, destination !== undefined, destination?.__id ?? 0);
    }
}

class AudioScheduledSourceNode extends AudioNode {
    constructor(token, context, id) {
        super(token, context, id);
        this.__started = false;
        this.__ended = false;
        Object.defineProperty(this, "onended", { value: null, writable: true, enumerable: true });
        context.__scheduledSources.set(id, this);
    }
    __dispatchEnded() {
        if (this.__ended) return;
        this.__ended = true;
        const event = new Event("ended");
        event.isTrusted = true;
        this.dispatchEvent(event);
    }
}

class AudioDestinationNode extends AudioNode {
    constructor(token, context) {
        super(token, context, 0);
        Object.defineProperties(this, {
            maxChannelCount: { value: context.__channels, enumerable: true },
            numberOfInputs: { value: 1, enumerable: true },
            numberOfOutputs: { value: 0, enumerable: true },
        });
    }
}

class ScriptProcessorNode extends AudioNode {
    constructor(token, context, id, bufferSize) {
        if (token !== construct) throw new TypeError("Illegal constructor");
        super(token, context, id);
        Object.defineProperties(this, {
            __bufferSize: { value: bufferSize },
            numberOfInputs: { value: 1, enumerable: true },
            numberOfOutputs: { value: 1, enumerable: true },
            onaudioprocess: { value: null, writable: true, enumerable: true },
        });
    }
    get bufferSize() { return this.__bufferSize; }
}

class AudioWorkletMessagePort extends EventTarget {
    constructor(token, context, node) {
        if (token !== construct) throw new TypeError("Illegal constructor");
        super();
        Object.defineProperties(this, {
            __context: { value: context },
            __node: { value: node },
            onmessage: { value: null, writable: true, enumerable: true },
            onmessageerror: { value: null, writable: true, enumerable: true },
        });
    }
    postMessage(value) {
        let encoded;
        try {
            encoded = JSON.stringify(value);
        } catch {
            throw new DOMException("The value cannot be cloned", "DataCloneError");
        }
        if (encoded === undefined) throw new DOMException("The value cannot be cloned", "DataCloneError");
        call("audioPostWorkletMessage", this.__context.__id, this.__node, encoded);
        this.__context.__scheduleWorkletPoll();
    }
    start() { this.__context.__scheduleWorkletPoll(); }
    close() {}
    __deliver(encoded) {
        let data;
        try {
            data = JSON.parse(encoded);
        } catch {
            const error = new MessageEvent("messageerror", { data: null });
            error.isTrusted = true;
            this.dispatchEvent(error);
            return;
        }
        const event = new MessageEvent("message", { data });
        event.isTrusted = true;
        this.dispatchEvent(event);
    }
}

class AudioWorkletNode extends AudioNode {
    constructor(context, name, options = {}) {
        if (!(context instanceof BaseAudioContext)) throw new TypeError("context must be a BaseAudioContext");
        name = String(name);
        if (options === null || typeof options !== "object") throw new TypeError("options must be an object");
        const numberOfInputs = options.numberOfInputs === undefined ? 1 : Number(options.numberOfInputs) >>> 0;
        const numberOfOutputs = options.numberOfOutputs === undefined ? 1 : Number(options.numberOfOutputs) >>> 0;
        const outputChannelCount = options.outputChannelCount === undefined
            ? []
            : Array.from(options.outputChannelCount, value => Number(value) >>> 0);
        const parameterData = Object.fromEntries(
            Object.entries(options.parameterData ?? {}).map(([key, value]) => [String(key), Number(value)]),
        );
        const request = {
            numberOfInputs,
            numberOfOutputs,
            outputChannelCount,
            parameterData,
            processorOptions: options.processorOptions ?? {},
            channelCount: options.channelCount === undefined ? null : Number(options.channelCount) >>> 0,
            channelCountMode: options.channelCountMode === undefined ? null : String(options.channelCountMode),
            channelInterpretation: options.channelInterpretation === undefined ? null : String(options.channelInterpretation),
        };
        let encoded;
        try {
            encoded = JSON.stringify(request);
        } catch {
            throw new DOMException("processorOptions cannot be cloned", "DataCloneError");
        }
        const metadata = JSON.parse(call("audioCreateWorkletNode", context.__id, name, encoded));
        super(construct, context, metadata.id);
        const parameters = new Map();
        for (const descriptor of metadata.descriptors) {
            parameters.set(descriptor.name, new AudioParam(
                construct,
                context.__id,
                metadata.id,
                descriptor.name,
                descriptor.defaultValue,
                descriptor.minValue,
                descriptor.maxValue,
            ));
        }
        const port = new AudioWorkletMessagePort(construct, context, metadata.id);
        Object.defineProperties(this, {
            numberOfInputs: { value: numberOfInputs, enumerable: true },
            numberOfOutputs: { value: numberOfOutputs, enumerable: true },
            parameters: { value: parameters, enumerable: true },
            port: { value: port, enumerable: true },
            onprocessorerror: { value: null, writable: true, enumerable: true },
        });
        context.__workletNodes.set(metadata.id, this);
        context.__scheduleWorkletPoll();
    }
}

class AudioWorklet {
    constructor(token, context) {
        if (token !== construct) throw new TypeError("Illegal constructor");
        Object.defineProperty(this, "__context", { value: context });
    }
    addModule(moduleURL, options = {}) {
        const credentials = options?.credentials === undefined ? "same-origin" : String(options.credentials);
        if (!["omit", "same-origin", "include"].includes(credentials)) {
            return Promise.reject(new TypeError("Invalid AudioWorklet credentials mode"));
        }
        return fetch(String(moduleURL), { credentials }).then(response => {
            if (!response.ok) throw new TypeError(`Could not load AudioWorklet module: HTTP ${response.status}`);
            return response.text();
        }).then(source => {
            call("audioRegisterWorkletModule", this.__context.__id, source);
        });
    }
}

class MediaStreamAudioDestinationNode extends AudioNode {
    constructor(tokenOrContext, contextOrOptions, id) {
        let context;
        let options;
        if (tokenOrContext === construct) {
            context = contextOrOptions;
            options = {};
        } else {
            context = tokenOrContext;
            options = contextOrOptions ?? {};
            if (!(context instanceof AudioContext)) throw new TypeError("context must be an AudioContext");
            id = call("audioCreateNode", context.__id, "media-stream-destination", 0);
        }
        super(construct, context, id);
        const track = new MediaStreamTrack(construct, context.__id, id, context.sampleRate);
        Object.defineProperties(this, {
            stream: { value: new MediaStream(construct, track, `brimp-stream-${context.__id}-${id}`), enumerable: true },
            numberOfInputs: { value: 1, enumerable: true },
            numberOfOutputs: { value: 0, enumerable: true },
        });
        if (options.channelCount !== undefined) this.channelCount = options.channelCount;
        if (options.channelCountMode !== undefined) this.channelCountMode = options.channelCountMode;
        if (options.channelInterpretation !== undefined) this.channelInterpretation = options.channelInterpretation;
    }
}

function mediaStreamAudioTrack(stream) {
    if (!(stream instanceof MediaStream)) throw new TypeError("mediaStream must be a MediaStream");
    const tracks = stream.getAudioTracks().sort((left, right) => left.id.localeCompare(right.id));
    if (tracks.length === 0) {
        throw new DOMException("The MediaStream has no audio track", "InvalidStateError");
    }
    return tracks[0];
}

class MediaStreamAudioSourceNode extends AudioNode {
    constructor(tokenOrContext, contextOrStream, idOrOptions) {
        let context;
        let stream;
        let id;
        if (tokenOrContext === construct) {
            context = contextOrStream;
            stream = idOrOptions.stream;
            id = idOrOptions.id;
        } else {
            context = tokenOrContext;
            stream = contextOrStream?.mediaStream;
            if (!(context instanceof AudioContext)) throw new TypeError("context must be an AudioContext");
            const track = mediaStreamAudioTrack(stream);
            id = call("audioCreateMediaStreamSource", context.__id, track.__context, track.__node, false);
        }
        super(construct, context, id);
        Object.defineProperties(this, {
            mediaStream: { value: stream, enumerable: true },
            numberOfInputs: { value: 0, enumerable: true },
            numberOfOutputs: { value: 1, enumerable: true },
        });
    }
}

class MediaStreamTrackAudioSourceNode extends AudioNode {
    constructor(tokenOrContext, contextOrTrack, idOrOptions) {
        let context;
        let track;
        let id;
        if (tokenOrContext === construct) {
            context = contextOrTrack;
            track = idOrOptions.track;
            id = idOrOptions.id;
        } else {
            context = tokenOrContext;
            track = contextOrTrack?.mediaStreamTrack;
            if (!(context instanceof AudioContext)) throw new TypeError("context must be an AudioContext");
            if (!(track instanceof MediaStreamTrack) || track.kind !== "audio") {
                throw new TypeError("mediaStreamTrack must be an audio MediaStreamTrack");
            }
            id = call("audioCreateMediaStreamSource", context.__id, track.__context, track.__node, true);
        }
        super(construct, context, id);
        Object.defineProperties(this, {
            mediaStreamTrack: { value: track, enumerable: true },
            numberOfInputs: { value: 0, enumerable: true },
            numberOfOutputs: { value: 1, enumerable: true },
        });
    }
}

class MediaElementController {
    constructor(context, node, element) {
        this.context = context;
        this.node = node;
        this.element = element;
        this.buffer = null;
        this.loading = null;
        this.loadedSrc = "";
        this.source = null;
        this.offset = 0;
        this.startedAt = 0;
        this.playing = false;
        this.endedFlag = false;
    }
    currentTime() {
        if (!this.playing || this.buffer === null) return this.offset;
        const elapsed = (this.context.currentTime - this.startedAt) * Math.abs(this.element.playbackRate);
        const time = this.offset + elapsed;
        if (this.element.loop && this.buffer.duration > 0) return time % this.buffer.duration;
        return Math.min(time, this.buffer.duration);
    }
    duration() { return this.buffer?.duration ?? NaN; }
    ended() { return this.endedFlag; }
    readyState() { return this.buffer === null ? (this.loading === null ? 0 : 1) : 4; }
    load() {
        const src = this.element.currentSrc;
        this.pause();
        this.offset = 0;
        this.buffer = null;
        this.loadedSrc = src;
        if (src === "") {
            this.loading = Promise.reject(new DOMException("The media element has no source", "NotSupportedError"));
            return this.loading;
        }
        this.loading = fetch(src)
            .then(response => {
                if (!response.ok) throw new DOMException(`Media request failed with ${response.status}`, "NotSupportedError");
                return response.arrayBuffer();
            })
            .then(bytes => this.context.decodeAudioData(bytes))
            .then(buffer => {
                const originClean = host("mediaElementOriginClean", this.element);
                this.buffer = originClean
                    ? buffer
                    : this.context.createBuffer(buffer.numberOfChannels, buffer.length, buffer.sampleRate);
                this.loading = null;
                const event = new Event("canplaythrough");
                event.isTrusted = true;
                this.element.dispatchEvent(event);
                return this.buffer;
            }, error => {
                this.loading = null;
                throw error;
            });
        return this.loading;
    }
    async play() {
        if (this.buffer === null) {
            if (this.loading === null || this.loadedSrc !== this.element.currentSrc) this.load();
            await this.loading;
        }
        if (this.playing) return;
        if (this.offset >= this.buffer.duration) this.offset = 0;
        const source = this.context.createBufferSource();
        source.buffer = this.buffer;
        source.loop = this.element.loop;
        source.playbackRate.value = this.element.playbackRate;
        call("audioConnect", this.context.__id, source.__id, this.node.__id, 0, 0);
        source.onended = () => {
            if (this.source !== source) return;
            this.playing = false;
            this.source = null;
            this.offset = this.buffer.duration;
            this.endedFlag = true;
            const event = new Event("ended");
            event.isTrusted = true;
            this.element.dispatchEvent(event);
        };
        call("audioSetParam", this.context.__id, this.node.__id, "gain",
            this.element.muted ? 0 : this.element.volume);
        source.start(0, this.offset);
        this.source = source;
        this.startedAt = this.context.currentTime;
        this.playing = true;
        this.endedFlag = false;
    }
    pause() {
        if (!this.playing) return;
        this.offset = this.currentTime();
        this.playing = false;
        if (this.source !== null) {
            const source = this.source;
            this.source = null;
            source.stop();
            source.disconnect();
        }
    }
    seek(value) {
        const wasPlaying = this.playing;
        this.pause();
        this.offset = this.buffer === null ? value : Math.min(value, this.buffer.duration);
        this.endedFlag = false;
        if (wasPlaying) void this.play();
    }
    setPlaybackRate(value) {
        if (this.source === null) return;
        this.offset = this.currentTime();
        this.startedAt = this.context.currentTime;
        this.source.playbackRate.value = value;
    }
    setVolume(value) {
        call("audioSetParam", this.context.__id, this.node.__id, "gain",
            this.element.muted ? 0 : value);
    }
}

class MediaElementAudioSourceNode extends AudioNode {
    constructor(tokenOrContext, contextOrElement, idOrOptions) {
        let context;
        let element;
        let id;
        if (tokenOrContext === construct) {
            context = contextOrElement;
            element = idOrOptions.element;
            id = idOrOptions.id;
        } else {
            context = tokenOrContext;
            element = contextOrElement?.mediaElement;
            if (!(context instanceof AudioContext)) throw new TypeError("context must be an AudioContext");
            if (!(element instanceof HTMLMediaElement)) throw new TypeError("mediaElement must be an HTMLMediaElement");
            if (mediaElementSources.has(element)) throw new DOMException("The media element already has a source node", "InvalidStateError");
            id = call("audioCreateNode", context.__id, "gain", 0);
        }
        super(construct, context, id);
        if (mediaElementSources.has(element)) throw new DOMException("The media element already has a source node", "InvalidStateError");
        const controller = new MediaElementController(context, this, element);
        Object.defineProperties(this, {
            mediaElement: { value: element, enumerable: true },
            numberOfInputs: { value: 0, enumerable: true },
            numberOfOutputs: { value: 1, enumerable: true },
            __controller: { value: controller },
        });
        mediaElementSources.set(element, this);
    }
}

Object.defineProperty(globalThis, "__brimpGetMediaElementController", {
    value(element) { return mediaElementSources.get(element)?.__controller ?? null; },
    configurable: true,
});

class OscillatorNode extends AudioScheduledSourceNode {
    constructor(token, context, id) {
        super(token, context, id);
        this.__type = "sine";
        Object.defineProperties(this, {
            frequency: { value: new AudioParam(construct, context.__id, id, "frequency", 440, -3.4e38, 3.4e38), enumerable: true },
            detune: { value: new AudioParam(construct, context.__id, id, "detune", 0, -3.4e38, 3.4e38), enumerable: true },
            numberOfInputs: { value: 0, enumerable: true },
            numberOfOutputs: { value: 1, enumerable: true },
        });
    }
    get type() { return this.__type; }
    set type(value) {
        value = String(value);
        if (!["sine", "square", "sawtooth", "triangle"].includes(value)) {
            throw new TypeError("Invalid oscillator type");
        }
        call("audioSetOscillatorType", this.context.__id, this.__id, value);
        this.__type = value;
    }
    start(when = 0) {
        if (this.__started) throw new DOMException("The source has already started", "InvalidStateError");
        call("audioStart", this.context.__id, this.__id, Number(when));
        this.__started = true;
    }
    stop(when = 0) {
        if (!this.__started) throw new DOMException("The source has not started", "InvalidStateError");
        call("audioStop", this.context.__id, this.__id, Number(when));
        this.context.__scheduleEndedPoll(Number(when), this);
    }
    setPeriodicWave(periodicWave) {
        if (!(periodicWave instanceof PeriodicWave) || periodicWave.__context !== this.context.__id) throw new TypeError("Invalid PeriodicWave");
        call("audioSetPeriodicWave", this.context.__id, this.__id, periodicWave.__id);
        this.__type = "custom";
    }
}

class DynamicsCompressorNode extends AudioNode {
    constructor(token, context, id) {
        super(token, context, id);
        const param = (name, value, min, max) => new AudioParam(construct, context.__id, id, name, value, min, max);
        Object.defineProperties(this, {
            threshold: { value: param("threshold", -24, -100, 0), enumerable: true },
            knee: { value: param("knee", 30, 0, 40), enumerable: true },
            ratio: { value: param("ratio", 12, 1, 20), enumerable: true },
            attack: { value: param("attack", 0.003, 0, 1), enumerable: true },
            release: { value: param("release", 0.25, 0, 1), enumerable: true },
            reduction: { value: 0, enumerable: true },
            numberOfInputs: { value: 1, enumerable: true },
            numberOfOutputs: { value: 1, enumerable: true },
        });
    }
}

class GainNode extends AudioNode {
    constructor(token, context, id) {
        super(token, context, id);
        Object.defineProperties(this, {
            gain: { value: new AudioParam(construct, context.__id, id, "gain", 1, -3.4e38, 3.4e38), enumerable: true },
            numberOfInputs: { value: 1, enumerable: true },
            numberOfOutputs: { value: 1, enumerable: true },
        });
    }
}

class BiquadFilterNode extends AudioNode {
    constructor(token, context, id) {
        super(token, context, id);
        const param = (name, value, min, max) => new AudioParam(construct, context.__id, id, name, value, min, max);
        this.__type = "lowpass";
        Object.defineProperties(this, {
            frequency: { value: param("frequency", 350, 0, context.sampleRate / 2), enumerable: true },
            detune: { value: param("detune", 0, -3.4e38, 3.4e38), enumerable: true },
            Q: { value: param("Q", 1, -3.4e38, 3.4e38), enumerable: true },
            gain: { value: param("gain", 0, -3.4e38, 3.4e38), enumerable: true },
            numberOfInputs: { value: 1, enumerable: true }, numberOfOutputs: { value: 1, enumerable: true },
        });
    }
    get type() { return this.__type; }
    set type(value) {
        value = String(value);
        if (!["lowpass", "highpass", "bandpass", "lowshelf", "highshelf", "peaking", "notch", "allpass"].includes(value)) throw new TypeError("Invalid biquad filter type");
        call("audioSetBiquadType", this.context.__id, this.__id, value);
        this.__type = value;
    }
    getFrequencyResponse(frequencyHz, magResponse, phaseResponse) {
        if (!(frequencyHz instanceof Float32Array) || !(magResponse instanceof Float32Array) || !(phaseResponse instanceof Float32Array)) throw new TypeError("Frequency response arguments must be Float32Array instances");
        if (frequencyHz.length !== magResponse.length || frequencyHz.length !== phaseResponse.length) throw new DOMException("Array lengths must match", "InvalidAccessError");
        magResponse.set(call("audioBiquadFrequencyResponse", this.context.__id, this.__id, frequencyHz, true));
        phaseResponse.set(call("audioBiquadFrequencyResponse", this.context.__id, this.__id, frequencyHz, false));
    }
}

class StereoPannerNode extends AudioNode {
    constructor(token, context, id) {
        super(token, context, id);
        Object.defineProperties(this, {
            pan: { value: new AudioParam(construct, context.__id, id, "pan", 0, -1, 1), enumerable: true },
            numberOfInputs: { value: 1, enumerable: true }, numberOfOutputs: { value: 1, enumerable: true },
        });
    }
}

class PannerNode extends AudioNode {
    constructor(token, context, id) {
        super(token, context, id);
        this.__panningModel = "equalpower";
        this.__distanceModel = "inverse";
        this.__refDistance = 1;
        this.__maxDistance = 10000;
        this.__rolloffFactor = 1;
        this.__coneInnerAngle = 360;
        this.__coneOuterAngle = 360;
        this.__coneOuterGain = 0;
        const param = (name, value) => new AudioParam(construct, context.__id, id, name, value, -3.4e38, 3.4e38);
        Object.defineProperties(this, {
            positionX: { value: param("positionX", 0), enumerable: true },
            positionY: { value: param("positionY", 0), enumerable: true },
            positionZ: { value: param("positionZ", 0), enumerable: true },
            orientationX: { value: param("orientationX", 1), enumerable: true },
            orientationY: { value: param("orientationY", 0), enumerable: true },
            orientationZ: { value: param("orientationZ", 0), enumerable: true },
            numberOfInputs: { value: 1, enumerable: true }, numberOfOutputs: { value: 1, enumerable: true },
        });
    }
    __setModel(property, allowed, value) {
        value = String(value);
        if (!allowed.includes(value)) throw new TypeError(`Invalid ${property}`);
        call("audioSetPannerModel", this.context.__id, this.__id, property, value);
        this[`__${property}`] = value;
    }
    __setNumber(property, value) {
        value = Number(value);
        call("audioSetPannerNumber", this.context.__id, this.__id, property, value);
        this[`__${property}`] = value;
    }
    get panningModel() { return this.__panningModel; }
    set panningModel(value) { this.__setModel("panningModel", ["equalpower", "HRTF"], value); }
    get distanceModel() { return this.__distanceModel; }
    set distanceModel(value) { this.__setModel("distanceModel", ["linear", "inverse", "exponential"], value); }
    get refDistance() { return this.__refDistance; }
    set refDistance(value) { this.__setNumber("refDistance", value); }
    get maxDistance() { return this.__maxDistance; }
    set maxDistance(value) { this.__setNumber("maxDistance", value); }
    get rolloffFactor() { return this.__rolloffFactor; }
    set rolloffFactor(value) { this.__setNumber("rolloffFactor", value); }
    get coneInnerAngle() { return this.__coneInnerAngle; }
    set coneInnerAngle(value) { this.__setNumber("coneInnerAngle", value); }
    get coneOuterAngle() { return this.__coneOuterAngle; }
    set coneOuterAngle(value) { this.__setNumber("coneOuterAngle", value); }
    get coneOuterGain() { return this.__coneOuterGain; }
    set coneOuterGain(value) { this.__setNumber("coneOuterGain", value); }
    setPosition(x, y, z) {
        this.positionX.value = Number(x); this.positionY.value = Number(y); this.positionZ.value = Number(z);
    }
    setOrientation(x, y, z) {
        this.orientationX.value = Number(x); this.orientationY.value = Number(y); this.orientationZ.value = Number(z);
    }
}

class AudioBufferSourceNode extends AudioScheduledSourceNode {
    constructor(token, context, id) {
        super(token, context, id);
        this.__buffer = null;
        this.__loop = false;
        this.__loopStart = 0;
        this.__loopEnd = 0;
        Object.defineProperties(this, {
            playbackRate: { value: new AudioParam(construct, context.__id, id, "playbackRate", 1, -3.4e38, 3.4e38), enumerable: true },
            detune: { value: new AudioParam(construct, context.__id, id, "detune", 0, -3.4e38, 3.4e38), enumerable: true },
            numberOfInputs: { value: 0, enumerable: true }, numberOfOutputs: { value: 1, enumerable: true },
        });
    }
    get buffer() { return this.__buffer; }
    set buffer(value) {
        if (value !== null && (!(value instanceof AudioBuffer) || value.__context !== this.context.__id)) throw new TypeError("Invalid AudioBuffer");
        if (this.__buffer !== null && value !== this.__buffer) throw new DOMException("The buffer is already assigned", "InvalidStateError");
        call("audioSetBufferSource", this.context.__id, this.__id, value?.__id ?? 0);
        this.__buffer = value;
    }
    get loop() { return this.__loop; }
    set loop(value) { this.__loop = Boolean(value); call("audioConfigureBufferSource", this.context.__id, this.__id, "loop", this.__loop ? 1 : 0); }
    get loopStart() { return this.__loopStart; }
    set loopStart(value) { value = Number(value); call("audioConfigureBufferSource", this.context.__id, this.__id, "loopStart", value); this.__loopStart = value; }
    get loopEnd() { return this.__loopEnd; }
    set loopEnd(value) { value = Number(value); call("audioConfigureBufferSource", this.context.__id, this.__id, "loopEnd", value); this.__loopEnd = value; }
    start(when = 0, offset = 0, duration = undefined) {
        if (this.__started) throw new DOMException("The source has already started", "InvalidStateError");
        call("audioStartBufferSource", this.context.__id, this.__id, Number(when), Number(offset), duration === undefined ? -1 : Number(duration));
        this.__started = true;
        if (this.context instanceof AudioContext && !this.__loop) {
            const rate = Math.abs(this.playbackRate.value * Math.pow(2, this.detune.value / 1200));
            const sourceDuration = duration === undefined
                ? Math.max(0, (this.__buffer?.duration ?? 0) - Number(offset))
                : Number(duration);
            if (rate > 0) this.context.__scheduleEndedPoll(Math.max(Number(when), this.context.currentTime) + sourceDuration / rate, this);
        }
    }
    stop(when = 0) {
        if (!this.__started) throw new DOMException("The source has not started", "InvalidStateError");
        call("audioStop", this.context.__id, this.__id, Number(when));
        this.context.__scheduleEndedPoll(Number(when), this);
    }
}

class AnalyserNode extends AudioNode {
    constructor(token, context, id) {
        super(token, context, id);
        Object.defineProperties(this, { numberOfInputs: { value: 1, enumerable: true }, numberOfOutputs: { value: 1, enumerable: true } });
    }
    __settings() { return JSON.parse(call("audioAnalyserSettings", this.context.__id, this.__id)); }
    get fftSize() { return this.__settings()[0]; }
    set fftSize(value) { call("audioSetAnalyser", this.context.__id, this.__id, "fftSize", Number(value)); }
    get frequencyBinCount() { return this.__settings()[1]; }
    get minDecibels() { return this.__settings()[2]; }
    set minDecibels(value) { call("audioSetAnalyser", this.context.__id, this.__id, "minDecibels", Number(value)); }
    get maxDecibels() { return this.__settings()[3]; }
    set maxDecibels(value) { call("audioSetAnalyser", this.context.__id, this.__id, "maxDecibels", Number(value)); }
    get smoothingTimeConstant() { return this.__settings()[4]; }
    set smoothingTimeConstant(value) { call("audioSetAnalyser", this.context.__id, this.__id, "smoothingTimeConstant", Number(value)); }
    getFloatFrequencyData(destination) {
        if (!(destination instanceof Float32Array)) throw new TypeError("Expected Float32Array");
        destination.set(call("audioAnalyserFloatData", this.context.__id, this.__id, true, destination.length));
    }
    getFloatTimeDomainData(destination) {
        if (!(destination instanceof Float32Array)) throw new TypeError("Expected Float32Array");
        destination.set(call("audioAnalyserFloatData", this.context.__id, this.__id, false, destination.length));
    }
    getByteFrequencyData(destination) {
        if (!(destination instanceof Uint8Array)) throw new TypeError("Expected Uint8Array");
        destination.set(call("audioAnalyserByteData", this.context.__id, this.__id, true, destination.length));
    }
    getByteTimeDomainData(destination) {
        if (!(destination instanceof Uint8Array)) throw new TypeError("Expected Uint8Array");
        destination.set(call("audioAnalyserByteData", this.context.__id, this.__id, false, destination.length));
    }
}

class ConstantSourceNode extends AudioScheduledSourceNode {
    constructor(token, context, id) {
        super(token, context, id);
        Object.defineProperties(this, {
            offset: { value: new AudioParam(construct, context.__id, id, "offset", 1, -3.4e38, 3.4e38), enumerable: true },
            numberOfInputs: { value: 0, enumerable: true }, numberOfOutputs: { value: 1, enumerable: true },
        });
    }
    start(when = 0) {
        if (this.__started) throw new DOMException("The source has already started", "InvalidStateError");
        call("audioStart", this.context.__id, this.__id, Number(when));
        this.__started = true;
    }
    stop(when = 0) {
        if (!this.__started) throw new DOMException("The source has not started", "InvalidStateError");
        call("audioStop", this.context.__id, this.__id, Number(when));
        this.context.__scheduleEndedPoll(Number(when), this);
    }
}

class DelayNode extends AudioNode {
    constructor(token, context, id, maxDelayTime) {
        super(token, context, id);
        Object.defineProperties(this, {
            delayTime: { value: new AudioParam(construct, context.__id, id, "delayTime", 0, 0, maxDelayTime), enumerable: true },
            numberOfInputs: { value: 1, enumerable: true }, numberOfOutputs: { value: 1, enumerable: true },
        });
    }
}

class WaveShaperNode extends AudioNode {
    constructor(token, context, id) {
        super(token, context, id);
        this.__curve = null;
        this.__oversample = "none";
        Object.defineProperties(this, {
            numberOfInputs: { value: 1, enumerable: true }, numberOfOutputs: { value: 1, enumerable: true },
        });
    }
    get curve() { return this.__curve; }
    set curve(value) {
        if (value === null && this.__curve === null) return;
        if (!(value instanceof Float32Array)) throw new TypeError("curve must be a Float32Array or null");
        if (this.__curve !== null) throw new DOMException("The curve is already assigned", "InvalidStateError");
        call("audioSetWaveShaperCurve", this.context.__id, this.__id, value);
        this.__curve = new Float32Array(value);
    }
    get oversample() { return this.__oversample; }
    set oversample(value) {
        value = String(value);
        if (!["none", "2x", "4x"].includes(value)) throw new TypeError("Invalid oversample value");
        call("audioSetWaveShaperOversample", this.context.__id, this.__id, value);
        this.__oversample = value;
    }
}

class ChannelSplitterNode extends AudioNode {
    constructor(token, context, id, outputs) {
        super(token, context, id);
        this.channelCount = outputs;
        this.channelCountMode = "explicit";
        this.channelInterpretation = "discrete";
        Object.defineProperties(this, {
            numberOfInputs: { value: 1, enumerable: true }, numberOfOutputs: { value: outputs, enumerable: true },
        });
    }
}

class ChannelMergerNode extends AudioNode {
    constructor(token, context, id, inputs) {
        super(token, context, id);
        this.channelCount = 1;
        this.channelCountMode = "explicit";
        Object.defineProperties(this, {
            numberOfInputs: { value: inputs, enumerable: true }, numberOfOutputs: { value: 1, enumerable: true },
        });
    }
}

class ConvolverNode extends AudioNode {
    constructor(token, context, id) {
        super(token, context, id);
        this.__buffer = null;
        this.__normalize = true;
        Object.defineProperties(this, {
            numberOfInputs: { value: 1, enumerable: true }, numberOfOutputs: { value: 1, enumerable: true },
        });
    }
    get buffer() { return this.__buffer; }
    set buffer(value) {
        if (value !== null && (!(value instanceof AudioBuffer) || value.__context !== this.context.__id)) throw new TypeError("Invalid AudioBuffer");
        if (value === null) {
            if (this.__buffer !== null) throw new DOMException("Clearing a ConvolverNode buffer is not supported", "NotSupportedError");
            return;
        }
        call("audioConfigureConvolver", this.context.__id, this.__id, "buffer", value.__id);
        this.__buffer = value;
    }
    get normalize() { return this.__normalize; }
    set normalize(value) {
        value = Boolean(value);
        call("audioConfigureConvolver", this.context.__id, this.__id, "normalize", value);
        this.__normalize = value;
    }
}

class IIRFilterNode extends AudioNode {
    constructor(token, context, id) {
        super(token, context, id);
        Object.defineProperties(this, {
            numberOfInputs: { value: 1, enumerable: true }, numberOfOutputs: { value: 1, enumerable: true },
        });
    }
    getFrequencyResponse(frequencyHz, magResponse, phaseResponse) {
        if (!(frequencyHz instanceof Float32Array) || !(magResponse instanceof Float32Array) || !(phaseResponse instanceof Float32Array)) throw new TypeError("Frequency response arguments must be Float32Array instances");
        if (frequencyHz.length !== magResponse.length || frequencyHz.length !== phaseResponse.length) throw new DOMException("Array lengths must match", "InvalidAccessError");
        magResponse.set(call("audioIirFrequencyResponse", this.context.__id, this.__id, frequencyHz, true));
        phaseResponse.set(call("audioIirFrequencyResponse", this.context.__id, this.__id, frequencyHz, false));
    }
}

class AudioListener {
    constructor(token, context) {
        if (token !== construct) throw new TypeError("Illegal constructor");
        const param = (name, value) => new AudioParam(construct, context.__id, listenerNode, name, value, -3.4e38, 3.4e38);
        Object.defineProperties(this, {
            positionX: { value: param("positionX", 0), enumerable: true },
            positionY: { value: param("positionY", 0), enumerable: true },
            positionZ: { value: param("positionZ", 0), enumerable: true },
            forwardX: { value: param("forwardX", 0), enumerable: true },
            forwardY: { value: param("forwardY", 0), enumerable: true },
            forwardZ: { value: param("forwardZ", -1), enumerable: true },
            upX: { value: param("upX", 0), enumerable: true },
            upY: { value: param("upY", 1), enumerable: true },
            upZ: { value: param("upZ", 0), enumerable: true },
        });
    }
    setPosition(x, y, z) {
        this.positionX.value = Number(x); this.positionY.value = Number(y); this.positionZ.value = Number(z);
    }
    setOrientation(forwardX, forwardY, forwardZ, upX, upY, upZ) {
        this.forwardX.value = Number(forwardX); this.forwardY.value = Number(forwardY); this.forwardZ.value = Number(forwardZ);
        this.upX.value = Number(upX); this.upY.value = Number(upY); this.upZ.value = Number(upZ);
    }
}

class PeriodicWave {
    constructor(token, context, id) {
        if (token !== construct) throw new TypeError("Illegal constructor");
        Object.defineProperties(this, { __context: { value: context }, __id: { value: id } });
    }
}

class AudioBuffer {
    constructor(token, context, metadata) {
        if (token !== construct) throw new TypeError("Illegal constructor");
        Object.defineProperties(this, {
            __context: { value: context }, __id: { value: metadata.id }, __channels: { value: new Map() },
            numberOfChannels: { value: metadata.channels, enumerable: true },
            length: { value: metadata.length, enumerable: true },
            sampleRate: { value: metadata.sampleRate, enumerable: true },
            duration: { value: metadata.length / metadata.sampleRate, enumerable: true },
        });
    }
    getChannelData(channel) {
        channel = Number(channel) >>> 0;
        if (channel >= this.numberOfChannels) throw new DOMException("Channel index is out of range", "IndexSizeError");
        if (!this.__channels.has(channel)) this.__channels.set(channel, call("audioChannel", this.__context, this.__id, channel));
        return this.__channels.get(channel);
    }
    copyFromChannel(destination, channel, offset = 0) {
        destination.set(this.getChannelData(channel).subarray(Number(offset) >>> 0, (Number(offset) >>> 0) + destination.length));
    }
    copyToChannel(source, channel, offset = 0) {
        if (!(source instanceof Float32Array)) throw new TypeError("The source must be a Float32Array");
        channel = Number(channel) >>> 0;
        offset = Number(offset) >>> 0;
        if (channel >= this.numberOfChannels) throw new DOMException("Channel index is out of range", "IndexSizeError");
        call("audioWriteChannel", this.__context, this.__id, channel, offset, source);
        if (this.__channels.has(channel)) this.__channels.get(channel).set(source.subarray(0, this.length - offset), offset);
    }
}

class AudioProcessingEvent extends Event {
    constructor(type, init) {
        if (!init || !(init.inputBuffer instanceof AudioBuffer)
            || !(init.outputBuffer instanceof AudioBuffer)
            || !("playbackTime" in init)) {
            throw new TypeError("AudioProcessingEvent requires playbackTime and input/output AudioBuffers");
        }
        const playbackTime = Number(init.playbackTime);
        if (!Number.isFinite(playbackTime)) throw new TypeError("playbackTime must be finite");
        super(type, init);
        Object.defineProperties(this, {
            __playbackTime: { value: playbackTime },
            __inputBuffer: { value: init.inputBuffer },
            __outputBuffer: { value: init.outputBuffer },
        });
    }
    get playbackTime() { return this.__playbackTime; }
    get inputBuffer() { return this.__inputBuffer; }
    get outputBuffer() { return this.__outputBuffer; }
}

class OfflineAudioCompletionEvent extends Event {
    constructor(type, init) {
        if (!init || !(init.renderedBuffer instanceof AudioBuffer)) throw new TypeError("An AudioBuffer is required");
        super(type, init);
        Object.defineProperty(this, "renderedBuffer", { value: init.renderedBuffer, enumerable: true });
    }
}

class BaseAudioContext extends EventTarget {
    constructor(token, id, channels, sampleRate) {
        if (token !== construct) throw new TypeError("Illegal constructor");
        super();
        Object.defineProperties(this, {
            __id: { value: id }, __channels: { value: channels },
            __scheduledSources: { value: new Map() },
            __processingNodes: { value: new Map() },
            __processingPollScheduled: { value: false, writable: true },
            __workletNodes: { value: new Map() },
            __workletPollScheduled: { value: false, writable: true },
            sampleRate: { value: sampleRate, enumerable: true },
        });
        Object.defineProperty(this, "destination", { value: new AudioDestinationNode(construct, this), enumerable: true });
        Object.defineProperty(this, "listener", { value: new AudioListener(construct, this), enumerable: true });
        Object.defineProperty(this, "audioWorklet", { value: new AudioWorklet(construct, this), enumerable: true });
    }
    __drainEndedEvents() {
        const ended = JSON.parse(call("audioTakeEnded", this.__id));
        for (const id of ended) {
            const source = this.__scheduledSources.get(id);
            if (!source || source.__ended) continue;
            queueMicrotask(() => source.__dispatchEnded());
        }
    }
    __scheduleEndedPoll(when, source) {
        if (!(this instanceof AudioContext)) return;
        const poll = delay => setTimeout(() => {
            this.__drainEndedEvents();
            if (!source.__ended) poll(25);
        }, delay);
        poll(Math.max(0, (Number(when) - this.currentTime) * 1000) + 5);
    }
    __scheduleProcessingPoll() {
        if (this.__processingPollScheduled) return;
        this.__processingPollScheduled = true;
        const poll = () => {
            if (this.state === "closed" || this.__processingNodes.size === 0) {
                this.__processingPollScheduled = false;
                return;
            }
            const events = JSON.parse(call("audioTakeProcessingEvents", this.__id));
            for (const pending of events) {
                const node = this.__processingNodes.get(pending.node);
                const inputBuffer = new AudioBuffer(construct, this.__id, pending.input);
                const outputBuffer = new AudioBuffer(construct, this.__id, pending.output);
                try {
                    if (node) {
                        const event = new AudioProcessingEvent("audioprocess", {
                            playbackTime: pending.playbackTime,
                            inputBuffer,
                            outputBuffer,
                            bubbles: true,
                        });
                        event.isTrusted = true;
                        node.dispatchEvent(event);
                    }
                } finally {
                    for (const [channel, samples] of outputBuffer.__channels) {
                        call(
                            "audioWriteChannel",
                            this.__id,
                            outputBuffer.__id,
                            channel,
                            0,
                            samples,
                        );
                    }
                    call(
                        "audioCompleteProcessingEvent",
                        this.__id,
                        pending.event,
                        outputBuffer.__id,
                    );
                }
            }
            setTimeout(poll, 1);
        };
        setTimeout(poll, 0);
    }
    __scheduleWorkletPoll() {
        if (this.__workletPollScheduled || this.__workletNodes.size === 0) return;
        this.__workletPollScheduled = true;
        const poll = () => {
            this.__drainWorkletMessages();
            if (this.state === "closed" || this.__workletNodes.size === 0) {
                this.__workletPollScheduled = false;
                return;
            }
            setTimeout(poll, 1);
        };
        setTimeout(poll, 0);
    }
    __drainWorkletMessages() {
        const messages = JSON.parse(call("audioTakeWorkletMessages", this.__id));
        for (const message of messages) {
            const node = this.__workletNodes.get(message.node);
            if (!node) continue;
            if (message.kind === "processorerror") {
                const event = new Event("processorerror");
                event.isTrusted = true;
                node.dispatchEvent(event);
            } else {
                node.port.__deliver(message.data);
            }
        }
    }
    __dispatchStateChange() {
        const event = new Event("statechange");
        event.isTrusted = true;
        this.dispatchEvent(event);
    }
    createOscillator() { return new OscillatorNode(construct, this, call("audioCreateNode", this.__id, "oscillator", 0)); }
    createDynamicsCompressor() { return new DynamicsCompressorNode(construct, this, call("audioCreateNode", this.__id, "compressor", 0)); }
    createGain() { return new GainNode(construct, this, call("audioCreateNode", this.__id, "gain", 0)); }
    createBiquadFilter() { return new BiquadFilterNode(construct, this, call("audioCreateNode", this.__id, "biquad", 0)); }
    createStereoPanner() { return new StereoPannerNode(construct, this, call("audioCreateNode", this.__id, "stereo-panner", 0)); }
    createBufferSource() { return new AudioBufferSourceNode(construct, this, call("audioCreateNode", this.__id, "buffer-source", 0)); }
    createAnalyser() { return new AnalyserNode(construct, this, call("audioCreateNode", this.__id, "analyser", 0)); }
    createScriptProcessor(bufferSize = 0, numberOfInputChannels = 2,
        numberOfOutputChannels = 2) {
        if (!(this instanceof AudioContext)) {
            throw new DOMException(
                "Offline ScriptProcessorNode rendering is not implemented",
                "NotSupportedError",
            );
        }
        const metadata = JSON.parse(call(
            "audioCreateScriptProcessor",
            this.__id,
            Number(bufferSize) >>> 0,
            Number(numberOfInputChannels) >>> 0,
            Number(numberOfOutputChannels) >>> 0,
        ));
        const node = new ScriptProcessorNode(
            construct, this, metadata.id, metadata.bufferSize);
        this.__processingNodes.set(metadata.id, node);
        this.__scheduleProcessingPoll();
        return node;
    }
    createConstantSource() { return new ConstantSourceNode(construct, this, call("audioCreateNode", this.__id, "constant-source", 0)); }
    createDelay(maxDelayTime = 1) {
        maxDelayTime = Number(maxDelayTime);
        return new DelayNode(construct, this, call("audioCreateNode", this.__id, "delay", maxDelayTime), maxDelayTime);
    }
    createWaveShaper() { return new WaveShaperNode(construct, this, call("audioCreateNode", this.__id, "wave-shaper", 0)); }
    createChannelSplitter(numberOfOutputs = 6) {
        numberOfOutputs = Number(numberOfOutputs);
        return new ChannelSplitterNode(construct, this, call("audioCreateNode", this.__id, "channel-splitter", numberOfOutputs), numberOfOutputs);
    }
    createChannelMerger(numberOfInputs = 6) {
        numberOfInputs = Number(numberOfInputs);
        return new ChannelMergerNode(construct, this, call("audioCreateNode", this.__id, "channel-merger", numberOfInputs), numberOfInputs);
    }
    createConvolver() { return new ConvolverNode(construct, this, call("audioCreateNode", this.__id, "convolver", 0)); }
    createPanner() { return new PannerNode(construct, this, call("audioCreateNode", this.__id, "panner", 0)); }
    createIIRFilter(feedforward, feedback) {
        const ff = Float64Array.from(feedforward);
        const fb = Float64Array.from(feedback);
        return new IIRFilterNode(construct, this, call("audioCreateIirFilter", this.__id, ff, fb));
    }
    createPeriodicWave(real, imag, constraints = {}) {
        const realCoefficients = Float32Array.from(real);
        const imagCoefficients = Float32Array.from(imag);
        const id = call("audioCreatePeriodicWave", this.__id, realCoefficients, imagCoefficients, Boolean(constraints.disableNormalization));
        return new PeriodicWave(construct, this.__id, id);
    }
    createBuffer(numberOfChannels, length, sampleRate) {
        const metadata = JSON.parse(call("audioCreateBuffer", this.__id, Number(numberOfChannels) >>> 0, Number(length) >>> 0, Number(sampleRate)));
        return new AudioBuffer(construct, this.__id, metadata);
    }
    decodeAudioData(audioData, successCallback = undefined, errorCallback = undefined) {
        if (!(audioData instanceof ArrayBuffer)) throw new TypeError("audioData must be an ArrayBuffer");
        try {
            const buffer = new AudioBuffer(construct, this.__id, JSON.parse(call("audioDecode", this.__id, new Uint8Array(audioData))));
            return Promise.resolve().then(() => { successCallback?.(buffer); return buffer; });
        } catch (error) {
            return Promise.resolve().then(() => { errorCallback?.(error); throw error; });
        }
    }
}

class OfflineAudioContext extends BaseAudioContext {
    constructor(numberOfChannels, length, sampleRate) {
        if (typeof numberOfChannels === "object" && numberOfChannels !== null) {
            ({ numberOfChannels, length, sampleRate } = numberOfChannels);
        }
        numberOfChannels = Number(numberOfChannels) >>> 0;
        length = Number(length) >>> 0;
        sampleRate = Number(sampleRate);
        const id = call("audioCreateOffline", numberOfChannels, length, sampleRate);
        super(construct, id, numberOfChannels, sampleRate);
        Object.defineProperties(this, {
            length: { value: length, enumerable: true },
            currentTime: { value: 0, writable: true, enumerable: true },
            state: { value: "suspended", writable: true, enumerable: true },
            oncomplete: { value: null, writable: true, enumerable: true },
            onstatechange: { value: null, writable: true, enumerable: true },
            __renderStarted: { value: false, writable: true },
            __activeSuspension: { value: false, writable: true },
        });
    }
    startRendering() {
        if (this.__renderStarted) return Promise.reject(new DOMException("Rendering already started", "InvalidStateError"));
        try {
            call("audioBeginRender", this.__id);
        } catch (error) {
            return Promise.reject(new DOMException(String(error?.message ?? error), "InvalidStateError"));
        }
        this.__renderStarted = true;
        this.state = "running";
        queueMicrotask(() => this.__dispatchStateChange());
        return new Promise((resolve, reject) => {
            const poll = () => {
                try {
                    const encoded = call("audioPollRender", this.__id);
                    if (encoded === null) {
                        setTimeout(poll, 0);
                        return;
                    }
                    const buffer = new AudioBuffer(construct, this.__id, JSON.parse(encoded));
                    this.currentTime = this.length / this.sampleRate;
                    this.state = "closed";
                    this.__drainEndedEvents();
                    this.__drainWorkletMessages();
                    queueMicrotask(() => this.__dispatchStateChange());
                    queueMicrotask(() => {
                        const event = new OfflineAudioCompletionEvent("complete", { renderedBuffer: buffer });
                        event.isTrusted = true;
                        this.dispatchEvent(event);
                    });
                    resolve(buffer);
                } catch (error) {
                    this.state = "closed";
                    reject(error);
                }
            };
            queueMicrotask(poll);
        });
    }
    suspend(suspendTime) {
        let suspension;
        try {
            suspension = call("audioScheduleSuspend", this.__id, Number(suspendTime));
        } catch (error) {
            return Promise.reject(new DOMException(String(error?.message ?? error), "InvalidStateError"));
        }
        return new Promise((resolve, reject) => {
            const poll = () => {
                try {
                    const currentTime = call("audioPollSuspend", this.__id, suspension);
                    if (currentTime === null) {
                        setTimeout(poll, 0);
                        return;
                    }
                    this.currentTime = Number(currentTime);
                    this.state = "suspended";
                    this.__activeSuspension = true;
                    queueMicrotask(() => this.__dispatchStateChange());
                    resolve();
                } catch (error) {
                    reject(error);
                }
            };
            poll();
        });
    }
    resume() {
        if (!this.__activeSuspension) {
            return Promise.reject(new DOMException("Offline rendering is not suspended", "InvalidStateError"));
        }
        try {
            call("audioBeginResume", this.__id);
        } catch (error) {
            return Promise.reject(new DOMException(String(error?.message ?? error), "InvalidStateError"));
        }
        this.__activeSuspension = false;
        this.state = "running";
        queueMicrotask(() => this.__dispatchStateChange());
        return new Promise((resolve, reject) => {
            const poll = () => {
                try {
                    if (!call("audioPollResume", this.__id)) {
                        setTimeout(poll, 0);
                        return;
                    }
                    resolve();
                } catch (error) {
                    reject(error);
                }
            };
            poll();
        });
    }
}

class AudioContext extends BaseAudioContext {
    constructor(options = {}) {
        const requestedRate = options.sampleRate === undefined ? 0 : Number(options.sampleRate);
        const requestedSink = audioSinkRequest(options.sinkId, hardwareOutputEnabled);
        const metadata = JSON.parse(call("audioCreateRealtime", requestedRate, requestedSink.native));
        super(construct, metadata.id, 2, metadata.sampleRate);
        Object.defineProperties(this, {
            __baseLatency: { value: metadata.baseLatency, writable: true },
            __sinkId: { value: metadata.sinkId === "none" ? new AudioSinkInfo(construct) : metadata.sinkId, writable: true },
            __playbackStats: { value: new AudioPlaybackStats(construct, metadata.id) },
            __lastOutputTimestamp: { value: { currentTime: 0, contextTime: 0, performanceTime: 0 }, writable: true },
            onstatechange: { value: null, writable: true, enumerable: true },
            onsinkchange: { value: null, writable: true, enumerable: true },
        });
    }
    get baseLatency() { return this.__baseLatency; }
    get outputLatency() { return JSON.parse(call("audioRealtimeState", this.__id)).outputLatency; }
    get sinkId() { return this.__sinkId; }
    get playbackStats() { return this.__playbackStats; }
    createMediaStreamDestination() {
        return new MediaStreamAudioDestinationNode(
            construct,
            this,
            call("audioCreateNode", this.__id, "media-stream-destination", 0),
        );
    }
    createMediaStreamSource(stream) {
        const track = mediaStreamAudioTrack(stream);
        return new MediaStreamAudioSourceNode(construct, this, {
            stream,
            id: call("audioCreateMediaStreamSource", this.__id, track.__context, track.__node, false),
        });
    }
    createMediaStreamTrackSource(track) {
        if (!(track instanceof MediaStreamTrack) || track.kind !== "audio") {
            throw new TypeError("track must be an audio MediaStreamTrack");
        }
        return new MediaStreamTrackAudioSourceNode(construct, this, {
            track,
            id: call("audioCreateMediaStreamSource", this.__id, track.__context, track.__node, true),
        });
    }
    createMediaElementSource(element) {
        if (!(element instanceof HTMLMediaElement)) {
            throw new TypeError("element must be an HTMLMediaElement");
        }
        if (mediaElementSources.has(element)) {
            throw new DOMException("The media element already has a source node", "InvalidStateError");
        }
        return new MediaElementAudioSourceNode(construct, this, {
            element,
            id: call("audioCreateNode", this.__id, "gain", 0),
        });
    }
    get currentTime() { return JSON.parse(call("audioRealtimeState", this.__id)).currentTime; }
    get state() { return JSON.parse(call("audioRealtimeState", this.__id)).state; }
    getOutputTimestamp() {
        const snapshot = JSON.parse(call("audioRealtimeState", this.__id));
        if (snapshot.currentTime === 0) return { contextTime: 0, performanceTime: 0 };
        if (snapshot.currentTime > this.__lastOutputTimestamp.currentTime) {
            const renderQuantum = 128 / this.sampleRate;
            this.__lastOutputTimestamp = {
                currentTime: snapshot.currentTime,
                contextTime: Math.max(0, snapshot.currentTime - Math.max(renderQuantum, snapshot.outputLatency)),
                performanceTime: performance.now(),
            };
        }
        return {
            contextTime: this.__lastOutputTimestamp.contextTime,
            performanceTime: this.__lastOutputTimestamp.performanceTime,
        };
    }
    __control(operation) {
        try {
            call("audioControlRealtime", this.__id, operation);
            queueMicrotask(() => this.__dispatchStateChange());
            return Promise.resolve();
        } catch (error) { return Promise.reject(error); }
    }
    suspend() { return this.__control("suspend"); }
    resume() { return this.__control("resume"); }
    close() { return this.__control("close"); }
}

Object.defineProperties(globalThis, {
    OfflineAudioContext: { value: OfflineAudioContext, writable: true, configurable: true },
    webkitOfflineAudioContext: { value: OfflineAudioContext, writable: true, configurable: true },
    AudioContext: { value: AudioContext, writable: true, configurable: true },
    webkitAudioContext: { value: AudioContext, writable: true, configurable: true },
    BaseAudioContext: { value: BaseAudioContext, writable: true, configurable: true },
    AudioSinkInfo: { value: AudioSinkInfo, writable: true, configurable: true },
    AudioPlaybackStats: { value: AudioPlaybackStats, writable: true, configurable: true },
    MediaStream: { value: MediaStream, writable: true, configurable: true },
    MediaStreamTrack: { value: MediaStreamTrack, writable: true, configurable: true },
    AudioBuffer: { value: AudioBuffer, writable: true, configurable: true },
    AudioProcessingEvent: { value: AudioProcessingEvent, writable: true, configurable: true },
    OfflineAudioCompletionEvent: { value: OfflineAudioCompletionEvent, writable: true, configurable: true },
    AudioNode: { value: AudioNode, writable: true, configurable: true },
    AudioScheduledSourceNode: { value: AudioScheduledSourceNode, writable: true, configurable: true },
    AudioParam: { value: AudioParam, writable: true, configurable: true },
    AudioDestinationNode: { value: AudioDestinationNode, writable: true, configurable: true },
    ScriptProcessorNode: { value: ScriptProcessorNode, writable: true, configurable: true },
    AudioWorkletNode: { value: AudioWorkletNode, writable: true, configurable: true },
    AudioWorklet: { value: AudioWorklet, writable: true, configurable: true },
    MediaStreamAudioDestinationNode: { value: MediaStreamAudioDestinationNode, writable: true, configurable: true },
    MediaStreamAudioSourceNode: { value: MediaStreamAudioSourceNode, writable: true, configurable: true },
    MediaStreamTrackAudioSourceNode: { value: MediaStreamTrackAudioSourceNode, writable: true, configurable: true },
    MediaElementAudioSourceNode: { value: MediaElementAudioSourceNode, writable: true, configurable: true },
    OscillatorNode: { value: OscillatorNode, writable: true, configurable: true },
    DynamicsCompressorNode: { value: DynamicsCompressorNode, writable: true, configurable: true },
    GainNode: { value: GainNode, writable: true, configurable: true },
    BiquadFilterNode: { value: BiquadFilterNode, writable: true, configurable: true },
    StereoPannerNode: { value: StereoPannerNode, writable: true, configurable: true },
    AudioBufferSourceNode: { value: AudioBufferSourceNode, writable: true, configurable: true },
    AnalyserNode: { value: AnalyserNode, writable: true, configurable: true },
    ConstantSourceNode: { value: ConstantSourceNode, writable: true, configurable: true },
    DelayNode: { value: DelayNode, writable: true, configurable: true },
    WaveShaperNode: { value: WaveShaperNode, writable: true, configurable: true },
    ChannelSplitterNode: { value: ChannelSplitterNode, writable: true, configurable: true },
    ChannelMergerNode: { value: ChannelMergerNode, writable: true, configurable: true },
    ConvolverNode: { value: ConvolverNode, writable: true, configurable: true },
    IIRFilterNode: { value: IIRFilterNode, writable: true, configurable: true },
    PannerNode: { value: PannerNode, writable: true, configurable: true },
    AudioListener: { value: AudioListener, writable: true, configurable: true },
    PeriodicWave: { value: PeriodicWave, writable: true, configurable: true },
});

for (const constructor of [BaseAudioContext, OfflineAudioContext, AudioContext, AudioSinkInfo, AudioPlaybackStats, MediaStream, MediaStreamTrack, AudioBuffer, AudioProcessingEvent, OfflineAudioCompletionEvent, AudioNode, AudioScheduledSourceNode, AudioParam, AudioDestinationNode, ScriptProcessorNode, AudioWorkletNode, AudioWorklet, AudioWorkletMessagePort, MediaStreamAudioDestinationNode, MediaStreamAudioSourceNode, MediaStreamTrackAudioSourceNode, MediaElementAudioSourceNode, OscillatorNode, DynamicsCompressorNode, GainNode, BiquadFilterNode, StereoPannerNode, PannerNode, AudioBufferSourceNode, AnalyserNode, ConstantSourceNode, DelayNode, WaveShaperNode, ChannelSplitterNode, ChannelMergerNode, ConvolverNode, IIRFilterNode, AudioListener, PeriodicWave]) {
    globalThis.__brimpMarkWebBuiltin?.(constructor);
    for (const key of Reflect.ownKeys(constructor.prototype)) {
        if (key === "constructor" || String(key).startsWith("__")) continue;
        const descriptor = Object.getOwnPropertyDescriptor(constructor.prototype, key);
        if (typeof descriptor?.value === "function") globalThis.__brimpMarkWebBuiltin?.(descriptor.value, `function ${String(key)}() { [native code] }`);
        if (typeof descriptor?.get === "function") globalThis.__brimpMarkWebBuiltin?.(descriptor.get, `function get ${String(key)}() { [native code] }`);
        if (typeof descriptor?.set === "function") globalThis.__brimpMarkWebBuiltin?.(descriptor.set, `function set ${String(key)}() { [native code] }`);
    }
}
})();
