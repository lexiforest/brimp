import {test} from 'node:test';
import assert from 'node:assert/strict';
import {createSession,pluginCatalog,hostStoragePlugin} from '../index.js';

test('plugin discovery and atomic host storage lifetime/origin boundaries',async()=>{
 const catalog=pluginCatalog();assert.equal(catalog.storage.lifetime,'realm');assert.equal(catalog['host-storage'].options.backendFactory.default,'session-memory');
 catalog.canvas.options.rgba.default[0]=255;assert.equal(pluginCatalog().canvas.options.rgba.default[0],0);
 const plugin=hostStoragePlugin(),session=await createSession({plugins:[plugin]});
 try {
  const page=await session.newPage();await page.route('https://*.test/**',r=>r.fulfill({body:'<p>storage</p>',headers:{'content-type':'text/html'}}));
  await assert.rejects(page.evaluate('hostStorage.set("x",1)'));
  await page.get('https://a.test/');await page.evaluate('hostStorage.set("x",{n:42})');
  await page.get('https://a.test/reload');assert.deepEqual(await page.evaluate('hostStorage.get("x")'),{n:42});
  const other=await session.newPage();await other.route('https://*.test/**',r=>r.fulfill({body:'<p>other</p>',headers:{'content-type':'text/html'}}));await other.get('https://a.test/');
  assert.deepEqual(await other.evaluate('hostStorage.get("x")'),{n:42});
  await assert.rejects(other.evaluate('hostStorage.transaction([{type:"set",key:"x",value:0},{type:"invalid"}])'));
  assert.deepEqual(await page.evaluate('hostStorage.get("x")'),{n:42});
  await other.get('https://b.test/');assert.equal(await other.evaluate('hostStorage.get("x")'),null);
  assert.equal(await page.evaluate('typeof localStorage'), 'undefined');
 }finally{await session.close();}
 const fresh=await createSession({plugins:[plugin]});
 try{const page=await fresh.newPage();await page.route('https://a.test/**',r=>r.fulfill({body:'ok',headers:{'content-type':'text/html'}}));await page.get('https://a.test/');assert.equal(await page.evaluate('hostStorage.get("x")'),null);}finally{await fresh.close();}
});

test('custom async backend batches serialize per origin and close after work',async()=>{
 let active=0,maximum=0,closed=0;
 const plugin=hostStoragePlugin({backendFactory:()=>({async transaction(origin,ops){assert.equal(origin,'https://a.test');maximum=Math.max(maximum,++active);await new Promise(r=>setTimeout(r,5));active--;return ops.map(()=>42);},close(){assert.equal(active,0);closed++;}})});
 const session=await createSession({plugins:[plugin]});
 try{const page=await session.newPage();await page.route('https://a.test/**',r=>r.fulfill({body:'ok',headers:{'content-type':'text/html'}}));await page.get('https://a.test/');assert.deepEqual(await page.evaluate('Promise.all([hostStorage.get("a"),hostStorage.get("b")])'),[42,42]);assert.equal(maximum,1);}finally{await session.close();}
 assert.equal(closed,1);
});
