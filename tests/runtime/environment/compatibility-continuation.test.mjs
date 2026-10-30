import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtemp, readFile, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { createSession, canvasPlugin, Plugin } from '../../../bindings/node/index.js';

test('blank-page storage works, remains isolated and is never persisted', async () => {
  const dir = await mkdtemp(join(tmpdir(), 'brimp-opaque-'));
  const storagePath = join(dir, 'storage.json');
  const session = await createSession({ storagePath });
  try {
    const a = await session.newPage(),
      b = await session.newPage();
    assert.deepEqual(
      await a.evaluate(`(async () => {
      localStorage.setItem('opaque-secret','one'); sessionStorage.setItem('opaque-secret','two');
      const cache = await caches.open('opaque-cache');
      await cache.put('https://fixture.test/resource',new Response('cached'));
      const db = await new Promise((resolve,reject) => {
        const r=indexedDB.open('opaque-db'); r.onupgradeneeded=()=>r.result.createObjectStore('items');
        r.onsuccess=()=>resolve(r.result); r.onerror=()=>reject(r.error);
      });
      await new Promise((resolve,reject)=>{
        const t=db.transaction('items','readwrite');t.objectStore('items').put('stored','key');
        t.oncomplete=resolve;t.onabort=()=>reject(t.error);
      });
      db.close();
      await cookieStore.set('ignored','value');
      document.body.textContent='continued';
      return [localStorage.getItem('opaque-secret'),sessionStorage.getItem('opaque-secret'),
        await (await cache.match('https://fixture.test/resource')).text(),
        (await indexedDB.databases()).map(d=>d.name),await navigator.storage.persisted(),
        await cookieStore.getAll(),document.body.textContent];
    })()`),
      ['one', 'two', 'cached', ['opaque-db'], false, [], 'continued'],
    );
    assert.deepEqual(
      await b.evaluate(`Promise.all([localStorage.getItem('opaque-secret'),
      sessionStorage.getItem('opaque-secret'),caches.keys(),indexedDB.databases()])`),
      [null, null, [], []],
    );
    await b.route('https://fixture.test/**', (route) => route.fulfill({ body: '<body></body>' }));
    await b.get('https://fixture.test/');
    await b.evaluate(`localStorage.setItem('normal','saved'); null`);
    assert.equal((await readFile(storagePath, 'utf8')).includes('opaque-secret'), false);
    await a.get('data:text/html,<body>new realm</body>');
    assert.deepEqual(
      await a.evaluate(`Promise.all([localStorage.getItem('opaque-secret'),
      sessionStorage.getItem('opaque-secret'),caches.keys(),indexedDB.databases()])`),
      [null, null, [], []],
    );
  } finally {
    await session.close();
    await rm(dir, { recursive: true, force: true });
  }
});

test('incomplete cookie calls settle harmlessly while valid cookies remain functional', async () => {
  const session = await createSession();
  try {
    const page = await session.newPage();
    await page.route('https://fixture.test/**', (route) =>
      route.fulfill({ body: '<body></body>' }),
    );
    await page.get('https://fixture.test/');
    assert.deepEqual(
      await page.evaluate(`(async () => {
      await Promise.all([cookieStore.set(),cookieStore.set(null),cookieStore.delete(),
        cookieStore.set('bad;name','ignored'),cookieStore.set('name','bad;value')]);
      const empty = await cookieStore.get(null);
      await cookieStore.set('valid','value');
      const value = (await cookieStore.get('valid')).value;
      await cookieStore.delete('valid');
      document.body.textContent='continued';
      return [empty,value,await cookieStore.getAll(null),document.body.textContent];
    })()`),
      [null, 'value', [], 'continued'],
    );
  } finally {
    await session.close();
  }
});

test('locks accept legacy option combinations and abort held requests without blocking successors', async () => {
  const session = await createSession();
  try {
    const page = await session.newPage();
    assert.deepEqual(
      await page.evaluate(`(async () => {
      const signal = new AbortController().signal;
      const results = [];
      for (const options of [{ifAvailable:true,signal},{steal:true,signal}])
        results.push(await navigator.locks.request('compat',options,lock=>lock.name));
      const controller = new AbortController();
      let started;const ready=new Promise(resolve=>started=resolve);
      const held=navigator.locks.request('held',{signal:controller.signal},()=>{
        started();return new Promise(()=>{});
      }).catch(error=>error);
      await ready; controller.abort('cancelled');
      results.push(await held);
      results.push(await navigator.locks.request('held',lock=>lock.name));
      results.push((await navigator.locks.query()).held.length);
      document.body.textContent='continued';results.push(document.body.textContent);
      return results;
    })()`),
      ['compat', 'compat', 'cancelled', 'held', 0, 'continued'],
    );
  } finally {
    await session.close();
  }
});

test('canvas normalizes context names and allows a later plugin to replace a factory', async () => {
  const override = new Plugin(
    'custom-canvas',
    `({canvas})=>{
    canvas.register('2D',function(){return {canvas:this,marker:'custom'}});
  }`,
  );
  for (const plugins of [[canvasPlugin()], [canvasPlugin(), override]]) {
    const session = await createSession({ plugins });
    try {
      const page = await session.newPage();
      assert.deepEqual(
        await page.evaluate(`(() => {
        const c=document.createElement('canvas'),context=c.getContext('2D');
        return [context===c.getContext('2d'),context.canvas===c,context.marker??'builtin'];
      })()`),
        [true, true, plugins.length === 1 ? 'builtin' : 'custom'],
      );
    } finally {
      await session.close();
    }
  }
});
