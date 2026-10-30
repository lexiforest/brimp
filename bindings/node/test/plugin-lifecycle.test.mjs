import { test } from 'node:test';
import assert from 'node:assert/strict';
import { createSession, Plugin } from '../index.js';
import { writeFile, mkdtemp, rm } from 'node:fs/promises';
import { join } from 'node:path';
import { tmpdir } from 'node:os';

test('realm library example receives options without exposing its installer', async () => {
  const plugin = Plugin.fromFile(
    new URL('../../../examples/plugins/realm-library.js', import.meta.url),
    { name: 'realm-library', options: { form: 'NFKC' } },
  );
  const session = await createSession({ plugins: [plugin] });
  try {
    const page = await session.newPage();
    assert.equal(await page.evaluate('textTools.normalize("Ａ")'), 'A');
    assert.equal(await page.evaluate('typeof installRealmLibrary'), 'undefined');
  } finally {
    await session.close();
  }
});

test('file plugins receive options and trusted session identity', async () => {
  const directory = await mkdtemp(join(tmpdir(), 'brimp-plugin-'));
  const path = join(directory, 'plugin.js');
  await writeFile(
    path,
    '({global, host, options}) => { global.config = options; global.context = () => host.call("context"); }',
  );
  const plugin = Plugin.fromFile(path, {
    name: 'file',
    options: { answer: 42 },
    handlers: {
      context: (c) => ({
        session: c.sessionId,
        page: c.pageId,
        realm: c.realmId,
        url: c.url,
        origin: c.origin,
      }),
    },
  });
  const first = await createSession({ plugins: [plugin] }),
    second = await createSession({ plugins: [plugin] });
  try {
    const a = await first.newPage(),
      b = await first.newPage(),
      c = await second.newPage();
    assert.deepEqual(await a.evaluate('config'), { answer: 42 });
    const [x, y, z] = await Promise.all([a, b, c].map((page) => page.evaluate('context()')));
    assert.equal(x.session, y.session);
    assert.notEqual(x.session, z.session);
    assert.notEqual(x.page, y.page);
    assert.equal(x.origin, null);
    assert.deepEqual(
      await a.evaluate(`(() => {
      let rejected = false;
      try {
        Object.defineProperty(globalThis, 'location', {value: {href: 'https://spoof.test'}, configurable: true});
      } catch (error) {
        rejected = error instanceof TypeError;
      }
      return [rejected, location.href !== 'https://spoof.test/'];
    })()`),
      [true, true],
    );
    assert.deepEqual(await a.evaluate('context()'), x);
  } finally {
    await first.close();
    await second.close();
    await rm(directory, { recursive: true, force: true });
  }
});

test('file plugin paths and host exception stacks stay outside the page', async () => {
  const directory = await mkdtemp(join(tmpdir(), 'brimp-private-'));
  const path = join(directory, 'private-plugin.js');
  await writeFile(
    path,
    '({global,host}) => { global.fail = () => host.call("fail"); global.localFailure = () => {throw new Error("public detail")}; }',
  );
  const plugin = Plugin.fromFile(path, {
    name: 'private',
    handlers: {
      fail() {
        throw new Error(`private host detail: ${path}`);
      },
    },
  });
  const session = await createSession({ plugins: [plugin] });
  try {
    const page = await session.newPage();
    const observed = new Promise((resolve) => page.on('error', resolve));
    const result = await page.evaluate(
      'fail().catch(e => ({name:e.name,message:e.message,stack:e.stack}))',
    );
    assert.equal(result.name, 'OperationError');
    assert.equal(result.message, 'The operation failed.');
    assert.ok(!JSON.stringify(result).includes(path));
    assert.match((await observed).message, /private host detail/);
    const stack = await page.evaluate('try {localFailure()} catch(e) {e.stack}');
    assert.ok(!stack.includes(path) && !stack.includes('file:///'));
    assert.ok(!(await page.evaluate('localFailure.toString()')).includes('public detail'));
  } finally {
    await session.close();
  }
  await writeFile(path, '() => {throw new Error("installation detail")}');
  const broken = await createSession({ plugins: [Plugin.fromFile(path, { name: 'broken' })] });
  try {
    await assert.rejects(
      broken.newPage(),
      (error) =>
        error.message.includes('installation detail') &&
        error.message.includes('private-plugin.js'),
    );
  } finally {
    await broken.close();
    await rm(directory, { recursive: true, force: true });
  }
});

test('session plugin state survives navigation, isolates origins/sessions, and disposes once', async () => {
  const { createServer } = await import('node:http');
  const server = createServer((_request, response) =>
    response.end('<!doctype html><p>fixture</p>'),
  );
  await new Promise((resolve) => server.listen(0, '127.0.0.1', resolve));
  const url = `http://127.0.0.1:${server.address().port}/`;
  let started = 0,
    disposed = 0;
  const plugin = Plugin.fromFile(
    new URL('../../../examples/plugins/preferences.js', import.meta.url),
    {
      name: 'preferences',
      setup() {
        started++;
        const state = new Map();
        const partition = (c) => {
          if (!c.origin) throw new Error('opaque');
          if (!state.has(c.origin)) state.set(c.origin, new Map());
          return state.get(c.origin);
        };
        return {
          handlers: {
            get: (c, k) => partition(c).get(k) ?? null,
            set: (c, k, v) => {
              partition(c).set(k, v);
              return null;
            },
          },
          dispose: () => {
            disposed++;
            state.clear();
          },
        };
      },
    },
  );
  const a = await createSession({ plugins: [plugin] }),
    b = await createSession({ plugins: [plugin] });
  try {
    const p = await a.get(url);
    await p.evaluate('preferences.set("theme","dark")');
    await p.get(url + 'next');
    assert.equal(await p.evaluate('preferences.get("theme")'), 'dark');
    const other = await a.get(url);
    assert.equal(await other.evaluate('preferences.get("theme")'), 'dark');
    const isolated = await b.get(url);
    assert.equal(await isolated.evaluate('preferences.get("theme")'), null);
    assert.equal(started, 2);
  } finally {
    await a.close();
    await a.close();
    await b.close();
    await new Promise((resolve) => server.close(resolve));
  }
  assert.equal(disposed, 2);
});

test('session plugin setup is rolled back if later setup fails', async () => {
  let disposed = 0;
  await assert.rejects(
    createSession({
      plugins: [
        new Plugin('one', '() => {}', {}, {}, () => ({
          dispose() {
            disposed++;
          },
        })),
        new Plugin('two', '() => {}', {}, {}, () => {
          throw new Error('setup failed');
        }),
      ],
    }),
    /setup failed/,
  );
  assert.equal(disposed, 1);
});
