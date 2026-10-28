function installWorkers({ global, profile: settings }, options = {}) {
  const _defaults = settings.values(
    'workers',
    {},
    {
      version: 1,
      notes:
        'Compatibility interfaces have no configurable values. Browser absence does not remove their placeholders.',
    },
  );
  const constructToken = Symbol('browser interface construction');
  const defineGlobalConstructor = (name, constructor) => {
    if (!Object.hasOwn(constructor.prototype, Symbol.toStringTag)) {
      Object.defineProperty(constructor.prototype, Symbol.toStringTag, {
        value: name,
        configurable: true,
      });
    }
    Object.defineProperty(global, name, {
      value: constructor,
      writable: true,
      configurable: true,
    });
  };
  const defineNavigatorValue = (name, value) => {
    const getter = function () {
      return value;
    };
    Object.defineProperty(getter, 'name', { value: `get ${name}`, configurable: true });
    Object.defineProperty(Navigator.prototype, name, {
      get: getter,
      enumerable: true,
      configurable: true,
    });
  };
  // Classic workers run in this realm behind a worker-shaped scope object. The
  // source comes from the object-URL registry (or a data: URL); no thread, no DOM
  // and no isolation boundary is created.
  const objectUrls = global.__brimpObjectUrls ?? null;
  const clone = (value) => {
    try {
      return structuredClone(value);
    } catch (error) {
      throw new DOMException('Worker message could not be cloned', 'DataCloneError');
    }
  };
  const workerLocation = (href) => {
    try {
      const parsed = new URL(href);
      return Object.freeze({
        href: parsed.href,
        origin: parsed.origin,
        protocol: parsed.protocol,
        host: parsed.host,
        hostname: parsed.hostname,
        port: parsed.port,
        pathname: parsed.pathname,
        search: parsed.search,
        hash: parsed.hash,
        toString() {
          return parsed.href;
        },
      });
    } catch (error) {
      return Object.freeze({ href, toString: () => href });
    }
  };
  class Worker extends EventTarget {
    #url;
    #name;
    #scope;
    #terminated = false;
    #queue = [];
    constructor(url, options = {}) {
      super();
      this.onmessage = this.onerror = this.onmessageerror = null;
      if (arguments.length === 0) throw new TypeError('1 argument required');
      this.#url = String(url);
      this.#name = options && options.name !== undefined ? String(options.name) : '';
      const scope = new EventTarget();
      this.#scope = scope;
      Object.defineProperties(scope, {
        name: { value: this.#name, writable: true, configurable: true },
        location: { value: workerLocation(this.#url), configurable: true },
        postMessage: {
          value: (data) => {
            if (this.#terminated) return;
            const cloned = clone(data);
            setTimeout(() => {
              this.dispatchEvent(new MessageEvent('message', { data: cloned }));
            }, 0);
          },
          configurable: true,
        },
        close: {
          value: () => this.terminate(),
          configurable: true,
        },
        importScripts: {
          value: () => {
            throw new DOMException('importScripts is not supported', 'NetworkError');
          },
          configurable: true,
        },
      });
      // Messages posted before the source is ready wait in order.
      this.#queue = [];
      const sourcePromise =
        objectUrls && typeof objectUrls.get === 'function'
          ? objectUrls.get(this.#url)
          : Promise.resolve(null);
      Promise.resolve(sourcePromise).then((blob) => {
        if (this.#terminated) return;
        if (blob === null || typeof blob.text !== 'function') {
          setTimeout(() => this.#fail(new Error(`Cannot load worker script ${this.#url}`)), 0);
          return;
        }
        blob.text().then((source) => {
          if (!this.#terminated) setTimeout(() => this.#start(source), 0);
        });
      });
    }
    postMessage(data) {
      if (this.#terminated) return;
      const cloned = clone(data);
      this.#queue.push(cloned);
      this.#drain();
    }
    terminate() {
      this.#terminated = true;
      this.#queue.length = 0;
    }
    #drain() {
      if (this.#terminated) return;
      const scope = this.#scope;
      while (this.#queue.length !== 0) {
        const data = this.#queue.shift();
        setTimeout(() => {
          if (!this.#terminated) scope.dispatchEvent(new MessageEvent('message', { data }));
        }, 0);
      }
    }
    #fail(error) {
      const event = new Event('error');
      Object.defineProperties(event, {
        message: { value: String(error && error.message) },
        error: { value: error },
      });
      this.dispatchEvent(event);
    }
    #start(source) {
      if (this.#terminated) return;
      const scope = this.#scope;
      try {
        // The worker script body sees `self`; other globals resolve to this realm.
        const run = new Function('self', 'postMessage', 'close', 'location', 'name', source);
        run(scope, scope.postMessage, scope.close, scope.location, this.#name);
      } catch (error) {
        this.#fail(error);
        return;
      }
      this.#drain();
    }
  }
  defineGlobalConstructor('Worker', Worker);
  class ServiceWorkerContainer extends EventTarget {
    #ready = new Promise(() => {});
    constructor(token) {
      super();
      if (token !== constructToken) throw new TypeError('Illegal constructor');
      this.oncontrollerchange = this.onmessage = this.onmessageerror = null;
    }
    get controller() {
      return null;
    }
    get ready() {
      return this.#ready;
    }
    getRegistration() {}
    getRegistrations() {}
    register() {}
    startMessages() {}
  }
  defineGlobalConstructor('ServiceWorkerContainer', ServiceWorkerContainer);
  defineNavigatorValue('serviceWorker', new ServiceWorkerContainer(constructToken));
}
