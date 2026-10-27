import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {createSession} from '../../../scripts/coverage/session.mjs';
import {inputEditingCases,measureInputEditing} from '../../../scripts/coverage/input-editing-cases.mjs';
import {inputValidityFlagNames} from '../../../scripts/coverage/input-validity-cases.mjs';

test('input editing preserves Safari length and bad-input flags across clone, script assignment and reset',async()=>{
  const reference=JSON.parse(fs.readFileSync(new URL('../fixtures/input-editing.safari.json',import.meta.url)));
  assert.deepEqual(reference.flags,inputValidityFlagNames);
  assert.deepEqual(reference.cases.map(({expected,...c})=>c),inputEditingCases);
  const session=await createSession();
  try{
    const actual=await measureInputEditing(await session.newPage(),inputEditingCases),differences=[];
    for(let i=0;i<actual.length;i++)if(JSON.stringify(actual[i])!==JSON.stringify(reference.cases[i].expected))differences.push({...reference.cases[i],actual:actual[i]});
    assert.deepEqual(differences,[]);
  }finally{await session.close();}
});

test('user edit state follows Safari attribute, script and DOM mutations',async()=>{
  const {inputEditingMutationCases,measureInputEditingMutations}=await import('../../../scripts/coverage/input-editing-cases.mjs');
  const reference=JSON.parse(fs.readFileSync(new URL('../fixtures/input-editing-mutations.safari.json',import.meta.url)));
  assert.deepEqual(reference.cases.map(({expected,...c})=>c),inputEditingMutationCases);
  const session=await createSession();
  try{
    const actual=await measureInputEditingMutations(await session.newPage(),inputEditingMutationCases),differences=[];
    for(let i=0;i<actual.length;i++)if(JSON.stringify(actual[i])!==JSON.stringify(reference.cases[i].expected))differences.push({...reference.cases[i],actual:actual[i]});
    assert.deepEqual(differences,[]);
  }finally{await session.close();}
});

test('continued typing after a script value assignment follows Safari buffer and caret state',async()=>{
  const {inputEditingContinuationCases,measureInputEditingContinuation}=await import('../../../scripts/coverage/input-editing-cases.mjs');
  const reference=JSON.parse(fs.readFileSync(new URL('../fixtures/input-editing-continuation.safari.json',import.meta.url)));
  assert.deepEqual(reference.cases.map(({expected,...c})=>c),inputEditingContinuationCases);
  const session=await createSession();
  try{
    const actual=await measureInputEditingContinuation(await session.newPage(),inputEditingContinuationCases),differences=[];
    for(let i=0;i<actual.length;i++)if(actual[i]!==reference.cases[i].expected)differences.push({...reference.cases[i],actual:actual[i]});
    assert.deepEqual(differences,[]);
  }finally{await session.close();}
});

async function withPage(run){const session=await createSession();try{await run(await session.newPage());}finally{await session.close();}}
const snapshot="(()=>{const i=document.getElementById('edit');return [i.value,i.validity.tooShort,i.validity.tooLong,i.validity.badInput]})()";

test('maxlength limits native edits by UTF-16 length and does not apply to numbers',()=>withPage(async page=>{
  for(const {type,text,max,expected} of [
    {type:'text',text:'💡a',max:1,expected:['a',true,false,false]},
    {type:'text',text:'éa',max:1,expected:['e',true,false,false]},
    {type:'text',text:'abc',max:2,expected:['ab',true,false,false]},
    {type:'email',text:'  ab ',max:3,expected:['a',true,false,false]},
    {type:'number',text:'1e',max:1,expected:['',false,false,true]},
  ]){
    await page.evaluate(`(()=>{document.body.innerHTML='<input id="edit" minlength="3">';const i=document.getElementById('edit');i.type=${JSON.stringify(type)};i.setAttribute('maxlength',${max});return null})()`);
    await page.typeText('#edit',text);assert.deepEqual(await page.evaluate(snapshot),expected);
  }
}));

test('incomplete numeric buffers survive deletion and selection while the public selection APIs stay unavailable',()=>withPage(async page=>{
  await page.evaluate(`(()=>{document.body.innerHTML='<input id="edit" type="number">';return null})()`);
  await page.typeText('#edit','1e');assert.deepEqual(await page.evaluate(snapshot),['',false,false,true]);
  await page.press('#edit','Backspace');assert.deepEqual(await page.evaluate(snapshot),['1',false,false,false]);
  await page.typeText('#edit','.2');assert.deepEqual(await page.evaluate(snapshot),['1.2',false,false,false]);
  await page.press('#edit','ArrowLeft');await page.press('#edit','ArrowLeft');await page.press('#edit','Delete');
  assert.deepEqual(await page.evaluate(snapshot),['12',false,false,false]);
  await page.evaluate(`(()=>{document.getElementById('edit').select();return null})()`);
  await page.typeText('#edit','-');assert.deepEqual(await page.evaluate(snapshot),['',false,false,true]);
  await page.evaluate(`(()=>{document.getElementById('edit').select();return null})()`);
  await page.typeText('#edit','3');assert.deepEqual(await page.evaluate(snapshot),['3',false,false,false]);
  assert.deepEqual(await page.evaluate(`(()=>{const i=document.getElementById('edit');let error;try{i.setSelectionRange(0,1)}catch(e){error=e.name}return [i.selectionStart,i.selectionEnd,error]})()`),[null,null,'InvalidStateError']);
}));

test('native edits use private state and expose live validity to events; synthetic and canceled edits do not change it',()=>withPage(async page=>{
  await page.evaluate(`(()=>{
    document.body.innerHTML='<input id="edit" minlength="3">';const i=document.getElementById('edit');
    globalThis.editEvents=[];
    i.addEventListener('input',e=>editEvents.push([i.validity.tooShort,e.isTrusted]));
    i.addEventListener('beforeinput',e=>{if(e.data==='x')e.preventDefault()});
    i.value='a';i.dispatchEvent(new InputEvent('input',{bubbles:true,data:'b'}));
    return null;
  })()`);
  assert.deepEqual(await page.evaluate(snapshot),['a',false,false,false]);
  await page.typeText('#edit','x');assert.deepEqual(await page.evaluate(snapshot),['a',false,false,false]);
  await page.typeText('#edit','b');assert.deepEqual(await page.evaluate(snapshot),['ab',true,false,false]);
  assert.deepEqual(await page.evaluate('editEvents'),[[false,false],[true,true]]);
  await page.evaluate(`(()=>{const i=document.getElementById('edit');for(const [name,value] of Object.entries({type:'number',value:'',minLength:0,maxLength:0,selectionStart:0,selectionEnd:0,setSelectionRange(){throw Error('public method')}}))Object.defineProperty(i,name,{configurable:true,value});return null})()`);
  await page.typeText('#edit','c');
  assert.deepEqual(await page.evaluate(`(()=>{const i=document.getElementById('edit');return [Object.getOwnPropertyDescriptor(HTMLInputElement.prototype,'value').get.call(i),i.validity.tooShort]})()`),['abc',false]);
}));

test('numeric scientific notation accepts an empty fractional part but not a trailing decimal separator',()=>withPage(async page=>{
  assert.deepEqual(await page.evaluate(`(()=>{const i=document.createElement('input');i.type='number';return ['1.','1.e2','1.e-2','+.5','01.'].map(value=>{i.value=value;return [i.value,Number.isNaN(i.valueAsNumber)?'NaN':i.valueAsNumber]})})()`),[['','NaN'],['1.e2',100],['1.e-2',0.01],['','NaN'],['','NaN']]);
}));

test('numeric setters and stepping replace the edit buffer and preserve Safari caret placement',()=>withPage(async page=>{
  for(const [write,expected] of [['i.valueAsNumber=77','774'],['i.stepUp()','134'],['i.valueAsNumber=12','412']]){
    await page.evaluate(`(()=>{document.body.innerHTML='<input id="edit" type="number">';return null})()`);
    await page.typeText('#edit','12');await page.press('#edit','ArrowLeft');await page.press('#edit','ArrowLeft');
    await page.evaluate(`(()=>{const i=document.getElementById('edit');${write};return null})()`);
    await page.typeText('#edit','4');assert.equal(await page.evaluate("document.getElementById('edit').value"),expected);
  }
}));
