const roots = [
  'standard',
  'headless',
  'empty-html',
  'empty-xml',
  'html-root',
  'html-div',
  'svg-root',
  'svg-g',
  'xml-root',
];
const layouts = [
  'none',
  'html',
  'svg',
  'null',
  'nested-html',
  'nested-svg',
  'duplicate-html',
  'duplicate-svg',
  'body-html',
  'template-html',
  'shadow-html',
];
const values = [
  'undefined',
  'null',
  'false',
  '0',
  '1n',
  'Symbol()',
  ...['', ' New \t title ', '\0a\ud800b\udc00', '\u00a0a\vb\fc\r\nd\u2028'].map(JSON.stringify),
  '({[Symbol.toPrimitive](hint){return hint}})',
  '({toString(){throw new RangeError("conversion")}})',
];
export const documentTitleCases = roots.flatMap((root) =>
  layouts.flatMap((layout) => values.map((source) => ({ root, layout, source }))),
);
export const documentDirCases = roots.flatMap((root) =>
  ['', 'LTR', 'rtl', 'auto', ' auto ', 'bogus', '\ud800'].flatMap((raw) =>
    [...values, ...['ltr', 'RTL', 'auto', 'AUTO', ' auto ', 'bad'].map(JSON.stringify)].map(
      (source) => ({ root, raw, source }),
    ),
  ),
);

export function measureDocumentTitleDir(cases, property) {
  const H = 'http://www.w3.org/1999/xhtml',
    S = 'http://www.w3.org/2000/svg';
  const snap = (node) =>
    node.nodeType === 3 || node.nodeType === 8
      ? [node.nodeType, node.data]
      : [
          node.nodeType,
          node.localName ?? null,
          node.namespaceURI ?? null,
          ...[...node.childNodes].map(snap),
        ];
  function make(root) {
    if (['standard', 'headless', 'empty-html'].includes(root)) {
      const d = document.implementation.createHTMLDocument();
      if (root === 'headless') d.head.remove();
      if (root === 'empty-html') d.documentElement.remove();
      return d;
    }
    const d = new Document();
    const spec = {
      'html-root': [H, 'html'],
      'html-div': [H, 'div'],
      'svg-root': [S, 'svg'],
      'svg-g': [S, 'g'],
      'xml-root': [null, 'root'],
    }[root];
    if (spec) d.appendChild(d.createElementNS(...spec));
    return d;
  }
  return cases.map((c) => {
    const d = make(c.root),
      root = d.documentElement;
    if (property === 'title' && root && c.layout !== 'none') {
      let parent = d.head ?? root;
      if (c.layout === 'body-html') parent = d.body ?? root;
      if (c.layout.startsWith('nested-')) {
        const box = d.createElementNS(H, 'div');
        parent.appendChild(box);
        parent = box;
      }
      if (c.layout === 'template-html') {
        const t = d.createElementNS(H, 'template');
        parent.appendChild(t);
        parent = t.content;
      }
      if (c.layout === 'shadow-html') {
        const host = d.createElementNS(H, 'div');
        parent.appendChild(host);
        parent = host.attachShadow({ mode: 'open' });
      }
      const ns = c.layout.includes('svg') ? S : c.layout === 'null' ? null : H;
      const t = d.createElementNS(ns, 'title');
      parent.appendChild(t);
      t.appendChild(d.createTextNode(' A\t B '));
      t.appendChild(d.createComment('ignored'));
      const span = d.createElementNS(H, 'span');
      span.appendChild(d.createTextNode(' nested '));
      t.appendChild(span);
      t.appendChild(d.createTextNode(' C\n '));
      if (c.layout.startsWith('duplicate-')) {
        const second = d.createElementNS(ns, 'title');
        second.textContent = 'second';
        parent.appendChild(second);
      }
    }
    if (property === 'dir' && root) root.setAttribute('dir', c.raw);
    const before = d[property];
    const observer = new MutationObserver(() => {});
    observer.observe(d, {
      subtree: true,
      childList: true,
      characterData: true,
      characterDataOldValue: true,
      attributes: true,
      attributeOldValue: true,
    });
    let error = null;
    try {
      d[property] = Function('d', 'return (' + c.source + ')')(d);
    } catch (e) {
      error = e.name;
    }
    const records = observer
      .takeRecords()
      .map((r) => [
        r.type,
        r.target.localName ?? '#document',
        r.target.namespaceURI ?? null,
        r.attributeName,
        r.attributeNamespace,
        r.oldValue,
        [...r.addedNodes].map(snap),
        [...r.removedNodes].map(snap),
      ]);
    observer.disconnect();
    return {
      before,
      after: d[property],
      error,
      dir: root?.getAttribute('dir') ?? null,
      tree: snap(d),
      records,
    };
  });
}
