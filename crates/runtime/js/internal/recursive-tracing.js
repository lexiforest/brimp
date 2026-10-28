function installRecursiveTracing(report) {
  // A realm-local membrane. Browser code runs on raw receivers; callbacks
  // re-enter the page side. Nothing is installed on Object.prototype.
  const root = globalThis;
  const R = {};
  for (const key of Reflect.ownKeys(Reflect)) R[key] = Reflect[key];
  const ProxyClass = Proxy;
  const isView = ArrayBuffer.isView;
  const bufferLength = R.getOwnPropertyDescriptor(ArrayBuffer.prototype, 'byteLength').get;
  const sharedBufferLength =
    typeof SharedArrayBuffer === 'function'
      ? R.getOwnPropertyDescriptor(SharedArrayBuffer.prototype, 'byteLength').get
      : null;
  const symbolString = Symbol.prototype.toString;
  const functionCall = Function.prototype.call;
  const functionApply = Function.prototype.apply;
  const functionBind = Function.prototype.bind;
  const slice = String.prototype.slice;
  const outgoing = new WeakMap(),
    originals = new WeakMap();
  const incoming = new WeakMap(),
    callbacks = new WeakMap();
  const paths = new WeakMap([[root, 'window']]);
  const intrinsics = new Set([
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
    'Proxy',
    'Reflect',
    'JSON',
    'Math',
    'Intl',
    'Atomics',
    'WebAssembly',
    'eval',
    'parseInt',
    'parseFloat',
    'isNaN',
    'isFinite',
    'decodeURI',
    'decodeURIComponent',
    'encodeURI',
    'encodeURIComponent',
    'escape',
    'unescape',
  ]);
  const untouched = new WeakSet();
  let internalDepth = 0;
  const objectLike = (value) =>
    value !== null && (typeof value === 'object' || typeof value === 'function');
  const raw = (value) => (objectLike(value) ? originals.get(value) || value : value);
  const bounded = (text) => R.apply(slice, text, [0, 512]);
  const propertyName = (key) => (typeof key === 'symbol' ? R.apply(symbolString, key, []) : key);
  const memberPath = (path, key) => bounded(`${path}.${propertyName(key)}`);
  function internal(fn) {
    internalDepth++;
    try {
      return fn();
    } finally {
      internalDepth--;
    }
  }
  function record(kind, path, key, result, args, note, extra) {
    path = bounded(path);
    const details = {
      layer: 'recursive',
      target: path,
      property: key === null ? null : bounded(propertyName(key)),
      note,
    };
    if (extra) {
      for (const name of Object.keys(extra)) details[name] = extra[name];
    }
    report(kind, key === null ? path : memberPath(path, key), result, args, details);
  }
  function failureName(error) {
    try {
      if (error === null) return 'null';
      const type = typeof error;
      if (type !== 'object' && type !== 'function') return type;
      let current = error;
      for (let depth = 0; current && depth < 4; depth++) {
        const d = R.getOwnPropertyDescriptor(current, 'name');
        if (d && 'value' in d && typeof d.value === 'string') return bounded(d.value);
        current = R.getPrototypeOf(current);
      }
    } catch (_) {
      /* a hostile error object must not break tracing */
    }
    return 'Error';
  }
  function observe(kind, path, key, args, fn, extra) {
    try {
      const result = internal(fn);
      let details = null;
      if (extra) {
        try {
          details = extra(result);
        } catch (_) {
          /* presence is optional detail */
        }
      }
      record(kind, path, key, result === null ? 'null' : typeof result, args, null, details);
      return result;
    } catch (error) {
      record(kind, path, key, 'throw', args, null, { failure: failureName(error) });
      throw error;
    }
  }
  function note(path, reason) {
    record('coverage', path, null, 'raw', [], reason);
  }
  function binary(value) {
    if (isView(value)) return true;
    try {
      R.apply(bufferLength, value, []);
      return true;
    } catch (_) {
      /* not an ArrayBuffer */
    }
    if (sharedBufferLength) {
      try {
        R.apply(sharedBufferLength, value, []);
        return true;
      } catch (_) {
        /* not a SharedArrayBuffer */
      }
    }
    return false;
  }
  // Values supplied by page code travel inward through stable adapters, so
  // event listeners (including handleEvent objects) can be removed by identity.
  function inward(value, path) {
    if (!objectLike(value)) return value;
    if (originals.has(value)) return originals.get(value);
    if (untouched.has(value) || binary(value)) return value;
    if (incoming.has(value)) return incoming.get(value);
    path = bounded(path);
    const adapter = new ProxyClass(value, {
      get(target, key) {
        const depth = internalDepth;
        internalDepth = 0;
        let result;
        try {
          result = R.get(target, key, target);
        } finally {
          internalDepth = depth;
        }
        const d = R.getOwnPropertyDescriptor(target, key);
        if (d && !d.configurable && 'value' in d && !d.writable) {
          if (objectLike(result))
            note(memberPath(path, key), 'Fixed callback property requires its original value');
          return result;
        }
        return inward(result, memberPath(path, key));
      },
      apply(target, receiver, args) {
        const depth = internalDepth;
        internalDepth = 0;
        try {
          return inward(
            R.apply(
              target,
              outward(receiver, `${path}.this`),
              args.map((arg, i) => outward(arg, `${path}.arguments.${i}`)),
            ),
            `${path}.result`,
          );
        } finally {
          internalDepth = depth;
        }
      },
    });
    incoming.set(value, adapter);
    callbacks.set(adapter, value);
    return adapter;
  }
  function outward(value, path) {
    if (!objectLike(value) || untouched.has(value)) return value;
    if (callbacks.has(value)) return callbacks.get(value);
    if (originals.has(value)) return value;
    if (outgoing.has(value)) return outgoing.get(value);
    if (binary(value)) {
      note(path, 'Binary buffers and views retain their native identity');
      return value;
    }
    path = paths.get(value) || bounded(path);
    paths.set(value, path);
    const handler = {
      get(target, key) {
        const result = observe(
          'get',
          path,
          key,
          [],
          () => R.get(target, key, target),
          (value) => (value === undefined ? { present: internal(() => R.has(target, key)) } : null),
        );
        const d = R.getOwnPropertyDescriptor(target, key);
        if (d && !d.configurable && 'value' in d && !d.writable) {
          if (objectLike(result))
            note(memberPath(path, key), 'Fixed property requires its original value');
          return result;
        }
        return outward(result, memberPath(path, key));
      },
      set(target, key, value) {
        return observe('set', path, key, [value], () =>
          R.set(
            target,
            key,
            typeof value === 'function' ? inward(value, memberPath(path, key)) : raw(value),
            target,
          ),
        );
      },
      has: (target, key) => observe('has', path, key, [], () => R.has(target, key)),
      deleteProperty: (target, key) =>
        observe('deleteProperty', path, key, [], () => R.deleteProperty(target, key)),
      ownKeys: (target) => observe('ownKeys', path, null, [], () => R.ownKeys(target)),
      getOwnPropertyDescriptor(target, key) {
        const d = observe('getOwnPropertyDescriptor', path, key, [], () =>
          R.getOwnPropertyDescriptor(target, key),
        );
        if (!d) return d;
        if (!d.configurable) {
          if (objectLike(d.value) || d.get || d.set)
            note(memberPath(path, key), 'Fixed descriptor exposes original references');
          return d;
        }
        if ('value' in d) d.value = outward(d.value, memberPath(path, key));
        if (d.get) d.get = outward(d.get, `${memberPath(path, key)}.get`);
        if (d.set) d.set = outward(d.set, `${memberPath(path, key)}.set`);
        return d;
      },
      defineProperty(target, key, descriptor) {
        // Non-configurable definitions must retain the exact supplied
        // references to satisfy the caller-facing Proxy invariants.
        return observe('defineProperty', path, key, [], () =>
          R.defineProperty(target, key, descriptor),
        );
      },
      getPrototypeOf(target) {
        const proto = observe('getPrototypeOf', path, null, [], () => R.getPrototypeOf(target));
        if (!R.isExtensible(target)) {
          if (proto) note(path, 'Non-extensible target requires its original prototype');
          return proto;
        }
        return outward(proto, `${path}.[[Prototype]]`);
      },
      setPrototypeOf: (target, proto) =>
        observe('setPrototypeOf', path, null, [], () => R.setPrototypeOf(target, raw(proto))),
      isExtensible: (target) =>
        observe('isExtensible', path, null, [], () => R.isExtensible(target)),
      preventExtensions: (target) =>
        observe('preventExtensions', path, null, [], () => R.preventExtensions(target)),
      apply(target, receiver, args) {
        if (internalDepth) return R.apply(target, raw(receiver), args.map(raw));
        // Binding/calling a traced function must retain its adapter.
        const functionOperation =
          target === functionCall || target === functionApply || target === functionBind;
        const result = observe('call', path, null, args, () =>
          R.apply(
            target,
            functionOperation ? receiver : raw(receiver),
            args.map((arg, i) => inward(arg, `${path}.arguments.${i}`)),
          ),
        );
        return outward(
          result,
          target === functionBind ? `${paths.get(raw(receiver)) || path}.bound` : `${path}.result`,
        );
      },
      construct(target, args, newTarget) {
        const result = observe('construct', path, null, args, () =>
          R.construct(
            target,
            args.map((arg, i) => inward(arg, `${path}.arguments.${i}`)),
            raw(newTarget),
          ),
        );
        return outward(result, `${path}.result`);
      },
    };
    const proxy = new ProxyClass(value, handler);
    outgoing.set(value, proxy);
    originals.set(proxy, value);
    return proxy;
  }
  const roots = R.ownKeys(root);
  for (const key of roots) {
    const d = R.getOwnPropertyDescriptor(root, key);
    if (typeof key === 'string' && intrinsics.has(key) && objectLike(d?.value)) {
      untouched.add(d.value);
      const proto = R.getOwnPropertyDescriptor(d.value, 'prototype')?.value;
      if (objectLike(proto)) untouched.add(proto);
    } else if (objectLike(d?.value) && !paths.has(d.value)) paths.set(d.value, propertyName(key));
  }
  // Class.prototype is commonly fixed. Adapt its configurable methods and
  // accessors as well, so extracted WebIDL getters accept traced receivers.
  // Internal browser calls bypass these adapters to retain private brands.
  const prototypes = new WeakSet();
  for (const key of roots) {
    if (typeof key !== 'string' || intrinsics.has(key)) continue;
    const value = R.getOwnPropertyDescriptor(root, key)?.value;
    if (typeof value !== 'function') continue;
    const proto = R.getOwnPropertyDescriptor(value, 'prototype')?.value;
    if (objectLike(proto) && !paths.has(proto)) paths.set(proto, `${key}.prototype`);
  }
  function adaptPrototype(proto, path) {
    if (!proto || untouched.has(proto) || prototypes.has(proto)) return;
    prototypes.add(proto);
    path = paths.get(proto) || path;
    paths.set(proto, path);
    for (const key of R.ownKeys(proto)) {
      if (key === 'constructor') continue;
      const d = R.getOwnPropertyDescriptor(proto, key);
      if (!d?.configurable) continue;
      for (const slot of ['value', 'get', 'set']) {
        if (typeof d[slot] === 'function') d[slot] = outward(d[slot], memberPath(path, key));
      }
      R.defineProperty(proto, key, d);
    }
    adaptPrototype(R.getPrototypeOf(proto), `${path}.[[Prototype]]`);
  }
  for (const key of roots) {
    if (typeof key !== 'string' || intrinsics.has(key) || key.startsWith('__')) continue;
    const value = R.getOwnPropertyDescriptor(root, key)?.value;
    if (typeof value === 'function')
      adaptPrototype(R.getOwnPropertyDescriptor(value, 'prototype')?.value, `${key}.prototype`);
  }
  for (const key of roots) {
    if (typeof key === 'string' && (intrinsics.has(key) || key.startsWith('__'))) continue;
    const d = R.getOwnPropertyDescriptor(root, key);
    if (!d?.configurable) {
      if (objectLike(d?.value) || d?.get || d?.set)
        note(
          memberPath('window', key),
          'Non-configurable global cannot install a traced root accessor',
        );
      continue;
    }
    // Do not evaluate accessors during setup.
    let value = d.value;
    const read = () => ('value' in d ? value : d.get ? R.apply(d.get, root, []) : undefined);
    const replacement = {
      enumerable: d.enumerable,
      configurable: true,
      get() {
        if (internalDepth) return read();
        return outward(
          observe('get', 'window', key, [], read, (value) =>
            value === undefined ? { present: internal(() => R.has(root, key)) } : null,
          ),
          propertyName(key),
        );
      },
    };
    if (d.writable || d.set)
      replacement.set = (next) => {
        const write = () => {
          if ('value' in d)
            value = typeof next === 'function' ? inward(next, propertyName(key)) : raw(next);
          else R.apply(d.set, root, [inward(next, propertyName(key))]);
        };
        if (internalDepth) write();
        else observe('set', 'window', key, [next], write);
      };
    R.defineProperty(root, key, replacement);
  }
  // Rust holds these controllers privately; their implementation must see raw
  // globals too, including native input and parser-triggered event delivery.
  return (_, controller) =>
    (...args) =>
      internal(() => R.apply(controller, undefined, args.map(raw)));
}
