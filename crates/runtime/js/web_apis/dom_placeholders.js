// Legacy callable coverage for browser UI and rendering operations. These core
// DOM methods intentionally return undefined without changing DOM, input or
// rendering state. They do not emulate result objects, promises or UI effects.
for (const [prototype, names] of [
  [
    Window.prototype,
    [
      'alert',
      'blur',
      'captureEvents',
      'close',
      'confirm',
      'find',
      'focus',
      'getComputedStyle',
      'moveBy',
      'moveTo',
      'open',
      'print',
      'prompt',
      'releaseEvents',
      'reportError',
      'resizeBy',
      'resizeTo',
      'scroll',
      'scrollBy',
      'scrollTo',
      'stop',
    ],
  ],
  [
    Element.prototype,
    [
      'animate',
      'getAnimations',
      'getBoundingClientRect',
      'getClientRects',
      'hasPointerCapture',
      'releasePointerCapture',
      'setPointerCapture',
      'requestFullscreen',
      'requestPointerLock',
      'webkitRequestFullScreen',
      'webkitRequestFullscreen',
      'scroll',
      'scrollBy',
      'scrollTo',
      'scrollIntoView',
      'scrollIntoViewIfNeeded',
      'computedStyleMap',
    ],
  ],
  [
    Document.prototype,
    [
      'captureEvents',
      'releaseEvents',
      'clear',
      'caretRangeFromPoint',
      'elementFromPoint',
      'elementsFromPoint',
      'execCommand',
      'queryCommandEnabled',
      'queryCommandIndeterm',
      'queryCommandState',
      'queryCommandSupported',
      'queryCommandValue',
      'getAnimations',
      'exitFullscreen',
      'exitPictureInPicture',
      'exitPointerLock',
      'webkitCancelFullScreen',
      'webkitExitFullscreen',
    ],
  ],
]) {
  for (const name of names) {
    Object.defineProperty(prototype, name, {
      value: { [name]() {} }[name],
      writable: true,
      enumerable: true,
      configurable: true,
    });
  }
}

// Empty geometry defaults and locally retained scroll values do not measure or
// move rendered boxes. Keep them on Element so HTML and SVG share the surface.
for (const name of [
  'clientHeight',
  'clientWidth',
  'clientTop',
  'clientLeft',
  'scrollHeight',
  'scrollWidth',
]) {
  Object.defineProperty(Element.prototype, name, {
    get() {
      return 0;
    },
    enumerable: true,
    configurable: true,
  });
}
for (const name of ['scrollTop', 'scrollLeft']) {
  const values = new WeakMap();
  Object.defineProperty(Element.prototype, name, {
    get() {
      return values.get(this) ?? 0;
    },
    set(value) {
      value = Number(value);
      values.set(this, Number.isFinite(value) ? value : 0);
    },
    enumerable: true,
    configurable: true,
  });
}

// Browser services have no producer in this runtime. Use saved Safari defaults
// for simple flags, and preserve the legacy undefined placeholders for objects
// whose behavior would require font, animation, navigation or HTMLDDA machinery.
for (const [name, value] of Object.entries({
  all: undefined,
  fragmentDirective: undefined,
  timeline: undefined,
  lastModified: undefined,
  referrer: '',
  fullscreen: false,
  fullscreenElement: null,
  fullscreenEnabled: true,
  pictureInPictureElement: null,
  pictureInPictureEnabled: true,
  pointerLockElement: null,
  webkitCurrentFullScreenElement: null,
  webkitFullscreenElement: null,
  webkitFullscreenEnabled: true,
  webkitIsFullScreen: false,
  xmlEncoding: null,
  xmlStandalone: false,
  xmlVersion: null,
})) {
  Object.defineProperty(Document.prototype, name, {
    get() {
      return value;
    },
    enumerable: true,
    configurable: true,
  });
}
Object.defineProperty(Document.prototype, 'scrollingElement', {
  // Common standards/quirks fallback; no CSS scrollability/layout inspection.
  get() {
    return this.compatMode === 'BackCompat' ? this.body : this.documentElement;
  },
  enumerable: true,
  configurable: true,
});
