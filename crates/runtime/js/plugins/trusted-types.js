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
  const policies = new Map();
  const trustedValue = (rules, kind, value) => {
    const create = rules?.[kind];
    const result = typeof create === 'function' ? create(String(value)) : String(value);
    return result == null ? '' : String(result);
  };
  class TrustedTypePolicy {
    constructor(name, rules) {
      this.__name = name;
      this.__rules = rules;
    }
    get name() {
      return this.__name;
    }
    createHTML(value) {
      return trustedValue(this.__rules, 'createHTML', value);
    }
    createScript(value) {
      return trustedValue(this.__rules, 'createScript', value);
    }
    createScriptURL(value) {
      return trustedValue(this.__rules, 'createScriptURL', value);
    }
  }
  class TrustedTypePolicyFactory {
    constructor(token) {
      if (token !== constructToken) throw new TypeError('Illegal constructor');
    }
    createPolicy(name, rules = {}) {
      name = String(name);
      if (policies.has(name)) throw new TypeError(`Policy ${name} already exists`);
      const policy = new TrustedTypePolicy(name, rules ?? {});
      policies.set(name, policy);
      return policy;
    }
    getAttributeType() {}
    getPropertyType() {}
    isHTML(value) {
      return typeof value === 'string';
    }
    isScript(value) {
      return typeof value === 'string';
    }
    isScriptURL(value) {
      return typeof value === 'string';
    }
    get defaultPolicy() {
      return policies.get('default') ?? null;
    }
    get emptyHTML() {
      return '';
    }
    get emptyScript() {
      return '';
    }
  }
  defineGlobalConstructor('TrustedTypePolicy', TrustedTypePolicy);
  defineGlobalConstructor('TrustedTypePolicyFactory', TrustedTypePolicyFactory);
  Object.defineProperty(global, 'trustedTypes', {
    value: new TrustedTypePolicyFactory(constructToken),
    enumerable: true,
    configurable: true,
  });
}
