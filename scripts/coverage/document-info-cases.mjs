import { createServer } from 'node:http';

export const documentInfoCases = [
  { id: 'HTML', path: '/html?x=1#fragment', contentType: 'text/html' },
  {
    id: 'HTML uppercase with parameters',
    path: '/upper#fragment',
    contentType: 'TEXT/HTML; charset=UTF-8',
  },
  { id: 'HTML quoted charset', path: '/quoted', contentType: 'text/html; charset="utf-8"' },
  { id: 'missing Content-Type', path: '/missing#fragment', contentType: null },
  { id: 'plain document', path: '/plain#fragment', contentType: 'text/plain; charset=utf-8' },
  { id: 'JSON document', path: '/json#fragment', contentType: 'application/json' },
  {
    id: 'redirect target fragment',
    path: '/redirect#initial',
    contentType: 'text/html',
    redirect: '/landing?x=2#target',
  },
  {
    id: 'redirect inherits fragment',
    path: '/redirect#initial',
    contentType: 'text/html',
    redirect: '/landing?x=2',
  },
  {
    id: 'redirect clears fragment',
    path: '/redirect#initial',
    contentType: 'text/html',
    redirect: '/landing?x=2#',
  },
  {
    id: 'redirect chain inherits replacement fragment',
    path: '/redirect#initial',
    contentType: 'text/html',
    redirect: ['/hop#middle', '/landing'],
  },
  {
    id: 'redirect chain preserves cleared fragment',
    path: '/redirect#initial',
    contentType: 'text/html',
    redirect: ['/hop#', '/landing'],
  },
  {
    id: 'escaped URL',
    path: '/path/%E9%9B%AA/%C3%A9?x=%E2%98%83#%C3%A9',
    contentType: 'text/html',
  },
];

export function measureDocumentInfo() {
  // Brimp exposes response metadata for non-HTML resources without rendering a
  // text/JSON viewer. Supply a DOM for the subsequent metadata-mutation checks.
  if (!document.documentElement) {
    const html = document.createElement('html');
    html.appendChild(document.createElement('head'));
    html.appendChild(document.createElement('body'));
    document.appendChild(html);
  }
  const read = (d) => [d.URL, d.documentURI, d.contentType];
  const result = { initial: read(document) };
  const docs = [
    new Document(),
    new DOMParser().parseFromString(
      '<!doctype html><base href="https://base.test/"><p>parsed</p>',
      'text/html',
    ),
    document.implementation.createHTMLDocument('created'),
    document.createElement('template').content.ownerDocument,
    document.cloneNode(false),
    document.cloneNode(true),
  ];
  result.created = docs.map(read);
  result.clones = docs.map((d) => [read(d.cloneNode(false)), read(d.cloneNode(true))]);
  const meta = document.createElement('meta');
  meta.httpEquiv = 'content-type';
  meta.content = 'application/json';
  document.head.appendChild(meta);
  const base = document.createElement('base');
  base.href = 'https://base.test/';
  document.head.appendChild(base);
  result.afterMetadata = read(document);
  history.replaceState(null, '', location.origin + '/changed?x=3#history');
  result.history = read(document);
  result.retained = docs.map(read);
  result.createdAfterHistory = read(new DOMParser().parseFromString('', 'text/html'));
  const all = [document, ...docs];
  result.canonical = all.map((d) => {
    const expected = read(d),
      getURL = Object.getOwnPropertyDescriptor(Document.prototype, 'URL').get,
      getType = Object.getOwnPropertyDescriptor(Document.prototype, 'contentType').get;
    Object.defineProperty(d, 'URL', {
      configurable: true,
      get() {
        throw Error('public URL');
      },
    });
    d.__contentType = 'spoofed';
    let aliases;
    try {
      aliases = [
        getURL.call(d),
        d.documentURI,
        getType.call(d),
        read(Node.prototype.cloneNode.call(d, false)),
      ];
    } finally {
      delete d.URL;
      delete d.__contentType;
    }
    return { expected, aliases };
  });
  result.open = docs.map((d) => {
    let error = null;
    const before = read(d);
    try {
      d.open('ignored', 'replace');
      d.write('<p>written</p>');
      d.close();
    } catch (e) {
      error = e.name;
    }
    return { before, after: read(d), error };
  });
  const frame = document.createElement('iframe');
  document.body.appendChild(frame);
  result.frame = read(frame.contentDocument);
  const frameDoc = frame.contentDocument;
  frameDoc.open();
  frameDoc.write('<p>frame</p>');
  frameDoc.close();
  result.frameWritten = [read(frameDoc), frame.contentWindow.location.href];
  frame.remove();
  result.removedFrame = read(frameDoc);
  history.replaceState(null, '', location.origin + '/after-frame#new');
  frameDoc.open();
  frameDoc.write('<p>detached frame</p>');
  frameDoc.close();
  result.removedFrameWritten = read(frameDoc);
  const before = read(document);
  document.open();
  document.write('<p>main</p>');
  document.close();
  result.mainOpen = [before, read(document)];
  return result;
}

export async function measureDocumentInfos(page, cases = documentInfoCases) {
  let current;
  const server = createServer((req, res) => {
    if (req.url.startsWith('/redirect') && current.redirect) {
      res.writeHead(302, {
        Location: Array.isArray(current.redirect) ? current.redirect[0] : current.redirect,
      });
      res.end();
      return;
    }
    if (req.url === '/hop' && Array.isArray(current.redirect)) {
      res.writeHead(302, { Location: current.redirect[1] });
      res.end();
      return;
    }
    res.writeHead(200, current.contentType === null ? {} : { 'Content-Type': current.contentType });
    res.end(
      '<!doctype html><html><head><meta charset=utf-8><title>Metadata</title></head><body>fixture</body></html>',
    );
  });
  await new Promise((resolve) => server.listen(0, '127.0.0.1', resolve));
  const origin = `http://127.0.0.1:${server.address().port}`;
  try {
    const results = [];
    for (const c of cases) {
      current = c;
      await page.get(origin + c.path);
      results.push(
        JSON.parse(
          (await page.evaluate(`JSON.stringify((${measureDocumentInfo})())`))
            .split(origin)
            .join('$ORIGIN'),
        ),
      );
    }
    return results;
  } finally {
    server.closeAllConnections();
    await new Promise((resolve) => server.close(resolve));
  }
}
