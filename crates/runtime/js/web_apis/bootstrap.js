'use strict';

const __nativeHost = globalThis.__brimp;
let __domVersion = 0;
const __domMutations = new Set([
  'setTextContent',
  'appendChild',
  'removeChild',
  'insertBefore',
  'replaceChild',
  'setAttribute',
  'removeAttribute',
  'setAttributeNS',
  'removeAttributeNS',
  'setInnerHTML',
  'styleSetCssText',
  'styleSetProperty',
  'styleRemoveProperty',
]);
function __callHost(operation, ...args) {
  const before = __domMutations.has(operation) ? __mutationBefore(operation, args) : null;
  const iteratorUpdates = __domMutations.has(operation)
    ? __iteratorMutationBefore(operation, args)
    : [];
  const updateRanges = __domMutations.has(operation)
    ? __rangeMutationBefore(operation, args)
    : null;
  const updateSheets = __domMutations.has(operation)
    ? __styleMutationBefore(operation, args)
    : null;
  const updateInput = __domMutations.has(operation) ? __inputMutationBefore(operation, args) : null;
  const updateInputValidation = __domMutations.has(operation)
    ? __inputValidationMutationBefore(operation, args)
    : null;
  const updateDisabled = __domMutations.has(operation)
    ? __disabledMutationBefore(operation, args)
    : null;
  const updateSelection = __domMutations.has(operation)
    ? __selectionMutationBefore(operation, args)
    : null;
  const updateLabels = __domMutations.has(operation)
    ? __labelMutationBefore(operation, args)
    : null;
  const updateChecked = __domMutations.has(operation)
    ? __checkedMutationBefore(operation, args)
    : null;
  const result = __nativeHost(operation, ...args);
  if (
    (operation === 'createElement' || operation === 'createElementNS') &&
    result instanceof HTMLScriptElement
  ) {
    __forceAsyncScripts.add(result);
  }
  if (operation === 'cloneNode') {
    __cloneFormControlState(args[0], result, args[1]);
  } else if (operation === 'importNode') __cloneFormControlState(args[1], result, args[2]);
  if (__domMutations.has(operation)) __domVersion++;
  if (updateDisabled) updateDisabled();
  if (updateLabels) updateLabels();
  if (updateChecked) updateChecked();
  if (updateRanges) updateRanges();
  for (const update of iteratorUpdates) update();
  if (before) __mutationAfter(before);
  if (updateInput) updateInput();
  if (updateInputValidation) updateInputValidation();
  if (updateSelection) updateSelection();
  if (__domMutations.has(operation)) __updateSlots();
  if (updateSheets) updateSheets();
  return result;
}
delete globalThis.__brimp;

// Capture the realm clock before any Web API captures Date.now.
if (__nativeHost('clockLogical')) {
  const OriginalDate = Date;
  const now = () => __nativeHost('clockOrigin') + __nativeHost('clockNow');
  function RealmDate(...args) {
    if (!new.target) return new OriginalDate(now()).toString();
    return Reflect.construct(OriginalDate, args.length ? args : [now()], new.target);
  }
  Object.setPrototypeOf(RealmDate, OriginalDate);
  RealmDate.prototype = OriginalDate.prototype;
  Object.defineProperty(RealmDate, 'name', { value: 'Date' });
  Object.defineProperty(RealmDate, 'length', { value: 7 });
  Object.defineProperty(RealmDate, 'now', { value: now, configurable: true, writable: true });
  Object.defineProperty(RealmDate.prototype, 'constructor', {
    value: RealmDate,
    configurable: true,
    writable: true,
  });
  globalThis.Date = RealmDate;
}

const __initialGlobalNames = new Set(Object.getOwnPropertyNames(globalThis));
const __originalDate = globalThis.Date;
const __nativeFunctions = new WeakSet();
const __nativeMembers = new WeakMap();
const __finalizedWebIdlPrototypes = new WeakSet();
const __originalFunctionToString = Function.prototype.toString;
// "webkit" is WebKit's multi-line native source; "chrome" is Blink's
// single-line form with get/set prefixes. Brimp's engine is JSC, but a profile
// can claim a Chromium identity, so the source style follows the identity.
let __nativeStyle = 'webkit';

function __nativeSource(member) {
  const name = member?.name ?? '';
  const kind = member?.kind ?? 'value';
  if (__nativeStyle === 'webkit') return `function ${name}() {\n    [native code]\n}`;
  if (kind === 'get' || kind === 'set') return `function ${kind} ${name}() { [native code] }`;
  return `function ${name}() { [native code] }`;
}

function __normalizeNativeName(fn, key, kind) {
  if (typeof fn !== 'function' || typeof key !== 'string') return;
  let name;
  if (kind === 'value') {
    if (fn.name !== '' && fn.name !== 'value' && fn.name !== 'anonymous') return;
    name = key;
  } else {
    name = `${kind} ${key}`;
  }
  try {
    if (fn.name !== name) Object.defineProperty(fn, 'name', { value: name, configurable: true });
    if (kind === 'get' && fn.length !== 0)
      Object.defineProperty(fn, 'length', { value: 0, configurable: true });
    if (kind === 'set' && fn.length !== 1)
      Object.defineProperty(fn, 'length', { value: 1, configurable: true });
  } catch {
    // Some engine functions reject redefinition; leave their observed shape.
  }
}

function __markWebBuiltin(fn, key = undefined, kind = 'value') {
  if (typeof fn !== 'function') return fn;
  __nativeFunctions.add(fn);
  __normalizeNativeName(fn, key, kind);
  const usableName =
    kind === 'value' && fn.name !== '' && fn.name !== 'value' && fn.name !== 'anonymous'
      ? fn.name
      : typeof key === 'string'
        ? key
        : fn.name || '';
  const existing = __nativeMembers.get(fn);
  if (existing === undefined) {
    __nativeMembers.set(fn, { name: usableName, kind });
  } else if ((existing.name === '' || existing.name === 'value') && usableName !== '') {
    existing.name = usableName;
  }
  return fn;
}

function __markWebBuiltinInterface(constructor) {
  if (typeof constructor !== 'function') return;
  __markWebBuiltin(constructor);
  const prototype = constructor.prototype;
  if (prototype === null || typeof prototype !== 'object') return;
  for (const key of Reflect.ownKeys(prototype)) {
    if (key === 'constructor') continue;
    const descriptor = Object.getOwnPropertyDescriptor(prototype, key);
    if (descriptor === undefined) continue;
    if (typeof descriptor.value === 'function') __markWebBuiltin(descriptor.value, key, 'value');
    if (typeof descriptor.get === 'function') __markWebBuiltin(descriptor.get, key, 'get');
    if (typeof descriptor.set === 'function') __markWebBuiltin(descriptor.set, key, 'set');
  }
}

function __functionToString() {
  const member = __nativeMembers.get(this);
  if (member !== undefined) return __nativeSource(member);
  if (__nativeFunctions.has(this)) return __nativeSource({ name: this.name || '', kind: 'value' });
  return __originalFunctionToString.call(this);
}

const __lenientThisFunctions = new WeakSet();
function __markConsoleMembers() {
  const target = globalThis.console;
  if (target === null || typeof target !== 'object') return;
  for (const key of Reflect.ownKeys(target)) {
    const descriptor = Object.getOwnPropertyDescriptor(target, key);
    if (descriptor !== undefined && typeof descriptor.value === 'function')
      __markWebBuiltin(descriptor.value);
  }
}

// Window IDL attributes belong to the global object itself. Brimp installs some
// as data properties and some on Window.prototype; browsers expose all of them
// as own accessors. This runs after every core API and plugin has installed its
// surface, and before page scripts execute.
const __windowNoSetter = new Set([
  'document',
  'window',
  'top',
  'navigator',
  'history',
  'crypto',
  'closed',
  'frameElement',
  'isSecureContext',
  'crossOriginIsolated',
  'indexedDB',
  'cookieStore',
  'caches',
  'trustedTypes',
  'speechSynthesis',
  'localStorage',
  'sessionStorage',
]);
const __windowConfigurableFalse = new Set(['document', 'window', 'top', 'location']);
const __windowDataNames = [
  'window',
  'self',
  'parent',
  'top',
  'opener',
  'document',
  'navigator',
  'history',
  'performance',
  'crypto',
  'crossOriginIsolated',
  'indexedDB',
  'cookieStore',
  'caches',
  'trustedTypes',
  'speechSynthesis',
  'localStorage',
  'sessionStorage',
  'location',
];
let __windowDocument = globalThis.document;
let __windowDocumentAccessorInstalled = false;
const __windowOverrides = new Map();

globalThis.__brimpSetDocument = (document) => {
  __windowDocument = document;
  if (!__windowDocumentAccessorInstalled) {
    Object.defineProperty(globalThis, 'document', {
      value: document,
      writable: true,
      enumerable: true,
      configurable: true,
    });
  }
};

function __windowGetter(name, originalGet) {
  const getter = function () {
    return __windowOverrides.has(name) ? __windowOverrides.get(name) : originalGet.call(globalThis);
  };
  return __markWebBuiltin(getter, name, 'get');
}

function __windowSetter(name) {
  const setter = function (value) {
    __windowOverrides.set(name, value);
  };
  return __markWebBuiltin(setter, name, 'set');
}

function __installWindowAccessors(diagnosticsRecursive = false) {
  const prototype = Object.getPrototypeOf(globalThis);
  if (prototype === null || prototype.constructor !== globalThis.Window) return;
  if (__windowDocument === undefined) __windowDocument = globalThis.document;

  // Move every Window.prototype member (IDL attributes and event handlers) onto
  // the global object, where browsers expose them.
  for (const key of Reflect.ownKeys(prototype)) {
    if (key === 'constructor') continue;
    const descriptor = Object.getOwnPropertyDescriptor(prototype, key);
    if (descriptor === undefined) continue;
    delete prototype[key];
    if (Object.hasOwn(globalThis, key)) continue;
    if (typeof key !== 'string') {
      Object.defineProperty(globalThis, key, descriptor);
      continue;
    }
    const originalGet =
      typeof descriptor.get === 'function' ? descriptor.get : () => descriptor.value;
    const setter = descriptor.set ?? (__windowNoSetter.has(key) ? undefined : __windowSetter(key));
    Object.defineProperty(globalThis, key, {
      get: __windowGetter(key, originalGet),
      set: setter,
      enumerable: true,
      configurable: diagnosticsRecursive || !__windowConfigurableFalse.has(key),
    });
  }

  for (const name of __windowDataNames) {
    const descriptor = Object.getOwnPropertyDescriptor(globalThis, name);
    if (descriptor === undefined) continue;
    let originalGet;
    if ('value' in descriptor) {
      const value = descriptor.value;
      originalGet = name === 'document' ? () => __windowDocument : () => value;
    } else if (typeof descriptor.get === 'function') {
      originalGet = () => descriptor.get.call(globalThis);
    } else {
      continue;
    }
    const setter =
      descriptor.set ?? (__windowNoSetter.has(name) ? undefined : __windowSetter(name));
    Object.defineProperty(globalThis, name, {
      get: __windowGetter(name, originalGet),
      set: setter,
      enumerable: true,
      configurable: diagnosticsRecursive || !__windowConfigurableFalse.has(name),
    });
  }
  __windowDocumentAccessorInstalled = true;
}

function __detectNativeStyle() {
  const agent = globalThis.__brimpProfile?.identity?.user_agent;
  __nativeStyle =
    typeof agent === 'string' && /Chrome\/|Chromium\/|Edg\//.test(agent) ? 'chrome' : 'webkit';
}

function __finalizeWebIdlInterfaces() {
  const diagnosticsRecursive = globalThis.__brimpDiagnosticsRecursive === true;
  delete globalThis.__brimpDiagnosticsRecursive;
  __detectNativeStyle();
  __installWindowAccessors(diagnosticsRecursive);
  __markConsoleMembers();
  for (const name of Object.getOwnPropertyNames(globalThis)) {
    if (__initialGlobalNames.has(name)) continue;
    const constructor = Object.getOwnPropertyDescriptor(globalThis, name)?.value;
    if (typeof constructor !== 'function' || constructor.prototype === undefined) continue;
    const prototype = constructor.prototype;
    if (
      prototype === null ||
      typeof prototype !== 'object' ||
      __finalizedWebIdlPrototypes.has(prototype)
    )
      continue;
    const nativeElement =
      prototype === Element.prototype ||
      Object.prototype.isPrototypeOf.call(Element.prototype, prototype);
    __finalizedWebIdlPrototypes.add(prototype);
    if (!Object.hasOwn(prototype, Symbol.toStringTag)) {
      Object.defineProperty(prototype, Symbol.toStringTag, {
        value: name,
        writable: false,
        enumerable: false,
        configurable: true,
      });
    }
    for (const key of Reflect.ownKeys(prototype)) {
      if (key === 'constructor') continue;
      const descriptor = Object.getOwnPropertyDescriptor(prototype, key);
      if (descriptor === undefined) continue;
      if (!descriptor.configurable) continue;
      for (const kind of ['value', 'get', 'set']) {
        // Safari exposes the generic Array iteration functions on DOMTokenList.
        if (
          (constructor === DOMTokenList || constructor === NodeList) &&
          kind === 'value' &&
          ['entries', 'keys', 'values', 'forEach', Symbol.iterator].includes(key)
        )
          continue;
        if (constructor === HTMLCollection && kind === 'value' && key === Symbol.iterator) continue;
        const fn = descriptor[kind];
        if (typeof fn !== 'function') continue;
        descriptor[kind] = new Proxy(fn, {
          apply(target, receiver, argumentsList) {
            // JSC resolves unqualified inherited global members with
            // undefined as receiver. Window globals use this realm.
            if (constructor === Window && receiver === undefined) receiver = globalThis;
            const objectReceiver =
              (typeof receiver === 'object' && receiver !== null) || typeof receiver === 'function';
            const compatible =
              constructor === Window
                ? __windowEventTargets.has(receiver)
                : constructor === Event
                  ? __eventData.has(receiver)
                  : constructor === CustomEvent
                    ? __customEventData.has(receiver)
                    : constructor === UIEvent
                      ? __uiEventData.has(receiver)
                      : constructor === TextEvent
                        ? __textEventData.has(receiver)
                        : constructor === MouseEvent
                          ? __mouseEventData.has(receiver)
                          : constructor === KeyboardEvent
                            ? __keyboardEventData.has(receiver)
                            : constructor === PointerEvent
                              ? __pointerEventData.has(receiver)
                              : constructor === EventTarget
                                ? objectReceiver &&
                                  (__eventTargets.has(receiver) ||
                                    __windowEventTargets.has(receiver) ||
                                    __nativeHost('isNativeNode', receiver))
                                : constructor === Document
                                  ? objectReceiver &&
                                    __nativeHost('isNativeNode', receiver) &&
                                    __nativeHost('nodeType', receiver) === 9
                                  : constructor === Element
                                    ? objectReceiver &&
                                      __nativeHost('isNativeNode', receiver) &&
                                      __nativeHost('nodeType', receiver) === 1
                                    : constructor === DocumentFragment
                                      ? objectReceiver &&
                                        __nativeHost('isNativeNode', receiver) &&
                                        __nativeHost('nodeType', receiver) === 11
                                      : constructor === CharacterData
                                        ? objectReceiver &&
                                          __nativeHost('isNativeNode', receiver) &&
                                          [3, 4, 7, 8].includes(__nativeHost('nodeType', receiver))
                                        : constructor === Text
                                          ? objectReceiver &&
                                            __nativeHost('isNativeNode', receiver) &&
                                            [3, 4].includes(__nativeHost('nodeType', receiver))
                                          : constructor === DOMTokenList
                                            ? __tokenListData.has(receiver)
                                            : constructor === HTMLCollection
                                              ? __htmlCollectionData.has(receiver)
                                              : constructor === NodeList
                                                ? __nodeListData.has(receiver)
                                                : objectReceiver &&
                                                  Object.prototype.isPrototypeOf.call(
                                                    prototype,
                                                    receiver,
                                                  ) &&
                                                  (!nativeElement ||
                                                    __nativeHost('isNativeNode', receiver));
            if (!compatible) {
              if (__lenientThisFunctions.has(fn)) return undefined;
              throw new TypeError('Illegal invocation');
            }
            return Reflect.apply(target, receiver, argumentsList);
          },
        });
        __markWebBuiltin(descriptor[kind], key, kind);
      }
      Object.defineProperty(prototype, key, descriptor);
    }
  }
  // Profile shims replace initial-global Intl/Date methods; mark them here so
  // their Function.prototype.toString is browser-shaped too.
  for (const name of [
    'DateTimeFormat',
    'NumberFormat',
    'Collator',
    'PluralRules',
    'RelativeTimeFormat',
    'ListFormat',
    'DisplayNames',
    'Segmenter',
  ]) {
    if (typeof Intl[name] === 'function') __markWebBuiltin(Intl[name]);
  }
  // A logical-clock realm replaces Date with a JS function; it must be marked
  // for every identity. Engine intrinsics stay untouched for WebKit so JSC's
  // own multi-line native source is preserved.
  if (globalThis.Date !== __originalDate) __markWebBuiltinInterface(globalThis.Date);
  for (const name of [
    'toLocaleString',
    'toLocaleDateString',
    'toLocaleTimeString',
    'getTimezoneOffset',
  ]) {
    if (typeof Date.prototype[name] === 'function') {
      __markWebBuiltin(Date.prototype[name], name, 'value');
    }
  }
  // Only run during trusted initialization. Inspect descriptors, never invoke
  // accessors, and preserve the descriptors/identity chosen by API installers.
  // A private graph walk includes namespace objects and static methods as well
  // as prototypes, so core and plugin APIs share the same source boundary.
  const visited = new WeakSet();
  function visit(object) {
    if (
      object === null ||
      (typeof object !== 'object' && typeof object !== 'function') ||
      visited.has(object)
    )
      return;
    visited.add(object);
    if (typeof object === 'function') __markWebBuiltin(object);
    for (const key of Reflect.ownKeys(object)) {
      const descriptor = Object.getOwnPropertyDescriptor(object, key);
      if (!descriptor) continue;
      if (typeof descriptor.get === 'function') __markWebBuiltin(descriptor.get, key, 'get');
      if (typeof descriptor.set === 'function') __markWebBuiltin(descriptor.set, key, 'set');
      if ('value' in descriptor) visit(descriptor.value);
    }
    visit(Object.getPrototypeOf(object));
  }
  // Some binding layers replace a core global after the initial marking pass
  // (streaming fetch, for example). Re-assert their native names and sources.
  for (const name of [
    'atob',
    'btoa',
    'fetch',
    'postMessage',
    'getSelection',
    'setTimeout',
    'clearTimeout',
    'setInterval',
    'clearInterval',
    'queueMicrotask',
    'requestAnimationFrame',
    'cancelAnimationFrame',
    'addEventListener',
    'removeEventListener',
    'dispatchEvent',
  ]) {
    const value = globalThis[name];
    if (typeof value !== 'function') continue;
    __nativeFunctions.add(value);
    try {
      Object.defineProperty(value, 'name', { value: name, configurable: true });
    } catch {
      // Bound host functions can reject the rename; the source still uses name.
    }
    __nativeMembers.set(value, { name, kind: 'value' });
  }
  for (const key of [
    '__brimpConfigureNavigator',
    '__brimpCanvasBackends',
    '__brimpObjectUrls',
    '__brimpProfile',
    '__brimpInstallWindowEnvironment',
    '__brimpInstallIntlProfile',
    '__brimpInstallMediaQueries',
    '__brimpInstallFonts',
    '__brimpInstallPerformance',
    '__brimpInstallScheduling',
    '__brimpPluginScopes',
  ]) {
    delete globalThis[key];
  }
  visited.add(globalThis);
  // Blink profiles need JSC's intrinsic sources rewritten too. Visiting the
  // initial globals reaches JSON, Math, Reflect, Intl and every constructor
  // prototype, not just the uppercase functions.
  if (__nativeStyle === 'chrome') {
    for (const key of Reflect.ownKeys(globalThis)) {
      if (!__initialGlobalNames.has(key)) continue;
      const descriptor = Object.getOwnPropertyDescriptor(globalThis, key);
      if ('value' in descriptor) visit(descriptor.value);
    }
  }
  for (const key of Reflect.ownKeys(globalThis)) {
    if (__initialGlobalNames.has(key)) continue;
    const descriptor = Object.getOwnPropertyDescriptor(globalThis, key);
    if ('value' in descriptor) visit(descriptor.value);
    else {
      if (descriptor.get) __markWebBuiltin(descriptor.get, key, 'get');
      if (descriptor.set) __markWebBuiltin(descriptor.set, key, 'set');
    }
  }
}

Object.defineProperty(Function.prototype, 'toString', {
  value: __functionToString,
  writable: true,
  enumerable: false,
  configurable: true,
});
__markWebBuiltin(__functionToString, 'toString', 'value');
Object.defineProperty(__functionToString, 'name', { value: 'toString', configurable: true });
__nativeMembers.set(__functionToString, { name: 'toString', kind: 'value' });

// Captured and removed by native initialization before plugin/page execution.
Object.defineProperty(globalThis, '__brimpFinalizeWebIdl', {
  value: __finalizeWebIdlInterfaces,
  writable: false,
  enumerable: false,
  configurable: true,
});

const __eventListeners = new WeakMap();
const __disconnectedNodeOrder = new WeakMap();
const __attributeMaps = new WeakMap();
const __inputValues = new WeakMap();
const __inputCheckedStates = new WeakMap();
const __inputIndeterminateStates = new WeakMap();
const __optionSelectedStates = new WeakMap();
const __iframeBrowsingContexts = new WeakMap();
let __activeElement = null;
let __nextDisconnectedNodeOrder = 1;
