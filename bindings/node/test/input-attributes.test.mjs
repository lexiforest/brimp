import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {createSession} from '../../../scripts/coverage/session.mjs';
import {inputAutocompleteCases as cases,measureAutocomplete} from '../../../scripts/coverage/input-autocomplete-cases.mjs';

const strings=['accept','align','alt','dirName','name','pattern','placeholder','useMap','formTarget'];
const booleans=['disabled','formNoValidate','multiple','readOnly','required'];

test('input string reflections preserve well-formed DOMStrings, attribute identity and canonical access',async()=>{
  const session=await createSession();
  try{assert.deepEqual(await(await session.newPage()).evaluate(`(()=>{
    const failures=[];
    for(const name of ${JSON.stringify(strings)}){
      const i=document.createElement('input'),attribute=name.toLowerCase();
      if(i[name]!=='')failures.push([name,'default']);
      for(const value of [null,undefined,0,-1,NaN,Infinity,123n,'','  a\\n\\rb\\t  ','你好💡',String.fromCharCode(0),{toString(){return 'object';}}]){
        i[name]=value;if(i[name]!==String(value)||i.getAttribute(attribute)!==String(value))failures.push([name,'conversion',String(value),i[name]]);
      }
      const before=i[name];for(const value of [Symbol(),{[Symbol.toPrimitive](){throw new RangeError('conversion');}}]){
        try{i[name]=value;failures.push([name,'expected throw']);}catch(e){if(!['TypeError','RangeError'].includes(e.name))failures.push([name,e.name]);}
        if(i[name]!==before)failures.push([name,'mutated before throw']);
      }
      const hints=[];i[name]={[Symbol.toPrimitive](hint){hints.push(hint);return 'converted';}};
      if(hints.join()!=='string')failures.push([name,'hint']);
      i.removeAttribute(attribute);i.setAttributeNS('urn:test',attribute,'foreign');if(i[name]!=='')failures.push([name,'namespace']);
      i[name]='canonical';if(i.getAttributeNS(null,attribute)!=='canonical'||i.getAttributeNS('urn:test',attribute)!=='foreign')failures.push([name,'identity']);
      i.getAttribute=i.setAttribute=i.hasAttribute=i.removeAttribute=()=>{throw Error('override');};i[name]='native';if(i[name]!=='native')failures.push([name,'override']);
      const d=Object.getOwnPropertyDescriptor(HTMLInputElement.prototype,name);
      if(!d.enumerable||!d.configurable||d.get.name!=='get '+name||d.set.name!=='set '+name||d.get.length!==0||d.set.length!==1)failures.push([name,'descriptor']);
      for(const receiver of [{},document.createElement('div'),Object.create(HTMLInputElement.prototype)])for(const fn of [d.get,d.set]){
        try{fn.call(receiver,'x');failures.push([name,'brand']);}catch(e){if(e.name!=='TypeError')failures.push([name,e.name]);}
      }
    }
    return failures;
  })()`),[]);}finally{await session.close();}
});

test('input boolean reflections use attribute presence and WebIDL boolean conversion',async()=>{
  const session=await createSession();
  try{assert.deepEqual(await(await session.newPage()).evaluate(`(()=>{
    const failures=[];
    for(const name of ${JSON.stringify(booleans)}){
      const i=document.createElement('input'),attribute=name.toLowerCase();
      if(i[name]!==false)failures.push([name,'default']);
      for(const value of [null,undefined,false,0,-0,NaN,0n,'',true,1,1n,'false','0',Symbol(),{[Symbol.toPrimitive](){throw Error('not converted');}}]){
        i[name]=value;if(i[name]!==Boolean(value)||i.hasAttribute(attribute)!==Boolean(value)||value&&i.getAttribute(attribute)!=='')failures.push([name,'conversion']);
      }
      i.setAttribute(attribute,'false');if(!i[name])failures.push([name,'presence']);
      i.removeAttribute(attribute);i.setAttributeNS('urn:test',attribute,'foreign');if(i[name])failures.push([name,'namespace']);
      i[name]=true;i[name]=false;if(i.getAttributeNS('urn:test',attribute)!=='foreign')failures.push([name,'identity']);
      i.getAttribute=i.setAttribute=i.hasAttribute=i.removeAttribute=()=>{throw Error('override');};i[name]=true;if(!i[name])failures.push([name,'override']);
      const d=Object.getOwnPropertyDescriptor(HTMLInputElement.prototype,name);
      if(!d.enumerable||!d.configurable||d.get.name!=='get '+name||d.set.name!=='set '+name)failures.push([name,'descriptor']);
      for(const receiver of [{},document.createElement('div'),Object.create(HTMLInputElement.prototype)])for(const fn of [d.get,d.set]){
        try{fn.call(receiver,true);failures.push([name,'brand']);}catch(e){if(e.name!=='TypeError')failures.push([name,e.name]);}
      }
    }
    const i=document.createElement('input');i.type='email';i.value='a@b, c@d';i.multiple=true;if(i.value!=='a@b,c@d')failures.push(['multiple',i.value]);
    const f=document.createElement('fieldset');f.disabled=true;f.appendChild(i);if(i.disabled)failures.push('disabled reflects own attribute only');
    if('incremental' in i||Object.getOwnPropertyDescriptor(HTMLInputElement.prototype,'incremental'))failures.push('incremental absent in Safari');
    return failures;
  })()`),[]);}finally{await session.close();}
});

test('input reflections emit Safari attribute records for repeated assignments and removals',async()=>{
  const session=await createSession();
  try{assert.deepEqual(await(await session.newPage()).evaluate(`(()=>{
    const failures=[];
    const names=${JSON.stringify([...strings,...booleans,'autocomplete'])};
    const i=document.createElement('input');
    for(const name of names){
      const attribute=name.toLowerCase(),boolean=${JSON.stringify(booleans)}.includes(name);
      const observer=new MutationObserver(()=>{});observer.observe(i,{attributes:true,attributeOldValue:true});
      i[name]=boolean?true:'one';i[name]=boolean?true:'one';i[name]=boolean?false:'two';if(boolean)i[name]=false;
      const expected=boolean?[[attribute,null,''],[attribute,'',''],[attribute,'',null]]:[[attribute,null,'one'],[attribute,'one','one'],[attribute,'one','two']];
      const records=observer.takeRecords().map(r=>[r.attributeName,r.oldValue,r.attributeNamespace]);observer.disconnect();
      if(JSON.stringify(records)!==JSON.stringify(expected.map(([a,old])=>[a,old,null])))failures.push([name,'records',records]);
    }
    return failures;
  })()`),[]);}finally{await session.close();}
});

test('input autocomplete follows the measured Safari token grammar and live type changes',async()=>{
  const reference=JSON.parse(fs.readFileSync(new URL('./fixtures/input-autocomplete.safari.json',import.meta.url),'utf8'));
  assert.deepEqual(reference.cases.map(({expected,...c})=>c),cases);
  const session=await createSession();
  try{
    const actual=await(await session.newPage()).evaluate(`(${measureAutocomplete})(${JSON.stringify(cases)})`);
    assert.deepEqual(cases.flatMap((c,i)=>JSON.stringify(actual[i])===JSON.stringify(reference.cases[i].expected)?[]:[{...c,actual:actual[i],expected:reference.cases[i].expected}]),[]);
  }finally{await session.close();}
});

test('autocomplete uses canonical attributes, DOMString conversion and Safari descriptors',async()=>{
  const session=await createSession();
  try{assert.deepEqual(await(await session.newPage()).evaluate(`(()=>{
    const failures=[],i=document.createElement('input');
    for(const value of [null,undefined,123n,{toString(){return 'SECTION-X SHIPPING EMAIL';}}]){i.autocomplete=value;if(i.getAttribute('autocomplete')!==String(value))failures.push('conversion');}
    try{i.autocomplete=Symbol();failures.push('symbol');}catch(e){if(e.name!=='TypeError')failures.push(e.name);}
    i.removeAttribute('autocomplete');i.setAttributeNS('urn:test','autocomplete','email');if(i.autocomplete!=='')failures.push('namespace');
    i.getAttribute=i.setAttribute=()=>{throw Error('override');};i.autocomplete='on';
    i.type='hidden';Object.defineProperty(i,'type',{value:'text'});if(i.autocomplete!=='')failures.push('canonical type');
    i.autocomplete='section-K HOME EMAIL';if(i.autocomplete!=='section-K home email')failures.push('ASCII folding');
    const d=Object.getOwnPropertyDescriptor(HTMLInputElement.prototype,'autocomplete');
    if(!d.enumerable||!d.configurable||d.get.name!=='get autocomplete'||d.set.name!=='set autocomplete')failures.push('descriptor');
    for(const fn of [d.get,d.set])try{fn.call({},'email');failures.push('brand');}catch(e){if(e.name!=='TypeError')failures.push(e.name);}
    return failures;
  })()`),[]);}finally{await session.close();}
});
