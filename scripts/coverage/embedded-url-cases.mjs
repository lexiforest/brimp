import { submitterActionCases } from './submitter-action-cases.mjs';
export const embeddedURLFields = [
  ['img', 'src'],
  ['img', 'lowsrc'],
  ['img', 'longDesc'],
  ['iframe', 'src'],
  ['iframe', 'longDesc'],
  ['frame', 'src'],
  ['frame', 'longDesc'],
  ['body', 'background'],
];
const extra = [
  'foo:opaque ?query#part',
  'data:text/plain, ?query',
  'mailto:a@b ?query',
  'file:///tmp/a b?é#中',
  'https://é.test/é?中#🦀',
  'blob:https://example.test/id',
  'https://example.test/?#',
  'http://[bad/\ud800',
];
export const embeddedURLCases = embeddedURLFields.flatMap(([tag, name]) => [
  ...submitterActionCases.map((c) => ({ tag, name, ...c })),
  ...extra.flatMap((value) =>
    ['attribute', 'property'].map((kind) => ({
      tag,
      name,
      base: 'https://reference.test/dir/',
      value,
      kind,
    })),
  ),
]);
export function measureEmbeddedURLs(cases) {
  return cases.map(({ tag, name, base, value, kind }) => {
    const owner = document.implementation.createHTMLDocument(),
      e = owner.createElement(tag),
      attribute = name.toLowerCase();
    if (base !== null) {
      const b = owner.createElement('base');
      b.setAttribute('href', base);
      owner.head.append(b);
    }
    owner.body.append(e);
    const observer = new MutationObserver(() => {});
    observer.observe(e, { attributes: true, attributeOldValue: true });
    let error = null;
    try {
      if (value !== null) {
        if (kind === 'attribute') e.setAttribute(attribute, value);
        else e[name] = value;
      }
    } catch (e) {
      error = e.name;
    }
    const result = e[name],
      raw = e.getAttribute(attribute),
      records = observer
        .takeRecords()
        .map((r) => [r.attributeName, r.attributeNamespace, r.oldValue]);
    observer.disconnect();
    return [error, result, raw, records];
  });
}
const conversions = [
  'undefined',
  'null',
  'false',
  '0',
  '1n',
  'Symbol()',
  '({})',
  '({[Symbol.toPrimitive](hint){if(hint!=="string")throw Error(hint);return "next"}})',
  '({toString(){throw new RangeError("conversion")}})',
  ...['', '\0', 'a\ud800b\udc00', 'http://[bad/\ud800', 'é?中#🦀'].map(JSON.stringify),
];
export const embeddedURLConversionCases = embeddedURLFields.flatMap(([tag, name]) =>
  conversions.map((source) => ({ tag, name, source })),
);
export function measureEmbeddedURLConversions(cases) {
  return cases.map(({ tag, name, source }) => {
    const owner = document.implementation.createHTMLDocument();
    owner.head.innerHTML = '<base href="https://reference.test/dir/">';
    const e = owner.createElement(tag),
      attribute = name.toLowerCase();
    e.setAttribute(attribute, 'before');
    const observer = new MutationObserver(() => {});
    observer.observe(e, { attributes: true, attributeOldValue: true });
    let error = null;
    try {
      e[name] = Function('return (' + source + ')')();
    } catch (e) {
      error = e.name;
    }
    const records = observer
      .takeRecords()
      .map((r) => [r.attributeName, r.attributeNamespace, r.oldValue]);
    observer.disconnect();
    return [error, e[name], e.getAttribute(attribute), records];
  });
}
