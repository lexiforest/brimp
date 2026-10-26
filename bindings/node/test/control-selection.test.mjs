import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {createSession} from '../../../scripts/coverage/session.mjs';
import {controlSelectionCases as cases,measureControlSelections as measure} from '../../../scripts/coverage/control-selection-cases.mjs';
const reference=JSON.parse(fs.readFileSync(new URL('./fixtures/control-selection.safari.json',import.meta.url),'utf8'));
async function evaluate(source){const s=await createSession();try{return await(await s.newPage()).evaluate(source);}finally{await s.close();}}

test('text-control selection and replacement match 640 measured Safari cases',async()=>{
 assert.deepEqual(reference.cases.map(({expected,...c})=>c),cases);
 const actual=await evaluate(`(${measure})(${JSON.stringify(cases)})`);
 assert.deepEqual(cases.flatMap((c,i)=>JSON.stringify(actual[i])===JSON.stringify(reference.cases[i].expected)?[]:[{...c,actual:actual[i],expected:reference.cases[i].expected}]),[]);
});

test('selection conversions, errors, API shape and canonical state follow Safari',async()=>{
 assert.deepEqual(await evaluate(`(()=>{const failures=[];
 for(const tag of ['input','textarea']){
  const i=document.createElement(tag);i.value='abcdef';i.setSelectionRange(2,4,'backward');
  for(const property of ['selectionStart','selectionEnd'])for(const value of [1n,Symbol(),{valueOf(){return 1n}}]){
   try{i[property]=value;failures.push([property,'conversion']);}catch(e){if(e.name!=='TypeError')failures.push([property,e.name]);}
  }
  for(const args of [[],[1],[1n,3],[1,3,Symbol()]]){try{i.setSelectionRange(...args);failures.push('range error');}catch(e){if(e.name!=='TypeError')failures.push(e.name);}}
  for(const args of [[],['x',1],[Symbol()],['x',1n,2]]){try{i.setRangeText(...args);failures.push('replace error');}catch(e){if(e.name!=='TypeError')failures.push(e.name);}}
  let converted=0;i.selectionStart={valueOf(){converted++;return 1;}};
  if(converted!==1||i.selectionStart!==1||i.selectionEnd!==4||i.selectionDirection!=='backward')failures.push('single conversion');
  const proto=tag==='input'?HTMLInputElement.prototype:HTMLTextAreaElement.prototype;
  for(const name of ['selectionStart','selectionEnd','selectionDirection','setSelectionRange','setRangeText','select']){
   const d=Object.getOwnPropertyDescriptor(proto,name);if(!d.enumerable||!d.configurable)failures.push([name,'descriptor']);
   const functions=d.value?[d.value]:[d.get,d.set];
   for(const fn of functions){try{fn.call({},1,2);failures.push([name,'brand']);}catch(e){if(e.name!=='TypeError')failures.push([name,e.name]);}}
  }
  if(proto.setSelectionRange.length!==2||proto.setRangeText.length!==1||proto.select.length!==0)failures.push('arity');
  const fail=()=>{throw Error('override')};
  Object.defineProperty(i,'value',{get:fail,set:fail});i.setSelectionRange=fail;
  if(tag==='input')Object.defineProperty(i,'type',{get:()=> 'number'});
  i.selectionStart=2;i.selectionEnd=5;i.selectionDirection={toString(){return 'backward'}};
  proto.setRangeText.call(i,'XY',2,4,'SELECT');
  const value=Object.getOwnPropertyDescriptor(proto,'value').get.call(i);
  if(value!=='abXYef'||i.selectionStart!==2||i.selectionEnd!==4||i.selectionDirection!=='none')failures.push('canonical');
 }
 const unsupported=document.createElement('input');unsupported.type='number';
 for(const action of [()=>unsupported.setSelectionRange(1,2),()=>unsupported.selectionStart=1,()=>unsupported.setRangeText('x')]){
  try{action();failures.push('unsupported');}catch(e){if(e.name!=='InvalidStateError')failures.push(e.name);}
 }
 return failures})()`),[]);
});

test('selection defaults, value assignments, reset and input type transitions follow Safari',async()=>{
 assert.deepEqual(await evaluate(String.raw`(()=>{const failures=[],snapshot=i=>[i.selectionStart,i.selectionEnd,i.selectionDirection];
 for(const tag of ['input','textarea']){
  const i=document.createElement(tag);i.defaultValue='abcdef';if(JSON.stringify(snapshot(i))!==JSON.stringify([0,0,'none']))failures.push([tag,'initial']);
  i.setSelectionRange(1,4,'backward');i.value='abcdef';if(JSON.stringify(snapshot(i))!==JSON.stringify([1,4,'backward']))failures.push([tag,'same']);
  i.value='abc';if(JSON.stringify(snapshot(i))!==JSON.stringify([3,3,'none']))failures.push([tag,'changed']);
  i.setSelectionRange(1,2,'backward');i.value='abc\r\n';const expected=tag==='input'?[1,2,'backward']:[4,4,'none'];
  if(JSON.stringify(snapshot(i))!==JSON.stringify(expected))failures.push([tag,'sanitized']);
  const clean=document.createElement(tag);clean.defaultValue='abcdef';clean.setSelectionRange(3,5,'backward');clean.defaultValue='xy';
  if(JSON.stringify(snapshot(clean))!==JSON.stringify(tag==='input'?[3,5,'backward']:[0,0,'none']))failures.push([tag,'default shrink']);
  clean.defaultValue='abcdef';clean.setSelectionRange(1,4,'backward');const form=document.createElement('form');form.appendChild(clean);form.reset();
  if(JSON.stringify(snapshot(clean))!==JSON.stringify(tag==='input'?[6,6,'none']:[1,4,'backward']))failures.push([tag,'reset']);
 }
 const supported=['text','search','url','tel','password'];
 for(const type of [...supported,'number','email','file','hidden','checkbox','date','time','datetime-local','range'])for(const focused of [false,true]){
  const i=document.createElement('input');i.value='abcdef';if(focused){document.body.appendChild(i);i.focus();}i.setSelectionRange(1,4,'backward');i.type=type;
  if(JSON.stringify(snapshot(i))!==JSON.stringify(supported.includes(type)?[1,4,'backward']:[null,null,null]))failures.push([type,'transition']);
  i.type='text';let returned=supported.includes(type)||focused&&type!=='number'?[1,4,'backward']:[0,0,'none'];
  if(focused&&['date','time','datetime-local'].includes(type))returned=[0,0,'forward'];
  if(focused&&type==='range')returned=[1,2,'backward'];
  if(JSON.stringify(snapshot(i))!==JSON.stringify(returned))failures.push([type,focused,'return',snapshot(i)]);i.remove();
 }
 const url=document.createElement('input');url.value='  ab  ';url.setSelectionRange(1,5,'backward');url.type='url';
 if(url.value!=='ab'||JSON.stringify(snapshot(url))!==JSON.stringify([2,2,'none']))failures.push('type sanitization');
 return failures})()`),[]);
});

async function flushEvents(page){
 if(process.env.COVERAGE_BROWSER!=='safari'){await page.runPendingTasks();return;}
 await page.evaluate('(()=>{globalThis.selectionTasksReady=false;setTimeout(()=>selectionTasksReady=true,20);return null})()');
 const deadline=Date.now()+5000;
 while(!await page.evaluate('selectionTasksReady')){if(Date.now()>deadline)throw Error('Safari selection tasks did not finish');}
}

test('select events are queued per change and selectionchange is coalesced with Safari flags',async()=>{
 const s=await createSession();try{const p=await s.newPage();
 await p.evaluate(`(()=>{globalThis.control=document.createElement('input');control.value='abcdef';globalThis.selectionLog=[];
 for(const name of ['select','selectionchange'])control.addEventListener(name,e=>selectionLog.push([name,e.bubbles,e.composed,e.isTrusted]));
 control.setSelectionRange(1,3);control.setSelectionRange(1,3);control.selectionDirection='backward';
 return selectionLog.length})()`).then(count=>assert.equal(count,0));
 await flushEvents(p);
 assert.deepEqual(await p.evaluate('selectionLog'),[['select',true,false,true],['selectionchange',true,false,true],['select',true,false,true]]);
 await p.evaluate(`(()=>{selectionLog=[];control.value='xyz';const f=document.createElement('form');f.appendChild(control);f.reset();return null})()`);
 await flushEvents(p);assert.deepEqual(await p.evaluate('selectionLog'),[]);
 }finally{await s.close();}
});

test('select focuses eligible connected controls and respects disabled controls and focused directions',async()=>{
 assert.deepEqual(await evaluate(`(()=>{const failures=[];
 for(const tag of ['input','textarea'])for(const disabled of [false,true]){
 const i=document.createElement(tag);i.value='abcdef';i.disabled=disabled;document.body.appendChild(i);i.select();
 if(i.selectionStart!==0||i.selectionEnd!==6||i.selectionDirection!=='none'||(document.activeElement===i)!==!disabled)failures.push([tag,disabled,'select']);
 if(!disabled){i.setSelectionRange(1,1,'backward');if(i.selectionDirection!=='forward')failures.push([tag,'collapsed']);
 i.defaultValue='x';if(i.value!=='abcdef')failures.push('dirty default');}
 i.remove();}
 const i=document.createElement('input');i.defaultValue='abcdef';document.body.appendChild(i);i.focus();i.setSelectionRange(3,5,'backward');i.defaultValue='xy';
 if(i.selectionStart!==0||i.selectionEnd!==0||i.selectionDirection!=='forward')failures.push('focused default');
 return failures})()`),[]);
});
