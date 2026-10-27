import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {createSession} from '../../../scripts/coverage/session.mjs';
import * as corpus from '../../../scripts/coverage/input-validity-cases.mjs';
async function evaluate(source){const s=await createSession();try{return await(await s.newPage()).evaluate(source);}finally{await s.close();}}
for(const [key,cases,measure] of [['cases',corpus.inputValidityCases,corpus.measureInputValidity],['radio_cases',corpus.radioValidityCases,corpus.measureRadioValidity]])
test(`input validity: ${cases.length} Safari ${key}`,async()=>{
  const reference=JSON.parse(fs.readFileSync(new URL('../fixtures/input-validity.safari.json',import.meta.url)));
  assert.deepEqual(reference.flags,corpus.inputValidityFlagNames);assert.deepEqual(reference[key].map(({expected,...c})=>c),cases);
  const differences=[];
  for(let start=0;start<cases.length;start+=210){
    const batch=cases.slice(start,start+210),actual=JSON.parse(await evaluate(`JSON.stringify((${measure})(${JSON.stringify(batch)},${JSON.stringify(reference.flags)}))`));
    for(let i=0;i<batch.length;i++)if(JSON.stringify(actual[i])!==JSON.stringify(reference[key][start+i].expected))differences.push({...batch[i],expected:reference[key][start+i].expected,actual:actual[i]});
  }
  assert.deepEqual(differences,[]);
});

test('input validity reads canonical constraints and stays live across mutation, cloning and adoption',async()=>{
  assert.deepEqual(await evaluate(`(() => {
    const failures=[],input=document.createElement('input'),state=input.validity;
    input.type='number';input.min='1';input.max='3';input.step='2';input.value='2';
    if(!state.stepMismatch||state.valid||input.validity!==state)failures.push('initial numeric');
    for(const name of ['type','value','min','max','step','required','readOnly','disabled','multiple','pattern'])Object.defineProperty(input,name,{get(){throw Error(name);},set(){throw Error(name);}});
    input.getAttribute=input.hasAttribute=input.getRootNode=()=>{throw Error('override');};
    if(!state.stepMismatch||state.rangeOverflow||state.rangeUnderflow)failures.push('canonical numeric');
    input.setAttributeNS(null,'min','3');if(!state.rangeUnderflow)failures.push('live bound');
    input.setAttributeNS(null,'type','email');input.setAttributeNS(null,'multiple','');
    Object.getOwnPropertyDescriptor(HTMLInputElement.prototype,'value').set.call(input,'a@b,c@d');
    input.setAttributeNS(null,'pattern','[^,]+');if(state.typeMismatch||state.patternMismatch)failures.push('per-email pattern');
    input.setAttributeNS(null,'pattern','a@b');if(!state.patternMismatch)failures.push('second address');
    const OriginalRegExp=globalThis.RegExp;try{globalThis.RegExp=function(){throw Error('constructor override');};if(!state.patternMismatch)failures.push('pattern intrinsic');}finally{globalThis.RegExp=OriginalRegExp;}
    const clone=input.cloneNode();if(clone.validity===state||!clone.validity.patternMismatch)failures.push('clone');
    const owner=document.implementation.createHTMLDocument();owner.adoptNode(input);if(input.validity!==state||!state.patternMismatch)failures.push('adopt');
    input.removeAttributeNS(null,'pattern');if(!state.valid)failures.push('clear pattern');
    return failures;
  })()`),[]);
});

test('required radio validity follows live group requirement and canonical checked/form/name state',async()=>{
  assert.deepEqual(await evaluate(`(() => {
    const failures=[],first=document.createElement('form'),second=document.createElement('form'),a=document.createElement('input'),b=document.createElement('input');
    for(const input of [a,b]){input.type='radio';input.name='g';first.append(input);}b.required=true;
    if(!a.validity.valueMissing||!b.validity.valueMissing)failures.push('group required');
    a.checked=true;if(a.validity.valueMissing||b.validity.valueMissing)failures.push('group checked');
    second.append(a);if(a.validity.valueMissing||!b.validity.valueMissing)failures.push('form boundary');
    first.append(a);for(const input of [a,b])for(const name of ['name','form','checked','required','type'])Object.defineProperty(input,name,{get(){throw Error(name);}});
    if(a.validity.valueMissing||b.validity.valueMissing)failures.push('canonical group');
    Object.getOwnPropertyDescriptor(HTMLInputElement.prototype,'checked').set.call(a,false);
    if(!a.validity.valueMissing||!b.validity.valueMissing)failures.push('live checked');
    b.removeAttribute('required');if(a.validity.valueMissing||b.validity.valueMissing)failures.push('live required');
    return failures;
  })()`),[]);
});

test('file valueMissing depends on the private file list even when disabled or readonly',async()=>{
  assert.deepEqual(await evaluate(`(() => {
    const failures=[],input=document.createElement('input');input.type='file';input.required=true;input.disabled=true;input.readOnly=true;
    const state=input.validity;if(!state.valueMissing)failures.push('empty');
    const transfer=new DataTransfer();transfer.items.add(new File(['content'],'a.txt'));input.files=transfer.files;
    if(state.valueMissing)failures.push('selected');
    Object.defineProperty(input,'files',{get(){throw Error('public files');}});if(state.valueMissing)failures.push('canonical files');
    input.value='';if(!state.valueMissing)failures.push('clear');input.required=false;if(state.valueMissing)failures.push('not required');
    return failures;
  })()`),[]);
});
