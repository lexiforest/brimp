import test from 'node:test';import assert from 'node:assert/strict';import fs from 'node:fs';
import {createSession} from '../../../scripts/coverage/session.mjs';import {inputFileCases as cases,measureInputFiles as measure}from'../../../scripts/coverage/input-files-cases.mjs';
async function evaluate(source){const s=await createSession();try{return await(await s.newPage()).evaluate(source);}finally{await s.close();}}

test('input.files assignment matches 78 Safari type, brand and side-effect cases',async()=>{
 const ref=JSON.parse(fs.readFileSync(new URL('../fixtures/input-files.safari.json',import.meta.url))).cases;assert.deepEqual(ref.map(({expected,...c})=>c),cases);
 const actual=JSON.parse(await evaluate(`JSON.stringify((${measure})(${JSON.stringify(cases)}))`));assert.deepEqual(ref.flatMap((c,i)=>JSON.stringify(c.expected)===JSON.stringify(actual[i])?[]:[{...c,actual:actual[i]}]),[]);
});

test('input.files shares a stable live DataTransfer list and clearing/reset preserves that identity',async()=>{
 assert.deepEqual(await evaluate(`(()=>{
 const out=[],dt=new DataTransfer(),i=document.createElement('input');i.type='file';const empty=i.files;
 dt.items.add(new File(['a'],'a.txt'));const list=dt.files;i.files=list;
 out.push(['assign',list===dt.files,i.files===list,empty.length,i.files.length]);
 dt.items.add(new File(['b'],'b.txt'));out.push(['add',i.files.length,list.length,i.value]);
 const clone=i.cloneNode();out.push(['clone',clone.files===list,clone.files.length,clone.files[0]===list[0]]);
 i.value='';out.push(['clear',i.files===list,i.files.length,list.length,dt.files.length,clone.files.length]);
 i.files=list;const form=document.createElement('form');form.append(i);form.reset();out.push(['reset',i.files===list,i.files.length,list.length]);
 i.files=list;i.type='text';i.type='file';out.push(['type',i.files===list,i.files.length,list.length]);
 dt.items.remove(0);out.push(['remove',list.length,list[0].name,clone.files.length,clone.files[0]?.name??null]);
 dt.items.clear();out.push(['empty',list.length,Object.keys(list)]);return out;
 })()`),[['assign',true,true,0,1],['add',2,2,'C:\\fakepath\\a.txt'],['clone',false,0,false],['clear',true,0,0,0,0],['reset',true,0,0],['type',false,0,0],['remove',1,'b.txt',0,null],['empty',0,[]]]);
});

test('files descriptors are branded and private file data drives values and required validity',async()=>{
 assert.deepEqual(await evaluate(`(()=>{
 const failures=[],d=Object.getOwnPropertyDescriptor(HTMLInputElement.prototype,'files');
 if(!d.enumerable||!d.configurable||d.get.name!=='get files'||d.set.name!=='set files'||d.get.length!==0||d.set.length!==1)failures.push('descriptor');
 for(const receiver of [{},document.createElement('div'),Object.create(HTMLInputElement.prototype)])for(const method of [d.get,d.set])try{method.call(receiver,null);failures.push('receiver')}catch(e){if(e.name!=='TypeError')failures.push(e.name)}
 const i=document.createElement('input');i.type='file';i.required=true;const dt=new DataTransfer();dt.items.add(new File(['x'],'x.txt'));const list=dt.files;
 Object.defineProperty(i,'type',{value:'text'});i.getAttribute=i.hasAttribute=()=>{throw Error('override')};i.files=list;
 if(i.files!==list||i.value!=='C:'+String.fromCharCode(92)+'fakepath'+String.fromCharCode(92)+'x.txt'||i.validity.valueMissing)failures.push('canonical');
 Object.defineProperty(list,'length',{value:0});Object.defineProperty(i,'files',{get(){throw Error('override')}});
 if(i.validity.valueMissing||i.value==='')failures.push('private list');
 const other=document.implementation.createHTMLDocument();other.adoptNode(i);if(d.get.call(i)!==list)failures.push('adoption');
 return failures;
 })()`),[]);
});
