import test from 'node:test';
import assert from 'node:assert/strict';
import {mkdtemp,readFile,rm} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {createSession,geometryPlugin,cssTypedOmPlugin,schedulingPlugin,graphicsPlugin,audioPlugin} from '../index.js';

test('logical clock, diagnostics and optional plugin arithmetic',async()=>{
  const dir=await mkdtemp(join(tmpdir(),'brimp-debug-'));
  const path=join(dir,'trace.jsonl');
  const session=await createSession({clock:{mode:'logical',epochMs:1000},diagnostics:{path,include:['Document.prototype','Reflect'],capacity:8},plugins:[geometryPlugin(),cssTypedOmPlugin(),schedulingPlugin()]});
  try {
    const page=await session.newPage();
    await page.evaluate('globalThis.seen=[];setTimeout(()=>seen.push([Date.now(),performance.now()]),20);null');
    assert.equal((await page.advanceTime(20)).now,20);
    assert.deepEqual(await page.evaluate('seen'),[[1020,20]]);
    assert.equal(await page.evaluate('new Promise(r=>setTimeout(()=>r(Date.now()),10000))'),11020);
    assert.equal(await page.evaluate("CSS.s(2).to('ms').value"),2000);
    await page.evaluate("document.querySelector('main');Reflect.ownKeys({a:1});null");
    const records=await page.takeDiagnostics();
    assert.ok(records.records.some(r=>r.path==='Document.prototype.querySelector'));
    assert.ok(records.records.every(r=>'pageId' in r && 'realmId' in r));
    assert.deepEqual((await page.takeDiagnostics()).records,[]);
    assert.equal(await (await session.newPage()).evaluate('performance.now()'),0);
  }finally {await session.close();}
  assert.ok((await readFile(path,'utf8')).trim().split('\n').map(JSON.parse).length>=2);
  await rm(dir,{recursive:true});
});
test('graphics/audio host extensions remain asynchronous',async()=>{
  const session=await createSession({clock:{mode:'logical'},plugins:[graphicsPlugin({}, {'graphics.submit':async(context,value)=>value.width}),audioPlugin({}, {'audio.process':async(context,value)=>value.samples})]});
  try{
    const page=await session.newPage();
    assert.equal(await page.evaluate("document.createElement('canvas').getContext('webgl').submitAsync([])"),300);
    assert.deepEqual(await page.evaluate('new AudioContext().processAsync(new Uint8Array([1,2])).then(v=>Array.from(v))'),[1,2]);
    await assert.rejects(()=>page.evaluate('new AudioContext().decodeAudioData(new ArrayBuffer(1))'),/decoding unavailable/);
  }finally{await session.close();}
});

test('inspect and summarize a retained function without consuming its trace', async () => {
  const session = await createSession({clock: {mode: 'logical', epochMs: 1000}, diagnostics: {include: ['Document.prototype.querySelector']}});
  try {
    const page = await session.newPage();
    const fn = await page.evaluateHandle('(() => { let calls = 0; return async n => { await new Promise(r => setTimeout(r, 5)); document.querySelector("body"); return [n + ++calls, Date.now()]; }; })()');
    try {
      await page.takeDiagnostics();
      for (const expected of [[41, 1005], [42, 1010]]) {
        const result = await fn.call('(fn, n) => fn(n)', 40);
        try { assert.deepEqual(await result.jsonValue(), expected); }
        finally { await result.dispose(); }
      }
      const before = await page.peekDiagnostics();
      const summary = await page.summarizeDiagnostics();
      assert.equal(summary.coverage.enabled, true);
      assert.equal(summary.writeError, null);
      assert.deepEqual(summary.dependencies, [{layer: 'api', path: 'Document.prototype.querySelector', kind: 'call', target: null, property: null, count: 2, throws: 0}]);
      assert.deepEqual(await page.takeDiagnostics(), before);
      assert.equal((await page.summarizeDiagnostics()).retained, 0);
    } finally { await fn.dispose(); }
  } finally { await session.close(); }
});
