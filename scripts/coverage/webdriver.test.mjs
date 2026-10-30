import test from 'node:test';
import assert from 'node:assert/strict';
import { createSafariSession } from './webdriver.mjs';
import { createSession } from './session.mjs';

test('Safari adapter requests Safari, runs the exact assertion source and closes its session', async (t) => {
  const calls = [];
  t.mock.method(globalThis, 'fetch', async (url, options) => {
    calls.push({
      path: new URL(url).pathname,
      method: options.method,
      body: options.body && JSON.parse(options.body),
    });
    const value =
      calls.length === 1
        ? { sessionId: 'reference', capabilities: { browserName: 'safari' } }
        : url.endsWith('/execute/sync')
          ? { ok: true, value: { failures: [], observed: null } }
          : null;
    return { ok: true, json: async () => ({ value }) };
  });
  const session = await createSafariSession();
  const page = await session.newPage();
  const source = '(() => ({failures: [], observed: null}))()';
  assert.deepEqual(await page.evaluate(source), { failures: [], observed: null });
  await session.close();
  assert.deepEqual(
    calls.map((c) => [c.method, c.path]),
    [
      ['POST', '/session'],
      ['POST', '/session/reference/url'],
      ['POST', '/session/reference/execute/sync'],
      ['DELETE', '/session/reference'],
    ],
  );
  assert.equal(calls[0].body.capabilities.alwaysMatch.browserName, 'safari');
  assert.match(decodeURIComponent(calls[1].body.url), /<!doctype html>/);
  assert.equal(calls[2].body.args[0], source);
  assert.match(calls[2].body.args[1], /^__brimpCoverage_/);
  assert.match(calls[2].body.script, /document.createElement\('script'\)/);
});

test('Safari session refusal propagates and unsupported browser choices fail', async (t) => {
  const prior = process.env.COVERAGE_BROWSER;
  t.after(() => {
    if (prior === undefined) delete process.env.COVERAGE_BROWSER;
    else process.env.COVERAGE_BROWSER = prior;
  });
  t.mock.method(globalThis, 'fetch', async () => ({
    ok: false,
    json: async () => ({
      value: { error: 'session not created', message: 'Allow Remote Automation is disabled' },
    }),
  }));
  process.env.COVERAGE_BROWSER = 'safari';
  await assert.rejects(createSession(), /Allow Remote Automation is disabled/);
  process.env.COVERAGE_BROWSER = 'chromium';
  await assert.rejects(createSession(), /must be brimp or safari/);
});

test('Safari page-script errors and failed script execution cannot produce a passing result', async (t) => {
  let result = {
    ok: false,
    name: 'TypeError',
    message: 'reference failure',
    stack: 'global code@reference:1',
  };
  t.mock.method(globalThis, 'fetch', async (url) => ({
    ok: true,
    json: async () => ({
      value: url.endsWith('/session')
        ? { sessionId: 'reference' }
        : url.endsWith('/execute/sync')
          ? result
          : null,
    }),
  }));
  const session = await createSafariSession();
  try {
    const page = await session.newPage();
    await assert.rejects(page.evaluate('throw Error()'), (error) => {
      assert.equal(error.name, 'TypeError');
      assert.match(error.stack, /TypeError: reference failure/);
      return true;
    });
    result = null;
    await assert.rejects(page.evaluate('42'), /did not execute/);
  } finally {
    await session.close();
  }
});
