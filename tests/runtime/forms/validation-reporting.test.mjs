import test from 'node:test';import assert from 'node:assert/strict';import fs from 'node:fs';
import {createSession} from '../../../scripts/coverage/session.mjs';
import {validationReportingCases as cases,measureValidationReporting as measure} from '../../../scripts/coverage/validation-reporting-cases.mjs';
async function evaluate(source){const session=await createSession();try{return await(await session.newPage()).evaluate(source);}finally{await session.close();}}

test('input/button reportValidity return values and events match 252 Safari cases',async()=>{
  const reference=JSON.parse(fs.readFileSync(new URL('../fixtures/validation-reporting.safari.json',import.meta.url)));
  assert.deepEqual(reference.cases.map(({expected,...c})=>c),cases);
  const actual=JSON.parse(await evaluate(`JSON.stringify((${measure})(${JSON.stringify(cases)}))`));
  assert.deepEqual(reference.cases.flatMap((c,i)=>JSON.stringify(c.expected)===JSON.stringify(actual[i])?[]:[{...c,actual:actual[i]}]),[]);
});

test('reportValidity has Safari method descriptors, receiver checks and canonical validation',async()=>{
  assert.deepEqual(await evaluate(`(()=>{
    const failures=[];
    for(const [tag,ctor]of [['input',HTMLInputElement],['button',HTMLButtonElement]]){
      const descriptor=Object.getOwnPropertyDescriptor(ctor.prototype,'reportValidity'),method=descriptor.value;
      if(!descriptor.enumerable||!descriptor.configurable||!descriptor.writable||method.name!=='reportValidity'||method.length!==0)failures.push([tag,'descriptor']);
      try{new method();failures.push('constructible')}catch(e){if(e.name!=='TypeError')failures.push(e.name)}
      for(const receiver of [{},document.createElement('div'),Object.create(ctor.prototype)])try{method.call(receiver);failures.push('receiver')}catch(e){if(e.name!=='TypeError')failures.push(e.name)}
      const control=document.createElement(tag);control.setCustomValidity('invalid');let events=0;control.addEventListener('invalid',e=>{e.preventDefault();events++});
      for(const name of ['checkValidity','validity','willValidate','validationMessage','dispatchEvent'])Object.defineProperty(control,name,{get(){throw Error(name)}});
      if(control.reportValidity()!==false||events!==1)failures.push([tag,'canonical']);control.setCustomValidity('');
      if(control.reportValidity()!==true||events!==1)failures.push([tag,'clear']);
    }return failures;
  })()`),[]);
});
