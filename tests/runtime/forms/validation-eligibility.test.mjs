import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {createSession} from '../../../scripts/coverage/session.mjs';
import {validationEligibilityCases as cases,measureValidationEligibility as measure} from '../../../scripts/coverage/validation-eligibility-cases.mjs';
import {validationEligibilityMutationCases,measureValidationEligibilityMutations} from '../../../scripts/coverage/validation-eligibility-cases.mjs';
async function evaluate(source){const s=await createSession();try{return await(await s.newPage()).evaluate(source);}finally{await s.close();}}

test('eligibility mutation sequences preserve Safari ancestor-disabled state and clone/adoption behavior',async()=>{
  const reference=JSON.parse(fs.readFileSync(new URL('../fixtures/validation-eligibility.safari.json',import.meta.url))).mutation_cases;
  assert.deepEqual(reference.map(({expected,...c})=>c),validationEligibilityMutationCases);
  const actual=await evaluate(`(${measureValidationEligibilityMutations})(${JSON.stringify(validationEligibilityMutationCases)})`);
  assert.deepEqual(reference.flatMap((c,i)=>JSON.stringify(c.expected)===JSON.stringify(actual[i])?[]:[{...c,actual:actual[i]}]),[]);
});

test(`validation eligibility matches ${cases.length} Safari cases`,async()=>{
  const reference=JSON.parse(fs.readFileSync(new URL('../fixtures/validation-eligibility.safari.json',import.meta.url))).cases;
  assert.deepEqual(reference.map(({expected,...c})=>c),cases);
  const differences=[];
  // These are independent cases. Fresh realms keep unrelated retained shadow
  // roots from turning a value contract into a cumulative mutation benchmark.
  for(let start=0;start<cases.length;start+=210){
    const batch=cases.slice(start,start+210),actual=await evaluate(`(${measure})(${JSON.stringify(batch)})`);
    for(let i=0;i<batch.length;i++)if(actual[i]!==reference[start+i].expected)differences.push({...batch[i],actual:actual[i],expected:reference[start+i].expected});
  }
  assert.deepEqual(differences,[]);
});

test('willValidate is readonly, branded and independent of public-property overrides',async()=>{
  assert.deepEqual(await evaluate(`(() => {
    const failures=[];
    for(const [tag,ctor] of [['input',HTMLInputElement],['button',HTMLButtonElement],['select',HTMLSelectElement],
      ['textarea',HTMLTextAreaElement],['fieldset',HTMLFieldSetElement],['output',HTMLOutputElement],['object',HTMLObjectElement]]){
      const control=document.createElement(tag),d=Object.getOwnPropertyDescriptor(ctor.prototype,'willValidate');
      if(!d.enumerable||!d.configurable||d.set!==undefined||d.get.name!=='get willValidate'||d.get.length!==0)failures.push([tag,'descriptor']);
      for(const receiver of [{},document.createElement('div'),Object.create(ctor.prototype)]){
        try{d.get.call(receiver);failures.push([tag,'receiver']);}catch(e){if(e.name!=='TypeError')failures.push([tag,e.name]);}
      }
      const before=control.willValidate;if(Reflect.set(control,'willValidate',!before)||control.willValidate!==before)failures.push([tag,'readonly']);
      control.getAttribute=control.hasAttribute=control.closest=control.getRootNode=()=>{throw Error('override');};
      for(const name of ['disabled','readOnly','type','parentNode','parentElement'])Object.defineProperty(control,name,{get(){throw Error(name);}});
      if(control.willValidate!==before)failures.push([tag,'canonical initial']);
      control.setAttributeNS('urn:foreign','disabled','');control.setAttributeNS('urn:foreign','readonly','');
      if(control.willValidate!==before)failures.push([tag,'foreign attributes']);
      control.setAttributeNS(null,'disabled','false');if(control.willValidate)failures.push([tag,'canonical disabled']);control.removeAttributeNS(null,'disabled');
      if(control.willValidate!==before)failures.push([tag,'removed disabled']);
      if(tag==='input'){control.setAttribute('type','image');if(control.willValidate)failures.push('canonical image');}
    }
    return failures;
  })()`),[]);
});

test('eligibility follows live legends, nearest disabled fieldsets and datalist reparenting',async()=>{
  assert.deepEqual(await evaluate(`(() => {
    const failures=[];
    for(const tag of ['input','button','textarea','select']){
      const outer=document.createElement('fieldset'),inner=document.createElement('fieldset'),legend=document.createElement('legend'),control=document.createElement(tag);
      outer.disabled=inner.disabled=true;outer.append(inner);inner.append(legend);legend.append(control);
      const states=[control.willValidate];inner.disabled=false;states.push(control.willValidate);outer.disabled=false;states.push(control.willValidate);
      inner.disabled=true;const first=document.createElement('legend');inner.prepend(first);states.push(control.willValidate);
      inner.prepend(legend);states.push(control.willValidate);inner.append(control);states.push(control.willValidate);
      legend.append(control);states.push(control.willValidate);
      const datalist=document.createElement('datalist');legend.append(datalist);datalist.append(control);states.push(control.willValidate);
      const clone=control.cloneNode();states.push(clone.willValidate);document.implementation.createHTMLDocument().adoptNode(control);states.push(control.willValidate);
      if(JSON.stringify(states)!==JSON.stringify([true,true,true,false,true,false,true,false,true,true]))failures.push([tag,states]);
    }
    const fieldset=document.createElement('fieldset'),legend=document.createElement('legend'),input=document.createElement('input');
    fieldset.disabled=true;fieldset.append(legend);legend.append(input);
    for(const [node,names] of [[fieldset,['disabled','children','firstChild']], [legend,['contains','parentNode']],[input,['parentNode','parentElement']]])
      for(const name of names)Object.defineProperty(node,name,{get(){throw Error('overridden '+name);}});
    if(!input.willValidate)failures.push('canonical legend');
    input.setAttributeNS(null,'disabled','');if(input.willValidate)failures.push('canonical own disabled');
    return failures;
  })()`),[]);
});

test('eligibility is unaffected by invalid values or novalidate, and reads emit no events or mutations',async()=>{
  assert.deepEqual(await evaluate(`(() => {
    const failures=[],form=document.createElement('form'),input=document.createElement('input');form.append(input);
    input.required=true;input.setCustomValidity('custom');form.noValidate=true;input.formNoValidate=true;
    if(!input.willValidate)failures.push('constraints do not bar validation');
    const observer=new MutationObserver(()=>{});observer.observe(input,{attributes:true});const events=[];
    for(const type of ['invalid','input','change'])input.addEventListener(type,()=>events.push(type));
    for(let i=0;i<5;i++)if(!input.willValidate)failures.push('repeated read');
    if(observer.takeRecords().length||events.length)failures.push('read side effects');observer.disconnect();
    for(const operation of [()=>input.setAttribute('readonly',''),()=>input.setAttributeNS(null,'readonly','false'),
      ()=>{const a=document.createAttribute('readonly');a.value='x';input.setAttributeNode(a);}]){
      operation();if(input.willValidate)failures.push('readonly presence');input.attributes.getNamedItem('readonly').value='';
      if(input.willValidate)failures.push('empty readonly');input.removeAttributeNS(null,'readonly');if(!input.willValidate)failures.push('readonly removed');
    }
    return failures;
  })()`),[]);
});

test('disabled fieldset eligibility agrees with script click and form-data exclusion',async()=>{
  assert.deepEqual(await evaluate(`(() => {
    const failures=[],form=document.createElement('form'),outer=document.createElement('fieldset'),inner=document.createElement('fieldset'),legend=document.createElement('legend'),input=document.createElement('input');
    input.type='checkbox';input.name='check';input.value='yes';form.append(outer);outer.append(inner);inner.append(legend);legend.append(input);
    document.body.append(form);outer.disabled=inner.disabled=true;
    input.click();if(!input.checked||!input.willValidate||new FormData(form).get('check')!=='yes')failures.push('nearest first legend exemption');
    inner.disabled=false;input.click();if(input.checked||!input.willValidate||new FormData(form).has('check'))failures.push('retained exemption');
    // Reinsert the control to refresh its ancestor-disabled state.
    legend.append(input);input.checked=true;input.click();if(!input.checked||input.willValidate||new FormData(form).has('check'))failures.push('outer fieldset applies');
    Object.defineProperty(outer,'disabled',{value:false});Object.defineProperty(input,'disabled',{value:false});
    input.click();if(!input.checked||input.willValidate||new FormData(form).has('check'))failures.push('canonical disabled');
    form.remove();return failures;
  })()`),[]);
});
