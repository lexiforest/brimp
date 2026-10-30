export const embeddedInterfaces = [
  ['body', 'HTMLBodyElement'],
  ['frame', 'HTMLFrameElement'],
  ['iframe', 'HTMLIFrameElement'],
  ['img', 'HTMLImageElement'],
];
export const embeddedFields = [
  ...['text', 'link', 'vLink', 'aLink', 'bgColor'].map((name) => [
    'body',
    name,
    name.toLowerCase(),
    'null-empty',
  ]),
  ...['name', 'scrolling', 'frameBorder'].map((name) => [
    'frame',
    name,
    name.toLowerCase(),
    'string',
  ]),
  ...['marginHeight', 'marginWidth'].map((name) => [
    'frame',
    name,
    name.toLowerCase(),
    'null-empty',
  ]),
  ['frame', 'noResize', 'noresize', 'boolean'],
  ...['srcdoc', 'name', 'width', 'height', 'align', 'scrolling', 'frameBorder', 'allow'].map(
    (name) => ['iframe', name, name.toLowerCase(), 'string'],
  ),
  ...['marginHeight', 'marginWidth'].map((name) => [
    'iframe',
    name,
    name.toLowerCase(),
    'null-empty',
  ]),
  ['iframe', 'allowFullscreen', 'allowfullscreen', 'boolean'],
  ['iframe', 'loading', 'loading', 'enum'],
  ['iframe', 'referrerPolicy', 'referrerpolicy', 'enum'],
  ['iframe', 'sandbox', 'sandbox', 'tokens'],
  ...['alt', 'useMap', 'name', 'align', 'sizes'].map((name) => [
    'img',
    name,
    name.toLowerCase(),
    'string',
  ]),
  ['img', 'srcset', 'srcset', 'usv-string'],
  ['img', 'border', 'border', 'null-empty'],
  ['img', 'isMap', 'ismap', 'boolean'],
  ...['crossOrigin', 'referrerPolicy', 'decoding', 'loading'].map((name) => [
    'img',
    name,
    name.toLowerCase(),
    'enum',
  ]),
];
const sources = [
  'undefined',
  'null',
  'false',
  'true',
  '0',
  '1n',
  'Symbol()',
  '({})',
  '({[Symbol.toPrimitive](hint){if(hint!=="string")throw Error("hint");return "converted"}})',
  '({toString(){throw new RangeError("conversion")}})',
  ...['', ' Mixed value ', '\0', 'a\ud800b\udc00', '😀', 'a\r\nb', 'allow-scripts allow-forms'].map(
    JSON.stringify,
  ),
];
const keywords = [
  'lazy',
  'eager',
  'auto',
  'async',
  'sync',
  'anonymous',
  'use-credentials',
  'no-referrer',
  'no-referrer-when-downgrade',
  'same-origin',
  'origin',
  'strict-origin',
  'origin-when-cross-origin',
  'strict-origin-when-cross-origin',
  'unsafe-url',
];
export const embeddedAttributeCases = embeddedFields.flatMap(([tag, name, attribute, kind]) => [
  ...sources.map((source) => ({ tag, name, attribute, kind, source, raw: false })),
  ...(kind === 'enum'
    ? [
        ...keywords,
        ...keywords.map((s) => s.toUpperCase()),
        ...keywords.map((s) => ' ' + s),
        ...keywords.map((s) => s + ' '),
        'ſame-origin',
        'ORİGIN',
      ].map((value) => ({ tag, name, attribute, kind, source: JSON.stringify(value), raw: true }))
    : []),
]);
export function measureEmbeddedAttributes(cases) {
  return cases.map(({ tag, name, attribute, kind, source, raw }) => {
    const e = document.createElement(tag);
    e.setAttribute(attribute, 'before');
    const observer = new MutationObserver(() => {});
    observer.observe(e, { attributes: true, attributeOldValue: true });
    let error = null;
    try {
      const value = Function('return (' + source + ')')();
      if (raw) e.setAttribute(attribute, value);
      else e[name] = value;
    } catch (e) {
      error = e.name;
    }
    const value = kind === 'tokens' ? e[name]?.value : e[name],
      records = observer
        .takeRecords()
        .map((r) => [r.attributeName, r.attributeNamespace, r.oldValue]);
    observer.disconnect();
    return [error, value, e.getAttribute(attribute), records];
  });
}
export const sandboxTokens = [
  'allow-downloads',
  'allow-forms',
  'allow-modals',
  'allow-orientation-lock',
  'allow-pointer-lock',
  'allow-popups',
  'allow-popups-to-escape-sandbox',
  'allow-presentation',
  'allow-same-origin',
  'allow-scripts',
  'allow-storage-access-by-user-activation',
  'allow-top-navigation',
  'allow-top-navigation-by-user-activation',
  'allow-top-navigation-to-custom-protocols',
  'ALLOW-SCRIPTS',
  'unknown',
  '',
  ' allow-scripts',
  'allow-scripts ',
];
export function measureSandboxSupport(tokens) {
  const e = document.createElement('iframe');
  return tokens.map((token) => {
    try {
      return [token, e.sandbox.supports(token)];
    } catch (error) {
      return [token, error.name];
    }
  });
}
