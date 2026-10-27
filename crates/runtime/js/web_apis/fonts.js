// FontFace/FontFaceSet metadata backed by the browser profile. There is no
// shaping, loading engine or layout: check() answers from profile families.
let __fontProfileFamilies = [];
let __fontMetrics = Object.create(null);
let __fontInstalled = false;
const __genericFamilies = new Set([
  'serif',
  'sans-serif',
  'monospace',
  'cursive',
  'fantasy',
  'system-ui',
  'ui-serif',
  'ui-sans-serif',
  'ui-monospace',
  'ui-rounded',
  'math',
  'emoji',
]);
const __fontFaces = new Set();
const __fontData = new WeakMap();
const __fontSetToken = Symbol('font set construction');

function __normalizeFontFamily(value) {
  return String(value)
    .trim()
    .replace(/^['"]|['"]$/g, '')
    .toLowerCase();
}

function __fontFamiliesOf(shorthand) {
  let value = String(shorthand).trim();
  if (!value) return [];
  // Drop style, variant, weight, stretch and size tokens up to the family.
  value = value.replace(
    /^(?:normal|italic|oblique|small-caps|bold|bolder|lighter|[1-9]00)\s+/i,
    '',
  );
  value = value.replace(
    /^(?:ultra-condensed|extra-condensed|condensed|semi-condensed|semi-expanded|expanded|extra-expanded|ultra-expanded)\s+/i,
    '',
  );
  value = value.replace(
    /^(?:\d+(?:\.\d+)?(?:px|pt|pc|in|cm|mm|em|rem|ex|ch|vw|vh|vmin|vmax|%))\s+(?:[1-9]\d*(?:\.\d+)?\s+)?/i,
    '',
  );
  return value.split(',').map(__normalizeFontFamily).filter(Boolean);
}

function __fontKnown(family) {
  return (
    __genericFamilies.has(family) ||
    __fontProfileFamilies.some((name) => __normalizeFontFamily(name) === family)
  );
}

class FontFace {
  constructor(family, source, descriptors = {}) {
    if (arguments.length < 2) throw new TypeError('FontFace requires family and source');
    family = String(family);
    if (!family.trim()) throw new SyntaxError('FontFace family must not be empty');
    if (descriptors === null || typeof descriptors !== 'object')
      throw new TypeError('FontFace descriptors must be an object');
    const data = {
      family,
      source,
      style: descriptors.style === undefined ? 'normal' : String(descriptors.style),
      weight: descriptors.weight === undefined ? 'normal' : String(descriptors.weight),
      stretch: descriptors.stretch === undefined ? 'normal' : String(descriptors.stretch),
      unicodeRange:
        descriptors.unicodeRange === undefined ? 'U+0-10FFFF' : String(descriptors.unicodeRange),
      featureSettings:
        descriptors.featureSettings === undefined ? 'normal' : String(descriptors.featureSettings),
      variationSettings:
        descriptors.variationSettings === undefined
          ? 'normal'
          : String(descriptors.variationSettings),
      display: descriptors.display === undefined ? 'auto' : String(descriptors.display),
      ascentOverride:
        descriptors.ascentOverride === undefined ? 'normal' : String(descriptors.ascentOverride),
      descentOverride:
        descriptors.descentOverride === undefined ? 'normal' : String(descriptors.descentOverride),
      lineGapOverride:
        descriptors.lineGapOverride === undefined ? 'normal' : String(descriptors.lineGapOverride),
      sizeAdjust: descriptors.sizeAdjust === undefined ? '100%' : String(descriptors.sizeAdjust),
      status: 'unloaded',
      loaded: null,
    };
    data.loaded = Promise.resolve(this);
    __fontData.set(this, data);
  }
  get family() {
    return __fontData.get(this).family;
  }
  get style() {
    return __fontData.get(this).style;
  }
  get weight() {
    return __fontData.get(this).weight;
  }
  get stretch() {
    return __fontData.get(this).stretch;
  }
  get unicodeRange() {
    return __fontData.get(this).unicodeRange;
  }
  get featureSettings() {
    return __fontData.get(this).featureSettings;
  }
  get variationSettings() {
    return __fontData.get(this).variationSettings;
  }
  get display() {
    return __fontData.get(this).display;
  }
  get ascentOverride() {
    return __fontData.get(this).ascentOverride;
  }
  get descentOverride() {
    return __fontData.get(this).descentOverride;
  }
  get lineGapOverride() {
    return __fontData.get(this).lineGapOverride;
  }
  get sizeAdjust() {
    return __fontData.get(this).sizeAdjust;
  }
  get status() {
    return __fontData.get(this).status;
  }
  get loaded() {
    return __fontData.get(this).loaded;
  }
  load() {
    const data = __fontData.get(this);
    data.status = 'loaded';
    return data.loaded;
  }
}

class FontFaceSet extends EventTarget {
  constructor(token) {
    super();
    if (token !== __fontSetToken) throw new TypeError('Illegal constructor');
    this.onloading = null;
    this.onloadingdone = null;
    this.onloadingerror = null;
  }
  get ready() {
    return Promise.resolve(this);
  }
  get status() {
    return 'loaded';
  }
  get size() {
    return __fontFaces.size;
  }
  add(font) {
    if (!(font instanceof FontFace)) throw new TypeError('FontFace required');
    __fontFaces.add(font);
    return this;
  }
  delete(font) {
    if (!(font instanceof FontFace)) throw new TypeError('FontFace required');
    return __fontFaces.delete(font);
  }
  has(font) {
    if (!(font instanceof FontFace)) throw new TypeError('FontFace required');
    return __fontFaces.has(font);
  }
  clear() {
    __fontFaces.clear();
  }
  check(font, text = 'BESbswy') {
    const families = __fontFamiliesOf(font);
    if (!families.length) return false;
    return families.some(__fontKnown);
  }
  load(font, text = 'BESbswy') {
    const families = __fontFamiliesOf(font);
    const matches = [...__fontFaces].filter((face) =>
      families.includes(__normalizeFontFamily(face.family)),
    );
    for (const face of matches) face.load();
    return Promise.resolve(matches);
  }
  forEach(callback, thisArg) {
    for (const face of __fontFaces) callback.call(thisArg, face, face, this);
  }
  [Symbol.iterator]() {
    return __fontFaces[Symbol.iterator]();
  }
  entries() {
    return [...__fontFaces].map((face) => [face, face])[Symbol.iterator]();
  }
  keys() {
    return __fontFaces[Symbol.iterator]();
  }
  values() {
    return __fontFaces[Symbol.iterator]();
  }
}

function __installFonts(profile) {
  __fontProfileFamilies = profile.fonts.families.slice();
  __fontMetrics = Object.assign(Object.create(null), profile.fonts.metrics);
  if (__fontInstalled) return;
  __fontInstalled = true;
  const set = new FontFaceSet(__fontSetToken);
  Object.defineProperty(Document.prototype, 'fonts', {
    get() {
      return set;
    },
    enumerable: true,
    configurable: true,
  });
  Object.assign(globalThis, { FontFace, FontFaceSet });
}

Object.defineProperty(globalThis, '__brimpInstallFonts', {
  value: __installFonts,
  writable: false,
  enumerable: false,
  configurable: true,
});
