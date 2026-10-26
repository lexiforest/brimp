import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {createSession} from '../../../scripts/coverage/session.mjs';
import {inputSteppingCases,measureInputStepping} from '../../../scripts/coverage/input-stepping-cases.mjs';
async function evaluate(source){const s=await createSession();try{return await(await s.newPage()).evaluate(source);}finally{await s.close();}}
test(`input stepping: ${inputSteppingCases.length} Safari reference cases`,async()=>{
  const reference=JSON.parse(fs.readFileSync(new URL('./fixtures/input-stepping.safari.json',import.meta.url))).cases;
  assert.deepEqual(reference.map(({expected,...c})=>c),inputSteppingCases);
  const actual=JSON.parse(await evaluate(`JSON.stringify((${measureInputStepping})(${JSON.stringify(inputSteppingCases)}))`));
  assert.deepEqual(inputSteppingCases.flatMap((c,i)=>JSON.stringify(actual[i])===JSON.stringify(reference[i].expected)?[]:[{...c,actual:actual[i],expected:reference[i].expected}]),[]);
});

test('stepping descriptors, receiver checks and count conversion follow Safari',async()=>{
  assert.deepEqual(await evaluate(`(() => {
    const failures=[];
    for(const name of ['stepUp','stepDown']){
      const d=Object.getOwnPropertyDescriptor(HTMLInputElement.prototype,name),fn=d.value;
      if(!d.enumerable||!d.configurable||!d.writable||fn.name!==name||fn.length!==0)failures.push(name+' descriptor');
      let converted=false;try{fn.call({}, {valueOf(){converted=true;return 1;}});failures.push('receiver');}catch(e){if(e.name!=='TypeError'||converted)failures.push('receiver conversion');}
      try{new fn();failures.push('constructor');}catch(e){if(e.name!=='TypeError')failures.push('constructor error');}
    }
    const input=document.createElement('input');let calls=0;
    input.stepUp({[Symbol.toPrimitive](hint){calls++;if(hint!=='number')failures.push('hint');input.type='number';input.value='0.5';input.step='0.2';return 2;}});
    if(input.value!=='0.8'||calls!==1)failures.push('coercion before state');
    let error;try{input.stepDown({valueOf(){input.type='text';return 1;}});}catch(e){error=e.name;}
    if(error!=='InvalidStateError'||input.value!=='0.8')failures.push('type mutation');
    input.type='number';input.step='any';calls=0;
    try{input.stepUp({valueOf(){calls++;throw new RangeError('conversion');}});}catch(e){error=e.name;}
    if(error!=='RangeError'||calls!==1)failures.push('conversion before step error');
    return failures;
  })()`),[]);
});

test('stepping uses canonical input state and preserves clone/reset and event semantics',async()=>{
  assert.deepEqual(await evaluate(`(() => {
    const failures=[],form=document.createElement('form'),input=document.createElement('input');
    document.body.append(form);form.append(input);input.type='number';input.defaultValue='0.2';input.step='0.1';
    const observer=new MutationObserver(()=>{});observer.observe(input,{attributes:true});const events=[];
    for(const type of ['input','change','invalid'])input.addEventListener(type,()=>events.push(type));
    input.stepUp();if(input.value!=='0.3')failures.push('value');
    const clone=input.cloneNode();input.stepDown();if(clone.value!=='0.3')failures.push('clone independence');
    if(observer.takeRecords().length||events.length)failures.push('events or attributes');
    input.defaultValue='0.5';if(input.value!=='0.2')failures.push('dirty');form.reset();if(input.value!=='0.5')failures.push('reset');
    const nativeValue=Object.getOwnPropertyDescriptor(HTMLInputElement.prototype,'value').get;
    input.getAttribute=input.hasAttribute=()=>{throw Error('attribute override');};
    for(const name of ['type','value','valueAsNumber','min','max','step','defaultValue'])Object.defineProperty(input,name,{get(){throw Error(name);},set(){throw Error(name);}});
    input.stepUp();if(nativeValue.call(input)!=='0.6')failures.push('canonical state');
    observer.disconnect();form.remove();return failures;
  })()`),[]);
});

test('repeated stepping avoids drift and handles calendar rollover and time saturation',async()=>{
  assert.deepEqual(await evaluate(`(() => {
    const result=[],input=document.createElement('input');input.type='number';input.step='0.1';input.value='0';
    for(let i=0;i<100;i++)input.stepUp();result.push(input.value);
    for(let i=0;i<100;i++)input.stepDown();result.push(input.value);
    input.type='date';input.step='1';input.value='2000-02-27';for(let i=0;i<4;i++)input.stepUp();result.push(input.value);
    input.type='time';input.step='0.001';input.value='23:59:59.99';for(let i=0;i<20;i++)input.stepUp();result.push(input.value);
    return result;
  })()`),['10','0','2000-03-02','23:59:59.999']);
});
