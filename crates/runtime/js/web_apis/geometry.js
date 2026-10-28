(({ global }) => {
  // Arithmetic only: these values never measure a document or fabricate layout.
  const points = new WeakMap(),
    rects = new WeakMap(),
    matrices = new WeakMap();
  const rectLists = new WeakMap();
  class DOMRectList {
    constructor(rectangles = []) {
      const items = rectangles.slice();
      rectLists.set(this, items);
      items.forEach((rect, index) =>
        Object.defineProperty(this, index, {
          value: rect,
          enumerable: true,
          configurable: true,
        }),
      );
    }
    get length() {
      return rectLists.get(this).length;
    }
    item(index) {
      return rectLists.get(this)[index] ?? null;
    }
    [Symbol.iterator]() {
      return rectLists.get(this)[Symbol.iterator]();
    }
    forEach(callback, thisArg) {
      rectLists.get(this).forEach(callback, thisArg);
    }
  }
  class DOMPointReadOnly {
    constructor(x = 0, y = 0, z = 0, w = 1) {
      points.set(this, [x, y, z, w].map(Number));
    }
    static fromPoint(p = {}) {
      return new this(p.x, p.y, p.z, p.w);
    }
    matrixTransform(matrix = {}) {
      return DOMMatrix.fromMatrix(matrix).transformPoint(this);
    }
    toJSON() {
      return { x: this.x, y: this.y, z: this.z, w: this.w };
    }
  }
  class DOMPoint extends DOMPointReadOnly {}
  class DOMRectReadOnly {
    constructor(x = 0, y = 0, width = 0, height = 0) {
      rects.set(this, [x, y, width, height].map(Number));
    }
    static fromRect(r = {}) {
      return new this(r.x, r.y, r.width, r.height);
    }
    get top() {
      return Math.min(this.y, this.y + this.height);
    }
    get bottom() {
      return Math.max(this.y, this.y + this.height);
    }
    get left() {
      return Math.min(this.x, this.x + this.width);
    }
    get right() {
      return Math.max(this.x, this.x + this.width);
    }
    toJSON() {
      return {
        x: this.x,
        y: this.y,
        width: this.width,
        height: this.height,
        top: this.top,
        right: this.right,
        bottom: this.bottom,
        left: this.left,
      };
    }
  }
  class DOMRect extends DOMRectReadOnly {}
  for (const [ro, rw, map, names] of [
    [DOMPointReadOnly, DOMPoint, points, ['x', 'y', 'z', 'w']],
    [DOMRectReadOnly, DOMRect, rects, ['x', 'y', 'width', 'height']],
  ]) {
    names.forEach((name, i) => {
      Object.defineProperty(ro.prototype, name, {
        get() {
          return map.get(this)[i];
        },
        enumerable: true,
        configurable: true,
      });
      Object.defineProperty(rw.prototype, name, {
        get() {
          return map.get(this)[i];
        },
        set(v) {
          map.get(this)[i] = Number(v);
        },
        enumerable: true,
        configurable: true,
      });
    });
  }
  class DOMQuad {
    constructor(p1 = {}, p2 = {}, p3 = {}, p4 = {}) {
      for (const [i, p] of [p1, p2, p3, p4].entries())
        Object.defineProperty(this, `p${i + 1}`, {
          value: DOMPoint.fromPoint(p),
          enumerable: true,
        });
    }
    static fromRect(r = {}) {
      const { x, y, width: w, height: h } = DOMRect.fromRect(r);
      return new this({ x, y }, { x: x + w, y }, { x: x + w, y: y + h }, { x, y: y + h });
    }
    static fromQuad(q = {}) {
      return new this(q.p1, q.p2, q.p3, q.p4);
    }
    getBounds() {
      const p = [this.p1, this.p2, this.p3, this.p4],
        x = Math.min(...p.map((p) => p.x)),
        y = Math.min(...p.map((p) => p.y));
      return new DOMRect(
        x,
        y,
        Math.max(...p.map((p) => p.x)) - x,
        Math.max(...p.map((p) => p.y)) - y,
      );
    }
    toJSON() {
      return {
        p1: this.p1.toJSON(),
        p2: this.p2.toJSON(),
        p3: this.p3.toJSON(),
        p4: this.p4.toJSON(),
      };
    }
  }
  function values(init) {
    if (init === undefined || init === '') return [1, 0, 0, 1, 0, 0];
    if (typeof init === 'string') {
      const m = /^matrix\(([^)]+)\)$/.exec(init.trim());
      if (!m) throw new DOMException('Only matrix(a,b,c,d,e,f) is supported', 'NotSupportedError');
      init = m[1].split(',');
    }
    let v = Array.from(init, Number);
    if (v.length === 16) {
      if ([2, 3, 6, 7, 8, 9, 11, 14].some((i) => v[i] !== 0) || v[10] !== 1 || v[15] !== 1)
        throw new DOMException('3D matrices unavailable', 'NotSupportedError');
      v = [v[0], v[1], v[4], v[5], v[12], v[13]];
    }
    if (v.length !== 6) throw new TypeError('This plugin supports six-component 2D matrices');
    return v;
  }
  const multiply = (a, b) => [
    a[0] * b[0] + a[2] * b[1],
    a[1] * b[0] + a[3] * b[1],
    a[0] * b[2] + a[2] * b[3],
    a[1] * b[2] + a[3] * b[3],
    a[0] * b[4] + a[2] * b[5] + a[4],
    a[1] * b[4] + a[3] * b[5] + a[5],
  ];
  class DOMMatrixReadOnly {
    constructor(init) {
      matrices.set(this, values(init));
    }
    static fromMatrix(m = {}) {
      if (
        m.is2D === false ||
        ['m13', 'm14', 'm23', 'm24', 'm31', 'm32', 'm34', 'm43'].some(
          (k) => m[k] !== undefined && Number(m[k]) !== 0,
        ) ||
        ['m33', 'm44'].some((k) => m[k] !== undefined && Number(m[k]) !== 1)
      )
        throw new DOMException('3D matrices unavailable', 'NotSupportedError');
      const aliases = ['m11', 'm12', 'm21', 'm22', 'm41', 'm42'];
      const v = ['a', 'b', 'c', 'd', 'e', 'f'].map((k, i) => {
        if (
          m[k] !== undefined &&
          m[aliases[i]] !== undefined &&
          Number(m[k]) !== Number(m[aliases[i]])
        )
          throw new TypeError('Conflicting matrix aliases');
        return Number(m[k] ?? m[aliases[i]] ?? ([0, 3].includes(i) ? 1 : 0));
      });
      return new this(v);
    }
    static fromFloat32Array(a) {
      return new this(a);
    }
    static fromFloat64Array(a) {
      return new this(a);
    }
    get is2D() {
      return true;
    }
    get isIdentity() {
      return matrices.get(this).every((n, i) => n === [1, 0, 0, 1, 0, 0][i]);
    }
    multiply(other = {}) {
      return new DOMMatrix(multiply(matrices.get(this), matrices.get(DOMMatrix.fromMatrix(other))));
    }
    translate(x = 0, y = 0, z = 0) {
      if (Number(z) !== 0) throw new DOMException('3D unavailable', 'NotSupportedError');
      return this.multiply({ e: Number(x), f: Number(y) });
    }
    scale(x = 1, y = x, z = 1, ox = 0, oy = 0, oz = 0) {
      if (Number(z) !== 1 || Number(oz) !== 0)
        throw new DOMException('3D unavailable', 'NotSupportedError');
      return this.translate(ox, oy)
        .multiply({ a: Number(x), d: Number(y) })
        .translate(-ox, -oy);
    }
    rotate(angle = 0) {
      if (arguments.length > 1) throw new DOMException('3D unavailable', 'NotSupportedError');
      const r = (Number(angle) * Math.PI) / 180,
        c = Math.cos(r),
        s = Math.sin(r);
      return this.multiply({ a: c, b: s, c: -s, d: c });
    }
    inverse() {
      const [a, b, c, d, e, f] = matrices.get(this),
        det = a * d - b * c;
      return new DOMMatrix(
        det
          ? [d / det, -b / det, -c / det, a / det, (c * f - d * e) / det, (b * e - a * f) / det]
          : Array(6).fill(NaN),
      );
    }
    transformPoint(p = {}) {
      const { x, y, z, w } = DOMPoint.fromPoint(p),
        [a, b, c, d, e, f] = matrices.get(this);
      return new DOMPoint(a * x + c * y + e * w, b * x + d * y + f * w, z, w);
    }
    toFloat32Array() {
      return new Float32Array(this.toFloat64Array());
    }
    toFloat64Array() {
      const [a, b, c, d, e, f] = matrices.get(this);
      return new Float64Array([a, b, 0, 0, c, d, 0, 0, 0, 0, 1, 0, e, f, 0, 1]);
    }
    toString() {
      return `matrix(${matrices.get(this).join(', ')})`;
    }
    toJSON() {
      const result = {
        a: this.a,
        b: this.b,
        c: this.c,
        d: this.d,
        e: this.e,
        f: this.f,
        is2D: this.is2D,
        isIdentity: this.isIdentity,
      };
      const v = this.toFloat64Array();
      for (let c = 1; c <= 4; c++)
        for (let r = 1; r <= 4; r++) result[`m${c}${r}`] = v[(c - 1) * 4 + r - 1];
      return result;
    }
  }
  class DOMMatrix extends DOMMatrixReadOnly {
    multiplySelf(m) {
      matrices.set(this, matrices.get(this.multiply(m)));
      return this;
    }
    preMultiplySelf(m) {
      matrices.set(this, matrices.get(DOMMatrix.fromMatrix(m).multiply(this)));
      return this;
    }
    translateSelf(...v) {
      matrices.set(this, matrices.get(this.translate(...v)));
      return this;
    }
    scaleSelf(...v) {
      matrices.set(this, matrices.get(this.scale(...v)));
      return this;
    }
    rotateSelf(...v) {
      matrices.set(this, matrices.get(this.rotate(...v)));
      return this;
    }
    invertSelf() {
      matrices.set(this, matrices.get(this.inverse()));
      return this;
    }
    setMatrixValue(value) {
      matrices.set(this, values(value));
      return this;
    }
  }
  ['a', 'b', 'c', 'd', 'e', 'f'].forEach((name, i) => {
    const alias = ['m11', 'm12', 'm21', 'm22', 'm41', 'm42'][i];
    for (const key of [name, alias]) {
      Object.defineProperty(DOMMatrixReadOnly.prototype, key, {
        get() {
          return matrices.get(this)[i];
        },
        enumerable: true,
      });
      Object.defineProperty(DOMMatrix.prototype, key, {
        get() {
          return matrices.get(this)[i];
        },
        set(v) {
          matrices.get(this)[i] = Number(v);
        },
        enumerable: true,
      });
    }
  });
  for (let c = 1; c <= 4; c++)
    for (let r = 1; r <= 4; r++) {
      const name = `m${c}${r}`;
      if (!(name in DOMMatrixReadOnly.prototype))
        Object.defineProperty(DOMMatrixReadOnly.prototype, name, {
          get() {
            return c === r ? 1 : 0;
          },
          enumerable: true,
        });
    }
  // There is no layout engine, so element geometry is a zero rectangle at the
  // origin rather than an absent value: scripts read `.width`, `.top`, `.right`
  // and `rects.length` without special-casing this runtime.
  const geometryElement = () =>
    typeof Element === 'function' ? Element.prototype : global.Element.prototype;
  for (const [name, method] of [
    ['getBoundingClientRect', () => new DOMRect(0, 0, 0, 0)],
    ['getClientRects', () => new DOMRectList([])],
  ])
    Object.defineProperty(geometryElement(), name, {
      value: {
        [name]() {
          if (!(this instanceof Element)) throw new TypeError('Illegal invocation');
          return method();
        },
      }[name],
      writable: true,
      configurable: true,
      enumerable: true,
    });
  for (const [name, value] of Object.entries({
    DOMPointReadOnly,
    DOMPoint,
    DOMRectReadOnly,
    DOMRect,
    DOMRectList,
    DOMQuad,
    DOMMatrixReadOnly,
    DOMMatrix,
  })) {
    Object.defineProperty(value.prototype, Symbol.toStringTag, {
      value: name,
      configurable: true,
    });
    Object.defineProperty(global, name, { value, writable: true, configurable: true });
  }
  // WebKit exposes its legacy CSS matrix interface as an alias of DOMMatrix.
  if (typeof global.WebKitCSSMatrix !== 'function') {
    Object.defineProperty(global, 'WebKitCSSMatrix', {
      value: DOMMatrix,
      writable: true,
      configurable: true,
    });
  }
})({ global: globalThis });
