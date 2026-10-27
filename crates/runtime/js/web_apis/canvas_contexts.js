(() => {
  const factories = new Map();
  const contexts = new WeakMap();
  Object.defineProperty(HTMLCanvasElement.prototype, 'getContext', {
    configurable: true,
    writable: true,
    value: function getContext(type, options) {
      type = String(type).toLowerCase();
      const current = contexts.get(this);
      if (current) return current.type === type ? current.context : null;
      const factory = factories.get(type);
      if (!factory) return null;
      const context = factory.call(this, options);
      if (context != null) contexts.set(this, { type, context });
      return context ?? null;
    },
  });
  Object.defineProperty(globalThis, '__brimpCanvasBackends', {
    configurable: true,
    value: Object.freeze({
      register(type, factory) {
        type = String(type).toLowerCase();
        if (typeof factory !== 'function') throw new TypeError('Canvas backend must be callable');
        factories.set(type, factory);
        return () => {
          if (factories.get(type) === factory) factories.delete(type);
        };
      },
    }),
  });
})();
