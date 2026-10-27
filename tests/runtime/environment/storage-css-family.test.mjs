import test from 'node:test';
import assert from 'node:assert/strict';
import {createSession} from '../../../bindings/node/index.js';

async function evaluate(source, plugins = []) {
  const session = await createSession({plugins});
  try {
    const page = await session.newPage();
    await page.route('https://core.test/**', route => route.fulfill({body:'<!doctype html><body></body>',headers:{'content-type':'text/html'}}));
    await page.get('https://core.test/');
    return await page.evaluate(source);
  }
  finally { await session.close(); }
}

test('storage names and cache lifecycle work without disk or filesystem services', async () => {
  assert.deepEqual(await evaluate(`(async () => {
    localStorage.answer=42;localStorage.setItem('second','value');sessionStorage.answer='session';
    const before=[localStorage.answer,localStorage.getItem('answer'),localStorage.key(0),localStorage.length,
      Object.keys(localStorage),sessionStorage.answer];
    delete localStorage.answer;localStorage.removeItem('second');localStorage.clear();
    const cache=await caches.open('empty'),created=[await caches.has('empty'),await caches.keys()];
    await cache.put('https://example.test/item',new Response('stored',{headers:{'X-Value':'yes'}}));
    const response=await caches.match('https://example.test/item');
    const stored=[await response.text(),response.headers.get('x-value')];
    await cache.put('https://example.test/binary',new Response(Uint8Array.of(0,255,128)));
    const bytes=Array.from(new Uint8Array(await (await caches.match('https://example.test/binary')).arrayBuffer()));
    const estimate=await navigator.storage.estimate();
    return [before,localStorage.length,created,stored,bytes,estimate.usage>0,estimate.quota,
      await navigator.storage.persist(),await navigator.storage.persisted(),navigator.storage.getDirectory()===undefined,
      await caches.delete('empty'),await caches.has('empty'),await caches.keys(),(await caches.match('https://example.test/item'))===undefined];
  })()`,[]),[['42','42','answer',2,['answer','second'],'session'],0,[true,['empty']],['stored','yes'],[0,255,128],true,null,false,false,true,true,false,[],true]);
});

test('IndexedDB instances retain vendor transactions, requests, schema upgrades and version-change events', async () => {
  assert.deepEqual(await evaluate(`(async () => {
    const resultOf=r=>new Promise((resolve,reject)=>{r.onsuccess=()=>resolve(r.result);r.onerror=()=>reject(r.error);});
    const opening=indexedDB.open('family',1),initial=[opening instanceof IDBOpenDBRequest,opening instanceof IDBRequest,
      opening.readyState,opening.source,opening.transaction,opening.onblocked,opening.onupgradeneeded];
    opening.onupgradeneeded=()=>{const db=opening.result;db.createObjectStore('items');db.createObjectStore('temporary');db.deleteObjectStore('temporary');};
    const db=await resultOf(opening);
    const metadata=[db.name,db.version,[...db.objectStoreNames],db.onabort,db.onclose,db.onerror,db.onversionchange,
      opening.readyState,opening.error,opening.transaction];
    const tx=db.transaction('items','readwrite'),store=tx.objectStore('items');
    const committed=new Promise((resolve,reject)=>{tx.oncomplete=resolve;tx.onabort=()=>reject(tx.error);});
    const put=store.put({answer:42},'key');
    const request=[put.source===store,put.transaction===tx,put.readyState,put.onsuccess,put.onerror];
    await resultOf(put);await committed;
    const found=await resultOf(db.transaction('items').objectStore('items').get('key'));
    const databases=await indexedDB.databases();let changed=0;
    db.onversionchange=()=>{changed++;db.close();};
    const upgrade=indexedDB.open('family',2);upgrade.onupgradeneeded=()=>upgrade.result.createObjectStore('more');
    const newer=await resultOf(upgrade),version=newer.version;newer.close();
    await resultOf(indexedDB.deleteDatabase('family'));
    return [initial,metadata,request,found,databases,changed,version,await indexedDB.databases(),indexedDB.cmp('a','b')];
  })()`,[]),[[true,true,'pending',null,null,null,null],['family',1,['items'],null,null,null,null,'done',null,null],
    [true,true,'pending',null,null],{answer:42},[{name:'family',version:1}],1,2,[],-1]);
});

test('CSS declarations retain DOM mutation and stylesheet ownership while registration is a no-op', async () => {
  assert.deepEqual(await evaluate(`(() => {
    const el=document.createElement('div');el.id='a:b';document.body.appendChild(el);
    const style=el.style;style.cssText='width: 12px; height: 7px;';style.cssFloat='left';
    style.setProperty('color','red','important');style.setProperty('--value','custom');
    const values=[style instanceof CSSStyleDeclaration,style.length,style.item(0),style.cssFloat,style.getPropertyValue('width'),
      style.getPropertyPriority('color'),style.getPropertyValue('--value'),style.parentRule,
      el.cloneNode().style.width,el.getAttribute('style').includes('width: 12px')];
    const sheet=new CSSStyleSheet();sheet.replaceSync('div { width: 4px; }');const rule=sheet.cssRules[0];
    const registered=CSS.registerProperty({name:'--registered',syntax:'<length>',inherits:false,initialValue:'3px'});
    return [values,style.removeProperty('height'),style.getPropertyValue('height'),rule.style.parentRule===rule,
      document.querySelector('#'+CSS.escape(el.id))===el,CSS.supports('width','4px'),CSS.supports('width','not-a-length'),
      registered===undefined,style.getPropertyValue('--registered')];
  })()`),[[true,5,'width','left','12px','important','custom',null,'12px',true],'7px','',true,true,true,false,true,'']);
});

test('geometry rectangles and CSS unit factories perform local arithmetic without layout', async () => {
  assert.deepEqual(await evaluate(`(() => {
    const r=new DOMRect(10,20,-4,-6),ro=DOMRectReadOnly.fromRect(r),initial=r.toJSON();r.x=3;r.width=5;
    const units=['number','percent','px','in','cm','mm','Q','pt','pc','deg','rad','grad','turn','s','ms','Hz','kHz',
      'dpi','dpcm','dppx','em','rem','ex','ch','vw','vh','vmin','vmax','fr'];
    return [initial,ro.toJSON(),r.x,r.width,r.left,r.right,
      units.every(unit=>CSS[unit](3).unit===unit&&CSS[unit](3).value===3),CSS.in(1).to('px').value,
      CSS.s(2).to('ms').value,CSS.turn(0.5).to('deg').value,CSS.percent(50).toString(),
      r instanceof DOMRectReadOnly,ro instanceof DOMRectReadOnly];
  })()`,[]),[
    {x:10,y:20,width:-4,height:-6,top:14,right:10,bottom:20,left:6},
    {x:10,y:20,width:-4,height:-6,top:14,right:10,bottom:20,left:6},3,5,3,8,true,96,2000,180,'50%',true,true]);
});
