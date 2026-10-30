const sources = [
  'undefined',
  'null',
  'false',
  'true',
  '0',
  '-0',
  '1',
  '-1',
  '1.9',
  '-1.9',
  'NaN',
  'Infinity',
  '-Infinity',
  '4294967296',
  '4294967297',
  '-4294967295',
  '1n',
  'Symbol()',
  ...[
    '',
    '0',
    '1',
    '01',
    'foo',
    'name',
    'missing',
    'null',
    'undefined',
    '\ud800',
    '4294967296',
  ].map(JSON.stringify),
  '({valueOf(){return 1}})',
  '({toString(){return "foo"}})',
  '({[Symbol.toPrimitive](hint){return hint==="number"?1:"foo"}})',
  '({toString(){throw new RangeError("conversion")}})',
];
export const htmlCollectionMethodCases = ['item', 'namedItem'].flatMap((method) =>
  sources.map((source) => ({ method, source })),
);
export const htmlCollectionPropertyCases = [
  '0',
  '1',
  '8',
  '01',
  '-1',
  '4294967294',
  '4294967295',
  'foo',
  'length',
  'item',
  'toString',
  '__proto__',
  'constructor',
  '__items',
].flatMap((key) =>
  ['id', 'name', 'foreign-id', 'foreign-name', 'none'].flatMap((match) =>
    ['read', 'set', 'define', 'delete', 'inherit', 'inherit-set'].map((operation) => ({
      key,
      match,
      operation,
    })),
  ),
);

export function measureHtmlCollectionMethods(cases) {
  return cases.map((c) => {
    const d = document.implementation.createHTMLDocument();
    d.body.innerHTML = '<img id="foo" name="name"><img id="bar" name="foo">';
    let result,
      error = null;
    try {
      const value = d.images[c.method](Function('return (' + c.source + ')')());
      result = value === null ? null : value.id;
    } catch (e) {
      error = e.name;
    }
    return { result: result ?? null, error };
  });
}
export function measureHtmlCollectionProperties(cases) {
  return cases.map((c) => {
    const d = document.implementation.createHTMLDocument(),
      e = d.createElement('img');
    if (c.match !== 'none')
      e.setAttributeNS(
        c.match.startsWith('foreign') ? 'urn:other' : null,
        c.match.endsWith('name') ? 'name' : 'id',
        c.key,
      );
    d.body.appendChild(e);
    const list = d.images,
      child = Object.create(list);
    const encode = (v) =>
      v === e
        ? 'element'
        : v === undefined
          ? 'undefined'
          : v === null
            ? null
            : typeof v === 'function'
              ? 'function:' + v.name
              : typeof v === 'object'
                ? Object.prototype.toString.call(v)
                : v;
    let result,
      error = null;
    try {
      if (c.operation === 'read') result = encode(list[c.key]);
      if (c.operation === 'set') result = Reflect.set(list, c.key, 42);
      if (c.operation === 'define')
        result = Reflect.defineProperty(list, c.key, {
          value: 42,
          configurable: true,
          writable: true,
          enumerable: true,
        });
      if (c.operation === 'delete') result = Reflect.deleteProperty(list, c.key);
      if (c.operation === 'inherit') result = encode(child[c.key]);
      if (c.operation === 'inherit-set')
        result = [
          Reflect.set(list, c.key, 42, child),
          Object.hasOwn(child, c.key),
          encode(child[c.key]),
        ];
    } catch (e) {
      error = e.name;
    }
    const descriptor = Object.getOwnPropertyDescriptor(list, c.key);
    return {
      result: result ?? null,
      error,
      value: encode(list[c.key]),
      has: c.key in list,
      own: !!descriptor,
      descriptor: descriptor
        ? {
            value: encode(descriptor.value),
            writable: descriptor.writable,
            enumerable: descriptor.enumerable,
            configurable: descriptor.configurable,
          }
        : null,
      keys: Reflect.ownKeys(list).map(String),
    };
  });
}
