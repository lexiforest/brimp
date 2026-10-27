(({ global }) => {
  // Numeric values and inline declarations only; no cascade or computed styles.
  const units = {
    number: ['number', 1],
    percent: ['percent', 1],
    px: ['length', 1],
    in: ['length', 96],
    cm: ['length', 96 / 2.54],
    mm: ['length', 96 / 25.4],
    Q: ['length', 96 / 101.6],
    pt: ['length', 96 / 72],
    pc: ['length', 16],
    deg: ['angle', 1],
    rad: ['angle', 180 / Math.PI],
    grad: ['angle', 0.9],
    turn: ['angle', 360],
    s: ['time', 1],
    ms: ['time', 0.001],
    Hz: ['frequency', 1],
    kHz: ['frequency', 1000],
    dpi: ['resolution', 1],
    dpcm: ['resolution', 2.54],
    dppx: ['resolution', 96],
  };
  for (const unit of ['em', 'rem', 'ex', 'ch', 'vw', 'vh', 'vmin', 'vmax', 'fr'])
    units[unit] = [unit, 1];
  const finite = (v) => {
    v = Number(v);
    if (!Number.isFinite(v)) throw new TypeError('Finite CSS number required');
    return v;
  };
  class CSSStyleValue {}
  class CSSNumericValue extends CSSStyleValue {
    static parse(text) {
      const m = /^\s*([+-]?(?:\d+\.?\d*|\.\d+)(?:e[+-]?\d+)?)\s*([a-zA-Z%]*)\s*$/.exec(
        String(text),
      );
      if (!m) throw new SyntaxError('Only unit values are supported');
      return new CSSUnitValue(Number(m[1]), m[2] === '%' ? 'percent' : m[2] || 'number');
    }
    add(...values) {
      return new CSSUnitValue(
        values.reduce((sum, v) => sum + numeric(v).to(this.unit).value, this.value),
        this.unit,
      );
    }
    sub(...values) {
      return new CSSUnitValue(
        values.reduce((sum, v) => sum - numeric(v).to(this.unit).value, this.value),
        this.unit,
      );
    }
    mul(...values) {
      let value = this.value;
      for (const v of values) {
        const n = numeric(v);
        if (n.unit !== 'number')
          throw new DOMException('Compound units unavailable', 'NotSupportedError');
        value *= n.value;
      }
      return new CSSUnitValue(value, this.unit);
    }
    div(...values) {
      let value = this.value;
      for (const v of values) {
        const n = numeric(v);
        if (n.unit !== 'number')
          throw new DOMException('Compound units unavailable', 'NotSupportedError');
        value /= n.value;
      }
      return new CSSUnitValue(value, this.unit);
    }
    equals(...values) {
      return values.every((v) => {
        try {
          return numeric(v).to(this.unit).value === this.value;
        } catch {
          return false;
        }
      });
    }
  }
  const numeric = (v) => (typeof v === 'number' ? new CSSUnitValue(v, 'number') : v);
  class CSSUnitValue extends CSSNumericValue {
    #value;
    #unit;
    constructor(value, unit) {
      super();
      unit = String(unit);
      if (!Object.hasOwn(units, unit)) throw new TypeError('Unsupported CSS unit');
      this.#value = finite(value);
      this.#unit = unit;
    }
    get value() {
      return this.#value;
    }
    set value(v) {
      this.#value = finite(v);
    }
    get unit() {
      return this.#unit;
    }
    to(unit) {
      unit = String(unit);
      if (!Object.hasOwn(units, unit) || units[unit][0] !== units[this.unit][0])
        throw new TypeError('Incompatible CSS units');
      return new CSSUnitValue((this.value * units[this.unit][1]) / units[unit][1], unit);
    }
    toString() {
      return `${this.value}${this.unit === 'number' ? '' : this.unit === 'percent' ? '%' : this.unit}`;
    }
  }
  class CSSKeywordValue extends CSSStyleValue {
    #value;
    constructor(value) {
      super();
      this.value = value;
    }
    get value() {
      return this.#value;
    }
    set value(v) {
      v = String(v);
      if (!v) throw new TypeError('Empty keyword');
      this.#value = v;
    }
    toString() {
      return this.value;
    }
  }
  class CSSVariableReferenceValue {
    #variable;
    constructor(variable, fallback = null) {
      this.variable = variable;
      if (fallback !== null && !(fallback instanceof CSSUnparsedValue))
        throw new TypeError('Invalid fallback');
      this.fallback = fallback;
    }
    get variable() {
      return this.#variable;
    }
    set variable(v) {
      v = String(v);
      if (!v.startsWith('--') || v.length < 3) throw new TypeError('Invalid custom property');
      this.#variable = v;
    }
  }
  class CSSUnparsedValue extends CSSStyleValue {
    #values;
    constructor(values) {
      super();
      this.#values = Array.from(values, (v) =>
        v instanceof CSSVariableReferenceValue ? v : String(v),
      );
      this.#values.forEach((_, i) =>
        Object.defineProperty(this, i, {
          get: () => this.#values[i],
          set: (v) => {
            this.#values[i] = v instanceof CSSVariableReferenceValue ? v : String(v);
          },
          enumerable: true,
        }),
      );
    }
    get length() {
      return this.#values.length;
    }
    [Symbol.iterator]() {
      return this.#values.values();
    }
    toString() {
      return this.#values
        .map((v) =>
          v instanceof CSSVariableReferenceValue
            ? `var(${v.variable}${v.fallback === null ? '' : `, ${v.fallback}`})`
            : v,
        )
        .join('');
    }
  }
  function parse(value) {
    try {
      return CSSNumericValue.parse(value);
    } catch {
      return /^[-\w]+$/.test(value) ? new CSSKeywordValue(value) : new CSSUnparsedValue([value]);
    }
  }
  const styles = new WeakMap(),
    maps = new WeakMap(),
    token = {};
  class StylePropertyMapReadOnly {
    constructor(key, style) {
      if (key !== token) throw new TypeError('Illegal constructor');
      styles.set(this, style);
    }
    get size() {
      return styles.get(this).length;
    }
    get(name) {
      const text = styles.get(this).getPropertyValue(String(name));
      return text === '' ? undefined : parse(text);
    }
    getAll(name) {
      const v = this.get(name);
      return v === undefined ? [] : [v];
    }
    has(name) {
      return this.get(name) !== undefined;
    }
    *entries() {
      const style = styles.get(this);
      for (let i = 0; i < style.length; i++) {
        const name = style.item(i);
        yield [name, this.getAll(name)];
      }
    }
    *keys() {
      for (const [name] of this) yield name;
    }
    *values() {
      for (const [, values] of this) yield values;
    }
    forEach(callback, thisArg) {
      for (const [key, value] of this) callback.call(thisArg, value, key, this);
    }
    [Symbol.iterator]() {
      return this.entries();
    }
  }
  class StylePropertyMap extends StylePropertyMapReadOnly {
    set(name, ...values) {
      if (values.length !== 1)
        throw new DOMException('Only single values supported', 'NotSupportedError');
      styles.get(this).setProperty(String(name), String(values[0]));
    }
    delete(name) {
      styles.get(this).removeProperty(String(name));
    }
    clear() {
      styles.get(this).cssText = '';
    }
  }
  Object.defineProperty(global.Element.prototype, 'attributeStyleMap', {
    configurable: true,
    get() {
      let map = maps.get(this);
      if (!map) {
        map = new StylePropertyMap(token, this.style);
        maps.set(this, map);
      }
      return map;
    },
  });
  const css = global.CSS ?? {};
  for (const unit of Object.keys(units))
    Object.defineProperty(css, unit, {
      value: (value) => new CSSUnitValue(value, unit),
      configurable: true,
    });
  if (!global.CSS) Object.defineProperty(global, 'CSS', { value: css, configurable: true });
  for (const [name, value] of Object.entries({
    CSSStyleValue,
    CSSNumericValue,
    CSSUnitValue,
    CSSKeywordValue,
    CSSVariableReferenceValue,
    CSSUnparsedValue,
    StylePropertyMapReadOnly,
    StylePropertyMap,
  })) {
    Object.defineProperty(value.prototype, Symbol.toStringTag, {
      value: name,
      configurable: true,
    });
    Object.defineProperty(global, name, { value, writable: true, configurable: true });
  }
})({ global: globalThis });
