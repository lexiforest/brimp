// Runs unchanged inside Brimp and Safari. No adopted source is executed.
export function probe(rows) {
  const cache = new Map(),
    recipes = {},
    failures = {},
    cleanup = [];
  const error = (e) => ({ name: e?.name || 'Error', message: String(e?.message || e) });
  const el = (tag) => document.createElement(tag);
  const globalPaths = {
    Window: 'window',
    Navigator: 'navigator',
    Document: 'document',
    HTMLDocument: 'document',
    Location: 'location',
    History: 'history',
    Screen: 'screen',
    ScreenOrientation: 'screen.orientation',
    VisualViewport: 'visualViewport',
    Performance: 'performance',
    PerformanceTiming: 'performance.timing',
    PerformanceNavigation: 'performance.navigation',
    MemoryInfo: 'performance.memory',
    Crypto: 'crypto',
    CustomElementRegistry: 'customElements',
    Storage: 'localStorage',
    CacheStorage: 'caches',
    IDBFactory: 'indexedDB',
    BarProp: 'locationbar',
    External: 'external',
    PluginArray: 'navigator.plugins',
    MimeTypeArray: 'navigator.mimeTypes',
    Plugin: 'navigator.plugins.0',
    MimeType: 'navigator.mimeTypes.0',
    NavigatorUAData: 'navigator.userAgentData',
    NetworkInformation: 'navigator.connection',
    Geolocation: 'navigator.geolocation',
    Permissions: 'navigator.permissions',
    MediaDevices: 'navigator.mediaDevices',
    MediaCapabilities: 'navigator.mediaCapabilities',
    MediaSession: 'navigator.mediaSession',
    CredentialsContainer: 'navigator.credentials',
    Clipboard: 'navigator.clipboard',
    StorageManager: 'navigator.storage',
    ServiceWorkerContainer: 'navigator.serviceWorker',
    Bluetooth: 'navigator.bluetooth',
    HID: 'navigator.hid',
    USB: 'navigator.usb',
    Serial: 'navigator.serial',
    Keyboard: 'navigator.keyboard',
    LockManager: 'navigator.locks',
    WakeLock: 'navigator.wakeLock',
    UserActivation: 'navigator.userActivation',
    VirtualKeyboard: 'navigator.virtualKeyboard',
    XRSystem: 'navigator.xr',
    Presentation: 'navigator.presentation',
    Ink: 'navigator.ink',
    NavigatorManagedData: 'navigator.managed',
    Scheduling: 'navigator.scheduling',
    Scheduler: 'scheduler',
    CookieStore: 'cookieStore',
    TrustedTypePolicyFactory: 'trustedTypes',
    DeprecatedStorageQuota: 'navigator.webkitTemporaryStorage',
  };
  const tags = {
    Element: 'div',
    HTMLElement: 'div',
    Node: 'div',
    HTMLAnchorElement: 'a',
    HTMLAudioElement: 'audio',
    HTMLBodyElement: 'body',
    HTMLButtonElement: 'button',
    HTMLCanvasElement: 'canvas',
    HTMLDivElement: 'div',
    HTMLFrameElement: 'frame',
    HTMLFrameSetElement: 'frameset',
    HTMLHeadElement: 'head',
    HTMLHtmlElement: 'html',
    HTMLIFrameElement: 'iframe',
    HTMLImageElement: 'img',
    HTMLInputElement: 'input',
    HTMLLabelElement: 'label',
    HTMLLIElement: 'li',
    HTMLLinkElement: 'link',
    HTMLMediaElement: 'video',
    HTMLMetaElement: 'meta',
    HTMLScriptElement: 'script',
    HTMLSpanElement: 'span',
    HTMLTitleElement: 'title',
    HTMLVideoElement: 'video',
  };
  function pathValue(path) {
    return path.split('.').reduce((o, k) => (o == null ? undefined : o[k]), globalThis);
  }
  function remember(name, recipe, make) {
    recipes[name] = recipe;
    try {
      return make();
    } catch (e) {
      failures[name] = error(e);
      return undefined;
    }
  }
  function receiver(name) {
    if (cache.has(name)) return cache.get(name);
    cache.set(name, undefined);
    const make = (recipe, fn) => remember(name, recipe, fn);
    let value;
    if (globalPaths[name]) value = make(globalPaths[name], () => pathValue(globalPaths[name]));
    else if (tags[name])
      value = make(`document.createElement(${JSON.stringify(tags[name])}) (detached)`, () =>
        el(tags[name]),
      );
    else if (
      [
        'Event',
        'CustomEvent',
        'UIEvent',
        'MouseEvent',
        'PointerEvent',
        'KeyboardEvent',
        'MediaEncryptedEvent',
        'RTCPeerConnectionIceEvent',
      ].includes(name)
    ) {
      value = make(`new ${name}("coverage")`, () => new globalThis[name]('coverage'));
    } else
      switch (name) {
        case 'TextEvent':
        case 'MutationEvent':
          value = make(`document.createEvent(${JSON.stringify(name)})`, () =>
            document.createEvent(name),
          );
          break;
        case 'Attr':
          value = make('document.createAttribute("data-coverage")', () =>
            document.createAttribute('data-coverage'),
          );
          break;
        case 'Text':
        case 'CharacterData':
          value = make('document.createTextNode("")', () => document.createTextNode(''));
          break;
        case 'CDATASection':
          value = make('XML document.createCDATASection("")', () =>
            document.implementation.createDocument(null, 'root').createCDATASection(''),
          );
          break;
        case 'DOMTokenList':
          value = make('detached div.classList', () => el('div').classList);
          break;
        case 'CSSStyleDeclaration':
          value = make('detached div.style', () => el('div').style);
          break;
        case 'NodeList':
          value = make('detached div.childNodes (empty)', () => el('div').childNodes);
          break;
        case 'HTMLCollection':
          value = make('detached div.children (empty)', () => el('div').children);
          break;
        case 'SVGElement':
          value = make('document.createElementNS(SVG, "svg")', () =>
            document.createElementNS('http://www.w3.org/2000/svg', 'svg'),
          );
          break;
        case 'SVGGraphicsElement':
          value = make('document.createElementNS(SVG, "g")', () =>
            document.createElementNS('http://www.w3.org/2000/svg', 'g'),
          );
          break;
        case 'Blob':
          value = make('new Blob([])', () => new Blob([]));
          break;
        case 'Request':
          value = make('new Request(location.href)', () => new Request(location.href));
          break;
        case 'Headers':
        case 'DOMParser':
        case 'DOMRect':
        case 'DOMRectReadOnly':
        case 'Path2D':
        case 'EventTarget':
        case 'XMLHttpRequest':
        case 'SpeechSynthesisUtterance':
        case 'MediaStream':
          value = make(`new ${name}()`, () => new globalThis[name]());
          break;
        case 'XMLHttpRequestEventTarget':
          value = make('new XMLHttpRequest().upload', () => receiver('XMLHttpRequest').upload);
          break;
        case 'PerformanceEntry':
          value = make('performance.mark("coverage", {startTime: 0})', () => {
            performance.mark('coverage', { startTime: 0 });
            cleanup.push(() => performance.clearMarks('coverage'));
            return performance.getEntriesByName('coverage')[0];
          });
          break;
        case 'PerformancePaintTiming':
          value = make(
            'performance.getEntriesByType("paint")[0]',
            () => performance.getEntriesByType('paint')[0],
          );
          break;
        case 'CanvasRenderingContext2D':
          value = make('detached canvas.getContext("2d")', () => el('canvas').getContext('2d'));
          break;
        case 'WebGLRenderingContext':
          value = make('detached canvas.getContext("webgl")', () => {
            const gl = el('canvas').getContext('webgl');
            if (gl) cleanup.push(() => gl.getExtension('WEBGL_lose_context')?.loseContext());
            return gl;
          });
          break;
        case 'WebGLDebugRendererInfo':
          value = make('webgl.getExtension("WEBGL_debug_renderer_info")', () =>
            receiver('WebGLRenderingContext')?.getExtension('WEBGL_debug_renderer_info'),
          );
          break;
        case 'OfflineAudioContext':
          value = make(
            'new OfflineAudioContext(1, 128, 44100)',
            () => new OfflineAudioContext(1, 128, 44100),
          );
          break;
        case 'BaseAudioContext':
          value = make('new AudioContext() (never resumed)', () => {
            const audio = new AudioContext();
            cleanup.push(() => audio.close().catch(() => {}));
            return audio;
          });
          break;
        case 'AudioDestinationNode':
          value = make('audioContext.destination', () => receiver('BaseAudioContext').destination);
          break;
        case 'AudioListener':
          value = make('audioContext.listener', () => receiver('BaseAudioContext').listener);
          break;
        case 'AudioWorklet':
        case 'Worklet':
          value = make(
            'audioContext.audioWorklet',
            () => receiver('BaseAudioContext').audioWorklet,
          );
          break;
        case 'AudioNode':
        case 'AudioScheduledSourceNode':
        case 'OscillatorNode':
          value = make('audioContext.createOscillator() (not started)', () =>
            receiver('BaseAudioContext').createOscillator(),
          );
          break;
        case 'AudioParam':
          value = make('oscillator.frequency', () => receiver('OscillatorNode').frequency);
          break;
        case 'DynamicsCompressorNode':
          value = make('audioContext.createDynamicsCompressor()', () =>
            receiver('BaseAudioContext').createDynamicsCompressor(),
          );
          break;
        case 'RTCPeerConnection':
          value = make('new RTCPeerConnection({iceServers: []}) (no negotiation)', () => {
            const pc = new RTCPeerConnection({ iceServers: [] });
            cleanup.push(() => pc.close());
            return pc;
          });
          break;
        case 'RTCDataChannel':
          value = make('peerConnection.createDataChannel("coverage") (no negotiation)', () =>
            receiver('RTCPeerConnection').createDataChannel('coverage'),
          );
          break;
        case 'RTCIceCandidate':
          value = make(
            'new RTCIceCandidate({candidate: ""})',
            () => new RTCIceCandidate({ candidate: '' }),
          );
          break;
        case 'RTCSessionDescription':
          value = make(
            'new RTCSessionDescription({type: "offer", sdp: ""})',
            () => new RTCSessionDescription({ type: 'offer', sdp: '' }),
          );
          break;
        // These require a workload/state transition. Inspect exposed prototype data
        // without inventing a receiver or invoking getters with an illegal receiver.
        default:
          recipes[name] = 'no representative receiver; exposed prototype data only';
      }
    cache.set(name, value);
    return value;
  }
  const key = (name) => (name.startsWith('[Symbol.') ? Symbol[name.slice(8, -1)] : name);
  const isObject = (x) => x !== null && (typeof x === 'object' || typeof x === 'function');
  const tag = (x) => Object.prototype.toString.call(x);
  function encode(value) {
    if (value === null) return { type: 'null' };
    const type = typeof value;
    if (type === 'undefined') return { type };
    if (type === 'number')
      return {
        type,
        value: Number.isFinite(value) ? (Object.is(value, -0) ? '-0' : value) : String(value),
      };
    if (type === 'boolean' || type === 'string') return { type, value };
    if (type === 'bigint' || type === 'symbol') return { type, value: String(value) };
    if (type === 'function') return { type, name: value.name, length: value.length };
    if (
      Array.isArray(value) &&
      value.length <= 32 &&
      value.every((v) => v == null || ['string', 'number', 'boolean'].includes(typeof v))
    )
      return { type: 'array', items: value.map(encode) };
    // Some prototype objects have a branded Symbol.toStringTag getter. A
    // failure to describe an object is distinct from failure to read the field.
    try {
      return { type, tag: tag(value) };
    } catch (e) {
      return { type, tag: null, tag_error: error(e) };
    }
  }
  function descriptor(object, k) {
    let depth = 0;
    for (let cur = object; isObject(cur); cur = Object.getPrototypeOf(cur), depth++) {
      const d = Object.getOwnPropertyDescriptor(cur, k);
      if (d)
        return {
          raw: d,
          depth,
          shape: {
            kind: 'value' in d ? 'data' : 'accessor',
            enumerable: d.enumerable,
            configurable: d.configurable,
            ...('value' in d
              ? { writable: d.writable }
              : { get: typeof d.get === 'function', set: typeof d.set === 'function' }),
          },
        };
    }
    return null;
  }
  const targets = {},
    result = {};
  function context(target) {
    if (targets[target]) return targets[target];
    let declared,
      sample,
      info = {};
    try {
      const match = /^(\w+)\.(prototype|instance)(.*)$/.exec(target);
      if (match) {
        const [, name, location, tail] = match;
        sample = receiver(name);
        declared = location === 'prototype' ? globalThis[name]?.prototype : sample;
        if (name === 'WindowProperties' && !declared && globalThis.Window)
          declared = Object.getPrototypeOf(Window.prototype);
        if (tail) {
          // The only nested surface in this source is Symbol.unscopables.
          for (const part of tail.match(/\[Symbol\.\w+\]|[^.]+/g) || [])
            declared = declared?.[key(part)];
          sample = declared;
        }
        info = {
          receiver: recipes[name],
          receiver_available: isObject(sample),
          declared_target_available: isObject(declared),
          constructor_exposed: typeof globalThis[name] === 'function',
        };
        if (failures[name]) info.receiver_error = failures[name];
      } else {
        declared = pathValue(target);
        sample = declared;
        info = {
          receiver: target,
          receiver_available: isObject(sample),
          declared_target_available: isObject(declared),
        };
      }
    } catch (e) {
      info.error = error(e);
    }
    const ctx = { declared, sample, info };
    targets[target] = ctx;
    return ctx;
  }
  for (const row of rows) {
    try {
      const { declared, sample, info } = context(row.target);
      if (row.name === '[index]' || row.name === '[name]') {
        result[row.id] = {
          presence: 'unverified',
          reason: 'dynamic_property_pattern_requires_populated_collection',
        };
        continue;
      }
      const k = key(row.name),
        declaredPresent = isObject(declared) ? k in declared : null;
      const onSample = isObject(sample) && k in sample;
      const object = onSample ? sample : declared;
      const present = onSample || declaredPresent === true;
      if (!present) {
        const unsampledInstance =
          row.target.endsWith('.instance') && info.constructor_exposed && !isObject(sample);
        result[row.id] = {
          presence: info.error || unsampledInstance ? 'unverified' : 'absent',
          reason: info.error
            ? 'target_resolution_error'
            : unsampledInstance
              ? 'no_representative_receiver'
              : isObject(sample) || isObject(declared)
                ? 'member_not_exposed'
                : 'target_and_receiver_not_exposed',
        };
        continue;
      }
      const d = descriptor(object, k);
      const record = {
        presence: 'present',
        declared_target: declaredPresent,
        declared_own: isObject(declared) ? Object.hasOwn(declared, k) : null,
        lookup: onSample ? 'receiver' : 'declared_target',
        depth: d?.depth ?? null,
        descriptor: d?.shape ?? null,
      };
      // Data descriptors may be read without a valid platform receiver.
      if (!onSample && d && d.shape.kind === 'accessor')
        record.value = { type: 'unverified', reason: 'no_representative_receiver' };
      else
        try {
          record.value = encode(Reflect.get(object, k, object));
        } catch (e) {
          record.value = { type: 'read_error', ...error(e) };
        }
      record.callable = record.value?.type === 'function';
      result[row.id] = record;
    } catch (e) {
      result[row.id] = { presence: 'unverified', reason: 'probe_error', error: error(e) };
    }
  }
  const environment = {
    url: location.href,
    user_agent: navigator.userAgent,
    platform: navigator.platform,
    secure_context: typeof isSecureContext === 'boolean' ? isSecureContext : null,
    cross_origin_isolated: typeof crossOriginIsolated === 'boolean' ? crossOriginIsolated : null,
    document_ready_state: document.readyState,
    compat_mode: document.compatMode,
    viewport: {
      width: globalThis.innerWidth ?? null,
      height: globalThis.innerHeight ?? null,
      device_pixel_ratio: globalThis.devicePixelRatio ?? null,
    },
  };
  const cleanupErrors = [];
  for (const dispose of cleanup.reverse())
    try {
      dispose();
    } catch (e) {
      cleanupErrors.push(error(e));
    }
  return {
    environment,
    targets: Object.fromEntries(Object.entries(targets).map(([k, v]) => [k, v.info])),
    fields: result,
    cleanup_errors: cleanupErrors,
  };
}
