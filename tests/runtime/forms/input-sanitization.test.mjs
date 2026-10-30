import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import { createSession } from '../../../scripts/coverage/session.mjs';
import {
  inputSanitizationCases as cases,
  measureInputSanitization as measure,
} from '../../../scripts/coverage/input-sanitization-cases.mjs';

const reference = JSON.parse(
  fs.readFileSync(new URL('../fixtures/input-sanitization.safari.json', import.meta.url), 'utf8'),
);

test('input temporal and numeric sanitization matches measured Safari across four mutation paths', async () => {
  assert.deepEqual(
    reference.cases.map(({ expected, ...c }) => c),
    cases,
    'Safari fixture must cover the current case list',
  );
  const session = await createSession();
  try {
    const actual = await (
      await session.newPage()
    ).evaluate(`(${measure})(${JSON.stringify(cases)})`);
    const failures = cases.flatMap((c, i) =>
      JSON.stringify(actual[i]) === JSON.stringify(reference.cases[i].expected)
        ? []
        : [{ ...c, actual: actual[i], expected: reference.cases[i].expected }],
    );
    assert.deepEqual(failures, []);
  } finally {
    await session.close();
  }
});

test('range constraint changes preserve Safari dirty values and reset restores the current default', async () => {
  const session = await createSession();
  try {
    assert.deepEqual(
      await (
        await session.newPage()
      ).evaluate(`(() => {
      const failures=[],changes=[['max','60'],['max','100'],['step','17'],['step','1'],['min','90'],['min','0'],['value','0.5'],['step','2'],['value','1']];
      for(const dirty of [false,true]) {
        const i=document.createElement('input');i.type='range';if(dirty)i.value='80';
        const expected=dirty?['60','60','60','60','90','90','90','90','90']:['30','50','51','50','95','50','1','0','2'];
        changes.forEach(([k,v],n)=>{i.setAttribute(k,v);if(i.value!==expected[n])failures.push([dirty,k,v,i.value,expected[n]])});
        const form=document.createElement('form');form.appendChild(i);form.reset();if(i.value!=='2')failures.push(['reset',i.value]);
      }
      const i=document.createElement('input');i.type='range';i.value='80';
      i.setAttributeNS('urn:test','max','10');if(i.value!=='80')failures.push('foreign constraint');
      i.setAttributeNS(null,'max','10');if(i.value!=='10')failures.push('canonical constraint');
      i.removeAttributeNS(null,'max');if(i.value!=='10')failures.push('restore dirty');
      i.getAttribute=i.setAttribute=()=>{throw Error('override');};
      Object.defineProperty(i,'min',{value:1000});i.value='6';if(i.value!=='6')failures.push('canonical minimum');
      return failures;
    })()`),
      [],
    );
  } finally {
    await session.close();
  }
});

test('input min/max/step reflect canonical DOMStrings and preserve Safari mutation behavior', async () => {
  const session = await createSession();
  try {
    assert.deepEqual(
      await (
        await session.newPage()
      ).evaluate(`(() => {
      const failures=[];
      for(const name of ['min','max','step']) {
        const i=document.createElement('input');if(i[name]!=='')failures.push([name,'default']);
        const observer=new MutationObserver(()=>{});observer.observe(i,{attributes:true,attributeOldValue:true});
        for(const value of [null,undefined,0,-1,NaN,Infinity,'ANY',' invalid ',{toString(){return 'object';}}]) {
          i[name]=value;if(i[name]!==String(value)||i.getAttribute(name)!==String(value))failures.push([name,'conversion']);
        }
        const before=i[name];try{i[name]=Symbol();failures.push([name,'symbol']);}catch(e){if(e.name!=='TypeError')failures.push([name,e.name]);}
        if(i[name]!==before)failures.push([name,'throw mutated']);
        const records=observer.takeRecords().map(r=>[r.attributeName,r.attributeNamespace,r.oldValue]);observer.disconnect();
        if(records.length!==9||JSON.stringify(records[0])!==JSON.stringify([name,null,null])||records[1][2]!=='null')failures.push([name,'records',records]);
        i.removeAttribute(name);i.setAttributeNS('urn:test',name,'foreign');if(i[name]!=='')failures.push([name,'namespace']);
        i.getAttribute=i.setAttribute=()=>{throw Error('override');};i[name]='canonical';if(i[name]!=='canonical')failures.push([name,'override']);
        const d=Object.getOwnPropertyDescriptor(HTMLInputElement.prototype,name);
        if(!d.enumerable||!d.configurable||d.get.name!=='get '+name||d.set.name!=='set '+name)failures.push([name,'descriptor']);
        for(const fn of [d.get,d.set]){try{fn.call({},'x');failures.push([name,'brand']);}catch(e){if(e.name!=='TypeError')failures.push([name,e.name]);}}
      }
      const i=document.createElement('input');i.type='range';i.value='80';i.max='60';i.max='100';if(i.value!=='60')failures.push('IDL maximum');
      i.min='90';i.min='0';i.step='17';if(i.value!=='90')failures.push('IDL minimum/step');
      i.value='80';if(i.value!=='85')failures.push('step applied on assignment');
      return failures;
    })()`),
      [],
    );
  } finally {
    await session.close();
  }
});
