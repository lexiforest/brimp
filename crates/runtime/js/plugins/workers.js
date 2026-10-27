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
  class Worker extends EventTarget {
    constructor(url) {
      super();
      this.onmessage = this.onerror = null;
    }
    postMessage() {}
    terminate() {}
  }
  // No inline-code execution, fetch, thread or worker scope is started here.
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
