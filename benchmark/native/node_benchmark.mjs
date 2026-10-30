import { createServer } from 'node:http';
import { once } from 'node:events';
import { performance } from 'node:perf_hooks';
import assert from 'node:assert/strict';
import { createSession } from '../../bindings/node/index.js';
const html =
  '<!doctype html><main>' +
  "<p class='item'>text</p>".repeat(3000) +
  '</main><script>globalThis.loaded = true;</script>';
const server = createServer((request, response) => {
  response.writeHead(200, { 'content-type': 'text/html' });
  response.end(html);
}).listen(0, '127.0.0.1');
await once(server, 'listening');
const start = performance.now();
try {
  const session = await createSession();
  try {
    const page = await session.newPage();
    for (let i = 0; i < 20; i++) {
      await page.get(`http://127.0.0.1:${server.address().port}`);
      assert.equal(await page.evaluate('loaded'), true);
      const node = await page.querySelector('main > .item');
      await node.setTextContent('edited');
      assert.equal(await node.textContent(), 'edited');
      await node.dispose();
      assert.equal(await page.evaluate("document.querySelectorAll('.item').length"), 3000);
    }
  } finally {
    await session.close();
  }
  console.log(
    JSON.stringify({
      iterations: 20,
      elapsed_ms: performance.now() - start,
      max_rss_kib: process.resourceUsage().maxRSS,
    }),
  );
} finally {
  server.closeAllConnections();
  await new Promise((resolve) => server.close(resolve));
}
