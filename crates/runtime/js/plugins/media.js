function installMedia({ global, profile: settings }, options = {}) {
  const mediaProfile = settings.values(
    'media',
    {
      media_devices_enabled: true,
      devices: [],
      supported_constraints: {},
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
  class MediaCapabilities {
    constructor(token) {
      if (token !== constructToken) throw new TypeError('Illegal constructor');
    }
    decodingInfo() {}
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
