// Complete the page's interface surface from WebKit's own IDL.
//
// The hand-written modules beside this file implement what pages actually call;
// this file makes the whole surface match the browser it claims to be. A script
// that probes names, order, kinds, arities, constants and descriptor flags then
// finds what Safari has, and nothing Safari lacks.
//
// `scripts/idl/extract.mjs` produced the shape document from
// `Source/WebCore/**/*.idl`; `scripts/idl/capture.mjs` recorded which names a real
// Safari exposes, the descriptor of each window property, and the shape of the
// few WebKit globals that have no IDL in the tree. `scripts/idl/check.mjs` diffs
// the result against that reference.
const __idlInterfaces = __brimpIdlDocument.interfaces;
const __idlCapture = __brimpIdlCapture;
const __idlExtra = __idlCapture.extra ?? {};
const __idlOmitted = __idlCapture.omitted ?? {};
const __idlSlots = new WeakMap();
const __idlUnresolved = [];
// Names a prototype never narrows away and never re-describes.
const __idlReserved = new Set([
  'constructor',
  'length',
  'name',
  'prototype',
  'caller',
  'arguments',
]);
// An interface brimp implements assigns to its own attributes in its
// constructors, so a placeholder accessor there always accepts a write; an
// interface this installer creates has no implementation to protect and stays as
// strict as the browser.
let __idlPermissive = false;
// A page calls these unqualified (`addEventListener('x', …)`), which in JSC
// resolves the method with `this === undefined`. They stay own globals so the
// runtime can treat that receiver as the window.
const __idlKeepGlobals = new Set(['addEventListener', 'removeEventListener', 'dispatchEvent']);
// Names the core modules defined, recorded by `install.js` before any plugin
// adds its own: a plugin global is not part of the browser surface and must
// survive the narrowing below.
let __idlCoreGlobalNames = null;
const __idlNoDelete = true;

function __idlSlot(owner, name) {
  let bag = __idlSlots.get(owner);
  if (bag === undefined) {
    bag = new Map();
    __idlSlots.set(owner, bag);
  }
  return bag;
}

// Each accessor closes over its own name: a computed-name object literal would
// allocate an object per member, which matters when a realm installs thousands.
function __idlGetter(name, globalReceiver) {
  const getter = function () {
    // JSC invokes unqualified global accessors with an undefined receiver.
    const bag = __idlSlot(this ?? globalReceiver, name);
    return bag.has(name) ? bag.get(name) : undefined;
  };
  return __markWebBuiltin(getter, name, 'get');
}

// A write becomes an own data property, which is what a WebKit attribute does
// for a receiver: later reads and `Object.assign` see the value without asking
// the accessor again.
function __idlSetter(name) {
  const setter = function (value) {
    __idlSlot(this, name).set(name, value);
    Object.defineProperty(this, name, {
      value,
      writable: true,
      enumerable: true,
      configurable: true,
    });
  };
  return __markWebBuiltin(setter, name, 'set');
}

function __idlMethod(name, length) {
  const method = function () {
    return undefined;
  };
  __markWebBuiltin(method, name, 'value');
  __idlSetShape(method, length ?? 0);
  return method;
}

function __idlSetShape(fn, length) {
  try {
    if (fn.length !== length)
      Object.defineProperty(fn, 'length', { value: length, configurable: true });
  } catch {}
  return fn;
}

function __idlConstructor(name, length) {
  const constructor = function () {
    throw new TypeError('Illegal constructor');
  };
  __markWebBuiltin(constructor);
  try {
    Object.defineProperty(constructor, 'name', { value: name, configurable: true });
  } catch {}
  __idlSetShape(constructor, length ?? 0);
  return constructor;
}

function __idlKey(name) {
  return name === 'Symbol.iterator' ? Symbol.iterator : name;
}

// The descriptor a window property carries, as Safari reported it:
// `[get, set, enumerable, configurable, writable, valueLength]`.
function __idlCapturedFlags(name) {
  const flags = __idlCapture.window[name];
  if (flags === undefined) return { enumerable: false, configurable: true, writable: true };
  return { enumerable: flags[2] !== 0, configurable: flags[3] !== 0, writable: flags[4] !== 0 };
}

function __idlDescribe(object, key, flags) {
  return Object.defineProperty(object, key, flags);
}

// Re-describe an existing property with the flags a browser reports, keeping the
// value behind an accessor when the reference has one.
function __idlAlignFlags(object, key, want) {
  const descriptor = Object.getOwnPropertyDescriptor(object, key);
  if (descriptor === undefined || descriptor.configurable !== true) return;
  const accessor = descriptor.get !== undefined || descriptor.set !== undefined;
  if (want.accessor === true && !accessor) {
    const value = descriptor.value;
    const name = typeof key === 'string' ? key : '';
    __idlDescribe(object, key, {
      get: __markWebBuiltin(
        function () {
          return value;
        },
        name,
        'get',
      ),
      set: want.setter
        ? __markWebBuiltin(
            function (next) {
              __idlSlot(object).set(key, next);
            },
            name,
            'set',
          )
        : undefined,
      enumerable: want.enumerable,
      configurable: want.configurable,
    });
    return;
  }
  if (want.accessor === false && accessor) return;
  const next = {
    enumerable: want.enumerable,
    configurable: want.configurable,
  };
  if (accessor) {
    next.get = descriptor.get;
    next.set = want.setter === false ? undefined : descriptor.set;
  } else {
    next.value = descriptor.value;
    next.writable = want.writable !== false;
  }
  __idlDescribe(object, key, next);
}

// A member brimp implements further up the chain still has to appear here: the
// browser declares it on this interface, so this interface owns the property.
function __idlInheritedDescriptor(target, key) {
  for (
    let proto = Object.getPrototypeOf(target);
    proto !== null;
    proto = Object.getPrototypeOf(proto)
  ) {
    const descriptor = Object.getOwnPropertyDescriptor(proto, key);
    if (descriptor !== undefined) return descriptor;
  }
  return undefined;
}

function __idlForwardMember(target, key, inherited) {
  if (inherited.get !== undefined || inherited.set !== undefined) {
    __idlDescribe(target, key, {
      get: inherited.get,
      set: inherited.set,
      enumerable: inherited.enumerable ?? true,
      configurable: true,
    });
    return;
  }
  __idlDescribe(target, key, {
    value: inherited.value,
    writable: inherited.writable ?? true,
    enumerable: inherited.enumerable ?? true,
    configurable: true,
  });
}

function __idlDefineMember(target, record, scope) {
  const key = __idlKey(record.name);
  const existing = Object.getOwnPropertyDescriptor(target, key);
  if (existing !== undefined) {
    __idlAlignMember(target, key, record, scope);
    return;
  }
  const inherited = __idlInheritedDescriptor(target, key);
  if (inherited !== undefined) {
    __idlForwardMember(target, key, inherited);
    __idlAlignMember(target, key, record, scope);
    return;
  }
  // Symbol-keyed members are never enumerable.
  const enumerable = typeof key !== 'symbol';
  if (record.kind === 'operation')
    __idlDescribe(target, key, {
      value: __idlMethod(record.name, record.length),
      writable: true,
      enumerable,
      configurable: true,
    });
  else if (record.kind === 'attribute' && scope === 'instance')
    __idlDescribe(target, key, {
      value: undefined,
      writable: true,
      enumerable,
      configurable: true,
    });
  else if (record.kind === 'attribute')
    __idlDescribe(target, key, {
      get: __idlGetter(record.name),
      set: record.setter || __idlPermissive ? __idlSetter(record.name) : undefined,
      enumerable,
      configurable: true,
    });
}

// WebKit's operations are plain functions; its attributes are accessor pairs, and
// a read-only one has no setter at all.
function __idlAlignMember(target, key, record, scope) {
  const descriptor = Object.getOwnPropertyDescriptor(target, key);
  if (descriptor === undefined) return;
  // A member the runtime implements keeps the shape its implementation chose:
  // event-handler accessors, for example, feed a map the dispatcher reads, and
  // re-describing them would disconnect it.
  if (__idlPermissive) return;
  if (record.kind === 'operation') {
    if (typeof descriptor.value === 'function') {
      __idlSetShape(descriptor.value, record.length ?? 0);
      __idlAlignFlags(target, key, {
        enumerable: typeof key !== 'symbol',
        configurable: true,
        writable: true,
      });
    }
    return;
  }
  if (record.kind !== 'attribute') return;
  if (scope === 'instance') {
    __idlAlignFlags(target, key, { enumerable: true, configurable: true, writable: true });
    return;
  }
  // An implemented interface keeps the property kind it chose: brimp stores
  // attribute state as own data properties, and turning those into accessors
  // would move the state somewhere `Object.assign` cannot see it.
  __idlAlignFlags(target, key, {
    accessor: __idlPermissive ? undefined : true,
    setter: record.setter === true || __idlPermissive,
    enumerable: true,
    configurable: true,
  });
}

function __idlConstant(object, constant) {
  const existing = Object.getOwnPropertyDescriptor(object, constant.name);
  if (existing === undefined)
    __idlDescribe(object, constant.name, {
      value: constant.value,
      writable: false,
      enumerable: true,
      configurable: false,
    });
  else if (existing.configurable === true)
    __idlDescribe(object, constant.name, {
      value: constant.value,
      writable: false,
      enumerable: true,
      configurable: false,
    });
}

// Drop the members WebKit does not declare, so a probe cannot find a name the
// reference browser lacks.
function __idlNarrow(object, keep) {
  for (const key of Reflect.ownKeys(object)) {
    if (typeof key !== 'string' || __idlReserved.has(key) || key.startsWith('__')) continue;
    if (keep.has(key)) continue;
    const descriptor = Object.getOwnPropertyDescriptor(object, key);
    if (descriptor === undefined || descriptor.configurable !== true) continue;
    delete object[key];
  }
}

// Delete the members this Safari does not expose. `scripts/idl/capture.mjs`
// measured the difference against the same IDL document, so the list only holds
// members WebKit declares behind a setting the release leaves off.
function __idlNarrowOmitted(object, interfaceName, location) {
  const omitted = __idlOmitted[interfaceName]?.[location];
  if (omitted === undefined) return 0;
  let removed = 0;
  for (const name of omitted) {
    if (__idlReserved.has(name)) continue;
    const descriptor = Object.getOwnPropertyDescriptor(object, name);
    if (descriptor === undefined || descriptor.configurable !== true) continue;
    delete object[name];
    removed += 1;
  }
  return removed;
}

function __idlMemberNames(entry) {
  const names = new Set();
  for (const record of [
    ...entry.members.prototype,
    ...entry.members.interface,
    ...entry.members.instance,
  ])
    names.add(record.name);
  for (const constant of entry.constants) names.add(constant.name);
  return names;
}

function __idlInstallInterface(interfaceName, entry, claimed) {
  let constructor = globalThis[interfaceName];
  // Brimp aliases `HTMLDocument` and `XMLDocument` to the `Document` class; a
  // browser gives each its own interface object.
  // Brimp sometimes aliases one class to several interface names. Replacing it
  // would strand every object the implementation already made, so the alias is
  // installed under the second name too; `HTMLDocument` and `XMLDocument` are the
  // exception because pages test `instanceof` against them.
  const shared =
    typeof constructor === 'function' &&
    claimed.has(constructor) &&
    claimed.get(constructor) !== interfaceName;
  const distinct = shared && (interfaceName === 'HTMLDocument' || interfaceName === 'XMLDocument');
  if (shared && !distinct) {
    __idlDescribe(globalThis, interfaceName, {
      value: constructor,
      ...__idlCapturedFlags(interfaceName),
    });
  }
  if (distinct) constructor = undefined;
  const created = typeof constructor !== 'function';
  if (created) {
    constructor = __idlConstructor(interfaceName, entry.construction?.length ?? 0);
    __idlDescribe(globalThis, interfaceName, {
      value: constructor,
      writable: true,
      enumerable: false,
      configurable: true,
    });
  }
  if (interfaceName === 'MediaSession') __idlSetShape(constructor, entry.construction?.length ?? 0);
  claimed.set(constructor, interfaceName);
  if (!created && constructor.name !== interfaceName) {
    try {
      Object.defineProperty(constructor, 'name', { value: interfaceName, configurable: true });
    } catch {}
  }
  // Extending an interface brimp already implements is deferred: a placeholder
  // in front of a working binding (IDB's request events, for instance) breaks
  // the implementation, and `TODO.md` records the remaining work.
  if (!created) return { created: false };
  const prototype = constructor.prototype;
  if (prototype !== null && typeof prototype === 'object') {
    const parent = entry.parent !== null ? globalThis[entry.parent] : undefined;
    if (typeof parent === 'function' && Object.getPrototypeOf(prototype) !== parent.prototype)
      Object.setPrototypeOf(prototype, parent.prototype);
    if (typeof parent === 'function' && Object.getPrototypeOf(constructor) !== parent)
      Object.setPrototypeOf(constructor, parent);
    if (Object.getOwnPropertyDescriptor(prototype, 'constructor') === undefined)
      __idlDescribe(prototype, 'constructor', {
        value: constructor,
        writable: true,
        enumerable: false,
        configurable: true,
      });
    __idlPermissive = !created;
    for (const record of entry.members.prototype) __idlDefineMember(prototype, record, 'prototype');
    __idlPermissive = false;
    __idlNarrowOmitted(prototype, interfaceName, 'prototype');
  }
  for (const record of entry.members.interface) __idlDefineMember(constructor, record, 'prototype');
  // `members.instance` describes properties of each instance, which only the
  // `[Global]` object installs itself (handled by `__idlInstallGlobalMember`).
  __idlNarrowOmitted(constructor, interfaceName, 'interface');
  for (const constant of entry.constants) {
    __idlConstant(constructor, constant);
    if (prototype !== null && typeof prototype === 'object') __idlConstant(prototype, constant);
  }
  for (const name of __idlOmitted[interfaceName]?.constants ?? []) {
    for (const target of [constructor, prototype])
      if (target !== null && typeof target === 'object') {
        const descriptor = Object.getOwnPropertyDescriptor(target, name);
        if (descriptor?.configurable === true) delete target[name];
      }
  }
  return { created };
}

function __idlInstallNamespace(name, entry) {
  let namespace = globalThis[name];
  if (namespace === null || typeof namespace !== 'object' || typeof namespace === 'function') {
    namespace = {};
    __idlDescribe(globalThis, name, {
      value: namespace,
      writable: true,
      enumerable: false,
      configurable: true,
    });
  }
  for (const record of entry.members.instance) __idlDefineMember(namespace, record, 'prototype');
  for (const constant of entry.constants) __idlConstant(namespace, constant);
  __idlNarrowOmitted(namespace, name, 'interface');
}

function __idlInstallFactory(name, entry, interfaceName) {
  const target = globalThis[interfaceName];
  if (typeof target !== 'function') return false;
  const constructor = __idlConstructor(name, 0);
  const prototype = Object.create(target.prototype);
  __idlDescribe(constructor, 'prototype', {
    value: prototype,
    writable: false,
    enumerable: false,
    configurable: false,
  });
  __idlDescribe(prototype, 'constructor', {
    value: constructor,
    writable: true,
    enumerable: false,
    configurable: true,
  });
  __idlDescribe(globalThis, name, {
    value: constructor,
    ...__idlCapturedFlags(name),
  });
  return true;
}

// A global WebKit object that the IDL in the tree does not describe
// (`GestureEvent`, the legacy `SVGPathSeg` family, Safari's own globals).
function __idlInstallCaptured(name, captured) {
  const flags = __idlCapturedFlags(name);
  if (captured.kind === 'object') {
    const object = {};
    for (const [member, tuple] of Object.entries(captured.own))
      __idlDescribeCaptured(object, member, tuple, 'object');
    __idlDescribe(globalThis, name, { value: object, ...flags });
    return;
  }
  const constructor = __idlConstructor(name, captured.length ?? 0);
  for (const [member, tuple] of Object.entries(captured.own))
    if (member !== 'length' && member !== 'name' && member !== 'prototype')
      __idlDescribeCaptured(constructor, member, tuple, 'object');
  const prototype = Object.create(null);
  Object.setPrototypeOf(prototype, Object.getPrototypeOf(constructor.prototype));
  __idlDescribe(constructor, 'prototype', {
    value: prototype,
    writable: false,
    enumerable: false,
    configurable: false,
  });
  const parentName = captured.parent_proto;
  const parent = typeof parentName === 'string' ? globalThis[parentName] : undefined;
  if (typeof parent === 'function') Object.setPrototypeOf(prototype, parent.prototype);
  for (const [member, tuple] of Object.entries(captured.proto ?? {}))
    if (member !== 'constructor') __idlDescribeCaptured(prototype, member, tuple, 'prototype');
  __idlDescribe(prototype, 'constructor', {
    value: constructor,
    writable: true,
    enumerable: false,
    configurable: true,
  });
  if (typeof parent === 'function' && Object.getPrototypeOf(constructor) !== parent)
    Object.setPrototypeOf(constructor, parent);
  __idlDescribe(globalThis, name, { value: constructor, ...flags });
}

// `[get, set, enumerable, configurable, writable, valueLength]` from the capture.
function __idlDescribeCaptured(target, member, tuple, scope) {
  if (!Array.isArray(tuple)) return;
  const enumerable = tuple[2] !== 0;
  const configurable = tuple[3] !== 0;
  const writable = tuple[4] !== 0;
  if (tuple[0] === 1 || tuple[1] === 1) {
    __idlDescribe(target, member, {
      get: tuple[0] === 1 ? __idlGetter(member) : undefined,
      set: tuple[1] === 1 ? __idlSetter(member) : undefined,
      enumerable,
      configurable,
    });
    return;
  }
  if (tuple[5] >= 0) {
    __idlDescribe(target, member, {
      value: __idlMethod(member, tuple[5]),
      writable,
      enumerable,
      configurable,
    });
    return;
  }
  __idlDescribe(target, member, { value: undefined, writable, enumerable, configurable });
}

// Which IDL entry a captured window name comes from.
function __idlResolve(name) {
  const direct = __idlInterfaces[name];
  if (direct !== undefined)
    return {
      entry: direct,
      interfaceName: name,
      kind: direct.namespace ? 'namespace' : 'interface',
    };
  for (const [interfaceName, entry] of Object.entries(__idlInterfaces)) {
    if (entry.aliases?.includes(name)) return { entry, interfaceName, kind: 'alias' };
    if (entry.factory?.includes(name)) return { entry, interfaceName, kind: 'factory' };
    if (entry.global === true || entry.namespace === true)
      for (const location of ['instance', 'prototype'])
        if (entry.members[location].some((member) => member.name === name))
          return { entry, interfaceName, kind: 'global-member' };
  }
  return null;
}

// Window IDL members are own properties of the global object, so the installer
// writes them there instead of on a prototype.
function __idlInstallGlobalMember(name, entry) {
  const record = [...entry.members.instance, ...entry.members.prototype].find(
    (member) => member.name === name,
  );
  if (record === undefined) return false;
  const flags = __idlCapturedFlags(name);
  const existing = Object.getOwnPropertyDescriptor(globalThis, name);
  if (existing !== undefined) {
    if (record.kind === 'operation') {
      if (typeof existing.value === 'function') {
        __idlSetShape(existing.value, record.length ?? 0);
        __idlAlignFlags(globalThis, name, {
          enumerable: flags.enumerable,
          configurable: flags.configurable,
          writable: true,
        });
      }
    } else
      __idlAlignFlags(globalThis, name, {
        accessor: flags.writable === false,
        setter: record.setter === true,
        enumerable: flags.enumerable,
        configurable: flags.configurable,
      });
    return true;
  }
  if (record.kind === 'operation')
    __idlDescribe(globalThis, name, {
      value: __idlMethod(record.name, record.length),
      writable: true,
      enumerable: flags.enumerable,
      configurable: flags.configurable,
    });
  else
    __idlDescribe(globalThis, name, {
      get: __idlGetter(record.name, globalThis),
      set: record.setter ? __idlSetter(record.name) : undefined,
      enumerable: flags.enumerable,
      configurable: flags.configurable,
    });
  return true;
}

function __idlInstallSurface() {
  const plan = new Map();
  for (const name of __idlCapture.names) {
    const resolved = __idlResolve(name);
    if (resolved === null) {
      if (__idlExtra[name] === undefined) __idlUnresolved.push(name);
      continue;
    }
    plan.set(name, resolved);
  }

  // Interfaces first, parents before children so a prototype chain resolves when
  // it is built.
  const interfaceNames = new Set();
  for (const resolved of plan.values())
    if (resolved.kind === 'interface' || resolved.kind === 'namespace')
      interfaceNames.add(resolved.interfaceName);
  const ordered = [];
  const visit = (interfaceName, seen = new Set()) => {
    if (seen.has(interfaceName) || ordered.includes(interfaceName)) return;
    seen.add(interfaceName);
    const parent = __idlInterfaces[interfaceName]?.parent;
    if (typeof parent === 'string' && interfaceNames.has(parent)) visit(parent, seen);
    ordered.push(interfaceName);
  };
  for (const interfaceName of interfaceNames) visit(interfaceName);
  const created = [];
  const claimed = new Map();
  for (const interfaceName of ordered) {
    const entry = __idlInterfaces[interfaceName];
    if (entry.namespace === true) __idlInstallNamespace(interfaceName, entry);
    else if (__idlInstallInterface(interfaceName, entry, claimed).created === true)
      created.push(interfaceName);
  }

  // Then every other name Safari carries, in its own property order.
  const unresolved = [];
  for (const name of __idlCapture.names) {
    const resolved = plan.get(name);
    if (resolved !== undefined) {
      if (resolved.kind === 'interface' || resolved.kind === 'namespace') continue;
      if (Object.getOwnPropertyDescriptor(globalThis, name) !== undefined) continue;
      if (resolved.kind === 'factory')
        __idlInstallFactory(name, resolved.entry, resolved.interfaceName);
      else if (resolved.kind === 'alias') {
        const target = globalThis[resolved.interfaceName];
        if (typeof target === 'function')
          __idlDescribe(globalThis, name, { value: target, ...__idlCapturedFlags(name) });
      } else if (__idlInstallGlobalMember(name, resolved.entry) !== true) unresolved.push(name);
      continue;
    }
    if (Object.getOwnPropertyDescriptor(globalThis, name) !== undefined) continue;
    const captured = __idlExtra[name];
    if (captured !== undefined) __idlInstallCaptured(name, captured);
  }

  // The page document was built before `HTMLDocument` existed.
  const pageDocument = globalThis.document;
  if (
    typeof globalThis.HTMLDocument === 'function' &&
    pageDocument !== null &&
    typeof pageDocument === 'object' &&
    !(pageDocument instanceof globalThis.HTMLDocument) &&
    pageDocument.contentType === 'text/html'
  )
    Object.setPrototypeOf(pageDocument, globalThis.HTMLDocument.prototype);

  // Finally drop names this Safari does not have, so a probe cannot find them.
  // `__`-prefixed names are the runtime's own and the engine's intrinsics stay:
  // the capture cannot name a binding the engine keeps unreachable.
  const known = new Set(__idlCapture.names);
  const removed = [];
  for (const name of Object.getOwnPropertyNames(globalThis)) {
    if (known.has(name) || name.startsWith('__') || __idlKeepGlobals.has(name)) continue;
    const descriptor = Object.getOwnPropertyDescriptor(globalThis, name);
    if (descriptor === undefined || descriptor.configurable !== true) continue;
    // An engine intrinsic (`Object`, `Infinity`, …) predates every script and
    // removing one can strand the runtime's own initialization.
    if (__initialGlobalNames.has(name)) continue;
    if (__idlCoreGlobalNames !== null && !__idlCoreGlobalNames.includes(name)) continue;
    if (__idlNoDelete) continue;
    delete globalThis[name];
    removed.push(name);
  }
  return {
    removed: removed.sort(),
    created: created.sort(),
    unresolved: [...unresolved, ...__idlUnresolved].sort(),
  };
}

globalThis.__brimpInstallIdlSurface = () => {
  try {
    return __idlInstallSurface();
  } catch (error) {
    return { error: String(error?.stack ?? error) };
  }
};
