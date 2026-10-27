function installExtraction({ global }) {
  // The vendored bundle is evaluated in this installer scope by the packager.
  const Constructor = global.Defuddle;
  delete global.Defuddle;
  if (typeof Constructor !== 'function') throw new Error('Defuddle constructor is unavailable');
  const key = Symbol.for('extract');
  Object.defineProperty(global, key, {
    configurable: true,
    value(optionsJson) {
      const options = JSON.parse(optionsJson);
      // Defuddle's guarded style/geometry checks naturally skip layout in this runtime.
      // Attribute and inline-style heuristics remain available on the canonical DOM.
      return JSON.stringify(new Constructor(document, options).parse());
    },
  });
  return () => {
    delete global[key];
  };
}
