import {submitterActionCases} from './submitter-action-cases.mjs';
export const anchorURLFields = ['href', 'origin', 'protocol', 'username', 'password', 'host', 'hostname', 'port', 'pathname', 'search', 'hash'];
export const anchorURLReadCases = submitterActionCases;
const hrefs = [null, '', '../relative?old#part', 'http://[bad', 'https://user:pass@EXAMPLE.test:8443/a/b?old#part',
  'http://example.test/', 'file:///tmp/a?old#part', 'mailto:test@example.test?old#part',
  'data:text/plain,x?old#part', 'foo://host:8000/a?old#part', 'foo:/a?old#part', 'foo:opaque ?old#part',
  'blob:https://example.test/id', 'https://example.test/?#'];
const common = ['undefined', 'null', 'true', '0', '1n', 'Symbol("x")', '({toString(){throw new RangeError("conversion")}})', '({[Symbol.toPrimitive](hint){return hint}})'];
const values = {
  href: ['', '../next', 'https://next.test/é?中#🦀', 'http://[bad', 'https://x/\ud800', '?x\0', ' #fragment'],
  protocol: ['', 'http', 'HTTPS:', 'http:ignored', 'file:', 'foo:', '1invalid', ' ht\ntp:'],
  username: ['', 'user:name@ /?', '\ud800', 'é🦀', 'a\tb\nc'],
  password: ['', 'pass:name@ /?', '\udc00', 'é🦀', 'a\tb\nc'],
  host: ['', 'next.test', 'next.test:123', 'next.test:', 'next.test:443', 'next.test:bad', 'next.test:12junk', '[::1]:80', 'user@next.test', ' next.test', 'next.test/path', 'next.test:99999', 'é.test'],
  hostname: ['', 'next.test', 'next.test:123', '[::1]', 'next.test/path', 'next.test?x', ' next.test', 'é.test', 'a\tb.test'],
  port: ['', '80', '443', '00012', '12junk', '+12', '-1', '65535', '65536', ' 80', '8\t0'],
  pathname: ['', 'next', '/a/../b', '\\next', 'é/🦀', '/x?y#z', '/\ud800'],
  search: ['', '?', '??two', 'next', '?x=é🦀', '?a#b', '?\ud800', 'a\tb\nc'],
  hash: ['', '#', '##two', 'next', '#é🦀', '#a?b', '#\udc00', 'a\tb\nc'],
};
export const anchorURLWriteCases = [null, 'https://reference.test/dir/'].flatMap(base => hrefs.flatMap(href =>
  Object.entries(values).flatMap(([name, strings]) => [...common, ...strings.map(v => JSON.stringify(v))].map(source => ({base, href, name, source})))));
// Host parsing has separate rejection and partial-update rules in Safari.
for (const href of ['https://u:p@original.test:8443/p', 'file://original.test/p', 'foo://host:8000/p']) {
  for (const name of ['host', 'hostname']) for (const value of [':', '::1', 'a:b:c', 'host:8\t0', 'host:65536', '[::1]:bad', '[::1]:99999', 'a@b', 'a b', 'bad[host'])
    anchorURLWriteCases.push({base: null, href, name, source: JSON.stringify(value)});
}
export function measureAnchorURLs(cases, writing = false) {
  const fields = ['href', 'origin', 'protocol', 'username', 'password', 'host', 'hostname', 'port', 'pathname', 'search', 'hash'];
  return cases.map(c => {
    const owner = document.implementation.createHTMLDocument(), a = owner.createElement('a');
    if (c.base !== null) { const b = owner.createElement('base'); b.setAttribute('href', c.base); owner.head.append(b); }
    owner.body.append(a);
    let error = null;
    if (writing) { if (c.href !== null) a.setAttribute('href', c.href); }
    const observer = new MutationObserver(() => {});
    observer.observe(a, {attributes: true, attributeOldValue: true});
    try {
      if (writing) a[c.name] = (0, eval)(c.source);
      else if (c.value !== null) { if (c.kind === 'attribute') a.setAttribute('href', c.value); else a.href = c.value; }
    } catch (e) { error = e.name; }
    const result = fields.map(name => { try { return a[name]; } catch (e) { return {error: e.name}; } });
    let string; try { string = a.toString(); } catch (e) { string = {error: e.name}; }
    const records = observer.takeRecords().map(r => [r.attributeName, r.attributeNamespace, r.oldValue]);
    observer.disconnect();
    return {error, values: result, string, attribute: a.getAttribute('href'), records};
  });
}
