import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {createSession} from '../../../scripts/coverage/session.mjs';
import {inputColorCases as cases, measureInputColors as measure, measureColorTransitions} from '../../../scripts/coverage/input-color-cases.mjs';

const reference=JSON.parse(fs.readFileSync(new URL('../fixtures/input-color.safari.json',import.meta.url),'utf8'));

test('color input sRGB and Display-P3 sanitization matches installed Safari across assignment, reset, type and parser paths', async () => {
  assert.deepEqual(reference.cases.map(({expected,...c})=>c),cases);
  const session=await createSession();
  try {
    const actual=await (await session.newPage()).evaluate(`(${measure})(${JSON.stringify(cases)})`);
    assert.deepEqual(cases.flatMap((c,i)=>JSON.stringify(actual[i])===JSON.stringify(reference.cases[i].expected)?[]:
      [{...c,actual:actual[i],expected:reference.cases[i].expected}]),[]);
  } finally {await session.close();}
});

test('color space mutations preserve Safari dirty/pristine values through five attribute paths, reset and cloning', async () => {
  const session=await createSession();
  try {
    assert.deepEqual(await (await session.newPage()).evaluate(`(${measureColorTransitions})()`),reference.transitions);
  } finally {await session.close();}
});

test('colorSpace is a canonical case-insensitive enum reflection with Safari conversions and descriptors', async () => {
  const session=await createSession();
  try {
    assert.deepEqual(await (await session.newPage()).evaluate(`(()=>{
      const failures=[],i=document.createElement('input');i.type='color';
      for(const value of [null,undefined,'','DISPLAY-P3','display-p3','limited-srgb',' display-p3','invalid',{toString(){return 'DISPLAY-P3';}}]) {
        i.colorSpace=value;const raw=String(value),expected=raw.toLowerCase()==='display-p3'?'display-p3':'limited-srgb';
        if(i.colorSpace!==expected||i.getAttribute('colorspace')!==raw)failures.push(['reflection',raw]);
      }
      const before=i.colorSpace;try{i.colorSpace=Symbol();failures.push('symbol');}catch(e){if(e.name!=='TypeError')failures.push(e.name);}
      if(i.colorSpace!==before)failures.push('conversion mutated');
      i.removeAttribute('colorspace');i.setAttributeNS('urn:foreign','colorspace','display-p3');
      if(i.colorSpace!=='limited-srgb')failures.push('namespace');
      const observer=new MutationObserver(()=>{});observer.observe(i,{attributes:true,attributeOldValue:true});
      i.colorSpace='display-p3';i.colorSpace='LIMITED-SRGB';
      const records=observer.takeRecords().map(r=>[r.attributeName,r.attributeNamespace,r.oldValue]);observer.disconnect();
      if(JSON.stringify(records)!==JSON.stringify([['colorspace',null,null],['colorspace',null,'display-p3']]))failures.push(['records',records]);
      i.getAttribute=i.setAttribute=()=>{throw Error('override');};i.colorSpace='display-p3';i.value='color(display-p3 .1 .2 .3)';
      if(i.value!=='color(display-p3 0.1 0.2 0.3)')failures.push('canonical');
      Object.defineProperty(i,'colorSpace',{value:'limited-srgb'});i.value='color(display-p3 .1 .2 .3)';
      if(i.value!=='color(display-p3 0.1 0.2 0.3)')failures.push('own override');
      const d=Object.getOwnPropertyDescriptor(HTMLInputElement.prototype,'colorSpace');
      if(!d.enumerable||!d.configurable||d.get.name!=='get colorSpace'||d.set.name!=='set colorSpace')failures.push('descriptor');
      for(const fn of [d.get,d.set])try{fn.call({},'display-p3');failures.push('brand');}catch(e){if(e.name!=='TypeError')failures.push(e.name);}
      if('__inputColor' in globalThis)failures.push('private parser leaked');
      return failures;
    })()`),[]);
  } finally {await session.close();}
});

test('color alpha reflection and mutations preserve dirty values, defaults and canonical access', async () => {
  const session=await createSession();
  try {
    assert.deepEqual(await (await session.newPage()).evaluate(`(()=>{
      const failures=[];
      for(const dirty of [false,true]) {
        const form=document.createElement('form'),i=document.createElement('input');form.appendChild(i);
        i.type='color';i.defaultValue='#12345678';if(dirty)i.value='#abcdef89';
        i.alpha=true;
        const expected=dirty?'color(srgb 0.670588 0.803922 0.937255)':'color(srgb 0.070588 0.203922 0.337255 / 0.470588)';
        if(i.value!==expected)failures.push(['enable',dirty,i.value]);
        i.alpha=false;if(i.value!==(dirty?'#abcdef':'#123456'))failures.push(['disable',dirty,i.value]);
        i.setAttributeNS('urn:foreign','alpha','');if(i.alpha)failures.push('namespace');
        i.setAttributeNS(null,'alpha','');if(i.value!==expected)failures.push(['canonical',dirty,i.value]);
        form.reset();if(i.value!=='color(srgb 0.070588 0.203922 0.337255 / 0.470588)')failures.push(['reset',dirty,i.value]);
        const clone=i.cloneNode();if(!clone.alpha||clone.value!==i.value)failures.push('clone');
      }
      const i=document.createElement('input');i.type='color';
      i.style.color='red';i.value='currentcolor';if(i.value!=='#000000')failures.push('currentColor');
      for(const value of [null,undefined,false,0,'',NaN,true,1,'false',{},Symbol()]) {
        i.alpha=value;if(i.alpha!==Boolean(value)||i.hasAttribute('alpha')!==Boolean(value))failures.push('boolean');
      }
      i.getAttribute=i.setAttribute=i.hasAttribute=i.removeAttribute=()=>{throw Error('override');};
      i.alpha=false;i.value='red';if(i.value!=='#ff0000')failures.push('overrides');
      Object.defineProperty(i,'alpha',{value:true});i.value='#12345678';if(i.value!=='#123456')failures.push('own alpha');
      const descriptor=Object.getOwnPropertyDescriptor(HTMLInputElement.prototype,'alpha');
      if(!descriptor.enumerable||!descriptor.configurable||descriptor.get.name!=='get alpha'||descriptor.set.name!=='set alpha')failures.push('descriptor');
      for(const fn of [descriptor.get,descriptor.set])try{fn.call({},true);failures.push('brand');}catch(e){if(e.name!=='TypeError')failures.push(e.name);}
      return failures;
    })()`),[]);
  } finally {await session.close();}
});
