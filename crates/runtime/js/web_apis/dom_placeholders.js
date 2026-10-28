// Legacy callable coverage for browser UI and rendering operations. These core
// DOM methods intentionally return undefined without changing DOM, input or
// rendering state. They do not emulate result objects, promises or UI effects.
// Element geometry is not in this list: it answers with a zero rectangle, because
// there is no layout engine to measure.
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

// No animation engine exists, so the sequence is always empty; this is the real
// return shape browsers use, not an ignored-result placeholder.
for (const prototype of [Element.prototype, Document.prototype]) {
  Object.defineProperty(prototype, 'getAnimations', {
    value: {
      getAnimations() {
        return [];
      },
    }.getAnimations,
    writable: true,
    enumerable: true,
    configurable: true,
  });
}

// Interfaces browsers expose that this runtime only names: scripts probe for
// presence, so each exists with its documented shape and no backing subsystem.
class MathMLElement extends Element {
  constructor() {
    throw new TypeError('Illegal constructor');
  }
}
class StaticRange {
  constructor(init = {}) {
    if (init === null || typeof init !== 'object') throw new TypeError('init must be an object');
    this.startContainer = init.startContainer ?? null;
    this.startOffset = Number(init.startOffset ?? 0);
    this.endContainer = init.endContainer ?? null;
    this.endOffset = Number(init.endOffset ?? 0);
  }
  get collapsed() {
    return this.startContainer === this.endContainer && this.startOffset === this.endOffset;
  }
}
class FragmentDirective {
  constructor() {
    throw new TypeError('Illegal constructor');
  }
}
class CSSFontPaletteValuesRule {
  constructor() {
    throw new TypeError('Illegal constructor');
  }
  get name() {
    return '';
  }
  get fontFamily() {
    return '';
  }
  get basePalette() {
    return '';
  }
  get overrideColors() {
    return '';
  }
}
class ContentVisibilityAutoStateChangeEvent extends Event {
  #skipped = false;
  constructor(type, init = {}) {
    super(type, init);
    this.#skipped = Boolean(init && init.skipped);
  }
  get skipped() {
    return this.#skipped;
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

// Browsers report the document's own modification time in a fixed local-time
// format, falling back to when the document was created. Response header
// plumbing is not available to the realm, so the navigation time origin is used.
const __documentLastModified = new WeakMap();
function __localDateTimeString(timestamp) {
  const date = new Date(timestamp);
  const pad = (value) => String(value).padStart(2, '0');
  return `${pad(date.getMonth() + 1)}/${pad(date.getDate())}/${date.getFullYear()} ${pad(date.getHours())}:${pad(date.getMinutes())}:${pad(date.getSeconds())}`;
}
Object.defineProperty(Document.prototype, 'lastModified', {
  get() {
    let value = __documentLastModified.get(this);
    if (value === undefined) {
      value = __localDateTimeString(performance.timeOrigin);
      __documentLastModified.set(this, value);
    }
    return value;
  },
  enumerable: true,
  configurable: true,
});

// Read-only timeline with no animation engine behind it: currentTime follows the
// realm clock and there are never any animations to enumerate.
class DocumentTimeline {
  constructor() {
    throw new TypeError('Illegal constructor');
  }
  get currentTime() {
    return performance.now();
  }
  get duration() {
    return null;
  }
}
Object.defineProperty(DocumentTimeline.prototype, Symbol.toStringTag, {
  value: 'DocumentTimeline',
  configurable: true,
});
const __documentTimelines = new WeakMap();
Object.defineProperty(Document.prototype, 'timeline', {
  get() {
    let timeline = __documentTimelines.get(this);
    if (timeline === undefined) {
      timeline = Object.create(DocumentTimeline.prototype);
      __documentTimelines.set(this, timeline);
    }
    return timeline;
  },
  enumerable: true,
  configurable: true,
});
