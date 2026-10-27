class HTMLBaseElement extends HTMLElement {
  constructor() {
    throw new TypeError('Illegal constructor');
  }
  get href() {
    return __elementURL(this, 'href');
  }
  set href(value) {
    this.setAttribute('href', value);
  }
}

function __defineStringReflections(prototype, definitions) {
  for (const [property, attribute = property.toLowerCase()] of definitions) {
    Object.defineProperty(
      prototype,
      property,
      Object.getOwnPropertyDescriptor(
        {
          get [property]() {
            return __getContentAttribute(this, attribute) ?? '';
          },
          set [property](value) {
            __setContentAttribute(this, attribute, `${value}`);
          },
        },
        property,
      ),
    );
  }
}

function __defineBooleanReflections(prototype, definitions) {
  for (const [property, attribute = property.toLowerCase()] of definitions) {
    Object.defineProperty(
      prototype,
      property,
      Object.getOwnPropertyDescriptor(
        {
          get [property]() {
            return __getContentAttribute(this, attribute) !== null;
          },
          set [property](value) {
            __setContentAttribute(this, attribute, value ? '' : null);
          },
        },
        property,
      ),
    );
  }
}

function __defineUSVStringReflection(prototype, property, attribute = property.toLowerCase()) {
  Object.defineProperty(
    prototype,
    property,
    Object.getOwnPropertyDescriptor(
      {
        get [property]() {
          return __getContentAttribute(this, attribute) ?? '';
        },
        set [property](value) {
          __setContentAttribute(this, attribute, __toUSVString(`${value}`));
        },
      },
      property,
    ),
  );
}

function __parseNonnegativeInteger(value) {
  const match = /^[\t\n\f\r ]*([+-]?\d+)/.exec(value);
  if (!match) return null;
  const number = Number(match[1]);
  if (number === 0) return 0;
  return Number.isSafeInteger(number) && number > 0 && number <= 2147483647 ? number : null;
}

function __defineUnsignedReflections(prototype, definitions) {
  for (const [property, attribute, defaultValue = 0] of definitions) {
    Object.defineProperty(
      prototype,
      property,
      Object.getOwnPropertyDescriptor(
        {
          get [property]() {
            const value = __getContentAttribute(this, attribute);
            return value === null
              ? defaultValue
              : (__parseNonnegativeInteger(value) ?? defaultValue);
          },
          set [property](value) {
            const converted = +value >>> 0;
            __setContentAttribute(
              this,
              attribute,
              String(converted <= 2147483647 ? converted : defaultValue),
            );
          },
        },
        property,
      ),
    );
  }
}

function __defineUrlReflections(prototype, definitions) {
  for (const [property, attribute = property.toLowerCase()] of definitions) {
    Object.defineProperty(
      prototype,
      property,
      Object.getOwnPropertyDescriptor(
        {
          get [property]() {
            return __getContentAttribute(this, attribute) !== null
              ? __elementURL(this, attribute)
              : '';
          },
          set [property](value) {
            __setContentAttribute(this, attribute, __toUSVString(`${value}`));
          },
        },
        property,
      ),
    );
  }
}

function __defineActionUrlReflection(prototype, property, attribute) {
  Object.defineProperty(
    prototype,
    property,
    Object.getOwnPropertyDescriptor(
      {
        get [property]() {
          const value = __getContentAttribute(this, attribute);
          return value === null || value === ''
            ? __documentURL(__callHost('ownerDocument', this))
            : __elementURL(this, attribute);
        },
        set [property](value) {
          __setContentAttribute(this, attribute, __toUSVString(`${value}`));
        },
      },
      property,
    ),
  );
}

const __elementDocumentEncoding = Object.getOwnPropertyDescriptor(
  Document.prototype,
  'characterSet',
).get;
function __elementURL(element, attribute) {
  const owner = __callHost('ownerDocument', element);
  return __callHost(
    'elementUrl',
    element,
    attribute,
    __documentURL(owner),
    __elementDocumentEncoding.call(owner),
  );
}

function __defineEnumReflection(
  prototype,
  property,
  attribute,
  keywords,
  defaultValue,
  invalidValue,
  aliases = {},
  nullable = false,
) {
  Object.defineProperty(
    prototype,
    property,
    Object.getOwnPropertyDescriptor(
      {
        get [property]() {
          const raw = __getContentAttribute(this, attribute);
          if (raw === null) return defaultValue;
          const value = __asciiLowercase(raw);
          if (Object.prototype.hasOwnProperty.call(aliases, value)) return aliases[value];
          return keywords.includes(value) ? value : invalidValue;
        },
        set [property](value) {
          __setContentAttribute(this, attribute, nullable && value == null ? null : `${value}`);
        },
      },
      property,
    ),
  );
}

function __asciiLowercase(value) {
  return String(value).replace(/[A-Z]/g, (character) =>
    String.fromCharCode(character.charCodeAt(0) + 32),
  );
}

function __defineNullAsEmptyStringReflection(prototype, property, attribute) {
  Object.defineProperty(
    prototype,
    property,
    Object.getOwnPropertyDescriptor(
      {
        get [property]() {
          return __getContentAttribute(this, attribute) ?? '';
        },
        set [property](value) {
          __setContentAttribute(this, attribute, value === null ? '' : `${value}`);
        },
      },
      property,
    ),
  );
}

function __defineTokenListReflection(prototype, property, attribute) {
  const lists = new WeakMap();
  const getList = (element) => {
    let list = lists.get(element);
    if (!list) {
      list = __createTokenList(element, attribute);
      lists.set(element, list);
    }
    return list;
  };
  Object.defineProperty(
    prototype,
    property,
    Object.getOwnPropertyDescriptor(
      {
        get [property]() {
          return getList(this);
        },
        set [property](value) {
          this[property].value = value;
        },
      },
      property,
    ),
  );
}

function __parseInteger(value) {
  const match = /^[\t\n\f\r ]*([+-]?\d+)/.exec(value);
  if (!match) return null;
  const number = Number(match[1]);
  if (number === 0) return 0;
  return Number.isSafeInteger(number) && number >= -2147483648 && number <= 2147483647
    ? number
    : null;
}

function __toLong(value) {
  let number = +value;
  if (!Number.isFinite(number) || number === 0) return 0;
  number = Math.trunc(number);
  number = ((number % 4294967296) + 4294967296) % 4294967296;
  return number >= 2147483648 ? number - 4294967296 : number;
}

function __defineLongReflections(prototype, definitions) {
  for (const [property, attribute = property.toLowerCase(), defaultValue = 0] of definitions) {
    Object.defineProperty(
      prototype,
      property,
      Object.getOwnPropertyDescriptor(
        {
          get [property]() {
            const parsed = __parseInteger(__getContentAttribute(this, attribute) ?? '');
            return parsed ?? defaultValue;
          },
          set [property](value) {
            const converted = __toLong(value);
            __setContentAttribute(this, attribute, String(converted));
          },
        },
        property,
      ),
    );
  }
}

function __defineLengthReflections(prototype) {
  for (const property of ['maxLength', 'minLength']) {
    const attribute = property.toLowerCase(),
      maximum = property === 'maxLength';
    Object.defineProperty(
      prototype,
      property,
      Object.getOwnPropertyDescriptor(
        {
          get [property]() {
            return __parseNonnegativeInteger(__getContentAttribute(this, attribute) ?? '') ?? -1;
          },
          set [property](value) {
            const converted = __toLong(value);
            const other = __parseNonnegativeInteger(
              __getContentAttribute(this, maximum ? 'minlength' : 'maxlength') ?? '',
            );
            if (
              converted < 0 ||
              (other !== null && (maximum ? converted < other : converted > other))
            )
              throw new DOMException('The length is outside the allowed range', 'IndexSizeError');
            __setContentAttribute(this, attribute, String(converted));
          },
        },
        property,
      ),
    );
  }
}

function __definePositiveUnsignedReflections(prototype, definitions) {
  for (const [
    property,
    attribute = property.toLowerCase(),
    defaultValue = 1,
    fallback = false,
  ] of definitions) {
    Object.defineProperty(
      prototype,
      property,
      Object.getOwnPropertyDescriptor(
        {
          get [property]() {
            const parsed = __parseNonnegativeInteger(__getContentAttribute(this, attribute) ?? '');
            return parsed === null || parsed === 0 ? defaultValue : parsed;
          },
          set [property](value) {
            const converted = value >>> 0;
            if (!fallback && converted === 0)
              throw new DOMException('value must be greater than zero', 'IndexSizeError');
            __setContentAttribute(
              this,
              attribute,
              String(converted > 0 && converted <= 2147483647 ? converted : defaultValue),
            );
          },
        },
        property,
      ),
    );
  }
}

function __defineClampedUnsignedReflections(prototype, definitions) {
  for (const [property, attribute, defaultValue, minimum, maximum] of definitions) {
    Object.defineProperty(prototype, property, {
      configurable: true,
      enumerable: true,
      get() {
        const match = /^[\t\n\f\r ]*([+-]?\d+)/.exec(this.getAttribute(attribute) ?? '');
        const parsed = match ? Number(match[1]) : NaN;
        if (!Number.isSafeInteger(parsed) || parsed < 0) return defaultValue;
        return Math.min(maximum, Math.max(minimum, parsed));
      },
      set(value) {
        const converted = Number(value) >>> 0;
        this.setAttribute(attribute, String(converted <= 2147483647 ? converted : defaultValue));
      },
    });
  }
}

function __parseFloatingPoint(value) {
  const match = /^[\t\n\f\r ]*([+-]?(?:\d+(?:\.\d*)?|\.\d+)(?:[eE][+-]?\d+)?)/.exec(value);
  if (!match) return null;
  const number = Number(match[1]);
  return Number.isFinite(number) ? (number === 0 ? 0 : number) : null;
}

function __defineDoubleReflections(prototype, definitions) {
  for (const [
    property,
    attribute = property.toLowerCase(),
    defaultValue = 0,
    positive = false,
  ] of definitions) {
    Object.defineProperty(prototype, property, {
      configurable: true,
      enumerable: true,
      get() {
        const parsed = __parseFloatingPoint(this.getAttribute(attribute) ?? '');
        return parsed === null || (positive && parsed <= 0) ? defaultValue : parsed;
      },
      set(value) {
        const number = Number(value);
        if (!Number.isFinite(number))
          throw new DOMException('value must be finite', 'NotSupportedError');
        if (!positive || number > 0)
          this.setAttribute(attribute, String(number === 0 ? 0 : number));
      },
    });
  }
}

function __defineNonceReflection(prototype) {
  const values = new WeakMap();
  Object.defineProperty(prototype, 'nonce', {
    configurable: true,
    enumerable: true,
    get() {
      return values.has(this) ? values.get(this) : (this.getAttribute('nonce') ?? '');
    },
    set(value) {
      values.set(this, String(value));
    },
  });
}

class HTMLPictureElement extends HTMLElement {
  constructor() {
    throw new TypeError('Illegal constructor');
  }
}
class HTMLImageElement extends HTMLElement {
  constructor() {
    throw new TypeError('Illegal constructor');
  }
  // Explicit legacy placeholders: no image resource selection, decoder or layout.
  get complete() {
    return true;
  }
  get currentSrc() {
    return '';
  }
  get naturalWidth() {
    return 0;
  }
  get naturalHeight() {
    return 0;
  }
  get x() {
    return 0;
  }
  get y() {
    return 0;
  }
  decode() {}
}

function __runInChildWindow(context, source) {
  return Function(
    'window',
    'document',
    'location',
    'parent',
    'self',
    'top',
    source,
  )(context.window, context.document, context.window.location, window, context.window, window);
}

function __childConstructor(context, creator) {
  return function (...arguments_) {
    return creator(context.document, ...arguments_);
  };
}

function __parseChildDocument(source) {
  if (source === '') source = '<!doctype html><html><head></head><body></body></html>';
  return new DOMParser().parseFromString(source, 'text/html');
}

class HTMLIFrameElement extends HTMLElement {
  constructor() {
    throw new TypeError('Illegal constructor');
  }
  getSVGDocument() {
    const doc = this.contentDocument;
    return doc?.rootElement ? doc : null;
  }
  get contentWindow() {
    let context = __iframeBrowsingContexts.get(this);
    if (!context) {
      const frame = this;
      let location = new URL('about:blank');
      const childWindow = new EventTarget();
      __windowEventTargets.add(childWindow);
      __defineEventHandlers(childWindow, __bodyWindowEventTypes);
      context = {
        document: __parseChildDocument(''),
        window: childWindow,
        navigate(url, srcdoc = null) {
          location = new URL(
            url,
            __documentURL(__callHost('ownerDocument', frame)) || 'about:blank',
          );
          __documentWindows.delete(context.document);
          context.document =
            srcdoc !== null
              ? __parseChildDocument(srcdoc)
              : location.protocol === 'data:'
                ? __callHost('dataDocumentParse', null, location.href)
                : __parseChildDocument('');
          __documentWindows.set(context.document, { window: childWindow, frame });
          __callHost('documentSetURL', context.document, location.href);
          for (const script of context.document.querySelectorAll('script')) {
            __runInChildWindow(context, script.textContent);
          }
          const onload = context.document.body?.getAttribute('onload');
          if (onload) __runInChildWindow(context, onload);
        },
      };
      __callHost('documentSetURL', context.document, 'about:blank');
      __documentWindows.set(context.document, { window: childWindow, frame });
      Object.defineProperties(childWindow, {
        location: {
          get() {
            const href = __documentURL(context.document);
            if (location.href !== href) location = new URL(href);
            return location;
          },
          set(value) {
            context.navigate(value);
          },
          enumerable: true,
        },
        document: {
          get() {
            return context.document;
          },
          enumerable: true,
        },
        frameElement: { value: this, enumerable: true },
        parent: { value: window, enumerable: true },
        top: { value: window, enumerable: true },
      });
      childWindow.window = childWindow;
      childWindow.self = childWindow;
      childWindow.setTimeout = setTimeout;
      childWindow.clearTimeout = clearTimeout;
      childWindow.setInterval = setInterval;
      childWindow.clearInterval = clearInterval;
      childWindow.queueMicrotask = queueMicrotask;
      childWindow.Comment = __childConstructor(context, (doc, data = '') =>
        doc.createComment(String(data)),
      );
      childWindow.Text = __childConstructor(context, (doc, data = '') =>
        doc.createTextNode(String(data)),
      );
      childWindow.DocumentFragment = __childConstructor(context, (doc) =>
        doc.createDocumentFragment(),
      );
      childWindow.postMessage = (message, targetOrigin = '*', transfer = []) => {
        const cloned = __prepareMessage(message, transfer);
        targetOrigin = String(targetOrigin);
        if (targetOrigin !== '*' && targetOrigin !== '/' && targetOrigin !== location.origin)
          return;
        setTimeout(() => {
          const event = new MessageEvent('message', {
            data: cloned,
            origin: document.location?.origin ?? location.origin,
            source: window,
          });
          __markTrustedEvent(event);
          childWindow.dispatchEvent(event);
        }, 0);
      };
      __iframeBrowsingContexts.set(this, context);
    }
    return context.window;
  }
  get contentDocument() {
    this.contentWindow;
    return __iframeBrowsingContexts.get(this).document;
  }
  __connected() {
    if (__iframeBrowsingContexts.get(this)?.connected) return;
    this.contentWindow;
    const context = __iframeBrowsingContexts.get(this);
    context.connected = true;
    const srcdoc = this.getAttribute('srcdoc');
    context.navigate(
      srcdoc !== null ? 'about:srcdoc' : this.getAttribute('src') || 'about:blank',
      srcdoc,
    );
    setTimeout(() => this.dispatchEvent(new Event('load')), 0);
  }
  __navigate(url) {
    this.contentWindow;
    const context = __iframeBrowsingContexts.get(this);
    context.navigate(url);
    setTimeout(() => this.dispatchEvent(new Event('load')), 0);
  }
}
class HTMLEmbedElement extends HTMLElement {
  constructor() {
    throw new TypeError('Illegal constructor');
  }
}
class HTMLObjectElement extends HTMLElement {
  constructor() {
    throw new TypeError('Illegal constructor');
  }
}
class HTMLParamElement extends HTMLElement {
  constructor() {
    throw new TypeError('Illegal constructor');
  }
}
const __mediaElementStates = new WeakMap();
function __mediaElementState(element) {
  let state = __mediaElementStates.get(element);
  if (state === undefined) {
    state = {
      currentTime: 0,
      defaultPlaybackRate: 1,
      playbackRate: 1,
      volume: 1,
      muted: false,
      paused: true,
      preservesPitch: true,
      srcObject: null,
    };
    __mediaElementStates.set(element, state);
  }
  return state;
}
function __mediaElementController(element) {
  return globalThis.__brimpGetMediaElementController?.(element) ?? null;
}
function __trustedMediaEvent(element, type) {
  const event = new Event(type);
  __markTrustedEvent(event);
  element.dispatchEvent(event);
}
class HTMLMediaElement extends HTMLElement {
  constructor() {
    throw new TypeError('Illegal constructor');
  }
  get currentSrc() {
    return __mediaElementController(this) === null ? '' : this.src;
  }
  get currentTime() {
    return __mediaElementController(this)?.currentTime() ?? __mediaElementState(this).currentTime;
  }
  set currentTime(value) {
    value = Number(value);
    if (!Number.isFinite(value) || value < 0)
      throw new RangeError('currentTime must be finite and non-negative');
    __mediaElementState(this).currentTime = value;
    __mediaElementController(this)?.seek(value);
  }
  get duration() {
    return __mediaElementController(this)?.duration() ?? NaN;
  }
  get paused() {
    return __mediaElementState(this).paused;
  }
  get ended() {
    return __mediaElementController(this)?.ended() ?? false;
  }
  get readyState() {
    return __mediaElementController(this)?.readyState() ?? 0;
  }
  get networkState() {
    return this.currentSrc === '' ? 0 : this.readyState === 0 ? 2 : 1;
  }
  get error() {
    return null;
  }
  get seeking() {
    return false;
  }
  get sinkId() {
    return '';
  }
  get preservesPitch() {
    return __mediaElementState(this).preservesPitch;
  }
  set preservesPitch(value) {
    __mediaElementState(this).preservesPitch = Boolean(value);
  }
  get srcObject() {
    return __mediaElementState(this).srcObject;
  }
  set srcObject(value) {
    __mediaElementState(this).srcObject = value;
  }
  // Legacy placeholders: no buffering, text-track or remote-playback backend.
  get buffered() {
    return '';
  }
  get played() {
    return '';
  }
  get seekable() {
    return '';
  }
  get textTracks() {
    return '';
  }
  get remote() {
    return '';
  }
  addTextTrack() {}
  setSinkId() {}
  get defaultPlaybackRate() {
    return __mediaElementState(this).defaultPlaybackRate;
  }
  set defaultPlaybackRate(value) {
    value = Number(value);
    if (!Number.isFinite(value) || value === 0)
      throw new DOMException('Invalid playback rate', 'NotSupportedError');
    __mediaElementState(this).defaultPlaybackRate = value;
  }
  get playbackRate() {
    return __mediaElementState(this).playbackRate;
  }
  set playbackRate(value) {
    value = Number(value);
    if (!Number.isFinite(value) || value === 0)
      throw new DOMException('Invalid playback rate', 'NotSupportedError');
    __mediaElementController(this)?.setPlaybackRate(value);
    __mediaElementState(this).playbackRate = value;
    __trustedMediaEvent(this, 'ratechange');
  }
  get volume() {
    return __mediaElementState(this).volume;
  }
  set volume(value) {
    value = Number(value);
    if (!Number.isFinite(value) || value < 0 || value > 1)
      throw new DOMException('Volume is outside [0, 1]', 'IndexSizeError');
    __mediaElementState(this).volume = value;
    __mediaElementController(this)?.setVolume(value);
    __trustedMediaEvent(this, 'volumechange');
  }
  get muted() {
    return __mediaElementState(this).muted;
  }
  set muted(value) {
    __mediaElementState(this).muted = Boolean(value);
    __mediaElementController(this)?.setVolume(this.volume);
    __trustedMediaEvent(this, 'volumechange');
  }
  play() {
    const controller = __mediaElementController(this);
    if (controller === null) {
      return Promise.reject(new DOMException('Media playback is unavailable', 'NotSupportedError'));
    }
    const state = __mediaElementState(this);
    state.paused = false;
    __trustedMediaEvent(this, 'play');
    return Promise.resolve(controller.play()).then(
      () => {
        __trustedMediaEvent(this, 'playing');
      },
      (error) => {
        state.paused = true;
        throw error;
      },
    );
  }
  pause() {
    const state = __mediaElementState(this);
    if (state.paused) return;
    state.currentTime = this.currentTime;
    state.paused = true;
    __mediaElementController(this)?.pause();
    __trustedMediaEvent(this, 'pause');
  }
  load() {
    const state = __mediaElementState(this);
    state.currentTime = 0;
    state.paused = true;
    __mediaElementController(this)?.load();
    __trustedMediaEvent(this, 'loadstart');
  }
  canPlayType(type) {
    type = String(type).toLowerCase();
    return /^(audio\/(wav|wave|x-wav|mpeg|mp4|aac|flac|ogg))\b/.test(type) ? 'probably' : '';
  }
}
for (const [name, value] of Object.entries({
  NETWORK_EMPTY: 0,
  NETWORK_IDLE: 1,
  NETWORK_LOADING: 2,
  NETWORK_NO_SOURCE: 3,
  HAVE_NOTHING: 0,
  HAVE_METADATA: 1,
  HAVE_CURRENT_DATA: 2,
  HAVE_FUTURE_DATA: 3,
  HAVE_ENOUGH_DATA: 4,
})) {
  Object.defineProperty(HTMLMediaElement, name, { value });
  Object.defineProperty(HTMLMediaElement.prototype, name, { value });
}
class HTMLVideoElement extends HTMLMediaElement {
  constructor() {
    throw new TypeError('Illegal constructor');
  }
  get videoWidth() {
    return 0;
  }
  get videoHeight() {
    return 0;
  }
  get webkitDisplayingFullscreen() {
    return false;
  }
  get webkitSupportsFullscreen() {
    return false;
  }
  // Legacy callable placeholders: there is no video decoder or display UI.
  cancelVideoFrameCallback() {}
  requestVideoFrameCallback() {}
  getVideoPlaybackQuality() {}
  requestPictureInPicture() {}
  webkitEnterFullscreen() {}
  webkitEnterFullScreen() {}
  webkitExitFullscreen() {}
  webkitExitFullScreen() {}
}
class HTMLAudioElement extends HTMLMediaElement {
  constructor() {
    throw new TypeError('Illegal constructor');
  }
}
class HTMLSourceElement extends HTMLElement {
  constructor() {
    throw new TypeError('Illegal constructor');
  }
}
class HTMLTrackElement extends HTMLElement {
  constructor() {
    throw new TypeError('Illegal constructor');
  }
}
class HTMLCanvasElement extends HTMLElement {
  constructor() {
    throw new TypeError('Illegal constructor');
  }
}
class HTMLMapElement extends HTMLElement {
  constructor() {
    throw new TypeError('Illegal constructor');
  }
}
class HTMLAreaElement extends HTMLElement {
  constructor() {
    throw new TypeError('Illegal constructor');
  }
}
class HTMLFormElement extends HTMLElement {
  constructor() {
    throw new TypeError('Illegal constructor');
  }
  get length() {
    return this.elements.length;
  }
  requestSubmit(submitter = null) {
    if (submitter !== null) {
      const isButton =
        submitter instanceof HTMLButtonElement && __buttonType(submitter) === 'submit';
      const isInput =
        submitter instanceof HTMLInputElement &&
        (submitter.type === 'submit' || submitter.type === 'image');
      if (!isButton && !isInput) throw new TypeError('submitter is not a submit button');
      const owner = __formOwner(submitter);
      if (owner !== this) {
        throw new DOMException('The submitter is not owned by this form', 'NotFoundError');
      }
    }
    if (!this.noValidate && !submitter?.formNoValidate && !this.checkValidity()) return;
    const event = new SubmitEvent('submit', {
      bubbles: true,
      cancelable: true,
      submitter,
    });
    if (this.dispatchEvent(event)) this.submit();
  }
  submit() {
    if (this.method !== 'get') {
      throw new DOMException('Only GET form submission is implemented', 'NotSupportedError');
    }
    const charset = this.acceptCharset || document.characterSet;
    const pairs = [];
    for (const control of this.querySelectorAll('input,button,select,textarea')) {
      if (!control.name || control.disabled) continue;
      if (
        control instanceof HTMLInputElement &&
        ['button', 'reset', 'submit', 'image', 'file'].includes(control.type)
      )
        continue;
      pairs.push(
        __callHost('formUrlEncode', window, charset, control.name) +
          '=' +
          __callHost('formUrlEncode', window, charset, control.value ?? ''),
      );
    }
    const destination = new URL(this.action || document.URL, document.URL);
    destination.search = pairs.join('&');
    const target = this.target;
    let frame = null;
    if (target) {
      for (const candidate of document.getElementsByTagName('iframe')) {
        if (candidate.name === target || candidate.id === target) {
          frame = candidate;
          break;
        }
      }
    }
    if (frame) frame.__navigate(destination.href);
    else location.href = destination.href;
  }
}
class HTMLFieldSetElement extends HTMLElement {
  constructor() {
    throw new TypeError('Illegal constructor');
  }
}
class HTMLLegendElement extends HTMLElement {
  constructor() {
    throw new TypeError('Illegal constructor');
  }
}
class HTMLLabelElement extends HTMLElement {
  constructor() {
    throw new TypeError('Illegal constructor');
  }
}
const __inputFiles = new WeakMap();
const __fileListData = new WeakMap();
const __inputDirectoryEntries = new WeakMap();
class FileList {
  constructor() {
    throw new TypeError('Illegal constructor');
  }
  get length() {
    return __fileListData.get(this).length;
  }
  item(index) {
    if (!__fileListData.has(this) || !arguments.length)
      throw new TypeError('Expected a FileList and an index');
    return __fileListData.get(this)[+index >>> 0] ?? null;
  }
  *[Symbol.iterator]() {
    yield* __fileListData.get(this);
  }
  get [Symbol.toStringTag]() {
    return 'FileList';
  }
}
globalThis.FileList = FileList;
function __fileList(files) {
  const list = Object.create(FileList.prototype);
  __fileListData.set(list, []);
  __replaceFileList(list, files);
  return list;
}
function __replaceFileList(list, files) {
  for (let i = 0; i < __fileListData.get(list).length; i++) delete list[i];
  const values = Array.from(files);
  __fileListData.set(list, values);
  for (let i = 0; i < values.length; i++)
    Object.defineProperty(list, i, { value: values[i], enumerable: true, configurable: true });
}
function __clearInputFiles(input) {
  const files = __inputFiles.get(input);
  if (files) __replaceFileList(files, []);
}
class HTMLInputElement extends HTMLElement {
  constructor() {
    throw new TypeError('Illegal constructor');
  }
  get autocomplete() {
    return __inputAutocomplete(this);
  }
  set autocomplete(value) {
    __setContentAttribute(this, 'autocomplete', `${value}`);
  }
  get list() {
    // Safari has no month/week state; those keywords use text behavior.
    const type = __asciiLowercase(__getContentAttribute(this, 'type') ?? 'text');
    if (
      [
        'hidden',
        'password',
        'date',
        'time',
        'datetime-local',
        'checkbox',
        'radio',
        'file',
        'submit',
        'image',
        'reset',
        'button',
      ].includes(type)
    )
      return null;
    const id = __getContentAttribute(this, 'list');
    if (!id) return null;
    let root = this,
      parent;
    while ((parent = __callHost('parentNode', root)) !== null) root = parent;
    // Safari excludes a plain fragment and the root element itself. A
    // shadow root is searchable, but association never crosses its boundary.
    if (__callHost('nodeType', root) === 11 && !(root instanceof ShadowRoot)) return null;
    // A first matching ID of another element type blocks later datalists.
    const elements = __callHost('querySelectorAll', root, '*');
    const target = elements.find((element) => __getContentAttribute(element, 'id') === id);
    return target &&
      __callHost('namespaceURI', target) === 'http://www.w3.org/1999/xhtml' &&
      __callHost('localName', target) === 'datalist'
      ? target
      : null;
  }
  get webkitdirectory() {
    return __getContentAttribute(this, 'webkitdirectory') !== null;
  }
  set webkitdirectory(value) {
    __setContentAttribute(this, 'webkitdirectory', value ? '' : null);
  }
  get webkitEntries() {
    // No directory picker/entry backend: the unselected state is a stable
    // frozen empty array, as in Safari. FileList assignment adds no entries.
    if (!__inputDirectoryEntries.has(this)) __inputDirectoryEntries.set(this, Object.freeze([]));
    return __inputDirectoryEntries.get(this);
  }
  get files() {
    if (__inputType(this) !== 'file') return null;
    if (!__inputFiles.has(this)) __inputFiles.set(this, __fileList([]));
    return __inputFiles.get(this);
  }
  set files(value) {
    if (value == null) return;
    if (!__fileListData.has(value)) throw new TypeError('Expected FileList');
    if (__inputType(this) !== 'file') return;
    __inputFiles.set(this, value);
  }
  __setFiles(files) {
    __inputFiles.set(this, __fileList(files));
  }

  get defaultValue() {
    return __getContentAttribute(this, 'value') ?? '';
  }
  set defaultValue(value) {
    __setContentAttribute(this, 'value', `${value}`);
  }
  get alpha() {
    return __getContentAttribute(this, 'alpha') !== null;
  }
  set alpha(value) {
    __setContentAttribute(this, 'alpha', value ? '' : null);
  }
  get min() {
    return __getContentAttribute(this, 'min') ?? '';
  }
  set min(value) {
    __setContentAttribute(this, 'min', `${value}`);
  }
  get max() {
    return __getContentAttribute(this, 'max') ?? '';
  }
  set max(value) {
    __setContentAttribute(this, 'max', `${value}`);
  }
  get step() {
    return __getContentAttribute(this, 'step') ?? '';
  }
  set step(value) {
    __setContentAttribute(this, 'step', `${value}`);
  }
  get value() {
    return __inputValue(this);
  }
  set value(value) {
    value = value === null ? '' : `${value}`;
    const type = __inputType(this),
      mode = __inputValueMode(type);
    if (mode === 'filename') {
      if (value !== '')
        throw new DOMException('File input value can only be cleared', 'InvalidStateError');
      __clearInputFiles(this);
      return;
    }
    const previous = __inputValue(this, type);
    const sanitized = __sanitizeInputValue(this, type, value);
    __inputScriptValue(this, previous, sanitized);
    __inputValues.set(this, sanitized);
    if (mode === 'default' || mode === 'default/on') __setContentAttribute(this, 'value', value);
    if (mode === 'value') __controlValueChanged(this, previous, sanitized);
  }
  get checked() {
    return __inputChecked(this);
  }
  set checked(value) {
    __setInputChecked(this, Boolean(value));
  }
  get indeterminate() {
    return __inputIndeterminateStates.get(this) ?? false;
  }
  set indeterminate(value) {
    __inputIndeterminateStates.set(this, Boolean(value));
  }
}
class HTMLButtonElement extends HTMLElement {
  constructor() {
    throw new TypeError('Illegal constructor');
  }
}
function __buttonType(button) {
  const type = __asciiLowercase(__getContentAttribute(button, 'type') ?? '');
  if (['submit', 'reset', 'button'].includes(type)) return type;
  // Safari resolves the auto state from attribute presence and the immediate
  // parent, including detached controls and commands without a valid target.
  return __getContentAttribute(button, 'command') !== null ||
    __getContentAttribute(button, 'commandfor') !== null ||
    __callHost('parentNode', button) instanceof HTMLSelectElement
    ? 'button'
    : 'submit';
}
class HTMLSelectElement extends HTMLElement {
  constructor() {
    throw new TypeError('Illegal constructor');
  }
}
class HTMLDataListElement extends HTMLElement {
  constructor() {
    throw new TypeError('Illegal constructor');
  }
}
class HTMLOptGroupElement extends HTMLElement {
  constructor() {
    throw new TypeError('Illegal constructor');
  }
}
class HTMLOptionElement extends HTMLElement {
  constructor() {
    throw new TypeError('Illegal constructor');
  }
  get selected() {
    return __optionSelectedStates.has(this)
      ? __optionSelectedStates.get(this)
      : this.defaultSelected;
  }
  set selected(value) {
    __optionSelectedStates.set(this, Boolean(value));
  }
}
class HTMLTextAreaElement extends HTMLElement {
  constructor() {
    throw new TypeError('Illegal constructor');
  }
  get defaultValue() {
    return __textareaDefaultValue(this);
  }
  set defaultValue(value) {
    __callHost('setTextContent', this, `${value}`);
  }
  get value() {
    return (
      __inputValues.has(this) ? __inputValues.get(this) : __textareaDefaultValue(this)
    ).replace(/\r\n?/g, '\n');
  }
  set value(value) {
    value = (value === null ? '' : `${value}`).replace(/\r\n?/g, '\n');
    const previous = (
      __inputValues.has(this) ? __inputValues.get(this) : __textareaDefaultValue(this)
    ).replace(/\r\n?/g, '\n');
    __inputValues.set(this, value);
    __controlValueChanged(this, previous, value);
  }
}
class HTMLOutputElement extends HTMLElement {
  constructor() {
    // Installed Safari exposes output as a constructible element factory.
    const element = __callHost('createElement', document, 'output');
    const prototype = new.target.prototype;
    Object.setPrototypeOf(
      element,
      (typeof prototype === 'object' && prototype !== null) || typeof prototype === 'function'
        ? prototype
        : HTMLOutputElement.prototype,
    );
    return element;
  }
}
class HTMLProgressElement extends HTMLElement {
  constructor() {
    throw new TypeError('Illegal constructor');
  }
}
class HTMLMeterElement extends HTMLElement {
  constructor() {
    throw new TypeError('Illegal constructor');
  }
}
class HTMLTableElement extends HTMLElement {
  constructor() {
    throw new TypeError('Illegal constructor');
  }
}
class HTMLTableCaptionElement extends HTMLElement {
  constructor() {
    throw new TypeError('Illegal constructor');
  }
}
class HTMLTableColElement extends HTMLElement {
  constructor() {
    throw new TypeError('Illegal constructor');
  }
}
class HTMLTableSectionElement extends HTMLElement {
  constructor() {
    throw new TypeError('Illegal constructor');
  }
}
class HTMLTableRowElement extends HTMLElement {
  constructor() {
    throw new TypeError('Illegal constructor');
  }
}
class HTMLTableCellElement extends HTMLElement {
  constructor() {
    throw new TypeError('Illegal constructor');
  }
}
class HTMLHeadElement extends HTMLElement {
  constructor() {
    throw new TypeError('Illegal constructor');
  }
}
class HTMLTitleElement extends HTMLElement {
  constructor() {
    throw new TypeError('Illegal constructor');
  }
  get text() {
    let text = '';
    for (const child of __callHost('childNodes', this)) {
      if (__callHost('nodeType', child) === 3) text += __callHost('textContent', child);
    }
    return text;
  }
  set text(value) {
    __callHost('setTextContent', this, `${value}`);
  }
}
class HTMLLinkElement extends HTMLElement {
  constructor() {
    throw new TypeError('Illegal constructor');
  }
  get disabled() {
    return __getContentAttribute(this, 'disabled') !== null;
  }
  set disabled(value) {
    __setContentAttribute(this, 'disabled', value ? '' : null);
  }
}
class HTMLMetaElement extends HTMLElement {
  constructor() {
    throw new TypeError('Illegal constructor');
  }
}
class HTMLStyleElement extends HTMLElement {
  constructor() {
    throw new TypeError('Illegal constructor');
  }
  get disabled() {
    return this.sheet?.disabled ?? false;
  }
  set disabled(value) {
    const sheet = this.sheet;
    if (sheet !== null) sheet.disabled = Boolean(value);
  }
}
class HTMLParagraphElement extends HTMLElement {
  constructor() {
    throw new TypeError('Illegal constructor');
  }
}
class HTMLHRElement extends HTMLElement {
  constructor() {
    throw new TypeError('Illegal constructor');
  }
}
class HTMLPreElement extends HTMLElement {
  constructor() {
    throw new TypeError('Illegal constructor');
  }
}
class HTMLQuoteElement extends HTMLElement {
  constructor() {
    throw new TypeError('Illegal constructor');
  }
}
class HTMLOListElement extends HTMLElement {
  constructor() {
    throw new TypeError('Illegal constructor');
  }
}
class HTMLUListElement extends HTMLElement {
  constructor() {
    throw new TypeError('Illegal constructor');
  }
}
class HTMLLIElement extends HTMLElement {
  constructor() {
    throw new TypeError('Illegal constructor');
  }
}
class HTMLDListElement extends HTMLElement {
  constructor() {
    throw new TypeError('Illegal constructor');
  }
}
class HTMLDivElement extends HTMLElement {
  constructor() {
    throw new TypeError('Illegal constructor');
  }
}
class HTMLSpanElement extends HTMLElement {
  constructor() {
    throw new TypeError('Illegal constructor');
  }
}
class HTMLDataElement extends HTMLElement {
  constructor() {
    throw new TypeError('Illegal constructor');
  }
}
class HTMLTimeElement extends HTMLElement {
  constructor() {
    throw new TypeError('Illegal constructor');
  }
}
class HTMLBRElement extends HTMLElement {
  constructor() {
    throw new TypeError('Illegal constructor');
  }
}
class HTMLBodyElement extends HTMLElement {
  constructor() {
    throw new TypeError('Illegal constructor');
  }
}
class HTMLHeadingElement extends HTMLElement {
  constructor() {
    throw new TypeError('Illegal constructor');
  }
}
class HTMLHtmlElement extends HTMLElement {
  constructor() {
    throw new TypeError('Illegal constructor');
  }
}
function __scriptText(element) {
  let text = '';
  for (const child of __callHost('childNodes', element)) {
    if (__callHost('nodeType', child) === 3) text += __callHost('textContent', child);
  }
  return text;
}
class HTMLScriptElement extends HTMLElement {
  constructor() {
    throw new TypeError('Illegal constructor');
  }
  get async() {
    return __forceAsyncScripts.has(this) || __getContentAttribute(this, 'async') !== null;
  }
  set async(value) {
    __forceAsyncScripts.delete(this);
    __setContentAttribute(this, 'async', value ? '' : null);
  }
  get text() {
    return __scriptText(this);
  }
  set text(value) {
    __callHost('setTextContent', this, `${value}`);
  }
}
class HTMLTemplateElement extends HTMLElement {
  constructor() {
    throw new TypeError('Illegal constructor');
  }
  get content() {
    return __callHost('templateContent', this);
  }
}
class HTMLSlotElement extends HTMLElement {
  constructor() {
    throw new TypeError('Illegal constructor');
  }
}
class HTMLModElement extends HTMLElement {
  constructor() {
    throw new TypeError('Illegal constructor');
  }
}
class HTMLDetailsElement extends HTMLElement {
  constructor() {
    throw new TypeError('Illegal constructor');
  }
}
class HTMLMenuElement extends HTMLElement {
  constructor() {
    throw new TypeError('Illegal constructor');
  }
}
class HTMLDialogElement extends HTMLElement {
  constructor() {
    throw new TypeError('Illegal constructor');
  }
}
class HTMLMarqueeElement extends HTMLElement {
  constructor() {
    throw new TypeError('Illegal constructor');
  }
}
class HTMLFrameSetElement extends HTMLElement {
  constructor() {
    throw new TypeError('Illegal constructor');
  }
}
class HTMLFrameElement extends HTMLElement {
  constructor() {
    throw new TypeError('Illegal constructor');
  }
  // No legacy frameset browsing-context engine.
  get contentDocument() {
    return null;
  }
  get contentWindow() {
    return null;
  }
}
class HTMLDirectoryElement extends HTMLElement {
  constructor() {
    throw new TypeError('Illegal constructor');
  }
}
class HTMLFontElement extends HTMLElement {
  constructor() {
    throw new TypeError('Illegal constructor');
  }
}

__defineStringReflections(HTMLImageElement.prototype, [
  ['alt'],
  ['sizes'],
  ['useMap', 'usemap'],
  ['name'],
  ['align'],
  ['border'],
]);
__defineUSVStringReflection(HTMLImageElement.prototype, 'srcset');
__defineUrlReflections(HTMLImageElement.prototype, [['src'], ['lowsrc'], ['longDesc', 'longdesc']]);
__defineBooleanReflections(HTMLImageElement.prototype, [['isMap', 'ismap']]);
__defineUnsignedReflections(HTMLImageElement.prototype, [
  ['width', 'width', 0],
  ['height', 'height', 0],
  ['hspace', 'hspace', 0],
  ['vspace', 'vspace', 0],
]);
__defineEnumReflection(
  HTMLImageElement.prototype,
  'crossOrigin',
  'crossorigin',
  ['anonymous', 'use-credentials'],
  null,
  'anonymous',
  { '': 'anonymous' },
  true,
);
__defineEnumReflection(
  HTMLImageElement.prototype,
  'referrerPolicy',
  'referrerpolicy',
  [
    '',
    'no-referrer',
    'no-referrer-when-downgrade',
    'same-origin',
    'origin',
    'strict-origin',
    'origin-when-cross-origin',
    'strict-origin-when-cross-origin',
    'unsafe-url',
  ],
  '',
  '',
);
__defineEnumReflection(
  HTMLImageElement.prototype,
  'decoding',
  'decoding',
  ['async', 'sync', 'auto'],
  'auto',
  'auto',
);
__defineEnumReflection(
  HTMLImageElement.prototype,
  'loading',
  'loading',
  ['lazy', 'eager'],
  'eager',
  'eager',
);
__defineNullAsEmptyStringReflection(HTMLImageElement.prototype, 'border', 'border');
const Image = new Proxy(
  function Image() {
    throw new TypeError("Constructor requires 'new'");
  },
  {
    construct(target, args, newTarget) {
      const dimensions = [];
      for (let index = 0; index < args.length && index < 2; index++)
        dimensions[index] = args[index] === undefined ? undefined : +args[index] >>> 0;
      const image = __callHost('createElement', document, 'img');
      for (let index = 0; index < dimensions.length; index++) {
        const value = dimensions[index];
        if (value !== undefined)
          __setContentAttribute(
            image,
            index === 0 ? 'width' : 'height',
            String(value <= 2147483647 ? value : 0),
          );
      }
      const prototype = newTarget.prototype;
      Object.setPrototypeOf(
        image,
        prototype !== null && (typeof prototype === 'object' || typeof prototype === 'function')
          ? prototype
          : HTMLImageElement.prototype,
      );
      return image;
    },
  },
);
Object.defineProperty(Image, 'prototype', { value: HTMLImageElement.prototype, writable: false });

__defineStringReflections(HTMLIFrameElement.prototype, [
  ['srcdoc'],
  ['name'],
  ['width'],
  ['height'],
  ['align'],
  ['scrolling'],
  ['allow'],
  ['frameBorder', 'frameborder'],
  ['marginHeight', 'marginheight'],
  ['marginWidth', 'marginwidth'],
]);
__defineUrlReflections(HTMLIFrameElement.prototype, [['src'], ['longDesc', 'longdesc']]);
__defineBooleanReflections(HTMLIFrameElement.prototype, [['allowFullscreen', 'allowfullscreen']]);
__defineEnumReflection(
  HTMLIFrameElement.prototype,
  'loading',
  'loading',
  ['lazy', 'eager'],
  'eager',
  'eager',
);
__defineTokenListReflection(HTMLIFrameElement.prototype, 'sandbox', 'sandbox');
__defineEnumReflection(
  HTMLIFrameElement.prototype,
  'referrerPolicy',
  'referrerpolicy',
  [
    '',
    'no-referrer',
    'no-referrer-when-downgrade',
    'same-origin',
    'origin',
    'strict-origin',
    'origin-when-cross-origin',
    'strict-origin-when-cross-origin',
    'unsafe-url',
  ],
  '',
  '',
);
__defineNullAsEmptyStringReflection(HTMLIFrameElement.prototype, 'marginHeight', 'marginheight');
__defineNullAsEmptyStringReflection(HTMLIFrameElement.prototype, 'marginWidth', 'marginwidth');

__defineStringReflections(HTMLEmbedElement.prototype, [
  ['type'],
  ['width'],
  ['height'],
  ['align'],
  ['name'],
]);
__defineUrlReflections(HTMLEmbedElement.prototype, [['src']]);
__defineStringReflections(HTMLObjectElement.prototype, [
  ['type'],
  ['name'],
  ['useMap', 'usemap'],
  ['width'],
  ['height'],
  ['align'],
  ['archive'],
  ['code'],
  ['standby'],
  ['codeType', 'codetype'],
  ['border'],
]);
__defineUrlReflections(HTMLObjectElement.prototype, [['data'], ['codeBase', 'codebase']]);
__defineBooleanReflections(HTMLObjectElement.prototype, [['declare']]);
__defineUnsignedReflections(HTMLObjectElement.prototype, [
  ['hspace', 'hspace', 0],
  ['vspace', 'vspace', 0],
]);
__defineNullAsEmptyStringReflection(HTMLObjectElement.prototype, 'border', 'border');
__defineStringReflections(HTMLParamElement.prototype, [
  ['name'],
  ['value'],
  ['type'],
  ['valueType', 'valuetype'],
]);

__defineUrlReflections(HTMLMediaElement.prototype, [['src']]);
__defineBooleanReflections(HTMLMediaElement.prototype, [
  ['autoplay'],
  ['loop'],
  ['controls'],
  ['defaultMuted', 'muted'],
  ['disableRemotePlayback', 'disableremoteplayback'],
]);
__defineEnumReflection(
  HTMLMediaElement.prototype,
  'crossOrigin',
  'crossorigin',
  ['anonymous', 'use-credentials'],
  null,
  'anonymous',
  { '': 'anonymous' },
  true,
);
__defineEnumReflection(
  HTMLMediaElement.prototype,
  'preload',
  'preload',
  ['none', 'metadata', 'auto'],
  'auto',
  'auto',
  { '': 'auto' },
);
__defineEnumReflection(
  HTMLMediaElement.prototype,
  'loading',
  'loading',
  ['lazy', 'eager'],
  'eager',
  'eager',
);
__defineUnsignedReflections(HTMLVideoElement.prototype, [
  ['width', 'width', 0],
  ['height', 'height', 0],
]);
__defineUrlReflections(HTMLVideoElement.prototype, [['poster']]);
__defineBooleanReflections(HTMLVideoElement.prototype, [
  ['playsInline', 'playsinline'],
  ['disablePictureInPicture', 'disablepictureinpicture'],
]);

__defineStringReflections(HTMLSourceElement.prototype, [
  ['type'],
  ['srcset'],
  ['sizes'],
  ['media'],
]);
__defineUrlReflections(HTMLSourceElement.prototype, [['src']]);
__defineStringReflections(HTMLTrackElement.prototype, [['srclang'], ['label']]);
__defineUrlReflections(HTMLTrackElement.prototype, [['src']]);
__defineBooleanReflections(HTMLTrackElement.prototype, [['default']]);
__defineEnumReflection(
  HTMLTrackElement.prototype,
  'kind',
  'kind',
  ['subtitles', 'captions', 'descriptions', 'chapters', 'metadata'],
  'subtitles',
  'metadata',
);
__defineUnsignedReflections(HTMLCanvasElement.prototype, [
  ['width', 'width', 300],
  ['height', 'height', 150],
]);
__defineStringReflections(HTMLMapElement.prototype, [['name']]);
__defineStringReflections(HTMLAreaElement.prototype, [
  ['alt'],
  ['coords'],
  ['shape'],
  ['target'],
  ['download'],
  ['ping'],
  ['rel'],
  ['hreflang'],
  ['type'],
]);
__defineUrlReflections(HTMLAreaElement.prototype, [['href']]);
__defineBooleanReflections(HTMLAreaElement.prototype, [['noHref', 'nohref']]);
__defineTokenListReflection(HTMLAreaElement.prototype, 'relList', 'rel');
__defineEnumReflection(
  HTMLAreaElement.prototype,
  'referrerPolicy',
  'referrerpolicy',
  [
    '',
    'no-referrer',
    'no-referrer-when-downgrade',
    'same-origin',
    'origin',
    'strict-origin',
    'origin-when-cross-origin',
    'strict-origin-when-cross-origin',
    'unsafe-url',
  ],
  '',
  '',
);

__defineStringReflections(HTMLAnchorElement.prototype, [
  ['target'],
  ['download'],
  ['rel'],
  ['hreflang'],
  ['type'],
  ['coords'],
  ['charset'],
  ['name'],
  ['rev'],
  ['shape'],
]);
__defineUSVStringReflection(HTMLAnchorElement.prototype, 'ping');
__defineTokenListReflection(HTMLAnchorElement.prototype, 'relList', 'rel');
__defineEnumReflection(
  HTMLAnchorElement.prototype,
  'referrerPolicy',
  'referrerpolicy',
  [
    '',
    'no-referrer',
    'no-referrer-when-downgrade',
    'same-origin',
    'origin',
    'strict-origin',
    'origin-when-cross-origin',
    'strict-origin-when-cross-origin',
    'unsafe-url',
  ],
  '',
  '',
);
__defineStringReflections(HTMLBaseElement.prototype, [['target']]);

__defineStringReflections(HTMLFormElement.prototype, [
  ['acceptCharset', 'accept-charset'],
  ['name'],
  ['target'],
]);
__defineActionUrlReflection(HTMLFormElement.prototype, 'action', 'action');
__defineBooleanReflections(HTMLFormElement.prototype, [['noValidate', 'novalidate']]);
__defineEnumReflection(
  HTMLFormElement.prototype,
  'autocomplete',
  'autocomplete',
  ['on', 'off'],
  'on',
  'on',
);
__defineEnumReflection(
  HTMLFormElement.prototype,
  'enctype',
  'enctype',
  ['application/x-www-form-urlencoded', 'multipart/form-data', 'text/plain'],
  'application/x-www-form-urlencoded',
  'application/x-www-form-urlencoded',
);
__defineEnumReflection(
  HTMLFormElement.prototype,
  'encoding',
  'enctype',
  ['application/x-www-form-urlencoded', 'multipart/form-data', 'text/plain'],
  'application/x-www-form-urlencoded',
  'application/x-www-form-urlencoded',
);
__defineEnumReflection(
  HTMLFormElement.prototype,
  'method',
  'method',
  ['get', 'post', 'dialog'],
  'get',
  'get',
);
__defineStringReflections(HTMLFieldSetElement.prototype, [['name']]);
__defineBooleanReflections(HTMLFieldSetElement.prototype, [['disabled']]);
__defineStringReflections(HTMLLegendElement.prototype, [['align']]);
__defineStringReflections(HTMLLabelElement.prototype, [['htmlFor', 'for']]);

__defineStringReflections(HTMLInputElement.prototype, [
  ['accept'],
  ['alt'],
  ['dirName', 'dirname'],
  ['name'],
  ['pattern'],
  ['placeholder'],
  ['align'],
  ['useMap', 'usemap'],
  ['formTarget', 'formtarget'],
]);
__defineBooleanReflections(HTMLInputElement.prototype, [
  ['defaultChecked', 'checked'],
  ['disabled'],
  ['formNoValidate', 'formnovalidate'],
  ['multiple'],
  ['readOnly', 'readonly'],
  ['required'],
]);
__defineUrlReflections(HTMLInputElement.prototype, [['src']]);
__defineActionUrlReflection(HTMLInputElement.prototype, 'formAction', 'formaction');
for (const name of ['height', 'width'])
  Object.defineProperty(
    HTMLInputElement.prototype,
    name,
    Object.getOwnPropertyDescriptor(
      {
        get [name]() {
          return __inputType(this) === 'image'
            ? (__parseNonnegativeInteger(__getContentAttribute(this, name) ?? '') ?? 0)
            : 0;
        },
        set [name](value) {
          const number = +value >>> 0;
          __setContentAttribute(this, name, String(number <= 2147483647 ? number : 0));
        },
      },
      name,
    ),
  );
__defineLengthReflections(HTMLInputElement.prototype);
__definePositiveUnsignedReflections(HTMLInputElement.prototype, [['size', 'size', 20, false]]);
__defineEnumReflection(
  HTMLInputElement.prototype,
  'formEnctype',
  'formenctype',
  ['application/x-www-form-urlencoded', 'multipart/form-data', 'text/plain'],
  '',
  'application/x-www-form-urlencoded',
);
__defineEnumReflection(
  HTMLInputElement.prototype,
  'formMethod',
  'formmethod',
  ['get', 'post', 'dialog'],
  '',
  'get',
);
__defineEnumReflection(HTMLInputElement.prototype, 'type', 'type', __inputTypes, 'text', 'text');
__defineEnumReflection(
  HTMLInputElement.prototype,
  'colorSpace',
  'colorspace',
  ['limited-srgb', 'display-p3'],
  'limited-srgb',
  'limited-srgb',
);

__defineStringReflections(HTMLButtonElement.prototype, [
  ['formTarget', 'formtarget'],
  ['name'],
  ['value'],
]);
__defineBooleanReflections(HTMLButtonElement.prototype, [
  ['disabled'],
  ['formNoValidate', 'formnovalidate'],
]);
__defineActionUrlReflection(HTMLButtonElement.prototype, 'formAction', 'formaction');
__defineEnumReflection(
  HTMLButtonElement.prototype,
  'formEnctype',
  'formenctype',
  ['application/x-www-form-urlencoded', 'multipart/form-data', 'text/plain'],
  '',
  'application/x-www-form-urlencoded',
);
__defineEnumReflection(
  HTMLButtonElement.prototype,
  'formMethod',
  'formmethod',
  ['get', 'post', 'dialog'],
  '',
  'get',
);
Object.defineProperty(
  HTMLButtonElement.prototype,
  'type',
  Object.getOwnPropertyDescriptor(
    {
      get type() {
        return __buttonType(this);
      },
      set type(value) {
        __setContentAttribute(this, 'type', `${value}`);
      },
    },
    'type',
  ),
);

__defineStringReflections(HTMLSelectElement.prototype, [['autocomplete'], ['name']]);
__defineBooleanReflections(HTMLSelectElement.prototype, [['disabled'], ['multiple'], ['required']]);
__defineUnsignedReflections(HTMLSelectElement.prototype, [['size', 'size', 0]]);
__defineStringReflections(HTMLOptGroupElement.prototype, [['label']]);
__defineBooleanReflections(HTMLOptGroupElement.prototype, [['disabled']]);
__defineStringReflections(HTMLOptionElement.prototype, [['label'], ['value']]);
__defineBooleanReflections(HTMLOptionElement.prototype, [
  ['disabled'],
  ['defaultSelected', 'selected'],
]);
__defineStringReflections(HTMLTextAreaElement.prototype, [
  ['autocomplete'],
  ['dirName', 'dirname'],
  ['name'],
  ['placeholder'],
  ['wrap'],
]);
__defineBooleanReflections(HTMLTextAreaElement.prototype, [
  ['disabled'],
  ['readOnly', 'readonly'],
  ['required'],
]);
__defineLengthReflections(HTMLTextAreaElement.prototype);
__definePositiveUnsignedReflections(HTMLTextAreaElement.prototype, [
  ['cols', 'cols', 20, true],
  ['rows', 'rows', 2, true],
]);
__defineTokenListReflection(HTMLOutputElement.prototype, 'htmlFor', 'for');
__defineStringReflections(HTMLOutputElement.prototype, [['name']]);
__defineDoubleReflections(HTMLProgressElement.prototype, [['max', 'max', 1, true]]);
__defineDoubleReflections(HTMLMeterElement.prototype, [
  ['value'],
  ['min'],
  ['max'],
  ['low'],
  ['high'],
  ['optimum'],
]);

__defineStringReflections(HTMLTableElement.prototype, [
  ['align'],
  ['border'],
  ['frame'],
  ['rules'],
  ['summary'],
  ['width'],
  ['bgColor', 'bgcolor'],
  ['cellPadding', 'cellpadding'],
  ['cellSpacing', 'cellspacing'],
]);
for (const [property, attribute] of [
  ['bgColor', 'bgcolor'],
  ['cellPadding', 'cellpadding'],
  ['cellSpacing', 'cellspacing'],
]) {
  __defineNullAsEmptyStringReflection(HTMLTableElement.prototype, property, attribute);
}
__defineStringReflections(HTMLTableCaptionElement.prototype, [['align']]);
__defineClampedUnsignedReflections(HTMLTableColElement.prototype, [['span', 'span', 1, 1, 1000]]);
__defineStringReflections(HTMLTableColElement.prototype, [
  ['align'],
  ['ch', 'char'],
  ['chOff', 'charoff'],
  ['vAlign', 'valign'],
  ['width'],
]);
__defineStringReflections(HTMLTableSectionElement.prototype, [
  ['align'],
  ['ch', 'char'],
  ['chOff', 'charoff'],
  ['vAlign', 'valign'],
]);
__defineStringReflections(HTMLTableRowElement.prototype, [
  ['align'],
  ['ch', 'char'],
  ['chOff', 'charoff'],
  ['vAlign', 'valign'],
  ['bgColor', 'bgcolor'],
]);
__defineNullAsEmptyStringReflection(HTMLTableRowElement.prototype, 'bgColor', 'bgcolor');
__defineClampedUnsignedReflections(HTMLTableCellElement.prototype, [
  ['colSpan', 'colspan', 1, 1, 1000],
  ['rowSpan', 'rowspan', 1, 0, 65534],
]);
__defineStringReflections(HTMLTableCellElement.prototype, [
  ['headers'],
  ['abbr'],
  ['align'],
  ['axis'],
  ['height'],
  ['width'],
  ['ch', 'char'],
  ['chOff', 'charoff'],
  ['vAlign', 'valign'],
  ['bgColor', 'bgcolor'],
]);
__defineBooleanReflections(HTMLTableCellElement.prototype, [['noWrap', 'nowrap']]);
__defineEnumReflection(
  HTMLTableCellElement.prototype,
  'scope',
  'scope',
  ['row', 'col', 'rowgroup', 'colgroup'],
  '',
  '',
);
__defineNullAsEmptyStringReflection(HTMLTableCellElement.prototype, 'bgColor', 'bgcolor');

__defineStringReflections(HTMLLinkElement.prototype, [
  ['rel'],
  ['media'],
  ['integrity'],
  ['hreflang'],
  ['type'],
  ['charset'],
  ['rev'],
  ['target'],
  ['imageSizes', 'imagesizes'],
]);
__defineUSVStringReflection(HTMLLinkElement.prototype, 'imageSrcset', 'imagesrcset');
__defineNonceReflection(HTMLLinkElement.prototype);
__defineUrlReflections(HTMLLinkElement.prototype, [['href']]);
__defineTokenListReflection(HTMLLinkElement.prototype, 'relList', 'rel');
__defineTokenListReflection(HTMLLinkElement.prototype, 'sizes', 'sizes');
__defineEnumReflection(
  HTMLLinkElement.prototype,
  'crossOrigin',
  'crossorigin',
  ['anonymous', 'use-credentials'],
  null,
  'anonymous',
  { '': 'anonymous' },
  true,
);
__defineEnumReflection(
  HTMLLinkElement.prototype,
  'as',
  'as',
  [
    'fetch',
    'audio',
    'document',
    'embed',
    'font',
    'image',
    'manifest',
    'object',
    'report',
    'script',
    'sharedworker',
    'style',
    'track',
    'video',
    'worker',
    'xslt',
  ],
  '',
  '',
);
__defineEnumReflection(
  HTMLLinkElement.prototype,
  'referrerPolicy',
  'referrerpolicy',
  [
    '',
    'no-referrer',
    'no-referrer-when-downgrade',
    'same-origin',
    'origin',
    'strict-origin',
    'origin-when-cross-origin',
    'strict-origin-when-cross-origin',
    'unsafe-url',
  ],
  '',
  '',
);
__defineStringReflections(HTMLMetaElement.prototype, [
  ['name'],
  ['httpEquiv', 'http-equiv'],
  ['content'],
  ['media'],
  ['scheme'],
]);
__defineStringReflections(HTMLStyleElement.prototype, [['media'], ['type']]);
__defineNonceReflection(HTMLStyleElement.prototype);

__defineStringReflections(HTMLParagraphElement.prototype, [['align']]);
__defineStringReflections(HTMLHRElement.prototype, [['align'], ['color'], ['size'], ['width']]);
__defineBooleanReflections(HTMLHRElement.prototype, [['noShade', 'noshade']]);
__defineLongReflections(HTMLPreElement.prototype, [['width', 'width', 0]]);
__defineUrlReflections(HTMLQuoteElement.prototype, [['cite']]);
__defineBooleanReflections(HTMLOListElement.prototype, [['reversed'], ['compact']]);
__defineLongReflections(HTMLOListElement.prototype, [['start', 'start', 1]]);
__defineStringReflections(HTMLOListElement.prototype, [['type']]);
__defineBooleanReflections(HTMLUListElement.prototype, [['compact']]);
__defineStringReflections(HTMLUListElement.prototype, [['type']]);
__defineLongReflections(HTMLLIElement.prototype, [['value', 'value', 0]]);
__defineStringReflections(HTMLLIElement.prototype, [['type']]);
__defineBooleanReflections(HTMLDListElement.prototype, [['compact']]);
__defineStringReflections(HTMLDivElement.prototype, [['align']]);
__defineStringReflections(HTMLDataElement.prototype, [['value']]);
__defineStringReflections(HTMLTimeElement.prototype, [['dateTime', 'datetime']]);
__defineStringReflections(HTMLBRElement.prototype, [['clear']]);

__defineStringReflections(HTMLBodyElement.prototype, [
  ['text'],
  ['link'],
  ['vLink', 'vlink'],
  ['aLink', 'alink'],
  ['bgColor', 'bgcolor'],
  ['background'],
]);
for (const [property, attribute] of [
  ['text', 'text'],
  ['link', 'link'],
  ['vLink', 'vlink'],
  ['aLink', 'alink'],
  ['bgColor', 'bgcolor'],
]) {
  __defineNullAsEmptyStringReflection(HTMLBodyElement.prototype, property, attribute);
}
__defineStringReflections(HTMLHeadingElement.prototype, [['align']]);
for (const [property, attribute] of [
  ['fgColor', 'text'],
  ['linkColor', 'link'],
  ['vlinkColor', 'vlink'],
  ['alinkColor', 'alink'],
  ['bgColor', 'bgcolor'],
]) {
  Object.defineProperty(Document.prototype, property, {
    configurable: true,
    enumerable: true,
    get() {
      return this.body?.getAttribute(attribute) ?? '';
    },
    set(value) {
      if (this.body) this.body.setAttribute(attribute, value === null ? '' : String(value));
    },
  });
}

__defineStringReflections(HTMLHtmlElement.prototype, [['version']]);
__defineStringReflections(HTMLScriptElement.prototype, [
  ['type'],
  ['charset'],
  ['integrity'],
  ['event'],
  ['htmlFor', 'for'],
]);
__defineUrlReflections(HTMLScriptElement.prototype, [['src']]);
__defineBooleanReflections(HTMLScriptElement.prototype, [['noModule', 'nomodule'], ['defer']]);
__defineEnumReflection(
  HTMLScriptElement.prototype,
  'crossOrigin',
  'crossorigin',
  ['anonymous', 'use-credentials'],
  null,
  'anonymous',
  { '': 'anonymous' },
  true,
);
__defineEnumReflection(
  HTMLScriptElement.prototype,
  'referrerPolicy',
  'referrerpolicy',
  [
    '',
    'no-referrer',
    'no-referrer-when-downgrade',
    'same-origin',
    'origin',
    'strict-origin',
    'origin-when-cross-origin',
    'strict-origin-when-cross-origin',
    'unsafe-url',
  ],
  '',
  '',
);
__defineStringReflections(HTMLSlotElement.prototype, [['name']]);
__defineUrlReflections(HTMLModElement.prototype, [['cite']]);
__defineStringReflections(HTMLModElement.prototype, [['dateTime', 'datetime']]);
__defineBooleanReflections(HTMLDetailsElement.prototype, [['open']]);
__defineBooleanReflections(HTMLMenuElement.prototype, [['compact']]);
__defineBooleanReflections(HTMLDialogElement.prototype, [['open']]);
__defineEnumReflection(
  HTMLElement.prototype,
  'enterKeyHint',
  'enterkeyhint',
  ['enter', 'done', 'go', 'next', 'previous', 'search', 'send'],
  '',
  '',
);
__defineEnumReflection(
  HTMLElement.prototype,
  'inputMode',
  'inputmode',
  ['none', 'text', 'tel', 'url', 'email', 'numeric', 'decimal', 'search'],
  '',
  '',
);

__defineStringReflections(HTMLMarqueeElement.prototype, [
  ['bgColor', 'bgcolor'],
  ['height'],
  ['width'],
]);
__defineUnsignedReflections(HTMLMarqueeElement.prototype, [
  ['hspace', 'hspace', 0],
  ['scrollAmount', 'scrollamount', 6],
  ['scrollDelay', 'scrolldelay', 85],
  ['vspace', 'vspace', 0],
]);
__defineBooleanReflections(HTMLMarqueeElement.prototype, [['trueSpeed', 'truespeed']]);
__defineEnumReflection(
  HTMLMarqueeElement.prototype,
  'behavior',
  'behavior',
  ['scroll', 'slide', 'alternate'],
  'scroll',
  'scroll',
);
__defineEnumReflection(
  HTMLMarqueeElement.prototype,
  'direction',
  'direction',
  ['up', 'right', 'down', 'left'],
  'left',
  'left',
);
__defineStringReflections(HTMLFrameSetElement.prototype, [['cols'], ['rows']]);
__defineStringReflections(HTMLFrameElement.prototype, [
  ['name'],
  ['scrolling'],
  ['frameBorder', 'frameborder'],
  ['marginHeight', 'marginheight'],
  ['marginWidth', 'marginwidth'],
]);
__defineUrlReflections(HTMLFrameElement.prototype, [['src'], ['longDesc', 'longdesc']]);
__defineBooleanReflections(HTMLFrameElement.prototype, [['noResize', 'noresize']]);
__defineNullAsEmptyStringReflection(HTMLFrameElement.prototype, 'marginHeight', 'marginheight');
__defineNullAsEmptyStringReflection(HTMLFrameElement.prototype, 'marginWidth', 'marginwidth');
__defineBooleanReflections(HTMLDirectoryElement.prototype, [['compact']]);
__defineStringReflections(HTMLFontElement.prototype, [['color'], ['face'], ['size']]);
__defineNullAsEmptyStringReflection(HTMLFontElement.prototype, 'color', 'color');
