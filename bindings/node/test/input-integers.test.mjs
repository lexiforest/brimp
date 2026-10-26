import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {createSession} from '../../../scripts/coverage/session.mjs';
import {inputIntegerCases as cases, measureInputIntegers as measure, inputLengthConstraintCases, measureLengthConstraints} from '../../../scripts/coverage/input-integer-cases.mjs';

async function evaluate(source) {
  const session=await createSession();
  try { return await (await session.newPage()).evaluate(source); }
  finally { await session.close(); }
}

test('input integer parsing, conversion, exceptions and records match 384 Safari cases', async () => {
  const reference=JSON.parse(fs.readFileSync(new URL('./fixtures/input-integers.safari.json',import.meta.url)));
  assert.deepEqual(reference.cases.map(({expected,...c})=>c),cases);
  const actual=JSON.parse(await evaluate(`JSON.stringify((${measure})(${JSON.stringify(cases)}))`));
  assert.deepEqual(cases.flatMap((c,i)=>JSON.stringify(actual[i])===JSON.stringify(reference.cases[i].expected)?[]:
    [{...c,actual:actual[i],expected:reference.cases[i].expected}]),[]);
});

test('input numeric metadata uses canonical attributes and remains independent of type and script values', async () => {
  assert.deepEqual(await evaluate(`(() => {
    const failures=[];
    for(const name of ['maxLength','minLength','size']) {
      const i=document.createElement('input'),attr=name.toLowerCase(),missing=name==='size'?20:-1;
      if(i[name]!==missing)failures.push([name,'default']);
      i.setAttributeNS('urn:test',attr,'43');if(i[name]!==missing)failures.push([name,'namespace']);
      i[name]=8;if(i.getAttributeNS(null,attr)!=='8'||i.getAttributeNS('urn:test',attr)!=='43')failures.push([name,'identity']);
      for(const type of ['text','search','tel','url','email','password','number','range','hidden','checkbox','radio','image','file']) {
        i.type=type;if(i[name]!==8)failures.push([name,type]);
      }
      const clone=i.cloneNode();i[name]=9;if(clone[name]!==8)failures.push([name,'clone']);
      i.getAttribute=i.setAttribute=i.hasAttribute=i.removeAttribute=()=>{throw Error('override');};
      i[name]=10;if(i[name]!==10)failures.push([name,'override']);
    }
    const i=document.createElement('input');i.value='long scripted value';i.maxLength=1;i.setAttribute('minlength','99');i.size=1;
    if(i.value!=='long scripted value'||i.maxLength!==1||i.minLength!==99||i.validity.tooLong||i.validity.tooShort)failures.push('script value');
    i.value='still longer';if(i.value!=='still longer')failures.push('script assignment is not truncated');
    return failures;
  })()`),[]);
});

test('length setters enforce the opposite canonical bound after conversion, as in Safari', async () => {
  const reference=JSON.parse(fs.readFileSync(new URL('./fixtures/input-integers.safari.json',import.meta.url)));
  assert.deepEqual(reference.constraints.map(({expected,...c})=>c),inputLengthConstraintCases);
  const actual=await evaluate(`(${measureLengthConstraints})(${JSON.stringify(inputLengthConstraintCases)})`);
  inputLengthConstraintCases.forEach((c,i)=>assert.deepEqual(actual[i],reference.constraints[i].expected,JSON.stringify(c)));
  assert.deepEqual(await evaluate(`(() => {
    const failures=[],i=document.createElement('input');
    i.setAttributeNS('urn:test','maxlength','0');i.minLength=5;
    Object.defineProperty(i,'minLength',{value:0});
    try{i.maxLength=4;failures.push('canonical lower bound');}catch(e){if(e.name!=='IndexSizeError')failures.push(e.name);}
    i.maxLength={valueOf(){i.setAttribute('minlength','2');return 3;}};
    if(i.maxLength!==3)failures.push('bound read after conversion');
    return failures;
  })()`),[]);
});

test('input numeric descriptors brand-check before conversion and coerce once with a number hint', async () => {
  assert.deepEqual(await evaluate(`(() => {
    const failures=[];
    for(const name of ['maxLength','minLength','size']) {
      const i=document.createElement('input'),d=Object.getOwnPropertyDescriptor(HTMLInputElement.prototype,name);
      if(!d.enumerable||!d.configurable||d.get.name!=='get '+name||d.set.name!=='set '+name||d.get.length!==0||d.set.length!==1)failures.push([name,'descriptor']);
      let hints=[];const value={[Symbol.toPrimitive](hint){hints.push(hint);return 12.9;}};
      d.set.call(i,value);if(i[name]!==12||hints.join()!=='number')failures.push([name,'conversion']);
      for(const receiver of [{},document.createElement('div'),Object.create(HTMLInputElement.prototype)]) {
        hints=[];try{d.set.call(receiver,value);failures.push([name,'brand']);}catch(e){if(e.name!=='TypeError')failures.push([name,e.name]);}
        if(hints.length)failures.push([name,'brand before conversion']);
        try{d.get.call(receiver);failures.push([name,'getter brand']);}catch(e){if(e.name!=='TypeError')failures.push([name,e.name]);}
      }
      i[name]=7;let calls=0;
      try{i[name]={valueOf(){calls++;i.setAttribute(name.toLowerCase(),'15');return name==='size'?0:-1;}};failures.push([name,'expected range error']);}
      catch(e){if(e.name!=='IndexSizeError'||e.code!==1)failures.push([name,'range error']);}
      if(calls!==1||i[name]!==15)failures.push([name,'conversion order']);
    }
    return failures;
  })()`),[]);
});
