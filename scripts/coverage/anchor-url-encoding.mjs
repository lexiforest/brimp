import { createServer } from 'node:http';
export async function measureAnchorURLEncoding(page) {
  const labels = ['UTF-8', 'windows-1252', 'Big5', 'ISO-2022-JP'];
  const server = createServer((req, res) => {
    const encoding = labels[Number(req.url.slice(1))] ?? 'UTF-8';
    res.writeHead(200, { 'Content-Type': `text/html; charset=${encoding}` });
    res.end(`<!doctype html><meta charset="${encoding}"><body>`);
  });
  await new Promise((resolve) => server.listen(0, '127.0.0.1', resolve));
  try {
    const results = [];
    for (let i = 0; i < labels.length; ++i) {
      await page.get(`http://127.0.0.1:${server.address().port}/${i}`);
      results.push(
        await page.evaluate(`(() => {
        const value = 'https://example.test/é?X　☃中é🦀#é';
        const out = [];
        for (const kind of ['attribute', 'href', 'search']) {
          const a = document.createElement('a');
          if (kind === 'attribute') a.setAttribute('href', value);
          else if (kind === 'href') a.href = value;
          else { a.href = 'https://example.test/'; a.search = '?X　☃中é🦀'; }
          out.push([kind, a.href, a.search, a.getAttribute('href'), a.cloneNode().href]);
        }
        const a = document.createElement('a'); a.href = value;
        const other = document.implementation.createHTMLDocument();
        other.adoptNode(a);
        out.push(['adopted', a.href, a.search, a.getAttribute('href')]);
        return out;
      })()`),
      );
    }
    return results;
  } finally {
    await new Promise((resolve) => server.close(resolve));
  }
}
