export const basicInterfaces = [
  ['head', 'HTMLHeadElement'],
  ['title', 'HTMLTitleElement'],
  ['meta', 'HTMLMetaElement'],
  ['div', 'HTMLDivElement'],
  ['html', 'HTMLHtmlElement'],
  ['li', 'HTMLLIElement'],
  ['span', 'HTMLSpanElement'],
  ['frameset', 'HTMLFrameSetElement'],
];
export const basicReflections = [
  ['meta', 'name', 'name'],
  ['meta', 'httpEquiv', 'http-equiv'],
  ['meta', 'content', 'content'],
  ['meta', 'media', 'media'],
  ['meta', 'scheme', 'scheme'],
  ['div', 'align', 'align'],
  ['html', 'version', 'version'],
  ['li', 'type', 'type'],
  ['frameset', 'cols', 'cols'],
  ['frameset', 'rows', 'rows'],
];
const sources = [
  'undefined',
  'null',
  'false',
  'true',
  '0',
  '-0',
  'NaN',
  'Infinity',
  '1n',
  'Symbol()',
  '({})',
  '({[Symbol.toPrimitive](hint){return hint}})',
  '({toString(){throw new RangeError("conversion")}})',
  ...['', ' Plain VALUE ', '\0', 'a\ud800b\udc00', '😀', 'a\r\nb'].map(JSON.stringify),
];
const integers = [
  '-Infinity',
  '2147483647',
  '2147483648',
  '-2147483648',
  '-2147483649',
  '4294967295',
  '4294967296',
  '9007199254740991',
  '1e100',
  '-1e100',
  '2.9',
  '-2.9',
  '({valueOf(){return 42}})',
  '({[Symbol.toPrimitive](hint){if(hint!=="number")throw Error(hint);return 7}})',
];
const rawIntegers = [
  '',
  '0',
  '-0',
  '+0',
  '-1',
  '+2',
  '  \t\n\r\f-3tail',
  '\v4',
  '\u00a05',
  '1.5',
  '2e3',
  '0xff',
  '--3',
  '+ 4',
  '12px',
  '2147483647',
  '2147483648',
  '-2147483648',
  '-2147483649',
  '4294967296',
  '99999999999999999999999',
  '0000000000000000000001',
  '١',
  '３',
];
export const basicAttributeCases = [
  ...[...basicReflections, ['title', 'text', 'text']].flatMap(([tag, property, attribute]) =>
    sources.map((source) => ({ tag, property, attribute, source, raw: false })),
  ),
  ...[...sources, ...integers].map((source) => ({
    tag: 'li',
    property: 'value',
    attribute: 'value',
    source,
    raw: false,
  })),
  ...rawIntegers.map((value) => ({
    tag: 'li',
    property: 'value',
    attribute: 'value',
    source: JSON.stringify(value),
    raw: true,
  })),
];
export function measureBasicAttributes(cases) {
  return cases.map(({ tag, property, attribute, source, raw }) => {
    const e = document.createElement(tag);
    e.append(document.createTextNode('before'), document.createComment('comment'));
    e.setAttribute(attribute, 'before');
    const observer = new MutationObserver(() => {});
    observer.observe(e, {
      attributes: true,
      attributeOldValue: true,
      childList: true,
      subtree: true,
    });
    let error = null;
    try {
      const value = Function('return (' + source + ')')();
      if (raw) e.setAttribute(attribute, value);
      else e[property] = value;
    } catch (e) {
      error = e.name;
    }
    const records = observer
      .takeRecords()
      .map((r) => [
        r.type,
        r.attributeName,
        r.attributeNamespace,
        r.oldValue,
        r.addedNodes.length,
        r.removedNodes.length,
      ]);
    observer.disconnect();
    return [error, e[property], e.getAttribute(attribute), e.textContent, records];
  });
}
