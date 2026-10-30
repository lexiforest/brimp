import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import { createSession } from '../../../scripts/coverage/session.mjs';
import * as corpus from '../../../scripts/coverage/input-value-conversion-cases.mjs';
async function evaluate(source) {
  const s = await createSession();
  try {
    return await (await s.newPage()).evaluate(source);
  } finally {
    await s.close();
  }
}
for (const [key, cases, measure] of [
  ['cases', corpus.inputValueConversionCases, corpus.measureInputValueConversions],
  ['time_cases', corpus.inputTimeSerializationCases, corpus.measureInputTimeSerialization],
  ['getter_cases', corpus.inputConversionGetterCases, corpus.measureInputConversionGetters],
])
  test(`input value conversions: ${cases.length} Safari ${key}`, async () => {
    const ref = JSON.parse(
      fs.readFileSync(new URL('../fixtures/input-value-conversions.safari.json', import.meta.url)),
    )[key];
    assert.deepEqual(
      ref.map(({ expected, ...c }) => c),
      cases,
    );
    const actual = JSON.parse(
      await evaluate(`JSON.stringify((${measure})(${JSON.stringify(cases)}))`),
    );
    assert.deepEqual(
      cases.flatMap((c, i) =>
        JSON.stringify(actual[i]) === JSON.stringify(ref[i].expected)
          ? []
          : [{ ...c, actual: actual[i], expected: ref[i].expected }],
      ),
      [],
    );
  });

test('value conversion descriptors and numeric coercion use canonical state after conversion', async () => {
  assert.deepEqual(
    await evaluate(`(() => {
    const failures=[],input=document.createElement('input');
    for(const name of ['valueAsNumber','valueAsDate']){
      const d=Object.getOwnPropertyDescriptor(HTMLInputElement.prototype,name);
      if(!d.enumerable||!d.configurable||d.get.name!=='get '+name||d.set.name!=='set '+name)failures.push([name,'descriptor']);
      let coerced=false;try{d.set.call({}, {[Symbol.toPrimitive](){coerced=true;return 1;}});failures.push('brand');}catch(e){if(e.name!=='TypeError'||coerced)failures.push('brand conversion');}
      try{d.get.call({});failures.push('getter brand');}catch(e){if(e.name!=='TypeError')failures.push(e.name);}
    }
    input.type='text';let count=0;input.valueAsNumber={[Symbol.toPrimitive](hint){count++;if(hint!=='number')failures.push('hint');input.type='range';input.min='10';input.max='20';input.step='2';return 15;}};
    if(input.value!=='16'||count!==1)failures.push('conversion state');
    input.type='number';input.value='3';let name='';try{input.valueAsNumber={valueOf(){input.type='text';return 12;}};}catch(e){name=e.name;}
    if(name!=='InvalidStateError'||input.value!=='3')failures.push('converted type');
    input.type='date';input.getAttribute=input.hasAttribute=()=>{throw Error('override');};
    Object.defineProperty(input,'type',{value:'text'});Object.defineProperty(input,'value',{get(){throw Error('override');},set(){throw Error('override');}});
    input.valueAsNumber=86400000;if(input.valueAsNumber!==86400000||input.valueAsDate.getTime()!==86400000)failures.push('canonical');
    input.valueAsDate=new Date(0);if(input.valueAsNumber!==0)failures.push('Date canonical');
    return failures;
  })()`),
    [],
  );
});

test('date conversion uses Date internals, returns fresh Date values and changes no attributes or input events', async () => {
  assert.deepEqual(
    await evaluate(`(() => {
    const failures=[],input=document.createElement('input');input.type='date';input.defaultValue='2000-01-01';
    const observer=new MutationObserver(()=>{});observer.observe(input,{attributes:true});const events=[];for(const type of ['input','change'])input.addEventListener(type,()=>events.push(type));
    const date=new Date(0);date.valueOf=date.getTime=()=>{throw Error('override');};input.valueAsDate=date;
    const a=input.valueAsDate,b=input.valueAsDate;if(a===b||a===date||a.getTime()!==0)failures.push('fresh Date');a.setTime(1);if(input.valueAsNumber!==0)failures.push('independence');
    const OriginalDate=Date,originalGetTime=Date.prototype.getTime,originalParse=Date.parse;
    try{globalThis.Date=function(){throw Error('constructor override');};OriginalDate.prototype.getTime=()=>{throw Error('getTime override');};OriginalDate.parse=()=>{throw Error('parse override');};
      input.valueAsDate=date;if(Reflect.apply(originalGetTime,input.valueAsDate,[])!==0)failures.push('intrinsic');input.valueAsDate='2001-01-01';
      if(input.valueAsNumber!==978307200000)failures.push('parse intrinsic');
    }finally{globalThis.Date=OriginalDate;OriginalDate.prototype.getTime=originalGetTime;OriginalDate.parse=originalParse;}
    input.valueAsDate={get [Symbol.toStringTag](){throw Error('tag');},[Symbol.toPrimitive](){throw Error('primitive');}};
    if(input.value!=='2000-01-01')failures.push('invalid restores default');input.defaultValue='2002-01-01';observer.takeRecords();
    input.valueAsNumber=0;if(observer.takeRecords().length||events.length)failures.push('side effects');
    const clone=input.cloneNode();input.valueAsNumber=86400000;if(clone.valueAsNumber!==0)failures.push('clone');
    const form=document.createElement('form');form.append(input);form.reset();if(input.value!=='2002-01-01')failures.push('reset');
    return failures;
  })()`),
    [],
  );
});
