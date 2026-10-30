import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import { createSession } from '../../../scripts/coverage/session.mjs';
import {
  embeddedURLFields,
  embeddedURLCases,
  measureEmbeddedURLs,
  embeddedURLConversionCases,
  measureEmbeddedURLConversions,
} from '../../../scripts/coverage/embedded-url-cases.mjs';
import { measureEmbeddedURLEncoding } from '../../../scripts/coverage/embedded-url-encoding.mjs';
const reference = JSON.parse(
  fs.readFileSync(new URL('../fixtures/embedded-urls.safari.json', import.meta.url)),
);
async function evaluate(source) {
  const s = await createSession();
  try {
    return await (await s.newPage()).evaluate(source);
  } finally {
    await s.close();
  }
}
for (const [key, cases, measure] of [
  ['urls', embeddedURLCases, measureEmbeddedURLs],
  ['conversions', embeddedURLConversionCases, measureEmbeddedURLConversions],
])
  test(`embedded URL ${key} match Safari resolution and mutation behavior`, async () => {
    assert.deepEqual(
      reference[key].map(({ expected, ...c }) => c),
      cases,
    );
    const actual = JSON.parse(
      await evaluate(`JSON.stringify((${measure})(${JSON.stringify(cases)}))`),
    );
    assert.deepEqual(
      reference[key].flatMap((c, i) =>
        JSON.stringify(c.expected) === JSON.stringify(actual[i])
          ? []
          : [{ ...c, actual: actual[i] }],
      ),
      [],
    );
  });

test('embedded URLs use live canonical bases, namespaces, cloning and adoption', async () => {
  assert.deepEqual(
    await evaluate(`(()=>{
    const failures=[];
    for(const[tag,name]of ${JSON.stringify(embeddedURLFields)}){
      const owner=document.implementation.createHTMLDocument(),other=document.implementation.createHTMLDocument(),attribute=name.toLowerCase();
      const resolved=(base,path)=>tag==='body'?path:base+path;
      owner.head.innerHTML='<base href="https://one.test/dir/"><base href="https://ignored.test/">';other.head.innerHTML='<base href="https://two.test/path/">';
      const e=owner.createElement(tag),d=Object.getOwnPropertyDescriptor(Object.getPrototypeOf(e),name);
      if(e[name]!=='')failures.push(name+' missing');e.setAttributeNS('urn:other',attribute,'foreign');if(e[name]!=='')failures.push('namespace');
      if(!d.enumerable||!d.configurable||d.get.name!=='get '+name||d.set.name!=='set '+name||d.get.length!==0||d.set.length!==1)failures.push('descriptor');
      for(const receiver of [{},document.createElement('div'),Object.create(Object.getPrototypeOf(e))]){
        let converted=false;try{d.set.call(receiver,{toString(){converted=true;return 'x'}});failures.push('receiver')}catch(err){if(err.name!=='TypeError'||converted)failures.push('brand conversion')}
      }
      e[name]='file';if(e[name]!==resolved('https://one.test/dir/','file'))failures.push('base');
      owner.head.firstChild.href='https://changed.test/';if(e[name]!==resolved('https://changed.test/','file'))failures.push('live base');
      const clone=e.cloneNode();other.adoptNode(e);
      if(e[name]!==resolved('https://two.test/path/','file')||clone[name]!==resolved('https://changed.test/','file'))failures.push('clone/adopt');
      const observer=new MutationObserver(()=>{});observer.observe(e,{attributes:true,attributeOldValue:true});
      e[name]={toString(){other.head.firstChild.href='https://conversion.test/';return 'next'}};
      if(e[name]!==resolved('https://conversion.test/','next')||observer.takeRecords().length!==1)failures.push('conversion order');
      e.getAttribute=e.setAttribute=e.hasAttribute=()=>{throw Error('public attributes')};Object.defineProperty(e,'ownerDocument',{get(){throw Error('public owner')}});
      Object.defineProperty(other,'URL',{get(){throw Error('public URL')}});Object.defineProperty(other,'baseURI',{get(){throw Error('public baseURI')}});
      e[name]='canonical';if(e[name]!==resolved('https://conversion.test/','canonical'))failures.push('canonical');
      observer.takeRecords();void e[name];if(observer.takeRecords().length)failures.push('read mutation');observer.disconnect();
    }
    return failures;
  })()`),
    [],
  );
});

test('embedded URL query encoding follows real document encodings and adoption', async () => {
  const s = await createSession();
  try {
    assert.deepEqual(await measureEmbeddedURLEncoding(await s.newPage()), reference.encoding);
  } finally {
    await s.close();
  }
});
