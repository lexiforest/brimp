// MediaQueryList and matchMedia evaluated against the active browser profile.
// There is no layout engine or live resize; values are stable until the profile
// changes, and no change events are generated for a static profile.
let __mediaProfile = null;
let __mediaInstalled = false;
const __mediaQueryData = new WeakMap();
const __mediaQueryToken = Symbol('media query construction');
const __mediaQueryHandlers = new WeakMap();

class MediaQueryList extends EventTarget {
  constructor(token, media, matches) {
    super();
    if (token !== __mediaQueryToken) throw new TypeError('Illegal constructor');
    __mediaQueryData.set(this, { media, matches });
  }
  get media() {
    return __mediaQueryData.get(this).media;
  }
  get matches() {
    return __mediaQueryData.get(this).matches;
  }
  get onchange() {
    return __mediaQueryHandlers.get(this) ?? null;
  }
  set onchange(value) {
    if (typeof value === 'function' || (typeof value === 'object' && value !== null)) {
      __mediaQueryHandlers.set(this, value);
      this.addEventListener('change', value);
    } else {
      __mediaQueryHandlers.set(this, null);
    }
  }
  addListener(listener) {
    if (listener != null) this.addEventListener('change', listener);
  }
  removeListener(listener) {
    if (listener != null) this.removeEventListener('change', listener);
  }
}

class MediaQueryListEvent extends Event {
  #media;
  #matches;
  constructor(type, init = {}) {
    if (arguments.length === 0) throw new TypeError('Event type is required');
    super(type, init);
    this.#media = init.media === undefined ? '' : String(init.media);
    this.#matches = Boolean(init.matches);
  }
  get media() {
    return this.#media;
  }
  get matches() {
    return this.#matches;
  }
}

function __mediaLength(value) {
  const match = /^(-?(?:\d+|\d*\.\d+))(px|em|rem)?$/.exec(String(value).trim().toLowerCase());
  if (!match) return NaN;
  const number = Number(match[1]);
  return match[2] === 'em' || match[2] === 'rem' ? number * 16 : number;
}

function __mediaResolution(value) {
  const match = /^(-?(?:\d+|\d*\.\d+))(dppx|dpi|dpcm)?$/.exec(String(value).trim().toLowerCase());
  if (!match) return NaN;
  const number = Number(match[1]);
  return match[2] === 'dpi' ? number / 96 : match[2] === 'dpcm' ? (number * 2.54) / 96 : number;
}

function __mediaBooleanFeature(feature) {
  const profile = __mediaProfile;
  switch (feature.trim().toLowerCase()) {
    case 'color':
      return profile.screen.color_depth > 0;
    case 'hover':
    case 'any-hover':
      return profile.media.hover === 'hover';
    case 'pointer':
    case 'any-pointer':
      return profile.media.pointer !== 'none';
    case 'width':
    case 'height':
    case 'device-width':
    case 'device-height':
      return true;
    case 'grid':
    case 'monochrome':
    case 'color-index':
      return false;
    default:
      return false;
  }
}

function __mediaFeature(feature, value) {
  const profile = __mediaProfile;
  const width = profile.window.inner_width;
  const height = profile.window.inner_height;
  const deviceWidth = profile.screen.width;
  const deviceHeight = profile.screen.height;
  const ratio = profile.screen.device_pixel_ratio;
  feature = feature.trim().toLowerCase();
  const prefix = feature.startsWith('min-') ? 'min' : feature.startsWith('max-') ? 'max' : null;
  const name = prefix ? feature.slice(4) : feature;
  const compare = (actual, expected) =>
    !Number.isFinite(expected)
      ? false
      : prefix === 'min'
        ? actual >= expected
        : prefix === 'max'
          ? actual <= expected
          : actual === expected;
  switch (name) {
    case 'width':
      return prefix
        ? compare(width, __mediaLength(value))
        : Number.isFinite(__mediaLength(value)) && width === __mediaLength(value);
    case 'height':
      return prefix
        ? compare(height, __mediaLength(value))
        : Number.isFinite(__mediaLength(value)) && height === __mediaLength(value);
    case 'device-width':
      return compare(deviceWidth, __mediaLength(value));
    case 'device-height':
      return compare(deviceHeight, __mediaLength(value));
    case 'aspect-ratio':
    case 'device-aspect-ratio': {
      const [a, b] = String(value).split('/').map(Number);
      if (!Number.isFinite(a) || !Number.isFinite(b) || b === 0) return false;
      const actual = name === 'aspect-ratio' ? width / height : deviceWidth / deviceHeight;
      return compare(actual, a / b);
    }
    case 'resolution':
      return compare(ratio, __mediaResolution(value));
    case 'device-pixel-ratio':
    case '-webkit-device-pixel-ratio':
      return compare(ratio, Number(value));
    case 'color':
      return prefix ? compare(profile.screen.color_depth, Number(value)) : Number(value) > 0;
    case 'color-index':
      return prefix ? compare(0, Number(value)) : false;
    case 'monochrome':
      return prefix ? compare(0, Number(value)) : false;
    case 'grid':
      return prefix ? compare(0, Number(value)) : value === '0';
    case 'orientation':
      return value === 'landscape'
        ? width > height
        : value === 'portrait'
          ? height >= width
          : false;
    case 'prefers-color-scheme':
      return value === profile.media.prefers_color_scheme;
    case 'prefers-reduced-motion':
      return value === profile.media.prefers_reduced_motion;
    case 'prefers-contrast':
      return value === profile.media.prefers_contrast;
    case 'pointer':
    case 'any-pointer':
      return value === profile.media.pointer;
    case 'hover':
    case 'any-hover':
      return value === profile.media.hover;
    case 'forced-colors':
      return value === 'none';
    case 'scripting':
      return value === 'enabled';
    case 'update':
      return value === 'fast';
    default:
      return false;
  }
}

function __evaluateMediaQuery(query) {
  if (__mediaProfile === null) return false;
  const text = String(query).trim();
  if (!text) return false;
  return text.split(',').some((part) => {
    let clause = part.trim().toLowerCase();
    if (!clause) return false;
    let negate = false;
    if (clause.startsWith('not ')) {
      negate = true;
      clause = clause.slice(4).trim();
    } else if (clause.startsWith('only ')) clause = clause.slice(5).trim();
    const conditions = clause.split(/\s+and\s+/).map((item) => item.trim());
    let result = true;
    for (const condition of conditions) {
      if (!condition) {
        result = false;
        break;
      }
      if (condition.startsWith('(') && condition.endsWith(')')) {
        const [feature, value] = condition
          .slice(1, -1)
          .split(':', 2)
          .map((item) => item.trim());
        if (!feature) result = false;
        else if (value === undefined) result = __mediaBooleanFeature(feature);
        else result = __mediaFeature(feature, value);
      } else if (['all', 'screen'].includes(condition)) {
        result = true;
      } else if (['print', 'speech'].includes(condition)) {
        result = false;
      } else {
        result = false;
      }
      if (!result) break;
    }
    return negate ? !result : result;
  });
}

function __installMediaQueries(profile) {
  __mediaProfile = profile;
  if (__mediaInstalled) return;
  __mediaInstalled = true;
  Object.defineProperty(Window.prototype, 'matchMedia', {
    value: function matchMedia(query) {
      query = String(query);
      return new MediaQueryList(__mediaQueryToken, query, __evaluateMediaQuery(query));
    },
    writable: true,
    enumerable: true,
    configurable: true,
  });
  Object.assign(globalThis, { MediaQueryList, MediaQueryListEvent });
}

Object.defineProperty(globalThis, '__brimpInstallMediaQueries', {
  value: __installMediaQueries,
  writable: false,
  enumerable: false,
  configurable: true,
});
