function installDevices({ global, profile: settings }, options = {}) {
  const deviceProfile = settings.values(
    'devices',
    {
      bluetooth_enabled: true,
      bluetooth_available: false,
    },
    {
      version: 1,
      notes:
        'bluetooth_enabled controls synthetic exposure and is not inferred from browser absence.',
      capture: async ({ read }) => ({
        bluetooth_available: await read(async () =>
          navigator.bluetooth ? await navigator.bluetooth.getAvailability() : undefined,
        ),
      }),
    },
  );
  const navigatorValues = { ...deviceProfile, ...options };
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
  if (navigatorValues.bluetooth_enabled) {
    defineNavigatorValue(
      'bluetooth',
      Object.freeze({
        getAvailability: () => Promise.resolve(navigatorValues.bluetooth_available),
      }),
    );
  } else {
    delete Navigator.prototype.bluetooth;
  }

  class Clipboard extends EventTarget {
    constructor(token) {
      super();
      if (token !== constructToken) throw new TypeError('Illegal constructor');
    }
    read() {}
    readText() {}
    write() {}
    writeText() {}
  }
  class Geolocation {
    constructor(token) {
      if (token !== constructToken) throw new TypeError('Illegal constructor');
    }
    clearWatch() {}
    getCurrentPosition() {}
    watchPosition() {}
  }
  class WakeLock {
    constructor(token) {
      if (token !== constructToken) throw new TypeError('Illegal constructor');
    }
    request() {}
  }
  defineGlobalConstructor('Clipboard', Clipboard);
  defineNavigatorValue('clipboard', new Clipboard(constructToken));
  defineGlobalConstructor('Geolocation', Geolocation);
  defineNavigatorValue('geolocation', new Geolocation(constructToken));
  defineGlobalConstructor('WakeLock', WakeLock);
  defineNavigatorValue('wakeLock', new WakeLock(constructToken));
  for (const name of ['canShare', 'getGamepads', 'share']) {
    Object.defineProperty(Navigator.prototype, name, {
      value: { [name]() {} }[name],
      writable: true,
      configurable: true,
      enumerable: true,
    });
  }
}
