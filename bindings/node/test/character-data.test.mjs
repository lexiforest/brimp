import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {createSession} from '../../../scripts/coverage/session.mjs';
import {characterDataCases,measureCharacterData} from '../../../scripts/coverage/character-data-cases.mjs';
const reference=JSON.parse(fs.readFileSync(new URL('./fixtures/character-data.safari.json',import.meta.url)));
async function evaluate(source){const session=await createSession();try{return await(await session.newPage()).evaluate(source)}finally{await session.close()}}

test('CharacterData matches Safari UTF-16 conversion, errors, mutation records and live range updates',async()=>{
  assert.deepEqual(reference.map(({expected,...c})=>c),characterDataCases);const actual=[];
  // Each evaluation owns its ranges; isolate batches to bound live-range scans.
  for(let start=0;start<characterDataCases.length;start+=200)actual.push(...JSON.parse(await evaluate(`JSON.stringify((${measureCharacterData})(${JSON.stringify(characterDataCases.slice(start,start+200))}))`)));
  const differences=reference.flatMap((c,i)=>JSON.stringify(c.expected)===JSON.stringify(actual[i])?[]:[{...c,actual:actual[i]}]);
  assert.deepEqual(differences.slice(0,6),[],`${differences.length} of ${reference.length} cases differ`);
});

test('CharacterData descriptors, brands and required arguments match Safari before conversion',async()=>{
  assert.deepEqual(await evaluate(`(()=>{
    const p=CharacterData.prototype,failures=[],node=document.createTextNode('abc');
    const arities={substringData:2,appendData:1,insertData:2,deleteData:2,replaceData:3};
    for(const name of ['data','length',...Object.keys(arities)]){
      const descriptor=Object.getOwnPropertyDescriptor(p,name),fn=descriptor.get??descriptor.value;
      if(!descriptor.enumerable||!descriptor.configurable||fn.name!==(descriptor.get?'get '+name:name)||fn.length!==(arities[name]??0)||descriptor.get&&name==='length'&&descriptor.set!==undefined||!descriptor.get&&!descriptor.writable)failures.push([name,'descriptor']);
      if(name==='data'&&(descriptor.set.name!=='set data'||descriptor.set.length!==1))failures.push('setter');
      const forged=document.createElement('div');Object.setPrototypeOf(forged,p);let conversions=0;const value={[Symbol.toPrimitive](){conversions++;return 0}};
      for(const receiver of [null,{},Object.create(p),forged,new Proxy(node,{})])for(const method of [fn,...descriptor.set?[descriptor.set]:[]])try{method.call(receiver,value,value,value);failures.push([name,'receiver'])}catch(e){if(e.name!=='TypeError')failures.push(e.name)}
      if(conversions!==0)failures.push([name,'brand order']);
      if(arities[name])for(let count=0;count<arities[name];count++)try{fn.apply(node,Array(count).fill(value));failures.push([name,'missing args'])}catch(e){if(e.name!=='TypeError'||conversions!==0)failures.push([name,'arity order'])}
    }
    node.length=99;if(node.length!==3||Object.hasOwn(node,'length'))failures.push('readonly length');
    try{(function(){'use strict';node.length=99})();failures.push('strict length')}catch(e){if(e.name!=='TypeError')failures.push(e.name)}
    return failures;
  })()`),[]);
});

test('CharacterData reads live data after ordered reentrant argument conversion',async()=>{
  assert.deepEqual(await evaluate(`(()=>{
    const out=[];
    for(const method of ['substringData','appendData','insertData','deleteData','replaceData']){
      const node=document.createTextNode('abcdef'),log=[];
      const offset={[Symbol.toPrimitive](hint){log.push('offset:'+hint);node.data='wxyz';return 1}};
      const count={[Symbol.toPrimitive](hint){log.push('count:'+hint);node.data='12345';return 2}};
      const data={[Symbol.toPrimitive](hint){log.push('data:'+hint);node.data='ABCDE';return 'q'}};
      const args=method==='appendData'?[data]:method==='insertData'?[offset,data]:method==='replaceData'?[offset,count,data]:[offset,count];
      const result=node[method](...args);out.push([method,result??null,node.data,log]);
    }
    return out;
  })()`),[
    ['substringData','23','12345',['offset:number','count:number']],
    ['appendData',null,'ABCDEq',['data:string']],
    ['insertData',null,'AqBCDE',['offset:number','data:string']],
    ['deleteData',null,'145',['offset:number','count:number']],
    ['replaceData',null,'AqDE',['offset:number','count:number','data:string']],
  ]);
});

test('CharacterData edits bypass public overrides and retain ranges, iterators and observers after prototype changes',async()=>{
  assert.deepEqual(await evaluate(`(()=>{
    const out=[],p=CharacterData.prototype;
    for(const kind of ['text','comment']){
      const parent=document.createElement('div'),node=kind==='text'?document.createTextNode('abcd'):document.createComment('abcd');parent.appendChild(node);
      const range=document.createRange();range.setStart(node,3);range.collapse(true);
      const iterator=document.createNodeIterator(parent);iterator.nextNode();iterator.nextNode();
      const observer=new MutationObserver(()=>{});observer.observe(parent,{subtree:true,characterData:true,characterDataOldValue:true});
      Object.setPrototypeOf(node,null);
      for(const key of ['data','length','textContent','nodeType','replaceData','appendData','substringData'])Object.defineProperty(node,key,{get(){throw Error('public '+key)}});
      p.replaceData.call(node,1,2,'XYZZ');
      const record=observer.takeRecords()[0];observer.disconnect();
      out.push([Object.getOwnPropertyDescriptor(p,'data').get.call(node),Object.getOwnPropertyDescriptor(p,'length').get.call(node),range.startContainer===node,range.startOffset,iterator.referenceNode===node,record?.type,record?.oldValue]);
    }
    return out;
  })()`),Array(2).fill(['aXYZZd',6,true,1,true,'characterData','abcd']));
});

test('CharacterData preserves all UTF-16 code units through data, substring and replacement',async()=>{
  assert.deepEqual(await evaluate(`(()=>{
    let data='';for(let unit=0;unit<=0xffff;unit++)data+=String.fromCharCode(unit)+'x';
    const out=[];for(const node of [document.createTextNode(''),document.createComment('')]){
      node.data=data;out.push(node.length===data.length,node.substringData(0,data.length)===data);
      node.replaceData(1,2,String.fromCharCode(0xdc00,0,0xd800));const expected=data.slice(0,1)+String.fromCharCode(0xdc00,0,0xd800)+data.slice(3);out.push(node.data===expected,node.length===expected.length);
    }return out;
  })()`),Array(8).fill(true));
});

test('CharacterData edits keep textarea selection and stylesheet reactions after prototype changes',async()=>{
  assert.deepEqual(await evaluate(`(()=>{
    const textarea=document.createElement('textarea');textarea.textContent='abcd';document.body.appendChild(textarea);textarea.setSelectionRange(3,3);
    const text=textarea.firstChild;Object.setPrototypeOf(text,null);CharacterData.prototype.replaceData.call(text,0,4,'x');
    const style=document.createElement('style');style.textContent='.before { color: red; }';document.head.appendChild(style);const sheet=style.sheet;
    const css=style.firstChild;Object.setPrototypeOf(css,null);Object.getOwnPropertyDescriptor(CharacterData.prototype,'data').set.call(css,'.after { color: blue; }');
    return [textarea.value,textarea.selectionStart,textarea.selectionEnd,sheet.cssRules[0].selectorText,style.sheet.cssRules[0].selectorText,sheet!==style.sheet];
  })()`),['x',0,0,'.before','.after',true]);
});
