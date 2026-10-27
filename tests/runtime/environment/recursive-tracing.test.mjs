import test from 'node:test';
import assert from 'node:assert/strict';
import {createSession, Plugin} from '../../../bindings/node/index.js';

const fixture = new Plugin('trace-fixture', `({global}) => {
  const nested = {value: 1}; nested.self = nested;
  const failure = new Error('fixture failure');
  let reads = 0;
  global.fixture = {
    nested, alias: nested, failure,
    get lazy() { reads++; return nested; }, get reads() { return reads; },
    fail() { throw failure; }, echo(value) { return value; },
    callback(fn) { return fn(nested); },
    [Symbol.for('trace')]: nested,
    fixed: Object.freeze({child: nested}), bytes: new Uint8Array([1, 2]),
  };
}`);

test('recursive environment operations, aliases, symbols, invariants and safe logging', async () => {
  const session = await createSession({plugins: [fixture], diagnostics: {mode: 'recursive', stack: true}});
  try {
    const page = await session.newPage();
    assert.equal(await page.evaluate('fixture.reads'), 0);
    assert.deepEqual(await page.evaluate(`(() => {
      const n = fixture.nested;
      const same = n === fixture.alias && n === n.self && n === fixture[Symbol.for('trace')];
      n.value = 3; void n.missing;
      let getterReads = 0;
      const original = {get dangerous() { getterReads++; throw Error('must not inspect'); },
        toJSON() { throw Error('must not serialize'); }};
      const roundTrip = fixture.echo(original) === original;
      const callbackValue = fixture.callback(value => { void value.callbackMissing; return value.value; });
      const absent = 'missing' in n;
      Object.defineProperty(n, 'added', {value: 7, configurable: true});
      const added = Object.getOwnPropertyDescriptor(n, 'added').value;
      Reflect.ownKeys(n); delete n.added;
      const proto = Object.getPrototypeOf(n); Object.setPrototypeOf(n, proto);
      Object.isExtensible(n); Object.preventExtensions(n); Object.getPrototypeOf(n);
      const fixed = fixture.fixed.child.value;
      const bytes = ArrayBuffer.isView(fixture.bytes);
      void fixture['x'.repeat(600)];
      let message; try { fixture.fail(); } catch(e) { message = e.message; }
      return [same, absent, added, fixed, bytes, message, fixture.lazy === n, fixture.reads,
        roundTrip, getterReads, callbackValue];
    })()`), [true, false, 7, 3, true, 'fixture failure', true, 1, true, 0, 3]);
    const batch = await page.peekDiagnostics();
    assert.equal(batch.coverage.mode, 'recursive');
    assert.equal(batch.dropped, 0);
    const records = batch.records.filter(r => r.details?.layer === 'recursive');
    for (const kind of ['get', 'set', 'has', 'defineProperty', 'getOwnPropertyDescriptor', 'ownKeys', 'deleteProperty', 'getPrototypeOf', 'setPrototypeOf', 'isExtensible', 'preventExtensions', 'call']) {
      assert.ok(records.some(r => r.kind === kind), kind);
    }
    assert.ok(records.some(r => r.path === 'fixture.nested.missing' && r.outcome === 'undefined'));
    assert.ok(records.some(r => r.path === 'fixture.nested.callbackMissing' && r.outcome === 'undefined'));
    assert.ok(records.some(r => r.kind === 'call' && r.path === 'fixture.fail' && r.outcome === 'throw'));
    assert.ok(records.some(r => r.kind === 'coverage' && r.details.note.includes('Fixed')));
    assert.ok(records.some(r => r.kind === 'coverage' && r.details.note.includes('Binary')));
    assert.ok(records.some(r => r.details.property === 'Symbol(trace)'));
    assert.ok(records.every(r => typeof r.stack === 'string'));
    assert.ok(records.every(r => r.path.length <= 512 && (r.details.property?.length ?? 0) <= 512));
    const summary = await page.summarizeDiagnostics();
    assert.ok(summary.dependencies.some(r => r.layer === 'recursive' && r.target === 'fixture.nested' && r.property === 'missing'));
    assert.deepEqual(await page.takeDiagnostics(), batch);
  } finally { await session.close(); }
});

test('recursive DOM brands, native input, callback identity and asynchronous results', async () => {
  const session = await createSession({clock: {mode: 'logical'}, diagnostics: {mode: 'recursive'}});
  try {
    const page = await session.newPage();
    await page.evaluate(`
      document.body.innerHTML = '<button id="b">before</button>';
      globalThis.seen = [];
      const button = document.querySelector('#b');
      const removed = () => seen.push('removed');
      button.addEventListener('click', removed); button.removeEventListener('click', removed);
      button.addEventListener('click', function(e) {
        seen.push([this === button, e.target === button, e.currentTarget === this, e.isTrusted]);
        this.textContent = 'after';
      });
      button.addEventListener('click', {handleEvent(e) { seen.push(e.target.textContent); }});
      null;
    `);
    const button = await page.querySelector('#b');
    try { await button.click(); } finally { await button.dispose(); }
    assert.deepEqual(await page.evaluate('seen'), [[true, true, true, true], 'after']);
    assert.deepEqual(await page.evaluate(`[window === self, window === globalThis, window.document === document,
      document instanceof Document, document.body instanceof Node,
      Object.getOwnPropertyDescriptor(Node.prototype, 'nodeType').get.call(document.body)]`), [true, true, true, true, true, 1]);
    assert.equal(await page.evaluate(`new Promise(resolve => setTimeout(() => resolve(document.body.tagName), 5))`), 'BODY');
    assert.equal(await page.evaluate(`new Response('hello').text().then(value => value.toUpperCase())`), 'HELLO');
    assert.equal(await page.evaluate(`new Promise(resolve => {
      const observer = new MutationObserver(records => { observer.disconnect(); resolve(records[0].target === document.body); });
      observer.observe(document.body, {attributes: true}); document.body.setAttribute('data-x', '1');
    })`), true);
    const batch = await page.peekDiagnostics();
    assert.ok(batch.records.some(r => r.kind === 'construct' && r.path === 'Response'));
    assert.ok(batch.records.some(r => r.kind === 'set' && r.path.endsWith('.textContent')));
  } finally { await session.close(); }
});

test('recursive dynamic roots and bounded filtered diagnostics', async () => {
  const session = await createSession({diagnostics: {mode: 'recursive', capacity: 3, include: ['window.dynamic']}});
  try {
    const page = await session.newPage();
    await page.evaluate(`window.dynamic = {nested: {value: 0}};
      for (let i=0; i<6; i++) window.dynamic.nested.value++; null;`);
    const summary = await page.summarizeDiagnostics();
    assert.equal(summary.retained, 3);
    assert.ok(summary.dropped > 3);
    assert.ok(summary.dependencies.every(r => r.path.startsWith('window.dynamic')));
    await page.takeDiagnostics();
    assert.equal((await page.peekDiagnostics()).dropped, 0);
  } finally { await session.close(); }
});

test('recursive wrappers preserve canonical document URLs and input metadata', async () => {
  const session=await createSession({diagnostics:{mode:'recursive'}});
  try {
    const page=await session.newPage();
    await page.get('data:text/html,<title>URL identity</title>');
    assert.deepEqual(await page.evaluate(`(() => {
      const failures=[],input=document.createElement('input');
      if(input.formAction!==location.href||document.URL!==location.href)failures.push('main document URL');
      const owner=document.implementation.createHTMLDocument(),child=owner.createElement('input');
      const base=owner.createElement('base');base.href='https://example.test/base/';owner.head.append(base);
      child.formAction='submit';child.maxLength=12;child.minLength=4;child.formMethod='DIALOG';
      if(child.formAction!=='https://example.test/base/submit'||child.maxLength!==12||child.minLength!==4||child.formMethod!=='dialog')failures.push('created document');
      return failures;
    })()`),[]);
  } finally { await session.close(); }
});
