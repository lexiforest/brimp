function installProfile(profile) {
  const identity = profile.identity;
  const state = {
    standalone: false,
    userAgent: identity.user_agent,
    appCodeName: identity.app_code_name,
    appName: identity.app_name,
    appVersion: identity.app_version,
    vendor: identity.vendor,
    vendorSub: identity.vendor_sub,
    product: identity.product,
    productSub: identity.product_sub,
    platform: identity.platform,
    language: identity.locale,
    languages: Object.freeze(identity.languages.slice()),
    hardwareConcurrency: profile.device.hardware_concurrency,
    maxTouchPoints: profile.device.max_touch_points,
    onLine: identity.onLine,
    cookieEnabled: identity.cookieEnabled,
    pdfViewerEnabled: profile.pdfViewerEnabled,
  };
  const defaults = { ...state };
  for (const name of Object.keys(state)) {
    const descriptor = Object.getOwnPropertyDescriptor(
      {
        get [name]() {
          return state[name];
        },
      },
      name,
    );
    Object.defineProperty(Navigator.prototype, name, {
      get: descriptor.get,
      enumerable: true,
      configurable: true,
    });
  }
  const defineValue = (target, name, value, enumerable = true) =>
    Object.defineProperty(target, name, { value, enumerable, configurable: true });
  const defineGlobalConstructor = (name, value) =>
    Object.defineProperty(globalThis, name, { value, writable: true, configurable: true });
  const defineNavigatorValue = (name, value) => {
    const descriptor = Object.getOwnPropertyDescriptor(
      {
        get [name]() {
          return value;
        },
      },
      name,
    );
    Object.defineProperty(Navigator.prototype, name, {
      get: descriptor.get,
      enumerable: true,
      configurable: true,
    });
  };
  class AudioSession {
    constructor() {
      throw new TypeError('Illegal constructor');
    }
    get type() {
      return 'auto';
    }
  }
  defineNavigatorValue('audioSession', Object.create(AudioSession.prototype));
  defineGlobalConstructor('AudioSession', AudioSession);
  defineNavigatorValue('webdriver', identity.webdriver);
  if (identity.doNotTrack !== null) defineNavigatorValue('doNotTrack', identity.doNotTrack);
  const arrayData = new WeakMap();
  class MimeType {
    constructor() {
      throw new TypeError('Illegal constructor');
    }
    get type() {
      return arrayData.get(this).type;
    }
    get suffixes() {
      return arrayData.get(this).suffixes;
    }
    get description() {
      return arrayData.get(this).description;
    }
    get enabledPlugin() {
      return arrayData.get(this).enabledPlugin;
    }
  }
  class Plugin {
    constructor() {
      throw new TypeError('Illegal constructor');
    }
    get name() {
      return arrayData.get(this).name;
    }
    get filename() {
      return arrayData.get(this).filename;
    }
    get description() {
      return arrayData.get(this).description;
    }
    get length() {
      return arrayData.get(this).items.length;
    }
    item(index) {
      return arrayData.get(this).items[Number(index)] ?? null;
    }
    namedItem(name) {
      return arrayData.get(this).names[String(name)] ?? null;
    }
    [Symbol.iterator]() {
      return arrayData.get(this).items[Symbol.iterator]();
    }
  }
  class PluginArray {
    constructor() {
      throw new TypeError('Illegal constructor');
    }
    get length() {
      return arrayData.get(this).items.length;
    }
    item(index) {
      return arrayData.get(this).items[Number(index)] ?? null;
    }
    namedItem(name) {
      return arrayData.get(this).names[String(name)] ?? null;
    }
    refresh() {}
    [Symbol.iterator]() {
      return arrayData.get(this).items[Symbol.iterator]();
    }
  }
  class MimeTypeArray {
    constructor() {
      throw new TypeError('Illegal constructor');
    }
    get length() {
      return arrayData.get(this).items.length;
    }
    item(index) {
      return arrayData.get(this).items[Number(index)] ?? null;
    }
    namedItem(name) {
      return arrayData.get(this).names[String(name)] ?? null;
    }
    [Symbol.iterator]() {
      return arrayData.get(this).items[Symbol.iterator]();
    }
  }
  const makeArrayLike = (constructor, items, names) => {
    const value = Object.create(constructor.prototype);
    arrayData.set(value, { items, names });
    items.forEach((item, index) => defineValue(value, index, item));
    for (const [name, item] of Object.entries(names)) {
      if (!(name in value)) defineValue(value, name, item, false);
    }
    return value;
  };
  for (const constructor of [MimeType, Plugin, PluginArray, MimeTypeArray]) {
    defineGlobalConstructor(constructor.name, constructor);
  }
  function installPluginCollections(pdfViewerEnabled) {
    const mimeByType = Object.create(null);
    const pluginByName = Object.create(null);
    const defaultPdfPlugin = (name) => ({
      name,
      filename: 'internal-pdf-viewer',
      description: 'Portable Document Format',
      mime_types: ['application/pdf', 'text/pdf'].map((type) => ({
        type,
        suffixes: 'pdf',
        description: 'Portable Document Format',
      })),
    });
    const pluginConfigs = profile?.plugins?.length
      ? profile.plugins.map((plugin) => ({
          name: plugin.name,
          filename: plugin.filename,
          description: plugin.description,
          mime_types: plugin.mime_types,
        }))
      : pdfViewerEnabled
        ? [
            'PDF Viewer',
            'Chrome PDF Viewer',
            'Chromium PDF Viewer',
            'Microsoft Edge PDF Viewer',
            'WebKit built-in PDF',
          ].map(defaultPdfPlugin)
        : [];
    const pluginEntries = pluginConfigs.map((pluginConfig) => {
      const plugin = Object.create(Plugin.prototype);
      const mimeNames = Object.create(null);
      const mimeEntries = pluginConfig.mime_types.map((mime) => {
        const value = Object.create(MimeType.prototype);
        arrayData.set(value, { ...mime, enabledPlugin: plugin });
        mimeNames[mime.type] = value;
        if (!(mime.type in mimeByType)) mimeByType[mime.type] = value;
        return value;
      });
      arrayData.set(plugin, {
        name: pluginConfig.name,
        filename: pluginConfig.filename,
        description: pluginConfig.description,
        items: mimeEntries,
        names: mimeNames,
      });
      mimeEntries.forEach((item, index) => defineValue(plugin, index, item));
      for (const [name, item] of Object.entries(mimeNames)) defineValue(plugin, name, item, false);
      pluginByName[pluginConfig.name] = plugin;
      return plugin;
    });
    const plugins = makeArrayLike(PluginArray, pluginEntries, pluginByName);
    const mimeTypes = makeArrayLike(MimeTypeArray, Object.values(mimeByType), mimeByType);
    defineNavigatorValue('plugins', plugins);
    defineNavigatorValue('mimeTypes', mimeTypes);
  }
  installPluginCollections(state.pdfViewerEnabled);
  // Chromium exposes Client Hints; WebKit and Gecko expose nothing at all, so the
  // surface follows the profile identity instead of being unconditional.
  const userAgentDataState = new WeakMap();
  class NavigatorUAData {
    constructor() {
      throw new TypeError('Illegal constructor');
    }
    get brands() {
      return userAgentDataState.get(this).brands;
    }
    get mobile() {
      return userAgentDataState.get(this).mobile;
    }
    get platform() {
      return userAgentDataState.get(this).platform;
    }
    // Derived from the user agent: the real build number, CPU architecture,
    // platform version and model are not observable, so they stay unspecified.
    getHighEntropyValues(hints) {
      if (arguments.length === 0) return Promise.reject(new TypeError('1 argument required'));
      const data = userAgentDataState.get(this);
      const values = {
        brands: data.brands,
        mobile: data.mobile,
        platform: data.platform,
        uaFullVersion: data.fullVersion,
        fullVersionList: data.brands.map((brand) => ({
          brand: brand.brand,
          version: brand.brand === 'Not:A-Brand' ? '8.0.0.0' : data.fullVersion,
        })),
        architecture: 'arm',
        bitness: '64',
        model: '',
        platformVersion: '15.0.0',
        wow64: false,
        formFactors: Object.freeze(['Desktop']),
      };
      const selected = {};
      for (const hint of Array.from(hints ?? [], String)) {
        if (hint in values) selected[hint] = values[hint];
      }
      return Promise.resolve(selected);
    }
    toJSON() {
      const data = userAgentDataState.get(this);
      return { brands: data.brands, mobile: data.mobile, platform: data.platform };
    }
  }
  function clientHintValues(userAgent, platform) {
    const version = /Chrome\/(\d+)/.exec(userAgent)?.[1];
    if (version === undefined) return null;
    const mobile = userAgent.includes('Mobile') || platform === 'Linux armv81';
    const hintPlatform =
      platform === 'MacIntel'
        ? 'macOS'
        : platform === 'Win32'
          ? 'Windows'
          : platform === 'iPhone' || platform === 'iPad'
            ? 'iOS'
            : mobile
              ? 'Android'
              : 'Linux';
    return {
      brands: Object.freeze(
        [
          { brand: 'Not:A-Brand', version: '8' },
          { brand: 'Chromium', version },
          { brand: 'Google Chrome', version },
        ].map((brand) => Object.freeze(brand)),
      ),
      mobile,
      platform: hintPlatform,
      fullVersion: `${version}.0.0.0`,
    };
  }
  const initialClientHints = clientHintValues(state.userAgent, state.platform);
  if (initialClientHints !== null) {
    const userAgentData = Object.create(NavigatorUAData.prototype);
    userAgentDataState.set(userAgentData, initialClientHints);
    defineNavigatorValue('userAgentData', userAgentData);
    defineGlobalConstructor('NavigatorUAData', NavigatorUAData);
  }
  Object.defineProperty(globalThis, '__brimpConfigureNavigator', {
    configurable: true,
    value(options) {
      for (const [option, name] of Object.entries({
        app_code_name: 'appCodeName',
        app_name: 'appName',
        app_version: 'appVersion',
        hardware_concurrency: 'hardwareConcurrency',
        max_touch_points: 'maxTouchPoints',
        platform: 'platform',
        vendor: 'vendor',
        product_sub: 'productSub',
        pdf_viewer_enabled: 'pdfViewerEnabled',
      })) {
        if (Object.hasOwn(options, option)) state[name] = options[option];
      }
      if (Object.hasOwn(options, 'pdf_viewer_enabled'))
        installPluginCollections(state.pdfViewerEnabled);
      if (options.device_memory_gb != null)
        defineNavigatorValue('deviceMemory', options.device_memory_gb);
      if (options.expose_global_privacy_control)
        defineNavigatorValue('globalPrivacyControl', options.global_privacy_control ?? false);
    },
  });

  globalThis.__brimpInstallWindowEnvironment(profile);
  globalThis.__brimpInstallIntlProfile(identity);
  globalThis.__brimpInstallMediaQueries(profile);
  globalThis.__brimpInstallFonts(profile);
  globalThis.__brimpInstallPerformance(profile);
  globalThis.__brimpInstallScheduling(identity);
  // Consumed by bundled plugins during installation, then removed by the
  // WebIDL finalizer so page scripts cannot address it.
  Object.defineProperty(globalThis, '__brimpProfile', {
    value: Object.freeze(profile),
    writable: false,
    enumerable: false,
    configurable: true,
  });
  return (serialized) => {
    const overrides = JSON.parse(serialized);
    for (const [name, value] of Object.entries(overrides)) {
      if (!(name in state)) continue;
      const previous = state[name];
      state[name] =
        value === null
          ? defaults[name]
          : name === 'languages'
            ? Object.freeze(Array.from(value, String))
            : name === 'onLine'
              ? Boolean(value)
              : String(value);
      if (name === 'onLine' && previous !== state[name]) {
        globalThis.dispatchEvent(new Event(state[name] ? 'online' : 'offline'));
      }
    }
    return true;
  };
}
