// Structured values persisted between realms. References preserve cycles and aliases.
export function encode(value) {
  const seen = new Map(),
    nodes = [];
  function visit(value) {
    if (value === undefined) return ['undefined'];
    if (typeof value === 'bigint') return ['bigint', String(value)];
    if (typeof value === 'number' && (!Number.isFinite(value) || Object.is(value, -0)))
      return ['number', Object.is(value, -0) ? '-0' : String(value)];
    if (value === null || typeof value !== 'object') return ['value', value];
    if (seen.has(value)) return ['ref', seen.get(value)];
    const id = nodes.length;
    seen.set(value, id);
    nodes.push(null);
    let node;
    if (value instanceof Date) node = ['Date', String(value.getTime())];
    else if (value instanceof RegExp) node = ['RegExp', value.source, value.flags];
    else if (value instanceof Map) node = ['Map', [...value].map(([k, v]) => [visit(k), visit(v)])];
    else if (value instanceof Set) node = ['Set', [...value].map(visit)];
    else if (value instanceof ArrayBuffer)
      node = ['ArrayBuffer', Array.from(new Uint8Array(value))];
    else if (ArrayBuffer.isView(value))
      node = [
        'View',
        value.constructor.name,
        visit(value.buffer),
        value.byteOffset,
        value instanceof DataView ? value.byteLength : value.length,
      ];
    else if (value instanceof Blob)
      node = [
        'Blob',
        Array.from(value.__bytes),
        value.type,
        value instanceof File ? [value.name, value.lastModified] : null,
      ];
    else if (value instanceof Error)
      node = ['Error', value.name, value.message, visit(value.cause)];
    else
      node = [
        Array.isArray(value) ? 'Array' : 'Object',
        Object.keys(value).map((key) => [key, visit(value[key])]),
        value.length,
      ];
    nodes[id] = node;
    return ['ref', id];
  }
  const root = visit(value);
  return JSON.stringify({ root, nodes });
}
export function decode(text) {
  const { root, nodes } = JSON.parse(text),
    values = new Map();
  function visit(wire) {
    const [tag, data] = wire;
    if (tag === 'value') return data;
    if (tag === 'undefined') return undefined;
    if (tag === 'bigint') return BigInt(data);
    if (tag === 'number') return Number(data);
    if (values.has(data)) return values.get(data);
    const [type, a, b, c, d] = nodes[data];
    let value;
    switch (type) {
      case 'Date':
        value = new Date(Number(a));
        break;
      case 'RegExp':
        value = new RegExp(a, b);
        break;
      case 'ArrayBuffer':
        value = new Uint8Array(a).buffer;
        break;
      case 'View':
        value = new globalThis[a](visit(b), c, d);
        break;
      case 'Blob':
        value = c
          ? new File([new Uint8Array(a)], c[0], { type: b, lastModified: c[1] })
          : new Blob([new Uint8Array(a)], { type: b });
        break;
      case 'Map':
        value = new Map();
        break;
      case 'Set':
        value = new Set();
        break;
      case 'Array':
        value = new Array(b);
        break;
      case 'Error':
        value = new Error(b);
        value.name = a;
        break;
      default:
        value = {};
    }
    values.set(data, value);
    if (type === 'Map') for (const [k, v] of a) value.set(visit(k), visit(v));
    else if (type === 'Set') for (const item of a) value.add(visit(item));
    else if (type === 'Object' || type === 'Array')
      for (const [k, v] of a)
        Object.defineProperty(value, k, {
          value: visit(v),
          writable: true,
          enumerable: true,
          configurable: true,
        });
    else if (type === 'Error') value.cause = visit(c);
    return value;
  }
  return visit(root);
}
