(() => {
  const host = globalThis.__brimpStorageHost;
  const token = {};
  const values = new WeakMap();
  class Lock {
    constructor(...args) {
      const [key, name, mode] = args;
      if (key !== token) throw new TypeError('Illegal constructor');
      values.set(this, { name, mode });
    }
    get name() {
      return values.get(this).name;
    }
    get mode() {
      return values.get(this).mode;
    }
  }
  class LockManager {
    constructor(...args) {
      const [key] = args;
      if (key !== token) throw new TypeError('Illegal constructor');
    }
    request(name, options, callback = undefined) {
      if (arguments.length < 2)
        throw new TypeError('LockManager.request requires a name and callback');
      name = String(name);
      if (name.startsWith('-')) {
        throw new DOMException("Lock names beginning with '-' are reserved", 'NotSupportedError');
      }
      if (typeof options === 'function') {
        callback = options;
        options = {};
      } else {
        options = options == null ? {} : Object(options);
      }
      if (typeof callback !== 'function')
        throw new TypeError('The lock callback must be a function');
      const mode = options.mode === undefined ? 'exclusive' : String(options.mode);
      if (mode !== 'exclusive' && mode !== 'shared') throw new TypeError('Invalid lock mode');
      const ifAvailable = Boolean(options.ifAvailable);
      const steal = Boolean(options.steal);
      if (steal && (ifAvailable || mode !== 'exclusive')) {
        throw new DOMException(
          'A steal request must be exclusive and cannot use ifAvailable',
          'NotSupportedError',
        );
      }
      const signal = options.signal;
      if (signal !== undefined && !(signal instanceof AbortSignal)) {
        throw new TypeError('signal must be an AbortSignal');
      }
      if (signal?.aborted) return Promise.reject(signal.reason);

      return new Promise((resolve, reject) => {
        let held = false,
          finished = false;
        const finish = (settle, value) => {
          if (finished) return;
          finished = true;
          signal?.removeEventListener('abort', abort);
          host({ op: 'lockRelease', id });
          settle(value);
        };
        // Retain the existing compatibility contract: abort can release a held request.
        const abort = () => finish(reject, signal.reason);
        const id = host({ op: 'lockRequest', name, mode, steal, ifAvailable }, (status) => {
          if (finished) return;
          if (status === 'stolen') {
            finish(reject, new DOMException('The lock was stolen', 'AbortError'));
            return;
          }
          held = status === 'granted';
          Promise.resolve()
            .then(() => callback(held ? new Lock(token, name, mode) : null))
            .then(
              (value) => finish(resolve, value),
              (error) => finish(reject, error),
            );
        });
        signal?.addEventListener('abort', abort, { once: true });
      });
    }
    query() {
      return Promise.resolve().then(() => host({ op: 'lockQuery' }));
    }
  }
  const locks = new LockManager(token);
  Object.defineProperty(Navigator.prototype, 'locks', {
    get() {
      return locks;
    },
    configurable: true,
    enumerable: true,
  });
  Object.assign(globalThis, { Lock, LockManager });
})();
