import {test} from 'node:test';
import assert from 'node:assert/strict';
import {createServer} from 'node:http';
import {createNodeDOM} from '../node-dom.mjs';
import {until} from '../scenarios.mjs';

for (const engine of ['jsdom', 'happy-dom']) {
  test(`${engine}: unchanged scripts, DOM input, async values, reload and resource routing`, async () => {
    const server = createServer((request, response) => {
      if (request.url === '/driver.js') {
        response.setHeader('Content-Type', 'application/javascript');
        response.end(`
          window.events = [];
          const field = document.querySelector('input');
          for (const type of ['input','change','keydown','keypress','keyup']) {
            field.addEventListener(type, event => events.push([type, field.value, event.key || null]));
          }
          document.querySelector('button').onclick = () => document.querySelector('output').textContent = field.value;
          console.warn('captured warning');
          setTimeout(() => { throw new Error('captured page error'); }, 0);
          window.loaded = true;
        `);
      } else {
        response.setHeader('Content-Type', 'text/html');
        response.end('<input><button>Copy</button><output></output><script defer src="/driver.js"></script><script src="http://outside.invalid/blocked.js"></script>');
      }
    });
    await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
    const origin = `http://127.0.0.1:${server.address().port}`;
    const records = {errors: [], console: [], blockedRequests: []};
    const adapter = await createNodeDOM(engine, origin, (key, value) => records[key].push(value));
    try {
      const {page} = adapter;
      await page.goto(origin);
      await until(page, 'window.loaded === true');
      await page.fill('input', 'Ada');
      await page.press('input', 'Enter');
      await page.click('button');
      assert.equal(await page.evaluate('document.querySelector("output").textContent'), 'Ada');
      assert.deepEqual(await page.evaluate('Promise.resolve(events)'), [
        ['input', 'Ada', null], ['change', 'Ada', null],
        ['keydown', 'Ada', 'Enter'], ['keypress', 'Ada', 'Enter'], ['keyup', 'Ada', 'Enter'],
      ]);
      assert.equal(await page.evaluate('typeof process'), 'undefined');
      await page.evaluate('document.cookie = "fixture=present"; window.previousRealm = true; null');
      await page.goto(origin);
      assert.equal(await page.evaluate('typeof previousRealm'), 'undefined');
      assert.equal(await page.evaluate('document.cookie'), 'fixture=present');
      await until(page, 'window.loaded === true');
      assert.ok(records.console.some(message => message.includes('captured warning')));
      assert.ok([...records.errors, ...records.console].some(message => message.includes('captured page error')));
      assert.ok(records.blockedRequests.includes('http://outside.invalid/blocked.js'));
      await assert.rejects(page.goto('http://outside.invalid/'), /External HTTP blocked/);
      await assert.rejects(page.evaluate('Promise.reject(new Error("rejected evaluation"))'), /rejected evaluation/);
    } finally {
      await adapter.close();
      await new Promise(resolve => server.close(resolve));
    }
  });
}
