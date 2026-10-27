function installDiagnostics(stackEnabled, installRecursive) {
  const emit = globalThis.__brimpTrace;
  delete globalThis.__brimpTrace;
  const ownKeys = Reflect.ownKeys;
  const descriptor = Object.getOwnPropertyDescriptor;
  const define = Object.defineProperty;
  const apply = Reflect.apply;
  const construct = Reflect.construct;
  const OriginalError = Error;
  const visited = new WeakSet();
  const paths = new WeakMap([[globalThis, 'window']]);
  const stringify = JSON.stringify;
  const create = Object.create;
  const setPrototypeOf = Object.setPrototypeOf;
  const substring = String.prototype.slice;
  const minimum = Math.min;
  let reporting = false;
  function report(kind, path, outcome, args = [], metadata = null) {
    if (reporting) return;
    reporting = true;
    try {
      const details = create(null),
        summaries = [];
      setPrototypeOf(summaries, null);
      for (let i = 0; i < minimum(args.length, 8); i++) {
        const v = args[i],
          type = typeof v;
        if (v === null || type === 'number' || type === 'boolean') summaries[i] = v;
        else if (type === 'string') summaries[i] = apply(substring, v, [0, 256]);
        else {
          const summary = create(null);
          summary.type = type;
          summaries[i] = summary;
        }
      }
      details.arguments = summaries;
      if (metadata) {
        details.layer = metadata.layer;
        details.target = metadata.target;
        details.property = metadata.property;
        if (metadata.note) details.note = metadata.note;
      }
      if (kind === 'reflection') {
        details.target = paths.get(args[0]) ?? null;
        details.property = typeof args[1] === 'string' ? apply(substring, args[1], [0, 256]) : null;
      }
      emit(
        kind,
        path,
        outcome,
        stackEnabled ? String(new OriginalError().stack || '') : '',
        stringify(details),
      );
    } finally {
      reporting = false;
    }
  }
  if (installRecursive) return installRecursive(report);
  function wrap(fn, kind, path) {
    return new Proxy(fn, {
      apply(target, receiver, args) {
        try {
          const result = apply(target, receiver, args);
          if (
            kind === 'get' &&
            result !== null &&
            (typeof result === 'object' || typeof result === 'function') &&
            !paths.has(result)
          )
            paths.set(result, path);
          report(kind, path, result === null ? 'null' : typeof result, args);
          return result;
        } catch (error) {
          report(kind, path, 'throw', args);
          throw error;
        }
      },
      construct(target, args, newTarget) {
        try {
          const value = construct(target, args, newTarget);
          paths.set(value, path);
          report('construct', path, 'object', args);
          return value;
        } catch (error) {
          report('construct', path, 'throw', args);
          throw error;
        }
      },
    });
  }
  function instrument(object, path) {
    if (!object || visited.has(object)) return;
    visited.add(object);
    paths.set(object, path);
    for (const key of ownKeys(object)) {
      if (typeof key !== 'string' || key === 'constructor' || key.startsWith('__')) continue;
      const d = descriptor(object, key);
      if (!d || !d.configurable) continue;
      for (const [slot, kind] of [
        ['value', 'call'],
        ['get', 'get'],
        ['set', 'set'],
      ]) {
        if (typeof d[slot] === 'function') d[slot] = wrap(d[slot], kind, `${path}.${key}`);
      }
      define(object, key, d);
    }
  }
  // Only browser constructors: engine intrinsics are deliberately left alone.
  const intrinsic = new Set([
    'Object',
    'Function',
    'Boolean',
    'Symbol',
    'Number',
    'BigInt',
    'String',
    'Array',
    'Date',
    'RegExp',
    'Error',
    'EvalError',
    'RangeError',
    'ReferenceError',
    'SyntaxError',
    'TypeError',
    'URIError',
    'AggregateError',
    'Promise',
    'Map',
    'Set',
    'WeakMap',
    'WeakSet',
    'WeakRef',
    'FinalizationRegistry',
    'ArrayBuffer',
    'SharedArrayBuffer',
    'DataView',
    'Int8Array',
    'Uint8Array',
    'Uint8ClampedArray',
    'Int16Array',
    'Uint16Array',
    'Int32Array',
    'Uint32Array',
    'Float16Array',
    'Float32Array',
    'Float64Array',
    'BigInt64Array',
    'BigUint64Array',
  ]);
  for (const key of ownKeys(globalThis)) {
    if (typeof key !== 'string') continue;
    const d = descriptor(globalThis, key);
    if (!d || typeof d.value !== 'function' || intrinsic.has(key) || key.startsWith('__')) continue;
    const prototype = descriptor(d.value, 'prototype')?.value;
    if (prototype) instrument(prototype, `${key}.prototype`);
    if (d.configurable) define(globalThis, key, { ...d, value: wrap(d.value, 'call', key) });
  }
  for (const key of [
    'navigator',
    'document',
    'location',
    'history',
    'performance',
    'console',
    'crypto',
    'screen',
    'scheduler',
  ]) {
    const d = descriptor(globalThis, key);
    if (d && d.value && typeof d.value === 'object') instrument(d.value, key);
  }
  // Explicit reflection instrumentation; no global-object Proxy and no raw values in logs.
  for (const [object, name, keys] of [
    [
      Object,
      'Object',
      [
        'keys',
        'values',
        'entries',
        'getOwnPropertyNames',
        'getOwnPropertySymbols',
        'getOwnPropertyDescriptor',
        'getOwnPropertyDescriptors',
        'getPrototypeOf',
        'defineProperty',
        'defineProperties',
        'hasOwn',
      ],
    ],
    [
      Reflect,
      'Reflect',
      [
        'ownKeys',
        'get',
        'set',
        'has',
        'getOwnPropertyDescriptor',
        'defineProperty',
        'getPrototypeOf',
      ],
    ],
    [Function.prototype, 'Function.prototype', ['toString']],
  ]) {
    for (const key of keys) {
      const d = descriptor(object, key);
      if (d?.configurable && typeof d.value === 'function')
        define(object, key, { ...d, value: wrap(d.value, 'reflection', `${name}.${key}`) });
    }
  }
}
