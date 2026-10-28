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
  const userAgent = globalThis.__brimpProfile.identity.user_agent;
  const chromium = /Chrome\/|Chromium\/|Edg\//.test(userAgent);
  global.__brimpConfigureNavigator({
    ...privacy,
    expose_global_privacy_control: privacy.global_privacy_control !== null,
    ...(chromium
      ? {
          vendor: 'Google Inc.',
          app_version: userAgent.replace(/^Mozilla\//, ''),
          device_memory_gb: 8,
        }
      : {}),
    ...options,
  });
  if (chromium) {
    const now = Date.now();
    const chrome = {
      app: {
        isInstalled: false,
        InstallState: {
          DISABLED: 'disabled',
          INSTALLED: 'installed',
          NOT_INSTALLED: 'not_installed',
        },
        RunningState: {
          CANNOT_RUN: 'cannot_run',
          READY_TO_RUN: 'ready_to_run',
          RUNNING: 'running',
        },
        getDetails() {
          return null;
        },
        getIsInstalled() {
          return false;
        },
        runningState() {
          return 'cannot_run';
        },
      },
      runtime: {
        OnInstalledReason: {
          CHROME_UPDATE: 'chrome_update',
          INSTALL: 'install',
          SHARED_MODULE_UPDATE: 'shared_module_update',
          UPDATE: 'update',
        },
        OnRestartRequiredReason: {
          APP_UPDATE: 'app_update',
          OS_UPDATE: 'os_update',
          PERIODIC: 'periodic',
        },
        PlatformArch: {
          ARM: 'arm',
          ARM64: 'arm64',
          MIPS: 'mips',
          MIPS64: 'mips64',
          X86_32: 'x86-32',
          X86_64: 'x86-64',
        },
        PlatformOs: {
          ANDROID: 'android',
          CROS: 'cros',
          LINUX: 'linux',
          MAC: 'mac',
          OPENBSD: 'openbsd',
          WIN: 'win',
        },
        connect() {},
        sendMessage() {},
      },
      csi() {
        return { onloadT: now, startE: now, pageT: performance.now(), tran: 15 };
      },
      loadTimes() {
        return {
          requestTime: now / 1000,
          startLoadTime: now / 1000,
          commitLoadTime: now / 1000,
          finishLoadTime: now / 1000,
          navigationType: 'Other',
          wasNpnNegotiated: true,
          npnNegotiatedProtocol: 'h2',
          wasAlternateProtocolAvailable: false,
          connectionInfo: 'h2',
        };
      },
    };
    Object.defineProperty(global, 'chrome', {
      value: chrome,
      enumerable: false,
      configurable: true,
    });
    if (!('memory' in performance)) {
      Object.defineProperty(performance, 'memory', {
        value: { jsHeapSizeLimit: 4294705152, totalJSHeapSize: 50000000, usedJSHeapSize: 30000000 },
        enumerable: true,
        configurable: true,
      });
    }
  }
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
  // Chromium-only surface: real Safari 27 exposes no navigator.connection.
  if (/Chrome\/|Chromium\/|Edg\//.test(globalThis.__brimpProfile.identity.user_agent))
    defineNavigatorValue('connection', new NetworkInformation(constructToken));
}
