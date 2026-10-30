import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import { isDeepStrictEqual } from 'node:util';
import { createSession } from '../../../scripts/coverage/session.mjs';
import {
  specialHandlerCases,
  measureSpecialHandlers,
} from '../../../scripts/coverage/special-handler-cases.mjs';
import { measureBodyHandlers } from '../../../scripts/coverage/body-handler-cases.mjs';
async function evaluate(source) {
  const s = await createSession();
  try {
    return await (await s.newPage()).evaluate(source);
  } finally {
    await s.close();
  }
}
const reference = JSON.parse(
  fs.readFileSync(new URL('../fixtures/special-handlers.safari.json', import.meta.url)),
);

test('error and beforeunload callbacks match Safari arguments, return conversion and cancellation', async () => {
  assert.deepEqual(
    reference.cases.map(({ expected, ...c }) => c),
    specialHandlerCases,
  );
  const actual = await evaluate(
    `(${measureSpecialHandlers})(${JSON.stringify(specialHandlerCases)})`,
  );
  assert.deepEqual(
    reference.cases.flatMap((c, i) =>
      isDeepStrictEqual(c.expected, actual[i]) ? [] : [{ ...c, actual: actual[i] }],
    ),
    [],
  );
});

test('special body/frameset handlers use Safari descriptors and shared window registration', async () => {
  assert.deepEqual(
    await evaluate(`(${measureBodyHandlers})(['error','beforeunload'])`),
    reference.forwarding,
  );
});

test('ErrorEvent payloads are readonly, branded, canonical and converted in Web IDL order', async () => {
  assert.deepEqual(
    await evaluate(`(()=>{
    const failures=[],order=[],value={};
    const options={};for(const name of ['bubbles','cancelable','colno','composed','error','filename','lineno','message'])Object.defineProperty(options,name,{get(){order.push(name);return ({bubbles:true,cancelable:true,colno:-1,composed:true,error:value,filename:'file',lineno:4294967297,message:'text'})[name]}});
    const event=new ErrorEvent({toString(){order.push('type');return 'error'}},options);
    if(order.join(',')!=='type,bubbles,cancelable,composed,colno,error,filename,lineno,message')failures.push(order);
    if(!event.bubbles||!event.cancelable||!event.composed||event.error!==value||event.colno!==4294967295||event.lineno!==1)failures.push('dictionary');
    for(const key of ['message','filename','lineno','colno','error']){
      const d=Object.getOwnPropertyDescriptor(ErrorEvent.prototype,key),before=event[key];
      if(!d.enumerable||!d.configurable||d.set!==undefined||d.get.name!=='get '+key||d.get.length!==0||Object.hasOwn(event,key))failures.push('descriptor '+key);
      Reflect.set(event,key,'changed');if(event[key]!==before)failures.push('readonly '+key);
      for(const receiver of [{},new Event('error'),Object.create(ErrorEvent.prototype)])try{d.get.call(receiver);failures.push('brand')}catch(e){if(e.name!=='TypeError')failures.push(e.name)}
      Object.defineProperty(event,key,{get(){throw Error('public payload override')}});
    }
    const prior=window.onerror;let captured;
    try{window.onerror=function(...args){captured=args;return true};window.dispatchEvent(event);}finally{window.onerror=prior}
    if(!captured||captured[0]!=='text'||captured[1]!=='file'||captured[2]!==1||captured[3]!==4294967295||captured[4]!==value||!event.defaultPrevented)failures.push('canonical payload');
    const source=String.fromCharCode(0xd800)+'x'+String.fromCharCode(0xdc00),utf=new ErrorEvent('error',{message:source,filename:source});
    if(utf.message!==source||utf.filename!=='�x�')failures.push('string conversion');
    const defaults=new ErrorEvent('error',null);
    if(defaults.message!==''||defaults.filename!==''||defaults.lineno!==0||defaults.colno!==0||defaults.error!==undefined)failures.push('defaults');
    for(const create of [()=>new ErrorEvent(),()=>new ErrorEvent(Symbol()),()=>new ErrorEvent('error',1),()=>new ErrorEvent('error',{lineno:1n}),()=>new ErrorEvent('error',{message:Symbol()})])try{create();failures.push('conversion accepted')}catch(e){if(e.name!=='TypeError')failures.push(e.name)}
    return failures;
  })()`),
    [],
  );
});

test('BeforeUnloadEvent returnValue persists across initialization and uses canonical state in handlers', async () => {
  assert.deepEqual(
    await evaluate(`(()=>{
    const failures=[],event=document.createEvent('BeforeUnloadEvent'),d=Object.getOwnPropertyDescriptor(BeforeUnloadEvent.prototype,'returnValue');
    if(!(event instanceof Event)||!(event instanceof BeforeUnloadEvent)||event.type!==''||event.returnValue!==''||event.cancelable)failures.push('creation');
    try{window.dispatchEvent(event);failures.push('uninitialized dispatch')}catch(e){if(e.name!=='InvalidStateError')failures.push(e.name)}
    if(!d.enumerable||!d.configurable||d.get.name!=='get returnValue'||d.set.name!=='set returnValue'||d.get.length!==0||d.set.length!==1)failures.push('descriptor');
    try{new BeforeUnloadEvent();failures.push('constructor')}catch(e){if(e.name!=='TypeError')failures.push(e.name)}
    for(const receiver of [{},new Event('beforeunload'),Object.create(BeforeUnloadEvent.prototype)]){
      let converted=false;try{d.set.call(receiver,{toString(){converted=true;return 'x'}});failures.push('brand')}catch(e){if(e.name!=='TypeError'||converted)failures.push('brand conversion')}
    }
    for(const value of ['',null,undefined,false,'text']){
      event.returnValue=value;if(event.returnValue!==String(value)||event.defaultPrevented)failures.push('setter');
      event.initEvent('beforeunload',false,true);if(event.returnValue!==String(value))failures.push('init retains value');
    }
    const prior=window.onbeforeunload;
    try{
      event.returnValue='';Object.defineProperty(event,'returnValue',{get(){throw Error('public getter')},set(){throw Error('public setter')}});
      window.onbeforeunload=()=>false;window.dispatchEvent(event);
      if(d.get.call(event)!=='false'||!event.defaultPrevented)failures.push('canonical cancellation');
      event.initEvent('beforeunload',false,true);d.set.call(event,'preset');window.onbeforeunload=()=> 'new';window.dispatchEvent(event);
      if(d.get.call(event)!=='preset'||!event.defaultPrevented)failures.push('preserve existing value');
    }finally{window.onbeforeunload=prior}
    return failures;
  })()`),
    [],
  );
});

test('special callbacks preserve Safari strict this binding and bypass public cancellation methods', async () => {
  assert.deepEqual(
    await evaluate(`(()=>{
    const failures=[],frame=document.createElement('iframe');document.body.append(frame);
    const priorError=window.onerror,priorUnload=window.onbeforeunload;
    try{
      for(const target of [window,frame.contentWindow]){
        let seen='unset';target.onerror=function(){'use strict';seen=this;return true};
        const error=new ErrorEvent('error',{cancelable:true});error.preventDefault=()=>{throw Error('override')};
        target.dispatchEvent(error);if(seen!==undefined||!error.defaultPrevented)failures.push('error callback');
        target.onbeforeunload=function(){'use strict';seen=this;return true};
        const unload=document.createEvent('BeforeUnloadEvent');unload.initEvent('beforeunload',false,true);unload.preventDefault=()=>{throw Error('override')};
        target.dispatchEvent(unload);if(seen!==target||!unload.defaultPrevented||unload.returnValue!=='true')failures.push('beforeunload callback');
        target.onerror=target.onbeforeunload=null;
      }
      const div=document.createElement('div');document.body.append(div);let args;
      window.onerror=function(){args=[...arguments];return true};
      const bubbling=new ErrorEvent('error',{message:'bubble',bubbles:true,cancelable:true});div.dispatchEvent(bubbling);div.remove();
      if(args?.length!==5||args[0]!=='bubble'||!bubbling.defaultPrevented)failures.push('bubbling error');
    }finally{frame.remove();window.onerror=priorError;window.onbeforeunload=priorUnload;}
    return failures;
  })()`),
    [],
  );
});
