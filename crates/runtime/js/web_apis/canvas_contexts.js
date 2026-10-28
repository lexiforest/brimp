(() => {
  const factories = new Map();
  const contexts = new WeakMap();
  const getContext = (canvas, type, options) => {
    type = String(type).toLowerCase();
    const current = contexts.get(canvas);
    if (current) return current.type === type ? current.context : null;
    const factory = factories.get(type);
    if (!factory) return null;
    const context = factory.call(canvas, options);
    if (context != null) contexts.set(canvas, { type, context });
    return context ?? null;
  };
  Object.defineProperty(HTMLCanvasElement.prototype, 'getContext', {
    configurable: true,
    writable: true,
    value: function getContext_(type, options) {
      return getContext(this, type, options);
    },
  });
  // Offscreen canvases reuse the same backends through a detached element, so no
  // separate renderer is needed for script-only canvases.
  class OffscreenCanvas {
    #element;
    constructor(width, height) {
      this.#element = document.createElement('canvas');
      this.#element.width = width === undefined ? 0 : width;
      this.#element.height = height === undefined ? 0 : height;
    }
    get width() {
      return this.#element.width;
    }
    set width(value) {
      this.#element.width = value;
    }
    get height() {
      return this.#element.height;
    }
    set height(value) {
      this.#element.height = value;
    }
    getContext(type, options) {
      return getContext(this.#element, type, options);
    }
    convertToBlob(options = {}) {
      return new Promise((resolve, reject) => {
        const element = this.#element;
        if (typeof element.toBlob !== 'function') {
          reject(new DOMException('OffscreenCanvas cannot be encoded', 'NotSupportedError'));
          return;
        }
        element.toBlob(
          (blob) => {
            if (blob) resolve(blob);
            else reject(new DOMException('OffscreenCanvas cannot be encoded', 'NotSupportedError'));
          },
          options.type === undefined ? 'image/png' : String(options.type),
          options.quality,
        );
      });
    }
  }
  Object.defineProperty(OffscreenCanvas.prototype, Symbol.toStringTag, {
    value: 'OffscreenCanvas',
    configurable: true,
  });
  Object.defineProperty(globalThis, 'OffscreenCanvas', {
    value: OffscreenCanvas,
    writable: true,
    configurable: true,
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
