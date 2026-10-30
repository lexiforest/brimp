import { test, before, after } from 'node:test';
import assert from 'node:assert/strict';
import { createServer } from 'node:http';
import { once } from 'node:events';
import { readFile } from 'node:fs/promises';
import { createSession, Timeout, Closed } from '../index.js';
const fixture = new URL('../../fixtures/', import.meta.url);
let server, url;
before(async () => {
  server = createServer(async (req, res) => {
    if (req.url === '/slow') {
      res.writeHead(200, { 'Content-Type': 'application/wasm' });
      res.flushHeaders();
      setTimeout(() => res.end(), 500);
      return;
    }
    try {
      const data = await readFile(
        new URL(
          req.url === '/wasm-import.wasm' ? 'wasm-import.wasm' : 'xxhash-wasm' + req.url,
          fixture,
        ),
      );
      res.writeHead(200, {
        'Content-Type': req.url.endsWith('.wasm') ? 'application/wasm' : 'text/javascript',
      });
      for (let offset = 0; offset < data.length; offset += 127)
        res.write(data.subarray(offset, offset + 127));
      res.end();
    } catch {
      res.writeHead(404);
      res.end();
    }
  }).listen(0, '127.0.0.1');
  await once(server, 'listening');
  url = `http://127.0.0.1:${server.address().port}`;
});
after(async () => {
  server.closeAllConnections();
  await new Promise((resolve) => server.close(resolve));
});

test('real browser WASM package, fetched modules and streaming', async () => {
  const session = await createSession();
  try {
    const page = await session.newPage();
    const source = await readFile(new URL('wasm-workload.js', fixture), 'utf8');
    assert.deepEqual(await page.evaluate(`${source}\nrunWasmWorkload(${JSON.stringify(url)})`), {
      hash32: 'fb0077f9',
      hash64: '26c7827d889f6da3',
      incremental: 0x26c7827d889f6da3n,
      streamed32: 0xfb0077f9,
      streamed64: 0x26c7827d889f6da3n,
      module: true,
      imported: 42,
      linkError: 'LinkError',
      memory: [1, 131072],
      global: 2n,
      table: 'function',
      again: 'function',
    });
  } finally {
    await session.close();
  }
});

test('pending WASM loading respects timeout and page closure', async () => {
  let session = await createSession({ timeout: 150 });
  try {
    const page = await session.newPage();
    await assert.rejects(
      page.evaluate(`WebAssembly.compileStreaming(fetch(${JSON.stringify(url + '/slow')}))`),
      Timeout,
    );
    assert.equal(await page.evaluate('42'), 42);
  } finally {
    await session.close();
  }
  session = await createSession();
  try {
    const page = await session.newPage();
    const result = assert.rejects(
      page.evaluate(`WebAssembly.compileStreaming(fetch(${JSON.stringify(url + '/slow')}))`),
      Closed,
    );
    await new Promise((resolve) => setTimeout(resolve, 30));
    await page.close();
    await result;
  } finally {
    await session.close();
  }
});
