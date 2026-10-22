import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { pathToFileURL } from 'node:url';
import { AsyncLocalStorage } from 'node:async_hooks';

export const callbackContext = new AsyncLocalStorage();

function encode(value, ancestors = new Set()) {
  if (value === null) return { type: 'null' };
  if (typeof value === 'boolean' || typeof value === 'string') return { type: typeof value, value };
  if (typeof value === 'number' && Number.isFinite(value)) return { type: 'number', value };
  if (value instanceof ArrayBuffer) return { type: 'binary', value: [...new Uint8Array(value)] };
  if (ArrayBuffer.isView(value)) return { type: 'binary', value: [...new Uint8Array(value.buffer, value.byteOffset, value.byteLength)] };
  if (!value || typeof value !== 'object' || ancestors.has(value)) throw new TypeError('host values must be acyclic JSON-like values or binary buffers');
  ancestors.add(value);
  try {
    if (Array.isArray(value)) return { type: 'array', value: value.map(item => encode(item, ancestors)) };
    if (Object.getPrototypeOf(value) === Object.prototype || Object.getPrototypeOf(value) === null) {
      return { type: 'object', value: Object.fromEntries(Object.entries(value).map(([key, item]) => [key, encode(item, ancestors)])) };
    }
    throw new TypeError('host values must be JSON-like values or binary buffers');
  } finally { ancestors.delete(value); }
}
function decode({ type, value }) {
  if (type === 'null') return null;
  if (type === 'binary') return Buffer.from(value);
  if (type === 'array') return value.map(decode);
  if (type === 'object') return Object.fromEntries(Object.entries(value).map(([key, item]) => [key, decode(item)]));
  return value;
}
export class Plugin {
  constructor(name, source, handlers = {}, options = {}, setup = null) {
    const serialized = JSON.stringify(options);
    this.name = name; this.source = `context => (${source}\n)({...context, options: ${serialized}})`; this.handlers = Object.freeze({ ...handlers }); this.setup = setup;
    Object.freeze(this);
  }
  static fromFile(path, {name, handlers = {}, options = {}, setup = null}) {
    const url = path instanceof URL ? path : pathToFileURL(resolve(path));
    return new Plugin(name, readFileSync(url, 'utf8') + `\n//# sourceURL=${url.href}\n`, handlers, options, setup);
  }
}
export class HostContext {
  #call;
  constructor(call) { this.#call = call; }
  get sessionId() { return this.#call.sessionId; }
  get url() { return this.#call.url ?? null; }
  get origin() { return this.#call.origin ?? null; }
  get pageId() { return this.#call.pageId; }
  get realmId() { return this.#call.realmId; }
  get cancelled() { return this.#call.cancelled; }
  emit(channel, value) { this.#call.emit(channel, JSON.stringify(encode(value))); }
}
export async function configurePlugins(plugins, session) {
  const handlers = new Map(), disposers = [];
  let disposed = false;
  const dispose = async () => {
    if (disposed) return;
    disposed = true;
    const errors = [];
    for (const cleanup of disposers.reverse()) { try { await cleanup(); } catch (error) { errors.push(error); } }
    handlers.clear();
    if (errors.length) throw new AggregateError(errors, 'Plugin session cleanup failed');
  };
  try {
    for (const plugin of plugins) {
      if (!plugin.name || handlers.has(plugin.name)) throw new TypeError('Plugin names must be nonempty and unique');
      const scope = plugin.setup?.() ?? {};
      if (scope && typeof scope.then === 'function') throw new TypeError('Plugin setup must be synchronous');
      if (scope.dispose) disposers.push(() => scope.dispose());
      handlers.set(plugin.name, new Map(Object.entries({...plugin.handlers, ...scope.handlers})));
    }
  } catch (error) { await dispose(); throw error; }
  return {
    dispose,
    descriptors: plugins.map(plugin => ({ name: plugin.name, source: plugin.source, handlers: [...handlers.get(plugin.name).keys()] })),
    dispatch(call) {
      if (call.cancelled) { call.reject('plugin realm is closed'); return; }
      callbackContext.run({ pageId: call.pageId, session }, async () => {
        try {
          const handler = handlers.get(call.pluginName).get(call.name);
          const result = await handler(new HostContext(call), ...JSON.parse(call.arguments).map(decode));
          call.resolve(JSON.stringify(encode(result)));
        } catch (error) { call.reject(String(error?.stack ?? error)); }
      });
    },
  };
}

function bundled(name, options) {
  let source = readFileSync(new URL(`./bundled/${name}.js`, import.meta.url), 'utf8');
  if (options !== undefined) source = `context => (${source})(context, ${JSON.stringify(options)})`;
  return new Plugin(name === 'extraction' ? 'defuddle' : name, source);
}
export const storagePlugin = () => bundled('storage');
export const canvasPlugin = (options = {}) => bundled('canvas', options);
export const servicesPlugin = (options = {}) => bundled('services', options);
export const extractionPlugin = () => bundled('extraction');

export const geometryPlugin = () => bundled('geometry');
export const cssTypedOmPlugin = () => bundled('css-typed-om');
export const schedulingPlugin = () => bundled('scheduling');
export const graphicsPlugin = (options = {}, handlers = {}) => new Plugin('graphics', bundled('graphics', options).source, handlers);
export const audioPlugin = (options = {}, handlers = {}) => new Plugin('audio', bundled('audio', options).source, handlers);

/** Host-side discovery only. Each call returns an independent catalog snapshot. */
export function pluginCatalog() {
  return JSON.parse(readFileSync(new URL('./bundled/catalog.json', import.meta.url), 'utf8'));
}

function memoryStorageBackend() {
  const origins = new Map();
  return {
    transaction(origin, operations) {
      const values = new Map(origins.get(origin));
      const results = operations.map(operation => {
        const {type, key, value} = operation;
        if (type === 'get') return values.get(key) ?? null;
        if (type === 'set') { values.set(key, value); return null; }
        if (type === 'delete') return values.delete(key);
        values.clear(); return null;
      });
      origins.set(origin, values);
      return results;
    },
    close() { origins.clear(); },
  };
}
function storageOperations(operations) {
  if (!Array.isArray(operations)) throw new TypeError('transaction requires an operation array');
  for (const op of operations) {
    if (!op || !['get','set','delete','clear'].includes(op.type) ||
        (op.type !== 'clear' && typeof op.key !== 'string') ||
        (op.type === 'set' && !Object.hasOwn(op,'value'))) throw new TypeError('invalid storage operation');
  }
  return operations;
}
/** backendFactory runs once per Session. Backends must atomically commit each batch. */
export function hostStoragePlugin({backendFactory = memoryStorageBackend} = {}) {
  return new Plugin('host-storage', bundled('host-storage').source, {}, {}, () => {
    const backend = backendFactory();
    if (!backend || typeof backend.transaction !== 'function') throw new TypeError('backend requires transaction(origin, operations)');
    const queues = new Map();
    return {
      handlers: {
        transaction(context, input) {
          if (context.origin === null) throw new Error('Host storage requires a non-opaque origin');
          const operations = storageOperations(input), origin = context.origin;
          const pending = (queues.get(origin) ?? Promise.resolve()).catch(() => {}).then(() => {
            if (context.cancelled) throw new Error('Storage request cancelled');
            return backend.transaction(origin, operations);
          });
          queues.set(origin, pending);
          pending.finally(() => {if (queues.get(origin) === pending) queues.delete(origin);}).catch(() => {});
          return pending;
        },
      },
      async dispose() { await Promise.allSettled(queues.values()); await backend.close?.(); },
    };
  });
}
