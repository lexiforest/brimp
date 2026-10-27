class DOMException extends Error {
  constructor(message = '', name = 'Error') {
    super(String(message));
    this.name = String(name);
  }
  get code() {
    return DOMException.__codes[this.name] || 0;
  }
}
DOMException.__codes = {
  IndexSizeError: 1,
  HierarchyRequestError: 3,
  WrongDocumentError: 4,
  InvalidCharacterError: 5,
  NoModificationAllowedError: 7,
  NotFoundError: 8,
  NotSupportedError: 9,
  InUseAttributeError: 10,
  InvalidStateError: 11,
  SyntaxError: 12,
  InvalidModificationError: 13,
  NamespaceError: 14,
  InvalidAccessError: 15,
  TypeMismatchError: 17,
  SecurityError: 18,
  NetworkError: 19,
  AbortError: 20,
  URLMismatchError: 21,
  QuotaExceededError: 22,
  TimeoutError: 23,
  InvalidNodeTypeError: 24,
  DataCloneError: 25,
};
for (const [name, code] of Object.entries(DOMException.__codes)) {
  const constant =
    name
      .replace(/Error$/, '')
      .replace(/([a-z])([A-Z])/g, '$1_$2')
      .toUpperCase() + '_ERR';
  Object.defineProperty(DOMException, constant, { value: code, enumerable: true });
  Object.defineProperty(DOMException.prototype, constant, { value: code, enumerable: true });
}

const __trustedEvents = new WeakSet();
const __pendingTrustedEvents = new WeakSet();
const __uninitializedEvents = new WeakSet();
const __eventData = new WeakMap();
function __eventRecord(event) {
  const record = __eventData.get(event);
  if (!record) throw new TypeError('Illegal invocation');
  return record;
}
function __markTrustedEvent(event) {
  __trustedEvents.add(event);
  __pendingTrustedEvents.add(event);
  if (__keyboardEventData.has(event)) {
    const data = __keyboardEventData.get(event);
    __uiEventData.get(event).which = event.type === 'keypress' ? data.charCode : data.keyCode;
  }
  return event;
}
Object.defineProperty(globalThis, '__brimpMarkTrustedEvent', {
  value: __markTrustedEvent,
  writable: false,
  enumerable: false,
  configurable: true,
});

class Event {
  constructor(type, options = {}) {
    if (arguments.length === 0) throw new TypeError('Event type is required');
    type = `${type}`;
    if (options == null) options = {};
    if (typeof options !== 'object' && typeof options !== 'function')
      throw new TypeError('Expected an event dictionary');
    __eventData.set(this, {
      type,
      bubbles: Boolean(options.bubbles),
      cancelable: Boolean(options.cancelable),
      composed: Boolean(options.composed),
      target: null,
      currentTarget: null,
      eventPhase: 0,
      defaultPrevented: false,
      timeStamp: __nativeHost('clockNow'),
    });
    Object.defineProperty(this, 'isTrusted', {
      enumerable: true,
      get() {
        __eventRecord(this);
        return __trustedEvents.has(this);
      },
    });
    this.__stopped = false;
    this.__immediateStopped = false;
    this.__dispatching = false;
    this.__inPassiveListener = false;
  }
  get type() {
    return __eventRecord(this).type;
  }
  get bubbles() {
    return __eventRecord(this).bubbles;
  }
  get cancelable() {
    return __eventRecord(this).cancelable;
  }
  get composed() {
    return __eventRecord(this).composed;
  }
  get target() {
    return __eventRecord(this).target;
  }
  get srcElement() {
    return __eventRecord(this).target;
  }
  get currentTarget() {
    return __eventRecord(this).currentTarget;
  }
  get eventPhase() {
    return __eventRecord(this).eventPhase;
  }
  get defaultPrevented() {
    return __eventRecord(this).defaultPrevented;
  }
  get timeStamp() {
    return __eventRecord(this).timeStamp;
  }
  preventDefault() {
    const record = __eventRecord(this);
    if (record.cancelable && !this.__inPassiveListener) record.defaultPrevented = true;
  }
  stopPropagation() {
    this.__stopped = true;
  }
  stopImmediatePropagation() {
    this.__immediateStopped = true;
    this.__stopped = true;
  }
  composedPath() {
    return __visibleEventPath(this);
  }
  initEvent(type, bubbles = false, cancelable = false) {
    if (arguments.length === 0) throw new TypeError('Event type is required');
    type = `${type}`;
    if (this.__dispatching) return;
    const record = __eventRecord(this);
    record.type = type;
    record.bubbles = Boolean(bubbles);
    record.cancelable = Boolean(cancelable);
    record.target = null;
    __uninitializedEvents.delete(this);
    __trustedEvents.delete(this);
    __pendingTrustedEvents.delete(this);
    record.defaultPrevented = false;
    this.__stopped = false;
    this.__immediateStopped = false;
  }
  get cancelBubble() {
    return this.__stopped;
  }
  set cancelBubble(value) {
    if (value) this.stopPropagation();
  }
  get returnValue() {
    return !this.defaultPrevented;
  }
  set returnValue(value) {
    if (!value) this.preventDefault();
  }
}
for (const [name, value] of Object.entries({
  NONE: 0,
  CAPTURING_PHASE: 1,
  AT_TARGET: 2,
  BUBBLING_PHASE: 3,
})) {
  Object.defineProperty(Event, name, { value, enumerable: true });
  Object.defineProperty(Event.prototype, name, { value, enumerable: true });
}
const __preventEventDefault = Event.prototype.preventDefault;
const __initializeEvent = Event.prototype.initEvent;

const __customEventData = new WeakMap();
class CustomEvent extends Event {
  constructor(type, options = {}) {
    if (arguments.length === 0) throw new TypeError('Event type is required');
    if (options == null) options = {};
    super(type, options);
    const detail = options.detail;
    __customEventData.set(this, { detail: detail === undefined ? null : detail });
  }
  get detail() {
    return __customEventData.get(this).detail;
  }
  initCustomEvent(type, bubbles = false, cancelable = false, detail = null) {
    if (arguments.length === 0) throw new TypeError('Event type is required');
    type = `${type}`;
    if (this.__dispatching) return;
    __initializeEvent.call(this, type, bubbles, cancelable);
    __customEventData.get(this).detail = detail;
  }
}

const __errorEventData = new WeakMap();
class ErrorEvent extends Event {
  constructor(type, options = {}) {
    if (arguments.length === 0) throw new TypeError('ErrorEvent type is required');
    type = `${type}`;
    if (options == null) options = {};
    if (typeof options !== 'object' && typeof options !== 'function')
      throw new TypeError('Expected an event dictionary');
    // Inherited dictionary members precede the derived members, with each
    // dictionary's own members read and converted in lexical order.
    const bubbles = Boolean(options.bubbles),
      cancelable = Boolean(options.cancelable);
    const composed = Boolean(options.composed);
    const colno = options.colno,
      column = colno === undefined ? 0 : +colno >>> 0;
    const error = options.error;
    const filename = options.filename,
      file = filename === undefined ? '' : __toUSVString(`${filename}`);
    const lineno = options.lineno,
      line = lineno === undefined ? 0 : +lineno >>> 0;
    const message = options.message,
      text = message === undefined ? '' : `${message}`;
    super(type, { bubbles, cancelable, composed });
    __errorEventData.set(this, {
      message: text,
      filename: file,
      lineno: line,
      colno: column,
      error,
    });
  }
  get message() {
    return __errorEventRecord(this).message;
  }
  get filename() {
    return __errorEventRecord(this).filename;
  }
  get lineno() {
    return __errorEventRecord(this).lineno;
  }
  get colno() {
    return __errorEventRecord(this).colno;
  }
  get error() {
    return __errorEventRecord(this).error;
  }
}
function __errorEventRecord(event) {
  const record = __errorEventData.get(event);
  if (!record) throw new TypeError('Illegal invocation');
  return record;
}

const __beforeUnloadEventData = new WeakMap();
class BeforeUnloadEvent extends Event {
  constructor() {
    throw new TypeError('Illegal constructor');
  }
  get returnValue() {
    if (!__beforeUnloadEventData.has(this)) throw new TypeError('Illegal invocation');
    return __beforeUnloadEventData.get(this);
  }
  set returnValue(value) {
    if (!__beforeUnloadEventData.has(this)) throw new TypeError('Illegal invocation');
    __beforeUnloadEventData.set(this, `${value}`);
  }
}
function __createBeforeUnloadEvent() {
  const event = new Event('');
  Object.setPrototypeOf(event, BeforeUnloadEvent.prototype);
  __beforeUnloadEventData.set(event, '');
  __uninitializedEvents.add(event);
  return event;
}

const __uiEventData = new WeakMap();
function __eventView(value) {
  if (value == null) return null;
  if (!__windowEventTargets.has(value)) throw new TypeError('Expected a Window');
  return value;
}
class UIEvent extends Event {
  constructor(type, options = {}) {
    if (arguments.length === 0) throw new TypeError('Event type is required');
    if (options == null) options = {};
    super(type, options);
    const detail = +options.detail >> 0,
      view = __eventView(options.view);
    __uiEventData.set(this, { view, detail, which: 0 });
  }
  get view() {
    return __uiEventData.get(this).view;
  }
  get detail() {
    return __uiEventData.get(this).detail;
  }
  get which() {
    return __uiEventData.get(this).which;
  }
  initUIEvent(type, bubbles = false, cancelable = false, view = null, detail = 0) {
    if (arguments.length === 0) throw new TypeError('Event type is required');
    type = `${type}`;
    view = __eventView(view);
    detail = +detail >> 0;
    if (this.__dispatching) return;
    __initializeEvent.call(this, type, bubbles, cancelable);
    Object.assign(__uiEventData.get(this), { view, detail });
  }
}

const __initializeUIEvent = UIEvent.prototype.initUIEvent;
const __textEventData = new WeakMap();
class TextEvent extends UIEvent {
  constructor() {
    throw new TypeError('Illegal constructor');
  }
  get data() {
    return __textEventData.get(this);
  }
  initTextEvent(type, bubbles = false, cancelable = false, view = null, data = undefined) {
    if (arguments.length === 0) throw new TypeError('Event type is required');
    type = `${type}`;
    view = __eventView(view);
    data = `${data}`;
    if (this.__dispatching) return;
    __initializeUIEvent.call(this, type, bubbles, cancelable, view, 0);
    __textEventData.set(this, data);
  }
}
function __createTextEvent() {
  const event = new UIEvent('');
  Object.setPrototypeOf(event, TextEvent.prototype);
  __textEventData.set(event, '');
  __uninitializedEvents.add(event);
  return event;
}

const __eventModifiers = new WeakMap();
function __initializeModifiers(event, options) {
  const state = Object.create(null);
  for (const [key, option] of Object.entries({
    Control: 'ctrlKey',
    Shift: 'shiftKey',
    Alt: 'altKey',
    Meta: 'metaKey',
    AltGraph: 'modifierAltGraph',
    CapsLock: 'modifierCapsLock',
    Fn: 'modifierFn',
    FnLock: 'modifierFnLock',
    Hyper: 'modifierHyper',
    NumLock: 'modifierNumLock',
    ScrollLock: 'modifierScrollLock',
    Super: 'modifierSuper',
    Symbol: 'modifierSymbol',
    SymbolLock: 'modifierSymbolLock',
  })) {
    state[key] = Boolean(options[option]);
  }
  __eventModifiers.set(event, state);
}
function __getModifierState(event, key, count) {
  if (!count) throw new TypeError('Not enough arguments');
  key = `${key}`;
  return __eventModifiers.get(event)[key] ?? false;
}
function __eventDouble(value) {
  return value === undefined ? 0 : +value;
}
function __eventRelatedTarget(value) {
  if (value == null) return null;
  if (
    !__eventTargets.has(value) &&
    !__windowEventTargets.has(value) &&
    !(
      (typeof value === 'object' || typeof value === 'function') &&
      __nativeHost('isNativeNode', value)
    )
  )
    throw new TypeError('Expected an EventTarget');
  return value;
}
const __mouseEventData = new WeakMap();
class MouseEvent extends UIEvent {
  constructor(type, options = {}) {
    if (!arguments.length) throw new TypeError('Event type is required');
    if (options == null) options = {};
    super(type, options);
    const data = {
      screenX: __eventDouble(options.screenX),
      screenY: __eventDouble(options.screenY),
      clientX: __eventDouble(options.clientX),
      clientY: __eventDouble(options.clientY),
      button: (+options.button << 16) >> 16,
      buttons: +options.buttons & 65535,
      relatedTarget: __eventRelatedTarget(options.relatedTarget),
      movementX: __eventDouble(options.movementX),
      movementY: __eventDouble(options.movementY),
    };
    __mouseEventData.set(this, data);
    __uiEventData.get(this).which = data.button + 1;
    __initializeModifiers(this, options);
  }
  get screenX() {
    return Math.floor(__mouseEventData.get(this).screenX);
  }
  get screenY() {
    return Math.floor(__mouseEventData.get(this).screenY);
  }
  get clientX() {
    return Math.floor(__mouseEventData.get(this).clientX);
  }
  get clientY() {
    return Math.floor(__mouseEventData.get(this).clientY);
  }
  get button() {
    return __mouseEventData.get(this).button;
  }
  get buttons() {
    return __mouseEventData.get(this).buttons;
  }
  get relatedTarget() {
    return __mouseEventData.get(this).relatedTarget;
  }
  get movementX() {
    return __mouseEventData.get(this).movementX;
  }
  get movementY() {
    return __mouseEventData.get(this).movementY;
  }
  get ctrlKey() {
    return __eventModifiers.get(this).Control;
  }
  get shiftKey() {
    return __eventModifiers.get(this).Shift;
  }
  get altKey() {
    return __eventModifiers.get(this).Alt;
  }
  get metaKey() {
    return __eventModifiers.get(this).Meta;
  }
  get fromElement() {
    return ['mouseout', 'mouseleave'].includes(this.type) ? this.target : this.relatedTarget;
  }
  get toElement() {
    return ['mouseout', 'mouseleave'].includes(this.type) ? this.relatedTarget : this.target;
  }
  get x() {
    return this.clientX;
  }
  get y() {
    return this.clientY;
  }
  // Explicit legacy placeholders: this runtime has no layout coordinate space.
  get pageX() {
    return 0;
  }
  get pageY() {
    return 0;
  }
  get offsetX() {
    return 0;
  }
  get offsetY() {
    return 0;
  }
  get layerX() {
    return 0;
  }
  get layerY() {
    return 0;
  }
  getModifierState(key) {
    return __getModifierState(this, key, arguments.length);
  }
  initMouseEvent(
    type,
    bubbles = false,
    cancelable = false,
    view = null,
    detail = 0,
    screenX = 0,
    screenY = 0,
    clientX = 0,
    clientY = 0,
    ctrlKey = false,
    altKey = false,
    shiftKey = false,
    metaKey = false,
    button = 0,
    relatedTarget = null,
  ) {
    if (!arguments.length) throw new TypeError('Event type is required');
    type = `${type}`;
    view = __eventView(view);
    detail = +detail >> 0;
    const data = {
      screenX: +screenX >> 0,
      screenY: +screenY >> 0,
      clientX: +clientX >> 0,
      clientY: +clientY >> 0,
      button: (+button << 16) >> 16,
      buttons: 0,
      relatedTarget: __eventRelatedTarget(relatedTarget),
      movementX: 0,
      movementY: 0,
    };
    if (this.__dispatching) return;
    __initializeUIEvent.call(this, type, bubbles, cancelable, view, detail);
    __mouseEventData.set(this, data);
    __uiEventData.get(this).which = data.button + 1;
    __initializeModifiers(this, { ctrlKey, altKey, shiftKey, metaKey });
  }
}

class InputEvent extends UIEvent {
  constructor(type, options = {}) {
    super(type, options);
    this.data = options.data ?? null;
    this.inputType = String(options.inputType ?? '');
    this.isComposing = Boolean(options.isComposing);
    this.dataTransfer = options.dataTransfer ?? null;
  }
  getTargetRanges() {
    return [];
  }
}
globalThis.InputEvent = InputEvent;

class WheelEvent extends MouseEvent {
  constructor(type, options = {}) {
    super(type, options);
    this.deltaX = Number(options.deltaX ?? 0);
    this.deltaY = Number(options.deltaY ?? 0);
    this.deltaZ = Number(options.deltaZ ?? 0);
    this.deltaMode = Number(options.deltaMode ?? WheelEvent.DOM_DELTA_PIXEL);
  }
}
WheelEvent.DOM_DELTA_PIXEL = 0;
WheelEvent.DOM_DELTA_LINE = 1;
WheelEvent.DOM_DELTA_PAGE = 2;

class FocusEvent extends UIEvent {
  constructor(type, options = {}) {
    super(type, options);
    this.relatedTarget = options.relatedTarget === undefined ? null : options.relatedTarget;
  }
}

class ProgressEvent extends Event {
  constructor(type, options = {}) {
    super(type, options);
    this.lengthComputable = Boolean(options.lengthComputable);
    this.loaded = Number(options.loaded ?? 0);
    this.total = Number(options.total ?? 0);
  }
}

class SubmitEvent extends Event {
  constructor(type, options = {}) {
    super(type, options);
    this.submitter = options.submitter === undefined ? null : options.submitter;
  }
}

const __keyboardEventData = new WeakMap();
class KeyboardEvent extends UIEvent {
  constructor(type, options = {}) {
    if (!arguments.length) throw new TypeError('Event type is required');
    if (options == null) options = {};
    super(type, options);
    const key = options.key,
      code = options.code;
    __keyboardEventData.set(this, {
      key: key === undefined ? '' : `${key}`,
      code: code === undefined ? '' : `${code}`,
      location: +options.location >>> 0,
      repeat: Boolean(options.repeat),
      isComposing: Boolean(options.isComposing),
      charCode: +options.charCode >>> 0,
      keyCode: +options.keyCode >>> 0,
    });
    __initializeModifiers(this, options);
  }
  get key() {
    return __keyboardEventData.get(this).key;
  }
  get code() {
    return __keyboardEventData.get(this).code;
  }
  get location() {
    return __keyboardEventData.get(this).location;
  }
  get repeat() {
    return __keyboardEventData.get(this).repeat;
  }
  get isComposing() {
    return __keyboardEventData.get(this).isComposing;
  }
  get charCode() {
    return __keyboardEventData.get(this).charCode;
  }
  get keyCode() {
    return __keyboardEventData.get(this).keyCode;
  }
  get ctrlKey() {
    return __eventModifiers.get(this).Control;
  }
  get shiftKey() {
    return __eventModifiers.get(this).Shift;
  }
  get altKey() {
    return __eventModifiers.get(this).Alt;
  }
  get metaKey() {
    return __eventModifiers.get(this).Meta;
  }
  getModifierState(key) {
    return __getModifierState(this, key, arguments.length);
  }
  initKeyboardEvent(
    type,
    bubbles = false,
    cancelable = false,
    view = null,
    keyIdentifier = '',
    location = 0,
    ctrlKey = false,
    altKey = false,
    shiftKey = false,
    metaKey = false,
  ) {
    if (!arguments.length) throw new TypeError('Event type is required');
    // Safari converts the legacy identifier but leaves key/code empty.
    type = `${type}`;
    view = __eventView(view);
    keyIdentifier = `${keyIdentifier}`;
    location = +location >>> 0;
    if (this.__dispatching) return;
    __initializeUIEvent.call(this, type, bubbles, cancelable, view, 0);
    __keyboardEventData.set(this, {
      key: '',
      code: '',
      location,
      repeat: false,
      isComposing: false,
      charCode: 0,
      keyCode: 0,
    });
    __uiEventData.get(this).which = 0;
    __initializeModifiers(this, { ctrlKey, altKey, shiftKey, metaKey });
  }
}
for (const [name, value] of Object.entries({
  DOM_KEY_LOCATION_STANDARD: 0,
  DOM_KEY_LOCATION_LEFT: 1,
  DOM_KEY_LOCATION_RIGHT: 2,
  DOM_KEY_LOCATION_NUMPAD: 3,
})) {
  Object.defineProperty(KeyboardEvent, name, { value, enumerable: true });
  Object.defineProperty(KeyboardEvent.prototype, name, { value, enumerable: true });
}

const __pointerEventData = new WeakMap();
function __pointerNumber(value, fallback = 0) {
  const number = value === undefined ? fallback : +value;
  if (!Number.isFinite(number)) throw new TypeError('Expected a finite number');
  return number;
}
class PointerEvent extends MouseEvent {
  constructor(type, options = {}) {
    if (!arguments.length) throw new TypeError('Event type is required');
    if (options == null) options = {};
    super(type, options);
    const pointerType = options.pointerType;
    const data = {
      pointerId: +options.pointerId >> 0,
      width: __pointerNumber(options.width, 1),
      height: __pointerNumber(options.height, 1),
      pressure: Math.fround(__pointerNumber(options.pressure)),
      tangentialPressure: Math.fround(__pointerNumber(options.tangentialPressure)),
      tiltX: +options.tiltX >> 0,
      tiltY: +options.tiltY >> 0,
      twist: +options.twist >>> 0,
      pointerType: pointerType === undefined ? '' : `${pointerType}`,
      isPrimary: Boolean(options.isPrimary),
    };
    const radians = Math.PI / 180,
      x = Math.tan(data.tiltX * radians),
      y = Math.tan(data.tiltY * radians);
    data.altitudeAngle = Math.atan2(1, Math.hypot(x, y));
    data.azimuthAngle = (Math.atan2(y, x) + 2 * Math.PI) % (2 * Math.PI);
    const altitude = options.altitudeAngle,
      azimuth = options.azimuthAngle;
    if (altitude !== undefined || azimuth !== undefined) {
      const a = __pointerNumber(altitude, Math.PI / 2),
        z = __pointerNumber(azimuth);
      // Safari exposes angles from the initial tilt while explicit angle
      // options determine the final tilt payload.
      data.tiltX = Math.round(Math.atan2(Math.cos(z), Math.tan(a)) / radians);
      data.tiltY = Math.round(Math.atan2(Math.sin(z), Math.tan(a)) / radians);
    }
    __pointerEventData.set(this, data);
  }
  get pointerId() {
    return __pointerEventData.get(this).pointerId;
  }
  get width() {
    return __pointerEventData.get(this).width;
  }
  get height() {
    return __pointerEventData.get(this).height;
  }
  get pressure() {
    return __pointerEventData.get(this).pressure;
  }
  get tangentialPressure() {
    return __pointerEventData.get(this).tangentialPressure;
  }
  get tiltX() {
    return __pointerEventData.get(this).tiltX;
  }
  get tiltY() {
    return __pointerEventData.get(this).tiltY;
  }
  get twist() {
    return __pointerEventData.get(this).twist;
  }
  get pointerType() {
    return __pointerEventData.get(this).pointerType;
  }
  get isPrimary() {
    return __pointerEventData.get(this).isPrimary;
  }
  get altitudeAngle() {
    return __pointerEventData.get(this).altitudeAngle;
  }
  get azimuthAngle() {
    return __pointerEventData.get(this).azimuthAngle;
  }
  // Legacy placeholders: no hardware sample history or prediction engine.
  getCoalescedEvents() {}
  getPredictedEvents() {}
}

class Touch {
  constructor(options = {}) {
    if (options.target === undefined) throw new TypeError('Touch target is required');
    this.identifier = Number(options.identifier ?? 0);
    this.target = options.target;
    this.clientX = Number(options.clientX ?? 0);
    this.clientY = Number(options.clientY ?? 0);
    this.screenX = Number(options.screenX ?? this.clientX);
    this.screenY = Number(options.screenY ?? this.clientY);
    this.pageX = Number(options.pageX ?? this.clientX);
    this.pageY = Number(options.pageY ?? this.clientY);
    this.radiusX = Number(options.radiusX ?? 0);
    this.radiusY = Number(options.radiusY ?? 0);
    this.rotationAngle = Number(options.rotationAngle ?? 0);
    this.force = Number(options.force ?? 0);
  }
}

class TouchEvent extends UIEvent {
  constructor(type, options = {}) {
    super(type, options);
    this.touches = Object.freeze(Array.from(options.touches ?? []));
    this.targetTouches = Object.freeze(Array.from(options.targetTouches ?? []));
    this.changedTouches = Object.freeze(Array.from(options.changedTouches ?? []));
    __initializeModifiers(this, options);
  }
  get ctrlKey() {
    return __eventModifiers.get(this).Control;
  }
  get shiftKey() {
    return __eventModifiers.get(this).Shift;
  }
  get altKey() {
    return __eventModifiers.get(this).Alt;
  }
  get metaKey() {
    return __eventModifiers.get(this).Meta;
  }
}

class MessageEvent extends Event {
  constructor(type, options = {}) {
    super(type, options);
    this.data = options.data === undefined ? null : options.data;
    this.origin = String(options.origin ?? '');
    this.lastEventId = String(options.lastEventId ?? '');
    this.source = options.source === undefined ? null : options.source;
    this.ports = Object.freeze(Array.from(options.ports ?? []));
  }
}

class StorageEvent extends Event {
  constructor(type, options = {}) {
    if (arguments.length === 0) throw new TypeError('StorageEvent type is required');
    super(type, options);
    this.key = options.key == null ? null : String(options.key);
    this.oldValue = options.oldValue == null ? null : String(options.oldValue);
    this.newValue = options.newValue == null ? null : String(options.newValue);
    this.url = options.url === undefined ? '' : String(options.url);
    this.storageArea = options.storageArea === undefined ? null : options.storageArea;
  }
  initStorageEvent(
    type,
    bubbles = false,
    cancelable = false,
    key = null,
    oldValue = null,
    newValue = null,
    url = '',
    storageArea = null,
  ) {
    if (arguments.length === 0) throw new TypeError('StorageEvent type is required');
    if (this.__dispatching) return;
    this.initEvent(type, bubbles, cancelable);
    this.key = key == null ? null : String(key);
    this.oldValue = oldValue == null ? null : String(oldValue);
    this.newValue = newValue == null ? null : String(newValue);
    this.url = String(url);
    this.storageArea = storageArea;
  }
}

class PopStateEvent extends Event {
  constructor(type, options = {}) {
    if (arguments.length === 0) throw new TypeError('PopStateEvent type is required');
    super(type, options);
    this.state = options.state === undefined ? null : options.state;
    this.hasUAVisualTransition = Boolean(options.hasUAVisualTransition);
  }
}

class HashChangeEvent extends Event {
  constructor(type, options = {}) {
    if (arguments.length === 0) throw new TypeError('HashChangeEvent type is required');
    super(type, options);
    this.oldURL = String(options.oldURL ?? '');
    this.newURL = String(options.newURL ?? '');
  }
}

function __listenerCapture(options) {
  return options !== null && (typeof options === 'object' || typeof options === 'function')
    ? Boolean(options.capture)
    : Boolean(options);
}
function __listenerArguments(type, callback, count) {
  if (count < 2) throw new TypeError('Not enough arguments');
  type = `${type}`;
  if (callback != null && typeof callback !== 'object' && typeof callback !== 'function')
    throw new TypeError('Expected an event listener');
  return type;
}

const __eventHandlerRecords = new WeakMap();
const __windowEventTargets = new WeakSet([globalThis]);
const __bodyWindowEventTypes = [
  'afterprint',
  'beforeprint',
  'blur',
  'focus',
  'hashchange',
  'languagechange',
  'load',
  'message',
  'messageerror',
  'offline',
  'online',
  'pagehide',
  'pageshow',
  'popstate',
  'rejectionhandled',
  'resize',
  'scroll',
  'storage',
  'unhandledrejection',
  'unload',
  'error',
  'beforeunload',
];
function __defaultPassiveListener(target, type) {
  if (!['touchstart', 'touchmove', 'wheel', 'mousewheel'].includes(type)) return false;
  if (target === globalThis || target instanceof Document) return true;
  const owner = target instanceof Node ? target.ownerDocument : null;
  return !!owner && (target === owner.body || target === owner.documentElement);
}
function __defineEventHandlers(prototype, types, windowReflecting = false) {
  // Forwarding belongs to the body/frameset accessor. Explicitly calling an
  // HTMLElement accessor on the same receiver still accesses its local handler.
  const handlerTarget = (receiver) =>
    windowReflecting ? __documentWindow(__callHost('ownerDocument', receiver)) : receiver;
  for (const type of types) {
    const property = `on${type}`;
    const eventType =
      {
        webkitanimationend: 'webkitAnimationEnd',
        webkitanimationiteration: 'webkitAnimationIteration',
        webkitanimationstart: 'webkitAnimationStart',
        webkittransitionend: 'webkitTransitionEnd',
      }[type] ?? type;
    const descriptor = Object.getOwnPropertyDescriptor(
      {
        get [property]() {
          return __eventHandlerRecords.get(handlerTarget(this))?.get(eventType)?.value ?? null;
        },
        set [property](value) {
          const target = handlerTarget(this);
          if (target === null) return;
          if ((typeof value !== 'object' && typeof value !== 'function') || value === null)
            value = null;
          let handlers = __eventHandlerRecords.get(target);
          if (!handlers) __eventHandlerRecords.set(target, (handlers = new Map()));
          let record = handlers.get(eventType);
          if (!record) handlers.set(eventType, (record = { value: null, listener: null }));
          record.value = value;
          let listeners = __eventListeners.get(target);
          if (!listeners) __eventListeners.set(target, (listeners = []));
          if (value === null && record.listener) {
            const index = listeners.indexOf(record.listener);
            if (index !== -1) listeners.splice(index, 1);
            record.listener = null;
          } else if (value !== null && !record.listener) {
            record.listener = {
              type: eventType,
              capture: false,
              once: false,
              passive: __defaultPassiveListener(target, eventType),
              callback(event) {
                if (typeof record.value !== 'function') return;
                const error =
                  eventType === 'error' &&
                  __windowEventTargets.has(this) &&
                  __errorEventData.get(event);
                const result = error
                  ? record.value.call(
                      undefined,
                      error.message,
                      error.filename,
                      error.lineno,
                      error.colno,
                      error.error,
                    )
                  : record.value.call(this, event);
                if (eventType === 'beforeunload') {
                  if (!__beforeUnloadEventData.has(event)) return;
                  const value = result == null ? null : `${result}`;
                  if (value !== null) {
                    __preventEventDefault.call(event);
                    if (__beforeUnloadEventData.get(event) === '')
                      __beforeUnloadEventData.set(event, value);
                  }
                } else if (error ? result === true : result === false)
                  __preventEventDefault.call(event);
              },
            };
            listeners.push(record.listener);
          }
        },
      },
      property,
    );
    if (
      [
        'beforecopy',
        'beforecut',
        'beforepaste',
        'webkitfullscreenchange',
        'webkitfullscreenerror',
        'webkitmouseforcechanged',
        'webkitmouseforcedown',
        'webkitmouseforceup',
        'webkitmouseforcewillbegin',
      ].includes(type)
    )
      descriptor.enumerable = false;
    if (['mouseenter', 'mouseleave', 'readystatechange'].includes(type)) {
      __lenientThisFunctions.add(descriptor.get);
      __lenientThisFunctions.add(descriptor.set);
    }
    Object.defineProperty(prototype, property, descriptor);
  }
}

function __invokeListeners(target, event, capture, phase) {
  const listeners = (__eventListeners.get(target) || []).slice();
  const record = __eventRecord(event);
  record.target = __retarget(event.__originalTarget, target);
  record.currentTarget = target;
  record.eventPhase = phase;
  event.__immediateStopped = false;
  for (const listener of listeners) {
    if (listener.type !== event.type || listener.capture !== capture) continue;
    if (!(__eventListeners.get(target) || []).includes(listener)) continue;
    if (listener.once) target.removeEventListener(event.type, listener.callback, listener.capture);
    event.__inPassiveListener = listener.passive;
    try {
      if (typeof listener.callback === 'function') listener.callback.call(target, event);
      else listener.callback.handleEvent(event);
    } catch (error) {
      setTimeout(() => {
        throw error;
      }, 0);
    } finally {
      event.__inPassiveListener = false;
    }
    if (event.__immediateStopped) break;
  }
  // Other EventTarget families (ports, abort signals and optional plugins)
  // still own their on* dispatch hooks; registered DOM handlers run above.
  if (
    !capture &&
    !event.__immediateStopped &&
    !(target instanceof Node) &&
    !__eventHandlerRecords.get(target)?.has(event.type)
  ) {
    const handler = target[`on${event.type}`];
    if (typeof handler === 'function' && handler.call(target, event) === false)
      event.preventDefault();
  }
}

const __eventTargets = new WeakSet();
class EventTarget {
  constructor() {
    __eventTargets.add(this);
  }
  addEventListener(type, callback, options = false) {
    type = __listenerArguments(type, callback, arguments.length);
    const capture = __listenerCapture(options);
    const dictionary =
      options !== null && (typeof options === 'object' || typeof options === 'function')
        ? options
        : null;
    const once = Boolean(dictionary?.once),
      passive = dictionary?.passive,
      signal = dictionary?.signal;
    if (signal !== undefined && !(signal instanceof AbortSignal))
      throw new TypeError('Expected an AbortSignal');
    if (callback == null) return;
    if (signal && signal.aborted) return;
    const listeners = __eventListeners.get(this) || [];
    if (
      !listeners.some(
        (item) => item.type === type && item.callback === callback && item.capture === capture,
      )
    ) {
      const listener = {
        type,
        callback,
        capture,
        once,
        passive: passive !== undefined ? Boolean(passive) : __defaultPassiveListener(this, type),
      };
      listeners.push(listener);
      __eventListeners.set(this, listeners);
      if (signal) {
        signal.addEventListener(
          'abort',
          () => {
            this.removeEventListener(type, callback, capture);
          },
          { once: true },
        );
      }
    }
  }
  removeEventListener(type, callback, options = false) {
    type = __listenerArguments(type, callback, arguments.length);
    const capture = __listenerCapture(options);
    const listeners = __eventListeners.get(this);
    if (!listeners) return;
    const index = listeners.findIndex(
      (item) => item.type === type && item.callback === callback && item.capture === capture,
    );
    if (index !== -1) listeners.splice(index, 1);
  }
  dispatchEvent(event) {
    const record = __eventRecord(event);
    if (__uninitializedEvents.has(event))
      throw new DOMException('Event has not been initialized', 'InvalidStateError');
    if (event.__dispatching)
      throw new DOMException('Event is already being dispatched', 'InvalidStateError');
    if (!__pendingTrustedEvents.delete(event)) __trustedEvents.delete(event);
    event.__dispatching = true;
    event.__originalTarget = this;
    record.target = this;
    const path = [];
    let ancestor = this instanceof Node ? __eventParent(this, event) : null;
    while (ancestor) {
      path.push(ancestor);
      ancestor = __eventParent(ancestor, event);
    }
    event.__path = [this, ...path];
    const completeActivation = __prepareInputActivation(event, event.__path);
    try {
      for (let i = path.length - 1; i >= 0 && !event.__stopped; i--) {
        __invokeListeners(
          path[i],
          event,
          true,
          __retarget(this, path[i]) === path[i] ? Event.AT_TARGET : Event.CAPTURING_PHASE,
        );
      }
      if (!event.__stopped) {
        __invokeListeners(this, event, true, Event.AT_TARGET);
        if (!event.__immediateStopped) __invokeListeners(this, event, false, Event.AT_TARGET);
      }
      if (!event.__stopped) {
        for (const target of path) {
          const atTarget = __retarget(this, target) === target;
          if (!event.bubbles && !atTarget) continue;
          __invokeListeners(
            target,
            event,
            false,
            atTarget ? Event.AT_TARGET : Event.BUBBLING_PHASE,
          );
          if (event.__stopped) break;
        }
      }
      return !event.defaultPrevented;
    } finally {
      record.currentTarget = null;
      record.eventPhase = Event.NONE;
      event.__dispatching = false;
      event.__stopped = false;
      event.__immediateStopped = false;
      event.__path = [];
      if (completeActivation) completeActivation(!event.defaultPrevented);
    }
  }
}
const __dispatchEvent = EventTarget.prototype.dispatchEvent;

function __prepareMessage(message, transferOrOptions) {
  const transfer = Array.isArray(transferOrOptions)
    ? transferOrOptions
    : (transferOrOptions?.transfer ?? []);
  return structuredClone(message, { transfer });
}

class MessagePort extends EventTarget {
  constructor() {
    super();
    this.__entangled = null;
    this.__queue = [];
    this.__started = false;
    this.__closed = false;
    this.__scheduled = false;
    this.__onmessage = null;
    this.onmessageerror = null;
  }
  get onmessage() {
    return this.__onmessage;
  }
  set onmessage(callback) {
    this.__onmessage = callback == null ? null : callback;
    if (this.__onmessage !== null) this.start();
  }
  postMessage(message, transferOrOptions = []) {
    if (arguments.length === 0) throw new TypeError('message is required');
    const cloned = __prepareMessage(message, transferOrOptions);
    const target = this.__entangled;
    if (!target || this.__closed || target.__closed) return;
    target.__queue.push(cloned);
    target.__schedule();
  }
  start() {
    if (this.__closed) return;
    this.__started = true;
    this.__schedule();
  }
  close() {
    this.__closed = true;
    this.__queue.length = 0;
    this.__entangled = null;
  }
  __schedule() {
    if (!this.__started || this.__scheduled || this.__closed || this.__queue.length === 0) return;
    this.__scheduled = true;
    setTimeout(() => {
      this.__scheduled = false;
      if (!this.__started || this.__closed) return;
      const hasMessage = this.__queue.length !== 0;
      const data = this.__queue.shift();
      if (hasMessage) {
        const event = new MessageEvent('message', { data });
        __markTrustedEvent(event);
        this.dispatchEvent(event);
      }
      this.__schedule();
    }, 0);
  }
}

class MessageChannel {
  constructor() {
    const port1 = new MessagePort();
    const port2 = new MessagePort();
    port1.__entangled = port2;
    port2.__entangled = port1;
    Object.defineProperties(this, {
      port1: { value: port1, enumerable: true },
      port2: { value: port2, enumerable: true },
    });
  }
}

function postMessage(message, targetOrigin = '/', transfer = []) {
  const cloned = __prepareMessage(message, transfer);
  targetOrigin = String(targetOrigin);
  if (targetOrigin !== '*' && targetOrigin !== '/' && targetOrigin !== location.origin) return;
  setTimeout(() => {
    const event = new MessageEvent('message', {
      data: cloned,
      origin: location.origin,
      source: window,
    });
    __markTrustedEvent(event);
    window.dispatchEvent(event);
  }, 0);
}

class AbortSignal extends EventTarget {
  constructor() {
    super();
    this.aborted = false;
    this.reason = undefined;
    this.onabort = null;
    this.__dependents = [];
  }
  throwIfAborted() {
    if (this.aborted) throw this.reason;
  }
  static abort(reason = undefined) {
    const signal = new AbortSignal();
    signal.__abort(reason);
    return signal;
  }
  static timeout(milliseconds) {
    milliseconds = Number(milliseconds);
    if (!Number.isFinite(milliseconds) || milliseconds < 0 || milliseconds > 0xffffffff) {
      throw new TypeError('timeout must be an unsigned long long');
    }
    const signal = new AbortSignal();
    setTimeout(
      () => signal.__abort(new DOMException('The operation timed out', 'TimeoutError')),
      milliseconds,
    );
    return signal;
  }
  static any(signals) {
    const inputs = Array.from(signals);
    for (const signal of inputs) {
      if (!(signal instanceof AbortSignal)) throw new TypeError('value is not an AbortSignal');
    }
    const result = new AbortSignal();
    const alreadyAborted = inputs.find((signal) => signal.aborted);
    if (alreadyAborted) {
      result.__abort(alreadyAborted.reason);
      return result;
    }
    for (const signal of new Set(inputs)) signal.__dependents.push(result);
    return result;
  }
  __abort(reason = undefined) {
    if (this.aborted) return;
    const abortReason =
      reason === undefined ? new DOMException('The operation was aborted', 'AbortError') : reason;
    const queue = [this];
    const aborted = [];
    while (queue.length) {
      const signal = queue.shift();
      if (signal.aborted) continue;
      signal.aborted = true;
      signal.reason = abortReason;
      aborted.push(signal);
      queue.push(...signal.__dependents);
    }
    for (const signal of aborted) {
      const event = new Event('abort');
      __markTrustedEvent(event);
      signal.dispatchEvent(event);
    }
  }
}

class AbortController {
  constructor() {
    this.signal = new AbortSignal();
  }
  abort(reason = undefined) {
    this.signal.__abort(reason);
  }
}
