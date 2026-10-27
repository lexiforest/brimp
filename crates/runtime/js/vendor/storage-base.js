export default ({ global, host }) => {
  const __storageData = new WeakMap();
  const __storageConstructorToken = {};

  function __storageMap(receiver) {
    const map = __storageData.get(receiver);
    if (map === undefined) throw new TypeError('Storage method called on an incompatible receiver');
    return map;
  }

  function __storeStorageValue(receiver, key, value) {
    __storageMap(receiver).set(key, value);
  }
  function __deleteStorageValue(receiver, key) {
    __storageMap(receiver).delete(key);
  }

  function __toUnsignedLong(value) {
    let number = Number(value);
    if (!Number.isFinite(number) || number === 0) return 0;
    number = Math.trunc(number) % 0x100000000;
    return number < 0 ? number + 0x100000000 : number;
  }

  const __storageProxyHandler = {
    get(target, property, receiver) {
      if (typeof property !== 'string' || Reflect.has(target, property)) {
        return Reflect.get(target, property, receiver);
      }
      return __storageMap(target).get(property);
    },
    set(target, property, value, receiver) {
      if (typeof property !== 'string') return Reflect.set(target, property, value, receiver);
      const converted = String(value);
      __storeStorageValue(target, property, converted);
      return true;
    },
    defineProperty(target, property, descriptor) {
      if (typeof property !== 'string') return Reflect.defineProperty(target, property, descriptor);
      const converted = String(descriptor.value);
      __storeStorageValue(target, property, converted);
      return true;
    },
    deleteProperty(target, property) {
      if (typeof property !== 'string') return Reflect.deleteProperty(target, property);
      __deleteStorageValue(target, property);
      return true;
    },
    has(target, property) {
      return (
        Reflect.has(target, property) ||
        (typeof property === 'string' && __storageMap(target).has(property))
      );
    },
    ownKeys(target) {
      const named = [];
      for (const key of __storageMap(target).keys()) {
        if (!Reflect.has(target, key)) named.push(key);
      }
      return named.concat(Reflect.ownKeys(target).filter((key) => typeof key === 'symbol'));
    },
    getOwnPropertyDescriptor(target, property) {
      if (typeof property !== 'string') return Reflect.getOwnPropertyDescriptor(target, property);
      const map = __storageMap(target);
      if (Reflect.has(target, property) || !map.has(property)) return undefined;
      return {
        value: map.get(property),
        writable: true,
        enumerable: true,
        configurable: true,
      };
    },
  };

  class Storage {
    constructor(token, namespace) {
      if (token !== __storageConstructorToken) throw new TypeError('Illegal constructor');
      const map = {
        get size() {
          return this.keys().length;
        },
        keys() {
          return host({ op: 'list', namespace }).map((key) => JSON.parse(key));
        },
        get(key) {
          const value = host({ op: 'get', namespace, key: JSON.stringify(key) });
          return value === null ? undefined : JSON.parse(value);
        },
        has(key) {
          return this.get(key) !== undefined;
        },
        set(key, value) {
          host({
            op: 'set',
            namespace,
            key: JSON.stringify(key),
            value: JSON.stringify(value),
          });
        },
        delete(key) {
          host({ op: 'delete', namespace, key: JSON.stringify(key) });
        },
        clear() {
          host({ op: 'clear', namespace });
        },
      };
      const proxy = new Proxy(this, __storageProxyHandler);
      __storageData.set(this, map);
      __storageData.set(proxy, map);
      return proxy;
    }
    get length() {
      return __storageMap(this).size;
    }
    key(index) {
      if (arguments.length === 0) throw new TypeError('Storage.key requires an index');
      return Array.from(__storageMap(this).keys())[__toUnsignedLong(index)] ?? null;
    }
    getItem(key) {
      if (arguments.length === 0) throw new TypeError('Storage.getItem requires a key');
      key = String(key);
      return __storageMap(this).has(key) ? __storageMap(this).get(key) : null;
    }
    setItem(key, value) {
      if (arguments.length < 2) throw new TypeError('Storage.setItem requires a key and value');
      key = String(key);
      value = String(value);
      __storeStorageValue(this, key, value);
    }
    removeItem(key) {
      if (arguments.length === 0) throw new TypeError('Storage.removeItem requires a key');
      __deleteStorageValue(this, String(key));
    }
    clear() {
      __storageMap(this).clear();
    }
    get [Symbol.toStringTag]() {
      return 'Storage';
    }
  }

  globalThis.Storage = Storage;
  for (const [name, namespace] of [
    ['localStorage', 'local'],
    ['sessionStorage', 'session'],
  ]) {
    let storage;
    Object.defineProperty(globalThis, name, {
      configurable: true,
      enumerable: true,
      get() {
        host({ op: 'list', namespace });
        return (storage ??= new Storage(__storageConstructorToken, namespace));
      },
    });
  }

  host({ op: 'subscribe' }, (serialized) => {
    const event = JSON.parse(serialized);
    for (const key of ['key', 'oldValue', 'newValue'])
      if (event[key] !== null) event[key] = JSON.parse(event[key]);
    event.storageArea = global.localStorage;
    global.dispatchEvent(new StorageEvent('storage', event));
  });

  function call(op, namespace, key, value) {
    const result = host({ op, namespace, key, value });
    return op === 'list' || op === 'estimate' ? JSON.stringify(result) : result;
  }
  const load = (namespace, key, fallback) => {
    const value = call('get', namespace, String(key));
    return value === null ? fallback : JSON.parse(value);
  };
  const save = (namespace, key, value) => {
    try {
      call('set', namespace, String(key), JSON.stringify(value));
    } catch (error) {
      if (String(error).includes('QuotaExceededError')) {
        throw new DOMException('The quota has been exceeded', 'QuotaExceededError');
      }
      throw error;
    }
  };

  class Cache {
    constructor(name) {
      this.__name = name;
    }
    __load() {
      return load('cache', this.__name, {});
    }
    match(request) {
      const key = new Request(request).url;
      const entry = this.__load()[key];
      return Promise.resolve(entry ? new Response(entry.body, entry) : undefined);
    }
    matchAll(request = undefined) {
      const entries = this.__load();
      const values =
        request === undefined
          ? Object.values(entries)
          : [entries[new Request(request).url]].filter(Boolean);
      return Promise.resolve(values.map((entry) => new Response(entry.body, entry)));
    }
    put(request, response) {
      const key = new Request(request).url;
      if (!(response instanceof Response))
        return Promise.reject(new TypeError('response must be a Response'));
      return response
        .clone()
        .arrayBuffer()
        .then((buffer) => {
          const body = Array.from(new Uint8Array(buffer));
          const entry = {
            body,
            status: response.status,
            statusText: response.statusText,
            headers: [...response.headers],
            url: response.url,
          };
          host({
            op: 'cachePut',
            namespace: 'cache',
            key: this.__name,
            url: key,
            value: JSON.stringify(entry),
          });
        });
    }
    add(request) {
      return fetch(request).then((response) => this.put(request, response));
    }
    addAll(requests) {
      return Promise.all(Array.from(requests, (request) => this.add(request))).then(
        () => undefined,
      );
    }
    delete(request) {
      const key = new Request(request).url;
      return Promise.resolve(
        host({ op: 'cacheDelete', namespace: 'cache', key: this.__name, url: key }),
      );
    }
    keys() {
      return Promise.resolve(Object.keys(this.__load()).map((url) => new Request(url)));
    }
  }

  class CacheStorage {
    open(name) {
      name = String(name);
      if (call('get', 'cache', name) === null) save('cache', name, {});
      return Promise.resolve(new Cache(name));
    }
    has(name) {
      return Promise.resolve(call('get', 'cache', String(name)) !== null);
    }
    delete(name) {
      name = String(name);
      const existed = call('get', 'cache', name) !== null;
      call('delete', 'cache', name);
      return Promise.resolve(existed);
    }
    keys() {
      return Promise.resolve(JSON.parse(call('list', 'cache')));
    }
    match(request) {
      return this.keys().then(async (names) => {
        for (const name of names) {
          const response = await new Cache(name).match(request);
          if (response) return response;
        }
        return undefined;
      });
    }
  }

  class StorageManager {
    estimate() {
      return Promise.resolve(JSON.parse(call('estimate')));
    }
    persist() {
      return Promise.resolve(host({ op: 'persistent' }));
    }
    persisted() {
      return Promise.resolve(host({ op: 'persistent' }));
    }
    // Legacy placeholder: no origin-private filesystem backend.
    getDirectory() {}
  }

  globalThis.Cache = Cache;
  globalThis.CacheStorage = CacheStorage;
  globalThis.caches = new CacheStorage();
  globalThis.StorageManager = StorageManager;
  Object.defineProperty(Navigator.prototype, 'storage', {
    value: new StorageManager(),
    enumerable: true,
    configurable: true,
  });

  class CookieStore extends EventTarget {
    constructor(token) {
      super();
      if (token !== __storageConstructorToken) throw new TypeError('Illegal constructor');
    }
    async getAll(options = {}) {
      const name = typeof options === 'string' ? options : options?.name;
      return host({ op: 'cookieGet' })
        .filter((cookie) => name === undefined || cookie.name === String(name))
        .map((cookie) => ({
          name: cookie.name,
          value: cookie.value,
          domain: cookie.host_only ? null : cookie.domain.replace(/^\./, ''),
          path: cookie.path,
          expires: cookie.expires === null ? null : cookie.expires * 1000,
          secure: cookie.secure,
          sameSite: (cookie.same_site ?? 'strict').toLowerCase(),
          partitioned: false,
        }));
    }
    async get(options) {
      return (await this.getAll(options))[0] ?? null;
    }
    async set(name, value) {
      const options = name !== null && typeof name === 'object' ? name : { name, value };
      if (options.name == null || options.value == null) return;
      const n = String(options.name),
        v = String(options.value);
      if (/[;=\r\n]/.test(n) || /[;\r\n]/.test(v)) return;
      let header = `${n}=${v}; Path=${options.path ?? '/'}; SameSite=${options.sameSite ?? 'Strict'}`;
      if (options.domain) header += `; Domain=${options.domain}`;
      if (options.expires != null) header += `; Expires=${new Date(options.expires).toUTCString()}`;
      if (options.secure ?? location.protocol === 'https:') header += '; Secure';
      host({ op: 'cookieSet', header });
    }
    async delete(options) {
      options = typeof options === 'string' ? { name: options } : options;
      return this.set({ ...options, value: '', expires: 0 });
    }
  }
  const handlers = new WeakMap();
  Object.defineProperty(CookieStore.prototype, 'onchange', {
    configurable: true,
    enumerable: true,
    get() {
      return handlers.get(this) ?? null;
    },
    set(value) {
      const old = handlers.get(this);
      if (old) this.removeEventListener('change', old);
      value = typeof value === 'function' ? value : null;
      handlers.set(this, value);
      if (value) this.addEventListener('change', value);
    },
  });
  Object.assign(global, { CookieStore, cookieStore: new CookieStore(__storageConstructorToken) });
  // Preserve the synthetic WebSQL payload contract; this does not execute SQL.
  class SQLTransaction {
    constructor() {
      throw new TypeError('Illegal constructor');
    }
    executeSql(sql, args, success) {
      if (typeof success === 'function')
        return success(this, { insertId: '', rows: { value: '0' }, rowsAffected: '' });
    }
  }
  Object.assign(global, { SQLTransaction });
  Object.defineProperty(global, 'openDatabase', { value: undefined, configurable: true });
};
