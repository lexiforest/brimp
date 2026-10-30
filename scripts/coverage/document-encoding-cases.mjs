import { createServer } from 'node:http';

export const documentEncodingCases = [];
const add = (id, head = '', header = 'text/html', textHex = 'e9', wire = 'latin1', bom = '') =>
  documentEncodingCases.push({ id, head, header, textHex, wire, bom });
add('unlabelled');
add('unlabelled UTF-8 bytes', '', 'text/html', 'c3a9');
for (const label of [
  'utf-8',
  'iso-8859-1',
  'windows-1252',
  'utf-16',
  'utf-16be',
  'x-user-defined',
  'replacement',
  'bogus',
  'ascii',
  'Big5',
  'ISO-2022-JP',
  'Shift_JIS',
  'EUC-KR',
]) {
  add(`header ${label}`, '', `text/html; charset=${label}`);
  add(`meta ${label}`, `<meta charset="${label}">`);
}
for (const header of [
  'text/html; charset=" UTF-8 "',
  'text/html; CHARSET="utf-8"',
  'text/html; charset=bogus',
  'text/html; charset=windows-1252; charset=utf-8',
  'text/html; charset=""; charset=utf-8',
  'text/html; foo="x;charset=windows-1252"; charset=utf-8',
])
  add(`header precedence ${header}`, '<meta charset=utf-8>', header);
for (const tag of [
  'script',
  'style',
  'title',
  'textarea',
  'noscript',
  'template',
  'svg',
  'xmp',
  'iframe',
  'noembed',
  'noframes',
])
  add(`nested ${tag}`, `<${tag}><meta charset=utf-8></${tag}>`);
for (const prefix of ['<!--', '<head>', '<body>', '<body><p>'])
  for (const size of [990, 1000, 1010, 1024, 1100, 5000])
    add(
      `scan ${prefix} ${size}`,
      prefix + ' '.repeat(size) + (prefix === '<!--' ? '-->' : '') + '<meta charset=utf-8>',
    );
for (const head of [
  '<meta charset=bogus><meta charset=utf-8>',
  '<meta charset="utf&#45;8">',
  '<meta charset=bogus charset=utf-8>',
  '<meta charset=utf-8 charset=bogus>',
  '<meta http-equiv=content-type content="text/html; charset=utf-8">',
  '<meta content="text/html; charset=utf-8" http-equiv=CONTENT-TYPE>',
  '<meta content="text/html; charset=utf-8">',
  '<meta http-equiv=" content-type " content="text/html; charset=utf-8">',
  '<meta charset=bogus http-equiv=content-type content="text/html; charset=utf-8">',
  '<meta content="text/html; charset=windows-1252" http-equiv=content-type charset=utf-8>',
  '<meta http-equiv=content-type content="charset=\'utf-8\'">',
  '<meta http-equiv=content-type content="xcharset = utf-8; other=x">',
  '<meta http-equiv=content-type content="charset=bogus; charset=utf-8">',
  '<!--<meta charset=windows-1252>--><meta charset=utf-8>',
  '<script>"<meta charset=windows-1252>"</script><meta charset=utf-8>',
  '<div data-meta="<meta charset=windows-1252>"></div><meta charset=utf-8>',
])
  add(`markup ${head}`, head);
for (const [label, textHex] of [
  ['windows-1252', 'e980'],
  ['Big5', 'a4a4a4e5'],
  ['Shift_JIS', '93fa967b'],
  ['ISO-2022-JP', '1b2442467c4b5c1b2842'],
  ['utf-8', 'c3a9e29883f09fa680'],
]) {
  add(`decoded header ${label}`, '', `text/html; charset=${label}`, textHex);
  add(`decoded meta ${label}`, `<meta charset=${label}>`, 'text/html', textHex);
}
for (const [wire, bom, textHex] of [
  ['utf8', 'efbbbf', 'c3a9'],
  ['utf16le', 'fffe', 'e900'],
  ['utf16be', 'feff', '00e9'],
])
  for (const header of ['text/html', 'text/html; charset=windows-1252', 'text/html; charset=utf-8'])
    add(`BOM ${wire} ${header}`, '<meta charset=Big5>', header, textHex, wire, bom);
for (const wire of ['utf16le', 'utf16be'])
  add(
    `header ${wire} real bytes`,
    '',
    `text/html; charset=${wire === 'utf16le' ? 'UTF-16LE' : 'UTF-16BE'}`,
    wire === 'utf16le' ? 'e900' : '00e9',
    wire,
  );

function encoded(text, wire) {
  if (wire === 'utf16be') return Buffer.from(text, 'utf16le').swap16();
  return Buffer.from(text, wire);
}
export function documentEncodingBody(c, trailer = '') {
  const text = Buffer.from(c.textHex, 'hex');
  return Buffer.concat([
    Buffer.from(c.bom, 'hex'),
    encoded('<!doctype html>' + c.head + '<body><p id="sample">', c.wire),
    text,
    encoded(
      '</p><script>globalThis.encodingDuringParse=document.characterSet;globalThis.decodedScript="',
      c.wire,
    ),
    text,
    encoded('";</script>' + trailer, c.wire),
  ]);
}
export function measureDocumentEncoding() {
  const read = (d) => [d.characterSet, d.charset, d.inputEncoding];
  const before = read(document),
    clones = [read(document.cloneNode(false)), read(document.cloneNode(true))];
  const created = [
    read(new DOMParser().parseFromString('<meta charset=Big5>', 'text/html')),
    read(document.implementation.createHTMLDocument()),
    read(new Document()),
    read(document.createElement('template').content.ownerDocument),
  ];
  document.querySelectorAll('meta').forEach((e) => e.remove());
  const meta = document.createElement('meta');
  meta.setAttribute('charset', 'Shift_JIS');
  document.head.appendChild(meta);
  const after = read(document);
  Object.defineProperty(document, 'characterSet', {
    configurable: true,
    get() {
      throw Error('public getter');
    },
  });
  let aliases;
  try {
    aliases = [document.charset, document.inputEncoding];
  } finally {
    delete document.characterSet;
  }
  document.__characterSet = 'spoofed';
  const canonical = read(document);
  delete document.__characterSet;
  return {
    before,
    clones,
    created,
    after,
    aliases,
    canonical,
    text: document.getElementById('sample')?.textContent ?? null,
    during: globalThis.encodingDuringParse ?? null,
    script: globalThis.decodedScript ?? null,
  };
}
export async function measureDocumentEncodings(page, cases = documentEncodingCases) {
  const server = createServer((req, res) => {
    const c = cases[Number(req.url.slice(1))];
    if (!c) {
      res.writeHead(404);
      res.end();
      return;
    }
    res.writeHead(200, { 'Content-Type': c.header });
    res.end(documentEncodingBody(c));
  });
  await new Promise((resolve) => server.listen(0, '127.0.0.1', resolve));
  try {
    const results = [];
    for (let i = 0; i < cases.length; i++) {
      await page.get(`http://127.0.0.1:${server.address().port}/${i}`);
      results.push(
        JSON.parse(await page.evaluate(`JSON.stringify((${measureDocumentEncoding})())`)),
      );
    }
    return results;
  } finally {
    server.closeAllConnections();
    await new Promise((resolve) => server.close(resolve));
  }
}
