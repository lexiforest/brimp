import { test } from 'node:test';
import assert from 'node:assert/strict';
import { createSession, Plugin, Reentry } from '../index.js';

const source = `async ({global, host}) => {
  const value = await host.call('init', {binary: new Uint8Array([3,5,8])});
  global.initial = Array.from(value.binary);
  global.messages = [];
  host.subscribe('message', value => messages.push(value));
  global.send = name => host.call(name);
  return async () => { await host.call('dispose'); };
}`;
test('async installers, Promise handlers, binary, notifications, errors, disposal', async () => {
  const disposed = [];
  const plugin = new Plugin('demo', source, {
    async init(context, value) {
      await new Promise(resolve => setTimeout(resolve, 10));
      assert.deepEqual(value, {binary: Buffer.from([3,5,8])});
      return value;
    },
    async notify(context) { for (let n = 0; n < 3; n++) context.emit('message', n); return null; },
    fail() { throw new Error('handler detail'); },
    dispose(context) { disposed.push(context.realmId); return null; },
  });
  const session = await createSession({plugins: [plugin]});
  try {
    const page = await session.newPage();
    assert.deepEqual(await page.evaluate('initial'), [3,5,8]);
    assert.deepEqual(await page.evaluate('send("notify").then(() => messages)'), [0,1,2]);
    assert.match(await page.evaluate('send("fail").catch(String)'), /OperationError/);
    assert.match(await page.evaluate('send("missing").catch(String)'), /OperationError/);
    await page.close();
  } finally { await session.close(); }
  assert.equal(disposed.length, 1);
});
test('async callback reentry rejected for page, handles and session', async () => {
  let page, other, handle, session;
  const plugin = new Plugin('reentry', '({global, host}) => {global.check = () => host.call("check");}', {
    async check() {
      await new Promise(resolve => setTimeout(resolve, 5));
      for (const action of [() => page.evaluate('1'), () => page.close(), () => session.close(), () => session.newPage(), () => handle.jsonValue()]) {
        await assert.rejects(action, Reentry);
      }
      return other.evaluate('6 * 7');
    },
  });
  session = await createSession({plugins:[plugin]});
  try {
    page = await session.newPage(); other = await session.newPage();
    handle = await page.evaluateHandle('({a:1})');
    assert.equal(await page.evaluate('check()'), 42);
    assert.equal(await page.evaluate('1 + 1'), 2);
    await handle.dispose();
  } finally { await session.close(); }
});
test('closure cancels callback and ignores late completion', async () => {
  let entered, finish, context;
  const started = new Promise(resolve => { entered = resolve; });
  const blocked = new Promise(resolve => { finish = resolve; });
  const plugin = new Plugin('pending', '({global, host}) => {global.wait = () => host.call("wait");}', {
    async wait(ctx) { context = ctx; entered(); await blocked; return Buffer.from('late'); },
  });
  const session = await createSession({plugins:[plugin]});
  try {
    const page = await session.newPage();
    const pending = assert.rejects(page.evaluate('wait()'), /closed/);
    await started;
    await page.close();
    assert.equal(context.cancelled, true);
    finish();
    await pending;
  } finally { finish(); await session.close(); }
});
test('installer failure includes plugin name and original error', async () => {
  const session = await createSession({plugins:[new Plugin('broken', '() => {throw new Error("install detail");}')]});
  try { await assert.rejects(session.newPage(), /broken.*install detail/); }
  finally { await session.close(); }
});

test('navigation cancels host calls and discards results from the previous realm', async () => {
  let entered, finish, context;
  const started = new Promise(resolve => { entered = resolve; });
  const blocked = new Promise(resolve => { finish = resolve; });
  const plugin = new Plugin('navigation',
    '({global,host}) => { global.wait = () => host.call("wait").then(v => global.late = v); }', {
      async wait(ctx) { context = ctx; entered(); await blocked; return 'old'; },
    });
  const session = await createSession({plugins:[plugin]});
  try {
    const page = await session.newPage();
    const pending = assert.rejects(page.evaluate('wait()'), /cancelled/);
    await started;
    await page.get('data:text/html,<title>new realm</title>');
    assert.equal(context.cancelled, true);
    finish();
    await pending;
    assert.deepEqual(await page.evaluate('[document.title, typeof late]'), ['new realm', 'undefined']);
  } finally { finish(); await session.close(); }
});

test('graphics and audio example uses state and explicit asynchronous host extensions', async () => {
  const {readFileSync} = await import('node:fs');
  const source = readFileSync(new URL('../../../examples/plugins/graphics-audio.js', import.meta.url), 'utf8');
  const plugin = new Plugin('graphics-audio', source, {
    async render(_context, command) {
      assert.deepEqual(command.clearColor, [1,0,0,1]);
      return Buffer.from([command.width, command.height]);
    },
    async processAudio(_context, command) {
      assert.equal(command.sampleRate, 48000);
      return command.samples;
    },
  });
  const session = await createSession({plugins:[plugin]});
  try {
    const page = await session.newPage();
    assert.deepEqual(await page.evaluate(`(async () => {
      const graphics = new MockWebGPU(12,34); graphics.setClearColor(1,0,0,1);
      const audio = new MockAudioContext(); await audio.resume();
      return [graphics.getParameter('MAX_TEXTURE_SIZE'), Array.from(await graphics.renderAsync([])),
              audio.state, Array.from(await audio.processAsync(new Uint8Array([2,4])))];
    })()`), [4096,[12,34],'running',[2,4]]);
  } finally { await session.close(); }
});

test('bundled plugins, direct input and extraction', async () => {
  const {storagePlugin, canvasPlugin, servicesPlugin, extractionPlugin} = await import('../index.js');
  const session = await createSession({plugins:[storagePlugin(), canvasPlugin(), servicesPlugin(), extractionPlugin()]});
  try {
    const page = await session.newPage();
    await page.evaluate("document.body.innerHTML='<article><h1>Native article</h1><p>This article is read from a DOM without a layout engine.</p></article><input id=edit><input id=check type=checkbox>'; null");
    await page.fill('#edit', 'ab'); await page.typeText('#edit', 'c'); await page.press('#edit', 'Backspace');
    assert.equal(await page.evaluate("document.querySelector('#edit').value"), 'ab');
    const checkbox = await page.querySelector('#check');
    await checkbox.click(); await checkbox.dispose();
    assert.equal(await page.evaluate("document.querySelector('#check').checked"), true);
    assert.match((await page.extract({contentSelector:'article'})).contentMarkdown, /DOM/);
    assert.equal(await page.evaluate("localStorage.x='yes';localStorage.getItem('x')"), 'yes');
    assert.deepEqual(await page.evaluate("Array.from(document.createElement('canvas').getContext('2d').getImageData(0,0,1,1).data)"), [0,0,0,0]);
  } finally { await session.close(); }
});
