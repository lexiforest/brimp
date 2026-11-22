import { test } from 'node:test';
import assert from 'node:assert/strict';
import { createServer } from 'node:http';
import { once } from 'node:events';
import { createSession, Timeout } from '../index.js';

for (const engine of ['runtime', 'webkit']) {
  test(
    `${engine}: shared navigation, value, input and cancellation contract`,
    {
      skip: engine === 'webkit' && !process.env.BRIMP_WEBKIT_EXECUTABLE,
    },
    async () => {
      const body = Buffer.from(
        '<meta charset=windows-1252><title>caf\xe9</title><input id=field style="width:150px;height:30px"><button id=go style="width:100px;height:40px">Go</button><script>document.querySelector("#go").onclick = e => window.clicked = e.isTrusted;</script>',
        'latin1',
      );
      const server = createServer((request, response) => {
        response.writeHead(201, { 'Content-Type': 'text/html; charset=windows-1252' });
        response.end(body);
      }).listen(0, '127.0.0.1');
      await once(server, 'listening');
      let session;
      try {
        session = await createSession({
          engine,
          plugins: [],
          timeout: 15000,
          executablePath: engine === 'webkit' ? process.env.BRIMP_WEBKIT_EXECUTABLE : undefined,
        });
        const page = await session.newPage();
        await page.get(`http://127.0.0.1:${server.address().port}/`);
        assert.equal(page.statusCode, 201);
        assert.deepEqual(page.content, body);
        assert.equal(await page.evaluate('document.title'), 'caf\xe9');
        await page.locator('#field').fill('hello');
        assert.equal(await page.evaluate('document.querySelector("#field").value'), 'hello');
        await page.locator('#go').click();
        assert.equal(await page.evaluate('clicked'), true);
        const handle = await page.evaluateHandle('({answer:42})');
        assert.deepEqual(await handle.jsonValue(), { answer: 42 });
        await handle.dispose();
        await assert.rejects(handle.jsonValue(), /stale/i);
        await assert.rejects(page.waitForSelector('#missing', { timeout: 50 }), Timeout);
        const abort = new AbortController();
        const pending = assert.rejects(
          page.waitForFunction('false', { signal: abort.signal, timeout: 0 }),
          /cancelled/,
        );
        setTimeout(() => abort.abort(), 50);
        await pending;
        assert.equal(await page.evaluate('6 * 7'), 42);
        assert.deepEqual(
          await page.evaluate(
            'Object.getOwnPropertyNames(window).filter(k => k.startsWith("__brimp"))',
          ),
          [],
        );
      } finally {
        await session?.close();
        server.closeAllConnections();
        await new Promise((resolve) => server.close(resolve));
      }
    },
  );
}
