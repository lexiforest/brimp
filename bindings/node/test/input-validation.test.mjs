import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {createSession} from '../../../scripts/coverage/session.mjs';
import {inputValidationCases,measureInputValidation} from '../../../scripts/coverage/input-validation-cases.mjs';
async function withPage(run){const session=await createSession();try{return await run(await session.newPage());}finally{await session.close();}}
async function evaluate(source){return withPage(page=>page.evaluate(source));}

test(`input validation messages and checkValidity match ${inputValidationCases.length} Safari cases`,async()=>{
  const reference=JSON.parse(fs.readFileSync(new URL('./fixtures/input-validation.safari.json',import.meta.url)));
  assert.deepEqual(reference.cases.map(({expected,...c})=>c),inputValidationCases);
  const differences=[];
  for(let start=0;start<inputValidationCases.length;start+=210){
    const batch=inputValidationCases.slice(start,start+210),actual=JSON.parse(await evaluate(`JSON.stringify((${measureInputValidation})(${JSON.stringify(batch)}))`));
    for(let i=0;i<batch.length;i++)if(JSON.stringify(actual[i])!==JSON.stringify(reference.cases[start+i].expected))differences.push({...batch[i],expected:reference.cases[start+i].expected,actual:actual[i]});
  }
  assert.deepEqual(differences,[]);
});

test('input validation message and method descriptors are readonly/branded with Safari shape',async()=>{
  assert.deepEqual(await evaluate(`(()=>{
    const failures=[],prototype=HTMLInputElement.prototype,input=document.createElement('input');
    const d=Object.getOwnPropertyDescriptor(prototype,'validationMessage');
    if(!d.enumerable||!d.configurable||d.set!==undefined||d.get.name!=='get validationMessage'||d.get.length!==0)failures.push('message descriptor');
    if(Reflect.set(input,'validationMessage','x'))failures.push('writable message');
    const m=Object.getOwnPropertyDescriptor(prototype,'checkValidity');
    if(!m.enumerable||!m.configurable||!m.writable||m.value.name!=='checkValidity'||m.value.length!==0)failures.push('method descriptor');
    try{new m.value();failures.push('constructible method')}catch(e){if(e.name!=='TypeError')failures.push(e.name)}
    for(const receiver of [{},document.createElement('button'),Object.create(prototype),null])for(const method of [d.get,m.value]){
      try{method.call(receiver);failures.push('receiver')}catch(e){if(e.name!=='TypeError')failures.push(e.name)}
    }
    return failures;
  })()`),[]);
});

test('input checks use canonical constraints, preserve focus, and return the result before invalid listeners mutate state',async()=>{
  assert.deepEqual(await evaluate(`(()=>{
    const failures=[],form=document.createElement('form'),input=document.createElement('input'),focused=document.createElement('button');
    form.append(input,focused);document.body.append(form);focused.focus();input.required=true;const log=[];
    form.addEventListener('invalid',e=>log.push(['capture',e.target===input]),true);
    form.addEventListener('invalid',()=>log.push(['bubble']));
    input.addEventListener('invalid',e=>{
      log.push(['target',e.isTrusted,e.bubbles,e.cancelable,e.composed,e.target===input]);e.preventDefault();input.value='filled';log.push(['nested',input.checkValidity()]);
    },{once:true});
    if(input.validationMessage!=='Fill out this field'||input.checkValidity()!==false||!input.checkValidity())failures.push('pre-event result');
    if(document.activeElement!==focused)failures.push('focus changed');
    if(JSON.stringify(log)!==JSON.stringify([['capture',true],['target',true,false,true,false,true],['nested',true]]))failures.push(['events',log]);
    input.value='';input.addEventListener('invalid',()=>{input.disabled=true;input.remove()},{once:true});
    if(input.checkValidity()!==false||!input.checkValidity()||input.validationMessage!=='')failures.push('disabled during event');
    input.disabled=false;let invalid=0;input.addEventListener('invalid',()=>invalid++);
    for(const name of ['value','type','required','disabled','readOnly','willValidate','validity','min','max','step','pattern','title','dispatchEvent'])Object.defineProperty(input,name,{get(){throw Error(name)}});
    input.getAttribute=input.hasAttribute=input.closest=()=>{throw Error('override')};
    if(input.validationMessage!=='Fill out this field'||input.checkValidity()!==false||invalid!==1)failures.push('canonical detached input');
    input.setCustomValidity('private');if(input.validationMessage!=='private')failures.push('private custom error');
    form.remove();return failures;
  })()`),[]);
});

test('native-edit messages and invalid events follow Safari error priority and length limits',async()=>{
  const {inputEditingCases}=await import('../../../scripts/coverage/input-editing-cases.mjs');
  const {measureInputValidationEditing}=await import('../../../scripts/coverage/input-validation-cases.mjs');
  const reference=JSON.parse(fs.readFileSync(new URL('./fixtures/input-validation-editing.safari.json',import.meta.url)));
  assert.deepEqual(reference.cases.map(({expected,...c})=>c),inputEditingCases);
  await withPage(async page=>{
    const actual=await measureInputValidationEditing(page,inputEditingCases),differences=[];
    for(let i=0;i<actual.length;i++)if(JSON.stringify(actual[i])!==JSON.stringify(reference.cases[i].expected))differences.push({...reference.cases[i],actual:actual[i]});
    assert.deepEqual(differences,[]);
  });
});

test('temporal checkValidity cache follows 792 Safari value and attribute state transitions independently of live flags',async()=>{
  const {temporalValidationCases:cases,temporalValidationActions:actions,measureTemporalValidation:measure}=await import('../../../scripts/coverage/input-validation-cache-cases.mjs');
  const reference=JSON.parse(fs.readFileSync(new URL('./fixtures/input-validation-cache.safari.json',import.meta.url)));
  assert.deepEqual(reference.cases.map(({expected,...c})=>c),cases);
  const actual=JSON.parse(await evaluate(`JSON.stringify((${measure})(${JSON.stringify(cases)},${JSON.stringify(actions)}))`));
  assert.deepEqual(reference.cases.flatMap((c,i)=>JSON.stringify(c.expected)===JSON.stringify(actual[i])?[]:[{...c,actual:actual[i]}]),[]);
});

test('temporal validation distinguishes parser initialization, focus, cloning and later value writes',async()=>{
  assert.deepEqual(await evaluate(`(()=>{
    const failures=[];
    for(const action of ['none','different','same','roundtrip','attribute','clone','focus']){
      const form=document.createElement('form');form.innerHTML='<input type="date" value="2000-01-01">';document.body.append(form);let i=form.firstChild;
      if(action==='different')i.value='2001-01-01';if(action==='same')i.value=i.value;
      if(action==='roundtrip'){i.type='text';i.type='date'}if(action==='attribute')i.defaultValue='2002-01-01';
      if(action==='clone')i=i.cloneNode();if(action==='focus')i.focus();
      i.value='';const expected=['none','same','clone'].includes(action);
      if(!i.validity.valid||i.validationMessage!==''||i.checkValidity()!==expected)failures.push(action);
      form.remove();i.remove();
    }
    return failures;
  })()`),[]);
});

test('reading validationMessage emits no DOM events or mutations and does not invoke public constraint getters',async()=>{
  assert.deepEqual(await evaluate(String.raw`(()=>{
    const i=document.createElement('input');i.setAttribute('pattern','a');i.setAttribute('title','  useful \n\t title ');i.value='b';
    const events=[];for(const type of ['invalid','input','change'])i.addEventListener(type,()=>events.push(type));
    const observer=new MutationObserver(()=>{});observer.observe(i,{attributes:true,childList:true,subtree:true});
    for(const name of ['value','pattern','title','validity','willValidate'])Object.defineProperty(i,name,{get(){throw Error(name)}});
    const first=i.validationMessage,second=i.validationMessage,records=observer.takeRecords();observer.disconnect();
    return [first,second,events,records.length];
  })()`),['Match the requested format: useful title','Match the requested format: useful title',[],0]);
});

test('inherited eligibility invalidates the temporal check cache while unchanged attributes and ordinary moves preserve it',async()=>{
  assert.deepEqual(await evaluate(`(()=>{return ['fieldset','datalist','form','required-same','pattern-same'].map(action=>{
    const i=document.createElement('input');i.type='date';i.value='2000-01-01';i.value='';let parent;
    if(action==='fieldset'){parent=document.createElement('fieldset');parent.append(i);parent.disabled=true;parent.disabled=false}
    if(action==='datalist'){parent=document.createElement('datalist');parent.append(i);i.remove()}
    if(action==='form'){parent=document.createElement('form');parent.append(i);i.remove()}
    if(action==='required-same')i.required=false;
    if(action==='pattern-same'){i.pattern='';i.value='2000-01-01';i.value='';i.pattern=''}
    return [action,i.validity.valid,i.checkValidity()];
  })})()`),[['fieldset',true,true],['datalist',true,true],['form',true,false],['required-same',true,false],['pattern-same',true,false]]);
});
