function installTrustedTypes({ global, profile: settings }, options = {}) {
  const _defaults = settings.values(
    'trusted-types',
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
  class TrustedTypePolicyFactory {
    constructor(token) {
      if (token !== constructToken) throw new TypeError('Illegal constructor');
    }
    createPolicy() {}
    getAttributeType() {}
    getPropertyType() {}
    isHTML() {}
    isScript() {}
    isScriptURL() {}
    get defaultPolicy() {
      return null;
    }
    get emptyHTML() {
      return '';
    }
    get emptyScript() {
      return '';
    }
  }
  defineGlobalConstructor('TrustedTypePolicyFactory', TrustedTypePolicyFactory);
  Object.defineProperty(global, 'trustedTypes', {
    value: new TrustedTypePolicyFactory(constructToken),
    enumerable: true,
    configurable: true,
  });
}
