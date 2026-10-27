import test from 'node:test';import assert from 'node:assert/strict';import fs from 'node:fs';
import {createSession} from '../../../scripts/coverage/session.mjs';import *as corpus from '../../../scripts/coverage/input-image-cases.mjs';
async function evaluate(source){const s=await createSession();try{return await(await s.newPage()).evaluate(source);}finally{await s.close();}}
for(const[key,cases,measure]of [['sources',corpus.inputSourceCases,corpus.measureInputSources],['dimensions',corpus.inputDimensionCases,corpus.measureInputDimensions]])
test(`input image ${key} match ${cases.length} Safari reflection cases`,async()=>{
 const ref=JSON.parse(fs.readFileSync(new URL('../fixtures/input-images.safari.json',import.meta.url)))[key];assert.deepEqual(ref.map(({expected,...c})=>c),cases);
 const actual=JSON.parse(await evaluate(`JSON.stringify((${measure})(${JSON.stringify(cases)}))`));
 assert.deepEqual(ref.flatMap((c,i)=>JSON.stringify(c.expected)===JSON.stringify(actual[i])?[]:[{...c,actual:actual[i]}]),[]);
});

test('input image reflections have Safari descriptors and canonical conversion/attribute behavior',async()=>{
 assert.deepEqual(await evaluate(`(()=>{
 const failures=[];
 for(const name of ['width','height','src']){
 const i=document.createElement('input');i.type='image';const d=Object.getOwnPropertyDescriptor(HTMLInputElement.prototype,name);
 if(!d.enumerable||!d.configurable||d.get.name!=='get '+name||d.set.name!=='set '+name||d.get.length!==0||d.set.length!==1)failures.push([name,'descriptor']);
 let hints=[];const value={[Symbol.toPrimitive](hint){hints.push(hint);return name==='src'?'https://example.test/a':12.9}};
 d.set.call(i,value);if(hints.join()!==(name==='src'?'string':'number'))failures.push([name,'hint']);
 for(const receiver of [{},document.createElement('div'),Object.create(HTMLInputElement.prototype)]){
 hints=[];try{d.set.call(receiver,value);failures.push('receiver')}catch(e){if(e.name!=='TypeError'||hints.length)failures.push('brand before conversion')}
 try{d.get.call(receiver);failures.push('getter receiver')}catch(e){if(e.name!=='TypeError')failures.push(e.name)}
 }
 const observer=new MutationObserver(()=>{});observer.observe(i,{attributes:true,attributeOldValue:true});
 const before=i.getAttribute(name);for(const bad of name==='src'?[Symbol()]:[Symbol(),1n,Object(1n)])try{i[name]=bad;failures.push('conversion accepted')}catch(e){if(e.name!=='TypeError')failures.push(e.name)}
 if(i.getAttribute(name)!==before||observer.takeRecords().length)failures.push('failed mutation');
 i.setAttributeNS('urn:test',name,'99');i[name]=name==='src'?'https://example.test/b':7;
 if(i.getAttributeNS('urn:test',name)!=='99')failures.push('namespace');
 Object.defineProperty(i,'type',{value:'text'});i.getAttribute=i.setAttribute=i.hasAttribute=()=>{throw Error('override')};
 if(i[name]!== (name==='src'?'https://example.test/b':7))failures.push('canonical');
 observer.disconnect();
 }return failures;
 })()`),[]);
});

test('input src follows live document bases and adoption while width/height follow the canonical input type',async()=>{
 assert.deepEqual(await evaluate(`(()=>{
 const failures=[],owner=document.implementation.createHTMLDocument(),other=document.implementation.createHTMLDocument();
 owner.head.innerHTML='<base href="https://one.test/a/">';other.head.innerHTML='<base href="https://two.test/b/">';
 const i=owner.createElement('input');if(i.src!=='')failures.push('missing src');i.src='';if(i.src!=='https://one.test/a/')failures.push('empty src');
 i.src='file.png';i.width=42;i.height=17;if(i.width!==0||i.height!==0)failures.push('text dimensions');i.type='image';
 if(i.src!=='https://one.test/a/file.png'||i.width!==42||i.height!==17)failures.push('image');
 owner.head.firstChild.href='https://changed.test/';if(i.src!=='https://changed.test/file.png')failures.push('live base');
 const clone=i.cloneNode();other.adoptNode(i);if(i.src!=='https://two.test/b/file.png'||clone.src!=='https://changed.test/file.png')failures.push('adoption');
 i.type='file';if(i.width!==0||i.height!==0||i.getAttribute('width')!=='42')failures.push('type changes');
 return failures;
 })()`),[]);
});
