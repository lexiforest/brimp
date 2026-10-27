function installEnvironment({ global, profile: settings }, options = {}) {
  const connection = settings.values(
    'connection',
    {
      type: 'wifi',
      effective_type: '4g',
      rtt_ms: 50,
      downlink_mbps: 10.0,
      save_data: false,
    },
    {
      version: 1,
      capture: async ({ read }) => {
        const c = navigator.connection;
        return {
          type: read(() => c?.type),
          rtt_ms: read(() => c?.rtt),
          downlink_mbps: read(() => c?.downlink),
          effective_type: read(() => c?.effectiveType),
          save_data: read(() => c?.saveData),
        };
      },
    },
  );
  const privacy = settings.values(
    'privacy',
    {
      device_memory_gb: null,
      global_privacy_control: null,
    },
    {
      version: 1,
      fields: {
        device_memory_gb: {
          type: 'number',
          nullable: true,
        },
        global_privacy_control: {
          type: 'boolean',
          nullable: true,
        },
      },
      capture: ({ read }) => ({
        device_memory_gb: read(() => navigator.deviceMemory),
        global_privacy_control: read(() => navigator.globalPrivacyControl),
      }),
    },
  );
  const navigatorValues = {
    connection_type: connection.type,
    connection_rtt_ms: connection.rtt_ms,
    connection_downlink_mbps: connection.downlink_mbps,
    connection_effective_type: connection.effective_type,
    connection_save_data: connection.save_data,
    ...options,
  };
  const constructToken = Symbol('browser interface construction');
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
  global.__brimpConfigureNavigator({
    ...privacy,
    expose_global_privacy_control: privacy.global_privacy_control !== null,
    ...options,
  });
  class NetworkInformation extends EventTarget {
    constructor(...args) {
      super();
      if (args[0] !== constructToken) throw new TypeError('Illegal constructor');
    }
    get type() {
      return navigatorValues.connection_type;
    }
    get rtt() {
      return navigatorValues.connection_rtt_ms;
    }
    get downlink() {
      return Number(navigatorValues.connection_downlink_mbps);
    }
    get effectiveType() {
      return navigatorValues.connection_effective_type;
    }
    get saveData() {
      return navigatorValues.connection_save_data;
    }
  }
  defineNavigatorValue('connection', new NetworkInformation(constructToken));
}
