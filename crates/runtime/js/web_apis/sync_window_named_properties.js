(() => {
  let previousFrameCount = 0;
  const parse = JSON.parse,
    define = Object.defineProperty;
  const fromCharCode = String.fromCharCode;
  return (serialized) => {
    for (const units of parse(serialized)) {
      let name = '';
      for (const unit of units) name += fromCharCode(unit);
      if (!(name in window)) {
        define(window, name, {
          configurable: true,
          enumerable: true,
          get() {
            const element = document.getElementById(name) ?? document.getElementsByName(name)[0];
            return element instanceof HTMLIFrameElement ? element.contentWindow : element;
          },
        });
      }
    }
    const frameCount = document.getElementsByTagName('iframe').length;
    for (let index = frameCount; index < previousFrameCount; index++) delete window[index];
    previousFrameCount = frameCount;
    for (let index = 0; index < frameCount; index++) {
      if (!(index in window)) {
        define(window, index, {
          configurable: true,
          enumerable: true,
          get() {
            return document.getElementsByTagName('iframe').item(index)?.contentWindow;
          },
        });
      }
    }
  };
})();
