function installAudio({ global, host, profile: settings }, options = {}) {
  class SpeechSynthesisUtterance extends EventTarget {
    constructor(text = '') {
      super();
      this.text = text;
    }
  }
  // Local utterance payloads only: no speech queue, voices or synthesis engine.
  for (const [name, initial] of Object.entries({
    text: '',
    lang: '',
    pitch: 1,
    rate: 1,
    volume: 1,
    voice: null,
  })) {
    const values = new WeakMap();
    Object.defineProperty(SpeechSynthesisUtterance.prototype, name, {
      get() {
        return values.has(this) ? values.get(this) : initial;
      },
      set(value) {
        values.set(
          this,
          initial === null ? value : typeof initial === 'string' ? String(value) : Number(value),
        );
      },
      enumerable: true,
      configurable: true,
    });
  }
  for (const type of ['boundary', 'end', 'error', 'mark', 'pause', 'resume', 'start']) {
    const handlers = new WeakMap();
    Object.defineProperty(SpeechSynthesisUtterance.prototype, 'on' + type, {
      get() {
        return handlers.get(this) ?? null;
      },
      set(value) {
        handlers.set(this, typeof value === 'function' ? value : null);
      },
      enumerable: true,
      configurable: true,
    });
  }
  global.SpeechSynthesisUtterance = SpeechSynthesisUtterance;
  const audioProfile = settings.values(
    'audio',
    {
      sample_rate: 44100,
      max_channel_count: 2,
      output_latency: 0.01,
    },
    {
      version: 1,
      fields: {
        sample_rate: {
          minimum: 8000,
          maximum: 192000,
        },
        max_channel_count: {
          integer: true,
          minimum: 1,
          maximum: 32,
        },
        output_latency: {
          minimum: 0,
        },
      },
      capture: async ({ read }) => {
        const Constructor = globalThis.AudioContext ?? globalThis.webkitAudioContext;
        if (!Constructor)
          return {
            sample_rate: read(() => undefined),
            max_channel_count: read(() => undefined),
            output_latency: read(() => undefined),
          };
        const audio = new Constructor();
        try {
          return {
            sample_rate: read(() => audio.sampleRate),
            max_channel_count: read(() => audio.destination.maxChannelCount),
            output_latency: read(() => audio.outputLatency),
          };
        } finally {
          await audio.close();
        }
      },
    },
  );

  const contexts = new Set();
  const construct = Symbol('audio interface construction');
  const validRate = (value) => {
    value = Number(value);
    if (!Number.isFinite(value) || value < 8000 || value > 192000)
      throw new DOMException('Unsupported sample rate', 'NotSupportedError');
    return value;
  };
  const validateBufferSize = (length, channels) => {
    if (
      !Number.isInteger(length) ||
      length <= 0 ||
      !Number.isInteger(channels) ||
      channels < 1 ||
      channels > 32 ||
      length * channels > 16_000_000
    )
      throw new DOMException('Audio buffer size limit', 'NotSupportedError');
  };
  class AudioBuffer {
    #channels;
    constructor({ length, numberOfChannels = 1, sampleRate }) {
      validateBufferSize(length, numberOfChannels);
      Object.defineProperties(this, {
        length: { value: length },
        sampleRate: { value: validRate(sampleRate) },
        numberOfChannels: { value: numberOfChannels },
      });
      this.#channels = Array.from({ length: numberOfChannels }, () => new Float32Array(length));
    }
    get duration() {
      return this.length / this.sampleRate;
    }
    getChannelData(channel) {
      if (!Number.isInteger(channel) || !this.#channels[channel])
        throw new DOMException('Invalid channel', 'IndexSizeError');
      return this.#channels[channel];
    }
    copyFromChannel(destination, channel, start = 0) {
      if (!(destination instanceof Float32Array)) throw new TypeError('Float32Array required');
      if (!Number.isInteger(start) || start < 0) throw new RangeError('Invalid offset');
      destination.set(this.getChannelData(channel).subarray(start, start + destination.length));
    }
    copyToChannel(source, channel, start = 0) {
      if (!(source instanceof Float32Array)) throw new TypeError('Float32Array required');
      const destination = this.getChannelData(channel);
      if (!Number.isInteger(start) || start < 0) throw new RangeError('Invalid offset');
      if (start < destination.length)
        destination.set(source.subarray(0, destination.length - start), start);
    }
  }
  class AudioParam {
    constructor(value) {
      this.value = value;
      this.defaultValue = value;
      this.minValue = -3.4028234663852886e38;
      this.maxValue = 3.4028234663852886e38;
      this.automationRate = 'a-rate';
    }
    setValueAtTime(value, time) {
      if (!Number.isFinite(Number(value)) || !Number.isFinite(Number(time)) || time < 0)
        throw new RangeError('Invalid audio parameter');
      this.value = Number(value);
      return this;
    }
    cancelScheduledValues() {
      return this;
    }
    cancelAndHoldAtTime() {
      return this;
    }
    // Legacy callable placeholders: automation does not schedule processing.
    linearRampToValueAtTime() {}
    exponentialRampToValueAtTime() {}
    setTargetAtTime() {}
    setValueCurveAtTime() {}
  }
  class AudioNode extends EventTarget {
    constructor(context) {
      super();
      if (!(context instanceof BaseAudioContext)) throw new TypeError('BaseAudioContext required');
      this.context = context;
      this.channelCount = 2;
      this.channelCountMode = 'max';
      this.channelInterpretation = 'speakers';
      this.numberOfInputs = 1;
      this.numberOfOutputs = 1;
      this._connections = new Set();
    }
    connect(destination) {
      if (this.context.state === 'closed')
        throw new DOMException('Context closed', 'InvalidStateError');
      if (!(destination instanceof AudioNode) && !(destination instanceof AudioParam))
        throw new TypeError('Invalid destination');
      if (destination.context && destination.context !== this.context)
        throw new DOMException('Different context', 'InvalidAccessError');
      this._connections.add(destination);
      return destination instanceof AudioNode ? destination : undefined;
    }
    disconnect() {
      this._connections.clear();
    }
  }
  class GainNode extends AudioNode {
    constructor(context, options = {}) {
      super(context);
      this.gain = new AudioParam(options.gain ?? 1);
    }
  }
  class AudioScheduledSourceNode extends AudioNode {
    constructor(context) {
      super(context);
      this.onended = null;
    }
    start(when = 0) {
      if (this._started) throw new DOMException('Already started', 'InvalidStateError');
      if (!Number.isFinite(Number(when)) || when < 0) throw new RangeError('Invalid start');
      this._started = true;
    }
    stop(when = 0) {
      if (!this._started) throw new DOMException('Not started', 'InvalidStateError');
      if (!Number.isFinite(Number(when)) || when < 0) throw new RangeError('Invalid stop');
      if (this._timer !== undefined) clearTimeout(this._timer);
      this._timer = setTimeout(
        () => {
          this.dispatchEvent(new Event('ended'));
          if (typeof this.onended === 'function') this.onended(new Event('ended'));
        },
        Math.max(0, (when - this.context.currentTime) * 1000),
      );
      this.context._timers.add(this._timer);
    }
  }
  class OscillatorNode extends AudioScheduledSourceNode {
    constructor(context, options = {}) {
      super(context);
      this.frequency = new AudioParam(options.frequency ?? 440);
      this.detune = new AudioParam(options.detune ?? 0);
      this.type = options.type ?? 'sine';
      this.numberOfInputs = 0;
    }
    setPeriodicWave() {}
  }
  class AudioDestinationNode extends AudioNode {
    constructor(context) {
      super(context);
      this.numberOfOutputs = 0;
      this.maxChannelCount = audioProfile.max_channel_count;
    }
  }
  class AudioListener {
    constructor(token) {
      if (token !== construct) throw new TypeError('Illegal constructor');
      for (const [name, value] of Object.entries({
        positionX: 0,
        positionY: 0,
        positionZ: 0,
        forwardX: 0,
        forwardY: 0,
        forwardZ: -1,
        upX: 0,
        upY: 1,
        upZ: 0,
      }))
        this[name] = new AudioParam(value);
    }
    setPosition(x, y, z) {
      this.positionX.value = Number(x);
      this.positionY.value = Number(y);
      this.positionZ.value = Number(z);
    }
    setOrientation(x, y, z, upX, upY, upZ) {
      for (const [name, value] of Object.entries({
        forwardX: x,
        forwardY: y,
        forwardZ: z,
        upX,
        upY,
        upZ,
      }))
        this[name].value = Number(value);
    }
  }
  class DynamicsCompressorNode extends AudioNode {
    constructor(context, options = {}) {
      super(context);
      this.channelCountMode = 'clamped-max';
      this.reduction = 0;
      for (const [name, value, min, max] of [
        ['attack', 0.003, 0, 1],
        ['knee', 30, 0, 40],
        ['ratio', 12, 1, 20],
        ['release', 0.25, 0, 1],
        ['threshold', -24, -100, 0],
      ]) {
        const param = new AudioParam(value);
        param.value = options[name] ?? value;
        param.minValue = min;
        param.maxValue = max;
        param.automationRate = 'k-rate';
        this[name] = param;
      }
    }
  }
  class Worklet {
    constructor(token) {
      if (token !== construct) throw new TypeError('Illegal constructor');
    }
    addModule() {}
  }
  class AudioWorklet extends Worklet {}
  class AnalyserNode extends AudioNode {
    #fftSize = 2048;
    get fftSize() {
      return this.#fftSize;
    }
    set fftSize(v) {
      v = Number(v);
      if (!Number.isInteger(v) || v < 32 || v > 32768 || v & (v - 1))
        throw new DOMException('Invalid FFT size', 'IndexSizeError');
      this.#fftSize = v;
    }
    get frequencyBinCount() {
      return this.fftSize / 2;
    }
    getFloatFrequencyData(array) {
      array.fill(-Infinity, 0, Math.min(array.length, this.frequencyBinCount));
    }
    getByteFrequencyData(array) {
      array.fill(0, 0, Math.min(array.length, this.frequencyBinCount));
    }
    getFloatTimeDomainData(array) {
      array.fill(0, 0, Math.min(array.length, this.fftSize));
    }
    getByteTimeDomainData(array) {
      array.fill(128, 0, Math.min(array.length, this.fftSize));
    }
  }
  class BaseAudioContext extends EventTarget {
    #state = 'suspended';
    #elapsed = 0;
    #started = 0;
    constructor(config = {}) {
      super();
      if (new.target === BaseAudioContext) throw new TypeError('Illegal constructor');
      this.sampleRate = validRate(
        config.sampleRate ?? options.sampleRate ?? audioProfile.sample_rate ?? 48000,
      );
      this.baseLatency = 0;
      this.outputLatency = Number(audioProfile.output_latency ?? 0);
      this.destination = new AudioDestinationNode(this);
      this.listener = new AudioListener(construct);
      this.audioWorklet = new AudioWorklet(construct);
      this.onstatechange = null;
      this._timers = new Set();
    }
    get state() {
      return this.#state;
    }
    get currentTime() {
      return (
        this.#elapsed + (this.#state === 'running' ? (performance.now() - this.#started) / 1000 : 0)
      );
    }
    _changeState(state) {
      if (this.#state === 'closed') throw new DOMException('Context closed', 'InvalidStateError');
      this.#elapsed = this.currentTime;
      this.#started = performance.now();
      this.#state = state;
      this.dispatchEvent(new Event('statechange'));
      if (typeof this.onstatechange === 'function') this.onstatechange(new Event('statechange'));
    }
    createBuffer(channels, length, sampleRate) {
      return new AudioBuffer({ numberOfChannels: channels, length, sampleRate });
    }
    createGain() {
      return new GainNode(this);
    }
    createOscillator() {
      return new OscillatorNode(this);
    }
    createAnalyser() {
      return new AnalyserNode(this);
    }
    createDynamicsCompressor() {
      return new DynamicsCompressorNode(this);
    }
    // The remaining legacy factories are callable without creating engines.
    createBiquadFilter() {}
    createBufferSource() {}
    createChannelMerger() {}
    createChannelSplitter() {}
    createConstantSource() {}
    createConvolver() {}
    createDelay() {}
    createIIRFilter() {}
    createPanner() {}
    createPeriodicWave() {}
    createScriptProcessor() {}
    createStereoPanner() {}
    createWaveShaper() {}
    async decodeAudioData() {
      throw new DOMException('Audio decoding unavailable', 'NotSupportedError');
    }
  }
  class AudioContext extends BaseAudioContext {
    constructor(config = {}) {
      super(config);
      contexts.add(this);
    }
    async resume() {
      this._changeState('running');
    }
    async suspend() {
      this._changeState('suspended');
    }
    async close() {
      this._changeState('closed');
      for (const timer of this._timers) clearTimeout(timer);
      this._timers.clear();
      contexts.delete(this);
    }
    processAsync(samples) {
      return host.call('audio.process', { sampleRate: this.sampleRate, samples });
    }
  }
  class OfflineAudioContext extends BaseAudioContext {
    #channels;
    constructor(channelsOrOptions, length, sampleRate) {
      const config =
        typeof channelsOrOptions === 'object'
          ? channelsOrOptions
          : { numberOfChannels: channelsOrOptions, length, sampleRate };
      super(config);
      this.#channels = config.numberOfChannels ?? 1;
      this.length = config.length;
      this.oncomplete = null;
      // Reuse buffer validation without introducing an offline engine.
      validateBufferSize(this.length, this.#channels);
      this.destination.channelCount = this.#channels;
    }
    // Legacy offline controls return promises that never complete.
    resume() {
      return new Promise(() => {});
    }
    suspend() {
      return new Promise(() => {});
    }
    startRendering() {
      return Promise.resolve(
        new AudioBuffer({
          numberOfChannels: this.#channels,
          length: this.length,
          sampleRate: this.sampleRate,
        }),
      );
    }
  }
  const speechProfile = settings.values(
    'speech',
    {
      // Safari 27.0's installed voices, as `speechSynthesis.getVoices()` reports
      // them; `brimp profile capture` replaces the list for another machine.
      voices: [
        { voiceURI: "com.apple.voice.super-compact.en-US.Samantha", name: "Samantha", lang: "en-US", localService: true, default: true },
        { voiceURI: "com.apple.speech.synthesis.voice.Albert", name: "Albert", lang: "en-US", localService: true, default: true },
        { voiceURI: "com.apple.speech.synthesis.voice.BadNews", name: "Bad News", lang: "en-US", localService: true, default: true },
        { voiceURI: "com.apple.speech.synthesis.voice.Bahh", name: "Bahh", lang: "en-US", localService: true, default: true },
        { voiceURI: "com.apple.speech.synthesis.voice.Bells", name: "Bells", lang: "en-US", localService: true, default: true },
        { voiceURI: "com.apple.speech.synthesis.voice.Boing", name: "Boing", lang: "en-US", localService: true, default: true },
        { voiceURI: "com.apple.speech.synthesis.voice.Bubbles", name: "Bubbles", lang: "en-US", localService: true, default: true },
        { voiceURI: "com.apple.speech.synthesis.voice.Cellos", name: "Cellos", lang: "en-US", localService: true, default: true },
        { voiceURI: "com.apple.speech.synthesis.voice.Deranged", name: "Wobble", lang: "en-US", localService: true, default: true },
        { voiceURI: "com.apple.speech.synthesis.voice.Fred", name: "Fred", lang: "en-US", localService: true, default: true },
        { voiceURI: "com.apple.speech.synthesis.voice.GoodNews", name: "Good News", lang: "en-US", localService: true, default: true },
        { voiceURI: "com.apple.speech.synthesis.voice.Hysterical", name: "Jester", lang: "en-US", localService: true, default: true },
        { voiceURI: "com.apple.speech.synthesis.voice.Junior", name: "Junior", lang: "en-US", localService: true, default: true },
        { voiceURI: "com.apple.speech.synthesis.voice.Kathy", name: "Kathy", lang: "en-US", localService: true, default: true },
        { voiceURI: "com.apple.speech.synthesis.voice.Organ", name: "Organ", lang: "en-US", localService: true, default: true },
        { voiceURI: "com.apple.speech.synthesis.voice.Princess", name: "Superstar", lang: "en-US", localService: true, default: true },
        { voiceURI: "com.apple.speech.synthesis.voice.Ralph", name: "Ralph", lang: "en-US", localService: true, default: true },
        { voiceURI: "com.apple.speech.synthesis.voice.Trinoids", name: "Trinoids", lang: "en-US", localService: true, default: true },
        { voiceURI: "com.apple.speech.synthesis.voice.Whisper", name: "Whisper", lang: "en-US", localService: true, default: true },
        { voiceURI: "com.apple.speech.synthesis.voice.Zarvox", name: "Zarvox", lang: "en-US", localService: true, default: true },
        { voiceURI: "com.apple.voice.super-compact.en-GB.Daniel", name: "Daniel", lang: "en-GB", localService: true, default: true },
        { voiceURI: "com.apple.voice.super-compact.en-AU.Karen", name: "Karen", lang: "en-AU", localService: true, default: true },
        { voiceURI: "com.apple.voice.super-compact.en-IE.Moira", name: "Moira", lang: "en-IE", localService: true, default: true },
        { voiceURI: "com.apple.voice.super-compact.en-IN.Rishi", name: "Rishi", lang: "en-IN", localService: true, default: true },
        { voiceURI: "com.apple.voice.super-compact.en-ZA.Tessa", name: "Tessa", lang: "en-ZA", localService: true, default: true },
        { voiceURI: "com.apple.voice.super-compact.it-IT.Alice", name: "Alice", lang: "it-IT", localService: true, default: true },
        { voiceURI: "com.apple.voice.super-compact.sv-SE.Alva", name: "Alva", lang: "sv-SE", localService: true, default: true },
        { voiceURI: "com.apple.voice.super-compact.fr-CA.Amelie", name: "Am\u00e9lie", lang: "fr-CA", localService: true, default: true },
        { voiceURI: "com.apple.voice.super-compact.ms-MY.Amira", name: "Amira", lang: "ms-MY", localService: true, default: true },
        { voiceURI: "com.apple.voice.super-compact.de-DE.Anna", name: "Anna", lang: "de-DE", localService: true, default: true },
        { voiceURI: "com.apple.voice.super-compact.he-IL.Carmit", name: "Carmit", lang: "he-IL", localService: true, default: true },
        { voiceURI: "com.apple.voice.super-compact.id-ID.Damayanti", name: "Damayanti", lang: "id-ID", localService: true, default: true },
        { voiceURI: "com.apple.voice.super-compact.bg-BG.Daria", name: "Daria", lang: "bg-BG", localService: true, default: true },
        { voiceURI: "com.apple.voice.super-compact.nl-BE.Ellen", name: "Ellen", lang: "nl-BE", localService: true, default: true },
        { voiceURI: "com.apple.voice.super-compact.ro-RO.Ioana", name: "Ioana", lang: "ro-RO", localService: true, default: true },
        { voiceURI: "com.apple.voice.super-compact.pt-PT.Joana", name: "Joana", lang: "pt-PT", localService: true, default: true },
        { voiceURI: "com.apple.voice.super-compact.th-TH.Kanya", name: "Kanya", lang: "th-TH", localService: true, default: true },
        { voiceURI: "com.apple.voice.super-compact.ja-JP.Kyoko", name: "Kyoko", lang: "ja-JP", localService: true, default: true },
        { voiceURI: "com.apple.voice.super-compact.hr-HR.Lana", name: "Lana", lang: "hr-HR", localService: true, default: true },
        { voiceURI: "com.apple.voice.super-compact.sk-SK.Laura", name: "Laura", lang: "sk-SK", localService: true, default: true },
        { voiceURI: "com.apple.voice.super-compact.hi-IN.Lekha", name: "Lekha", lang: "hi-IN", localService: true, default: true },
        { voiceURI: "com.apple.voice.super-compact.uk-UA.Lesya", name: "Lesya", lang: "uk-UA", localService: true, default: true },
        { voiceURI: "com.apple.voice.super-compact.vi-VN.Linh", name: "Linh", lang: "vi-VN", localService: true, default: true },
        { voiceURI: "com.apple.voice.super-compact.pt-BR.Luciana", name: "Luciana", lang: "pt-BR", localService: true, default: true },
        { voiceURI: "com.apple.voice.super-compact.ar-001.Maged", name: "Majed", lang: "ar-001", localService: true, default: true },
        { voiceURI: "com.apple.voice.super-compact.hu-HU.Mariska", name: "T\u00fcnde", lang: "hu-HU", localService: true, default: true },
        { voiceURI: "com.apple.voice.super-compact.zh-TW.Meijia", name: "Meijia", lang: "zh-TW", localService: true, default: true },
        { voiceURI: "com.apple.voice.super-compact.el-GR.Melina", name: "Melina", lang: "el-GR", localService: true, default: true },
        { voiceURI: "com.apple.voice.super-compact.ru-RU.Milena", name: "Milena", lang: "ru-RU", localService: true, default: true },
        { voiceURI: "com.apple.voice.super-compact.es-ES.Monica", name: "M\u00f3nica", lang: "es-ES", localService: true, default: true },
        { voiceURI: "com.apple.voice.super-compact.ca-ES.Montserrat", name: "Montse", lang: "ca-ES", localService: true, default: true },
        { voiceURI: "com.apple.voice.super-compact.nb-NO.Nora", name: "Nora", lang: "nb-NO", localService: true, default: true },
        { voiceURI: "com.apple.voice.super-compact.es-MX.Paulina", name: "Paulina", lang: "es-MX", localService: true, default: true },
        { voiceURI: "com.apple.voice.super-compact.da-DK.Sara", name: "Sara", lang: "da-DK", localService: true, default: true },
        { voiceURI: "com.apple.voice.super-compact.fi-FI.Satu", name: "Satu", lang: "fi-FI", localService: true, default: true },
        { voiceURI: "com.apple.voice.super-compact.zh-HK.Sinji", name: "Sinji", lang: "yue-HK", localService: true, default: true },
        { voiceURI: "com.apple.voice.super-compact.fr-FR.Thomas", name: "Thomas", lang: "fr-FR", localService: true, default: true },
        { voiceURI: "com.apple.voice.super-compact.sl-SI.Tina", name: "Tina", lang: "sl-SI", localService: true, default: true },
        { voiceURI: "com.apple.voice.super-compact.zh-CN.Tingting", name: "Tingting", lang: "zh-CN", localService: true, default: true },
        { voiceURI: "com.apple.voice.super-compact.nl-NL.Xander", name: "Xander", lang: "nl-NL", localService: true, default: true },
        { voiceURI: "com.apple.voice.super-compact.tr-TR.Yelda", name: "Yelda", lang: "tr-TR", localService: true, default: true },
        { voiceURI: "com.apple.voice.super-compact.ko-KR.Yuna", name: "Yuna", lang: "ko-KR", localService: true, default: true },
        { voiceURI: "com.apple.voice.super-compact.pl-PL.Zosia", name: "Zosia", lang: "pl-PL", localService: true, default: true },
        { voiceURI: "com.apple.voice.super-compact.cs-CZ.Zuzana", name: "Zuzana", lang: "cs-CZ", localService: true, default: true },
        { voiceURI: "com.apple.voice.super-compact.ta-IN.Vani", name: "Vani", lang: "ta-IN", localService: true, default: true },
        { voiceURI: "com.apple.voice.compact.kn-IN.Alpana", name: "Soumya", lang: "kn-IN", localService: true, default: true },
        { voiceURI: "com.apple.voice.compact.te-IN.Geeta", name: "Geeta", lang: "te-IN", localService: true, default: true },
        { voiceURI: "com.apple.voice.compact.bn-IN.Paya", name: "Piya", lang: "bn-IN", localService: true, default: true },
      ],
    },
    {
      version: 1,
      fields: {
        voices: {
          items: {
            type: 'object',
            additionalProperties: {
              type: 'json',
            },
          },
        },
      },
      notes:
        'Installed voice list captured from the claimed browser. Speech output and the utterance queue remain local placeholders.',
      capture: async ({ read }) => {
        const synthesis = globalThis.speechSynthesis;
        const list = () =>
          Array.from(synthesis?.getVoices?.() ?? [], (voice) => ({
            voiceURI: voice.voiceURI,
            name: voice.name,
            lang: voice.lang,
            localService: voice.localService,
            default: voice.default,
          }));
        return {
          voices: await read(async () => {
            if (!synthesis) return undefined;
            if (synthesis.getVoices().length > 0) return list();
            // Safari publishes its installed voices after the first call.
            await new Promise((resolve) => {
              const timer = setTimeout(resolve, 1500);
              synthesis.addEventListener(
                'voiceschanged',
                () => {
                  clearTimeout(timer);
                  resolve();
                },
                { once: true },
              );
            });
            return list();
          }),
        };
      },
    },
  );

  const voiceToken = Symbol('SpeechSynthesisVoice construction');
  class SpeechSynthesisVoice {
    #voice;
    constructor(token, voice) {
      if (token !== voiceToken) throw new TypeError('Illegal constructor');
      this.#voice = voice;
    }
    get voiceURI() {
      return this.#voice.voiceURI;
    }
    get name() {
      return this.#voice.name;
    }
    get lang() {
      return this.#voice.lang;
    }
    get localService() {
      return this.#voice.localService;
    }
    get default() {
      return this.#voice.default;
    }
  }
  Object.defineProperty(SpeechSynthesisVoice.prototype, Symbol.toStringTag, {
    value: 'SpeechSynthesisVoice',
    configurable: true,
  });
  Object.defineProperty(global, 'SpeechSynthesisVoice', {
    value: SpeechSynthesisVoice,
    writable: true,
    configurable: true,
  });
  const installedVoices = speechProfile.voices.map(
    (voice) =>
      new SpeechSynthesisVoice(voiceToken, {
        voiceURI: String(voice.voiceURI ?? voice.name ?? ''),
        name: String(voice.name ?? ''),
        lang: String(voice.lang ?? ''),
        localService: Boolean(voice.localService ?? true),
        default: Boolean(voice.default ?? false),
      }),
  );
  // Silent engine: utterances are accepted and dropped. The captured voice list
  // is published on the first getVoices() call, as Safari does after startup.
  const speechSynthesisHandlers = new WeakMap();
  class SpeechSynthesis extends EventTarget {
    #published = false;
    #scheduled = false;
    constructor() {
      super();
    }
    get pending() {
      return false;
    }
    get speaking() {
      return false;
    }
    get paused() {
      return false;
    }
    speak() {}
    cancel() {}
    pause() {}
    resume() {}
    getVoices() {
      // The list is part of the profile, so it is already known: Safari returns
      // its installed voices immediately once they are loaded. `voiceschanged`
      // still fires once, for code that waits for it.
      if (!this.#scheduled) {
        this.#scheduled = true;
        setTimeout(() => {
          this.#published = true;
          this.dispatchEvent(new Event('voiceschanged'));
        }, 0);
      }
      return installedVoices.slice();
    }
  }
  Object.defineProperty(SpeechSynthesis.prototype, 'onvoiceschanged', {
    get() {
      return speechSynthesisHandlers.get(this) ?? null;
    },
    set(value) {
      speechSynthesisHandlers.set(this, typeof value === 'function' ? value : null);
    },
    enumerable: true,
    configurable: true,
  });
  Object.defineProperty(SpeechSynthesis.prototype, Symbol.toStringTag, {
    value: 'SpeechSynthesis',
    configurable: true,
  });
  Object.defineProperty(SpeechSynthesis.prototype, 'constructor', {
    value: SpeechSynthesis,
    writable: true,
    configurable: true,
  });
  Object.defineProperty(global, 'SpeechSynthesis', {
    value: SpeechSynthesis,
    writable: true,
    configurable: true,
  });
  Object.defineProperty(global, 'speechSynthesis', {
    value: new SpeechSynthesis(),
    configurable: true,
  });
  for (const [name, value] of Object.entries({
    BaseAudioContext,
    AudioContext,
    OfflineAudioContext,
    AudioBuffer,
    AudioParam,
    AudioNode,
    AudioScheduledSourceNode,
    AudioDestinationNode,
    AudioListener,
    DynamicsCompressorNode,
    Worklet,
    AudioWorklet,
    GainNode,
    OscillatorNode,
    AnalyserNode,
  }))
    Object.defineProperty(global, name, { value, writable: true, configurable: true });
  return () => Promise.all([...contexts].map((context) => context.close()));
}
