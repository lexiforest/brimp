function installMedia({ global, profile: settings }, options = {}) {
  const mediaProfile = settings.values(
    'media',
    {
      media_devices_enabled: true,
      devices: [],
      supported_constraints: {},
      decoding: [],
    },
    {
      version: 1,
      fields: {
        devices: {
          items: {
            type: 'object',
            additionalProperties: {
              type: 'string',
            },
          },
        },
        supported_constraints: {
          type: 'object',
          additionalProperties: {
            type: 'boolean',
          },
        },
        decoding: {
          items: {
            type: 'object',
            additionalProperties: {
              type: 'json',
            },
          },
        },
      },
      notes:
        'Device enumeration does not request capture permission. Labels may be empty. media_devices_enabled remains a runtime exposure choice.',
      capture: async ({ read }) => ({
        devices: await read(async () =>
          navigator.mediaDevices
            ? Array.from(await navigator.mediaDevices.enumerateDevices(), (d) => ({
                deviceId: d.deviceId,
                groupId: d.groupId,
                kind: d.kind,
                label: d.label,
              }))
            : undefined,
        ),
        supported_constraints: read(() => navigator.mediaDevices?.getSupportedConstraints()),
        decoding: await read(async () => {
          const capabilities = navigator.mediaCapabilities;
          if (!capabilities?.decodingInfo) return undefined;
          const configurations = [
            {
              type: 'file',
              video: {
                contentType: 'video/webm; codecs="vp9"',
                width: 1920,
                height: 1080,
                bitrate: 4000000,
                framerate: 30,
              },
            },
            {
              type: 'file',
              video: {
                contentType: 'video/mp4; codecs="avc1.42E01E"',
                width: 1920,
                height: 1080,
                bitrate: 4000000,
                framerate: 30,
              },
            },
            {
              type: 'file',
              video: {
                contentType: 'video/mp4; codecs="hvc1.1.6.L93.B0"',
                width: 1920,
                height: 1080,
                bitrate: 4000000,
                framerate: 30,
              },
            },
            {
              type: 'file',
              audio: {
                contentType: 'audio/mp4; codecs="mp4a.40.2"',
                channels: 2,
                bitrate: 128000,
                samplerate: 44100,
              },
            },
            {
              type: 'file',
              audio: {
                contentType: 'audio/webm; codecs="opus"',
                channels: 2,
                bitrate: 128000,
                samplerate: 48000,
              },
            },
            {
              type: 'media-source',
              audio: {
                contentType: 'audio/mp4; codecs="mp4a.40.2"',
                channels: 2,
                bitrate: 128000,
                samplerate: 44100,
              },
            },
            {
              type: 'webrtc',
              video: {
                contentType: 'video/VP8',
                width: 640,
                height: 480,
                bitrate: 1000000,
                framerate: 30,
              },
            },
          ];
          const results = [];
          for (const configuration of configurations) {
            try {
              const info = await capabilities.decodingInfo(configuration);
              results.push({
                ...configuration,
                supported: Boolean(info.supported),
                smooth: Boolean(info.smooth),
                powerEfficient: Boolean(info.powerEfficient),
              });
            } catch (error) {}
          }
          return results;
        }),
      }),
    },
  );
  const navigatorValues = { ...mediaProfile, ...options };
  const constructToken = Symbol('browser interface construction');
  const defineGlobalConstructor = (name, constructor) => {
    if (!Object.hasOwn(constructor.prototype, Symbol.toStringTag)) {
      Object.defineProperty(constructor.prototype, Symbol.toStringTag, {
        value: name,
        configurable: true,
      });
    }
    Object.defineProperty(global, name, {
      value: constructor,
      writable: true,
      configurable: true,
    });
  };
  const defineNavigatorValue = (name, value) => {
    const getter = function () {
      return value;
    };
    Object.defineProperty(getter, 'name', { value: `get ${name}`, configurable: true });
    Object.defineProperty(Navigator.prototype, name, {
      get: getter,
      enumerable: true,
      configurable: true,
    });
  };
  class MediaDevices extends EventTarget {
    constructor(token) {
      super();
      if (token !== constructToken) throw new TypeError('Illegal constructor');
      this.ondevicechange = null;
    }
    enumerateDevices() {
      return Promise.resolve(mediaProfile.devices.map((device) => ({ ...device })));
    }
    getSupportedConstraints() {
      return Object.freeze({ ...mediaProfile.supported_constraints });
    }
    getUserMedia() {}
    getDisplayMedia() {}
  }
  defineGlobalConstructor('MediaDevices', MediaDevices);
  if (navigatorValues.media_devices_enabled) {
    defineNavigatorValue('mediaDevices', new MediaDevices(constructToken));
  } else {
    delete Navigator.prototype.mediaDevices;
  }

  // Legacy media call coverage without tracks, capture, buffering or decoding.
  class MediaStream extends EventTarget {
    #id = crypto.randomUUID();
    get id() {
      return this.#id;
    }
    get active() {
      return false;
    }
    addTrack() {}
    removeTrack() {}
    clone() {}
    getAudioTracks() {}
    getVideoTracks() {}
    getTracks() {}
    getTrackById() {}
  }
  class SourceBuffer extends EventTarget {
    constructor() {
      super();
      throw new TypeError('Illegal constructor');
    }
    abort() {}
    appendBuffer() {}
    changeType() {}
    remove() {}
    get buffered() {
      return '';
    }
    get updating() {
      return '';
    }
  }
  for (const name of ['appendWindowEnd', 'appendWindowStart', 'mode', 'timestampOffset']) {
    const values = new WeakMap();
    Object.defineProperty(SourceBuffer.prototype, name, {
      get() {
        return values.get(this) ?? '';
      },
      set(value) {
        values.set(this, String(value));
      },
      enumerable: true,
      configurable: true,
    });
  }
  for (const [constructor, names] of [
    [MediaStream, ['addtrack', 'removetrack']],
    [SourceBuffer, ['abort', 'error', 'update', 'updateend', 'updatestart']],
  ]) {
    for (const type of names) {
      const handlers = new WeakMap();
      Object.defineProperty(constructor.prototype, 'on' + type, {
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
    defineGlobalConstructor(constructor.name, constructor);
  }

  const __arrayBufferByteLength = Object.getOwnPropertyDescriptor(
    ArrayBuffer.prototype,
    'byteLength',
  ).get;
  class MediaEncryptedEvent extends Event {
    #initData;
    #initDataType;
    constructor(type, options = {}) {
      if (arguments.length === 0) throw new TypeError('Event type is required');
      if (options == null) options = {};
      super(type, options);
      const data = options.initData;
      if (data != null) __arrayBufferByteLength.call(data);
      this.#initData = data ?? null;
      this.#initDataType = options.initDataType === undefined ? '' : `${options.initDataType}`;
    }
    get initData() {
      return this.#initData;
    }
    get initDataType() {
      return this.#initDataType;
    }
  }
  for (const name of ['initData', 'initDataType']) {
    Object.defineProperty(MediaEncryptedEvent.prototype, name, { enumerable: true });
  }

  global.MediaEncryptedEvent = MediaEncryptedEvent;
  const unsupportedCapability = { supported: false, smooth: false, powerEfficient: false };
  class MediaCapabilities {
    constructor(token) {
      if (token !== constructToken) throw new TypeError('Illegal constructor');
    }
    decodingInfo(configuration = {}) {
      const contentType = (part) => part?.contentType ?? null;
      const match = (mediaProfile.decoding ?? []).find(
        (entry) =>
          entry.type === configuration.type &&
          contentType(entry.video) === contentType(configuration.video) &&
          contentType(entry.audio) === contentType(configuration.audio),
      );
      return Promise.resolve(
        match
          ? {
              supported: Boolean(match.supported),
              smooth: Boolean(match.smooth),
              powerEfficient: Boolean(match.powerEfficient),
            }
          : { ...unsupportedCapability },
      );
    }
    encodingInfo() {
      return Promise.resolve({ ...unsupportedCapability });
    }
  }
  class MediaSession {
    #metadata = null;
    #playbackState = 'none';
    constructor(token) {
      if (token !== constructToken) throw new TypeError('Illegal constructor');
    }
    get metadata() {
      return this.#metadata;
    }
    set metadata(value) {
      this.#metadata = value;
    }
    get playbackState() {
      return this.#playbackState;
    }
    set playbackState(value) {
      value = String(value);
      if (!['none', 'paused', 'playing'].includes(value))
        throw new TypeError('Invalid playback state');
      this.#playbackState = value;
    }
    setActionHandler() {}
    setCameraActive() {}
    setMicrophoneActive() {}
    setPositionState() {}
  }
  // Named interfaces without a media pipeline behind them: scripts probe for
  // presence, so the constructors exist and refuse direct construction.
  class MediaSource extends EventTarget {
    constructor() {
      super();
      throw new TypeError('Illegal constructor');
    }
    static isTypeSupported(type) {
      const base = String(type).toLowerCase().split(';')[0].trim();
      return ['audio/mp4', 'audio/webm', 'video/mp4', 'video/webm'].includes(base);
    }
    get readyState() {
      return 'closed';
    }
    get sourceBuffers() {
      return Object.freeze([]);
    }
    get activeSourceBuffers() {
      return Object.freeze([]);
    }
    get duration() {
      return NaN;
    }
    addSourceBuffer() {
      throw new DOMException('MediaSource is unavailable', 'NotSupportedError');
    }
    removeSourceBuffer() {}
    endOfStream() {}
    setLiveSeekableRange() {}
    clearLiveSeekableRange() {}
  }
  class MediaStreamTrack extends EventTarget {
    constructor() {
      super();
      throw new TypeError('Illegal constructor');
    }
    get kind() {
      return '';
    }
    get label() {
      return '';
    }
    get enabled() {
      return false;
    }
    set enabled(value) {}
    get muted() {
      return false;
    }
    get readyState() {
      return 'ended';
    }
    stop() {}
    clone() {
      throw new DOMException('MediaStreamTrack is unavailable', 'NotSupportedError');
    }
    getSettings() {
      return {};
    }
    getCapabilities() {
      return {};
    }
    getConstraints() {
      return {};
    }
    applyConstraints() {
      return Promise.resolve();
    }
  }
  class VideoTrackList extends EventTarget {
    constructor() {
      super();
      throw new TypeError('Illegal constructor');
    }
    get length() {
      return 0;
    }
    get selectedIndex() {
      return -1;
    }
    getTrackById() {
      return null;
    }
    item() {
      return null;
    }
    [Symbol.iterator]() {
      return [][Symbol.iterator]();
    }
  }
  class WebKitMediaKeyError {
    constructor() {
      throw new TypeError('Illegal constructor');
    }
    get code() {
      return 0;
    }
  }
  for (const [target, names] of [
    [MediaSource, ['isTypeSupported']],
    [MediaSource.prototype, ['addSourceBuffer', 'removeSourceBuffer', 'endOfStream']],
  ]) {
    for (const name of names) {
      const descriptor = Object.getOwnPropertyDescriptor(target, name);
      if (descriptor) Object.defineProperty(target, name, { ...descriptor, enumerable: true });
    }
  }
  const userAgent = globalThis.navigator?.userAgent ?? '';
  const exposeVideoTrackList = !/Firefox\//.test(userAgent);
  const exposeWebKitMediaKeyError =
    /AppleWebKit\//.test(userAgent) && !/Chrome\/|Chromium\/|Edg\//.test(userAgent);
  for (const [name, value] of Object.entries({
    MediaSource,
    MediaStreamTrack,
    ...(exposeVideoTrackList ? { VideoTrackList } : {}),
    ...(exposeWebKitMediaKeyError ? { WebKitMediaKeyError } : {}),
  })) {
    defineGlobalConstructor(name, value);
  }
  defineGlobalConstructor('MediaCapabilities', MediaCapabilities);
  defineNavigatorValue('mediaCapabilities', new MediaCapabilities(constructToken));
  defineGlobalConstructor('MediaSession', MediaSession);
  defineNavigatorValue('mediaSession', new MediaSession(constructToken));
  for (const name of ['requestMediaKeySystemAccess']) {
    Object.defineProperty(Navigator.prototype, name, {
      value: { [name]() {} }[name],
      writable: true,
      configurable: true,
      enumerable: true,
    });
  }
}
