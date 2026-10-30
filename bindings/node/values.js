/** The same tagged contract is used by evaluation, handles and plugins. */
export const referenceValue = Symbol('brimp.reference');
export class HostFunction {
  constructor(name) {
    this.name = name;
    Object.freeze(this);
  }
}
export function encode(value, ancestors = new Set()) {
  if (value === undefined) return { type: 'undefined' };
  if (value === null) return { type: 'null' };
  if (typeof value === 'boolean' || typeof value === 'string') return { type: typeof value, value };
  if (typeof value === 'bigint') return { type: 'bigInt', value: String(value) };
  if (typeof value === 'number')
    return Number.isFinite(value) && !Object.is(value, -0)
      ? { type: 'number', value }
      : { type: 'specialNumber', value: Object.is(value, -0) ? '-0' : String(value) };
  if (value instanceof HostFunction) return { type: 'hostFunction', value: value.name };
  if (typeof value?.[referenceValue] === 'function')
    return { type: 'reference', value: value[referenceValue]() };
  if (value instanceof ArrayBuffer) return { type: 'binary', value: [...new Uint8Array(value)] };
  if (ArrayBuffer.isView(value))
    return {
      type: 'binary',
      value: [...new Uint8Array(value.buffer, value.byteOffset, value.byteLength)],
    };
  if (!value || typeof value !== 'object' || ancestors.has(value))
    throw new TypeError('Unsupported or cyclic host data; use copied values or a JSHandle');
  ancestors.add(value);
  try {
    if (Array.isArray(value))
      return { type: 'array', value: Array.from(value, (item) => encode(item, ancestors)) };
    if (Object.getPrototypeOf(value) === Object.prototype || Object.getPrototypeOf(value) === null)
      return {
        type: 'object',
        value: Object.fromEntries(
          Object.entries(value).map(([key, item]) => [key, encode(item, ancestors)]),
        ),
      };
    throw new TypeError('Unsupported bridge value; use copied data or a JSHandle');
  } finally {
    ancestors.delete(value);
  }
}
export function decode({ type, value }, reference) {
  switch (type) {
    case 'undefined':
      return undefined;
    case 'null':
      return null;
    case 'specialNumber':
      return { NaN, Infinity, '-Infinity': -Infinity, '-0': -0 }[value];
    case 'bigInt':
      return BigInt(value);
    case 'binary':
      return Buffer.from(value);
    case 'array':
      return value.map((item) => decode(item, reference));
    case 'object':
      return Object.fromEntries(
        Object.entries(value).map(([key, item]) => [key, decode(item, reference)]),
      );
    case 'reference':
      if (!reference) throw new TypeError('References require a live realm');
      return reference(value);
    case 'boolean':
    case 'number':
    case 'string':
      return value;
    default:
      throw new TypeError(`Invalid bridge value: ${type}`);
  }
}
