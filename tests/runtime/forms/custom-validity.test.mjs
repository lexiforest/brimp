import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import { createSession } from '../../../scripts/coverage/session.mjs';
import {
  customValidityCases as cases,
  measureCustomValidity as measure,
} from '../../../scripts/coverage/custom-validity-cases.mjs';
async function evaluate(source) {
  const s = await createSession();
  try {
    return await (await s.newPage()).evaluate(source);
  } finally {
    await s.close();
  }
}

test(`custom validity matches ${cases.length} Safari conversion and event cases`, async () => {
  const reference = JSON.parse(
    fs.readFileSync(new URL('../fixtures/custom-validity.safari.json', import.meta.url)),
  ).cases;
  assert.deepEqual(
    reference.map(({ expected, ...c }) => c),
    cases,
  );
  const actual = JSON.parse(
    await evaluate(`JSON.stringify((${measure})(${JSON.stringify(cases)}))`),
  );
  assert.deepEqual(
    reference.flatMap((c, i) =>
      JSON.stringify(c.expected) === JSON.stringify(actual[i]) ? [] : [{ ...c, actual: actual[i] }],
    ),
    [],
  );
});

test('custom validity methods and button validity descriptors are branded with Safari shape', async () => {
  assert.deepEqual(
    await evaluate(`(() => {
    const failures=[];
    for(const [tag,ctor] of [['input',HTMLInputElement],['button',HTMLButtonElement]]){
      const control=document.createElement(tag),method=ctor.prototype.setCustomValidity;
      const d=Object.getOwnPropertyDescriptor(ctor.prototype,'setCustomValidity');
      if(!d.enumerable||!d.configurable||!d.writable||method.name!=='setCustomValidity'||method.length!==1)failures.push([tag,'method descriptor']);
      try{new method('x');failures.push('constructible method');}catch(e){if(e.name!=='TypeError')failures.push(e.name);}
      for(const receiver of [{},document.createElement('div'),Object.create(ctor.prototype)]){
        let converted=false;try{method.call(receiver,{toString(){converted=true;return 'x';}});failures.push('receiver');}catch(e){if(e.name!=='TypeError'||converted)failures.push('brand before conversion');}
      }
      control.setCustomValidity('before');try{control.setCustomValidity();failures.push('missing argument');}catch(e){if(e.name!=='TypeError')failures.push(e.name);}
      if(control.validationMessage!=='before')failures.push('missing changed state');
    }
    const button=document.createElement('button');
    for(const name of ['validity','validationMessage']){
      const d=Object.getOwnPropertyDescriptor(HTMLButtonElement.prototype,name);
      if(!d.enumerable||!d.configurable||d.set!==undefined||d.get.name!=='get '+name||d.get.length!==0)failures.push([name,'descriptor']);
      try{d.get.call({});failures.push([name,'receiver']);}catch(e){if(e.name!=='TypeError')failures.push(e.name);}
      if(Reflect.set(button,name,{}))failures.push([name,'readonly']);
    }
    const d=Object.getOwnPropertyDescriptor(HTMLButtonElement.prototype,'checkValidity');
    if(!d.enumerable||!d.configurable||!d.writable||d.value.name!=='checkValidity'||d.value.length!==0)failures.push('checkValidity descriptor');
    try{d.value.call({});failures.push('checkValidity receiver');}catch(e){if(e.name!=='TypeError')failures.push(e.name);}
    const state=button.validity;if(state!==button.validity||Object.getPrototypeOf(state)!==ValidityState.prototype||Object.prototype.toString.call(state)!=='[object ValidityState]')failures.push('state identity');
    const flags=['valueMissing','typeMismatch','patternMismatch','tooLong','tooShort','rangeUnderflow','rangeOverflow','stepMismatch','badInput','customError','valid'];
    for(const flag of flags){const descriptor=Object.getOwnPropertyDescriptor(ValidityState.prototype,flag);
      if(!descriptor.enumerable||!descriptor.configurable||descriptor.set!==undefined||descriptor.get.name!=='get '+flag||descriptor.get.length!==0)failures.push([flag,'descriptor']);
      for(const receiver of [{},Object.create(ValidityState.prototype),button])try{descriptor.get.call(receiver);failures.push([flag,'brand']);}catch(e){if(e.name!=='TypeError')failures.push(e.name);}
      if(Reflect.set(state,flag,true))failures.push([flag,'readonly']);
    }
    if(Object.getOwnPropertyNames(ValidityState.prototype).some(name=>name!=='constructor'&&!flags.includes(name)))failures.push('extra public helpers');
    try{new ValidityState();failures.push('constructible state');}catch(e){if(e.name!=='TypeError')failures.push(e.name);}
    return failures;
  })()`),
    [],
  );
});

test('button validity is live custom-error state independent of input-like properties and public overrides', async () => {
  assert.deepEqual(
    await evaluate(`(() => {
    const failures=[],button=document.createElement('button'),state=button.validity;
    button.required=true;button.pattern='x';button.min='10';button.max='1';button.minLength=20;button.maxLength=1;
    button.setAttribute('required','');button.setAttribute('pattern','x');button.value='';
    const otherFlags=['valueMissing','typeMismatch','patternMismatch','tooLong','tooShort','rangeUnderflow','rangeOverflow','stepMismatch','badInput'];
    for(const type of ['submit','reset','button','auto']){
      button.type=type;button.setCustomValidity('');if(!state.valid||state.customError||otherFlags.some(name=>state[name]))failures.push([type,'clear flags']);
      button.setCustomValidity('message');if(state.valid||!state.customError||otherFlags.some(name=>state[name]))failures.push([type,'custom flags']);
      if(button.validationMessage!==(['reset','button'].includes(type)?'':'message'))failures.push([type,'eligibility']);
    }
    button.type='submit';let invalid=0;button.addEventListener('invalid',()=>invalid++);
    for(const name of ['value','required','type','disabled','willValidate','validity','dispatchEvent'])Object.defineProperty(button,name,{get(){throw Error(name);}});
    button.getAttribute=button.hasAttribute=button.closest=()=>{throw Error('override');};
    if(button.checkValidity()!==false||invalid!==1||button.validationMessage!=='message'||state.valid)failures.push('canonical button');
    button.setCustomValidity('');if(!button.checkValidity()||button.validationMessage!==''||!state.valid||state.customError)failures.push('canonical clear');
    const input=document.createElement('input'),inputState=input.validity;
    Object.defineProperty(input,'value',{get(){throw Error('input value');}});Object.defineProperty(input,'required',{get(){throw Error('input required');}});
    input.setCustomValidity('private');if(!inputState.customError)failures.push('independent custom flag');input.setCustomValidity('');if(inputState.customError)failures.push('independent clear');
    return failures;
  })()`),
    [],
  );
});

test('custom validity survives reset/adoption but is not cloned, and assignments emit no DOM events or mutations', async () => {
  assert.deepEqual(
    await evaluate(`(() => {
    const failures=[];
    for(const tag of ['input','button']){
      const form=document.createElement('form'),control=document.createElement(tag);form.append(control);document.body.append(form);
      const state=control.validity,observer=new MutationObserver(()=>{});observer.observe(control,{attributes:true,childList:true,subtree:true});
      const events=[];for(const type of ['invalid','input','change'])control.addEventListener(type,()=>events.push(type));
      control.setCustomValidity('persistent');control.setCustomValidity('persistent');
      if(observer.takeRecords().length||events.length)failures.push([tag,'setter side effects']);observer.disconnect();
      form.reset();if(!state.customError||control.validationMessage!=='persistent')failures.push([tag,'reset']);
      const owner=document.implementation.createHTMLDocument(),clone=control.cloneNode(true),imported=owner.importNode(control,true);
      for(const copy of [clone,imported])if(copy.validity.customError||!copy.validity.valid||copy.validationMessage!==''||copy.validity===state)failures.push([tag,'clone/import']);
      owner.adoptNode(control);if(control.validity!==state||!state.customError||control.validationMessage!=='persistent')failures.push([tag,'adopt']);
      control.disabled=true;if(control.validationMessage!==''||!state.customError)failures.push([tag,'disabled persists']);
      control.disabled=false;if(control.validationMessage!=='persistent')failures.push([tag,'reenabled']);form.remove();
    }
    return failures;
  })()`),
    [],
  );
});

test('button checkValidity dispatches a trusted cancelable invalid event and returns the pre-event result', async () => {
  assert.deepEqual(
    await evaluate(`(() => {
    const failures=[],form=document.createElement('form'),button=document.createElement('button');form.append(button);document.body.append(form);
    button.setCustomValidity('first');const log=[];
    form.addEventListener('invalid',e=>log.push(['capture',e.target===button]),true);
    form.addEventListener('invalid',()=>log.push(['bubble']));
    button.addEventListener('invalid',e=>{log.push(['target',e.isTrusted,e.bubbles,e.cancelable,e.composed,e.target===button]);e.preventDefault();button.setCustomValidity('');log.push(['nested',button.checkValidity()]);},{once:true});
    if(button.checkValidity()!==false||!button.checkValidity())failures.push('result after clear');
    if(JSON.stringify(log)!==JSON.stringify([['capture',true],['target',true,false,true,false,true],['nested',true]]))failures.push(['events',log]);
    button.setCustomValidity('second');button.addEventListener('invalid',()=>{button.disabled=true;button.remove();},{once:true});
    if(button.checkValidity()!==false||!button.checkValidity()||!button.validity.customError)failures.push('disabled and detached during event');
    button.disabled=false;let detached=0;button.addEventListener('invalid',()=>detached++);if(button.checkValidity()!==false||detached!==1)failures.push('detached invalid');
    form.remove();return failures;
  })()`),
    [],
  );
});
