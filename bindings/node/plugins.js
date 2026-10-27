import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { pathToFileURL } from 'node:url';
import { AsyncLocalStorage } from 'node:async_hooks';

export const callbackContext = new AsyncLocalStorage();

import { encode, decode, HostFunction } from './values.js';
import { JSHandle } from './index.js';
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
  function(name) { return new HostFunction(name); }
  _fromReference(token) { return new JSHandle(this.#call.bindReference(JSON.stringify(token)), this.#call.pageId); }
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
          const context = new HostContext(call);
          const result = await handler(context, ...JSON.parse(call.arguments).map(arg => decode(arg, token => context._fromReference(token))));
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
export const canvasPlugin = (options = {}) => bundled('canvas', options);
export const environmentPlugin = (options = {}) => bundled('environment', options);
export const permissionsPlugin = (options = {}) => bundled('permissions', options);
export const devicesPlugin = (options = {}) => bundled('devices', options);
export const mediaPlugin = (options = {}) => bundled('media', options);
export const workersPlugin = (options = {}) => bundled('workers', options);
export const trustedTypesPlugin = (options = {}) => bundled('trusted-types', options);
export const extractionPlugin = () => bundled('extraction');

export const rtcPlugin = () => bundled('rtc');
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

/** Default browser subsystems. Each call returns fresh descriptors. */
export function browserPlugins() {
  return ['environment', 'permissions', 'devices', 'media', 'workers', 'trusted-types',
    'canvas', 'graphics', 'audio', 'rtc'].map(name => bundled(name, {}));
}
