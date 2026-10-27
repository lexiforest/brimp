import test from 'node:test';
import assert from 'node:assert/strict';
import {createSession} from '../../../scripts/coverage/session.mjs';

async function evaluate(source) {
  const session=await createSession();
  try { return await (await session.newPage()).evaluate(source); }
  finally { await session.close(); }
}

test('Safari legacy event availability excludes MutationEvent and directly constructed TextEvent', async () => {
  assert.deepEqual(await evaluate(`(() => {
    const errors=[];for(const op of [()=>new TextEvent(),()=>document.createEvent('MutationEvent'),
      ()=>document.createEvent('MutationEvents'),()=>document.createEvent('TextEvents')]) {
      try{op();errors.push('none');}catch(e){errors.push(e.name);}
    }
    const e=document.createEvent('TextEvent');
    return [typeof MutationEvent,errors,e.type,e.data,e.detail,e.view,e instanceof UIEvent,
      Object.getPrototypeOf(TextEvent.prototype)===UIEvent.prototype,TextEvent.prototype.constructor===TextEvent,Object.prototype.toString.call(e)];
  })()`), ['undefined',['TypeError','NotSupportedError','NotSupportedError','NotSupportedError'],'','',0,null,true,true,true,'[object TextEvent]']);
});

test('TextEvent initializes readonly text and preserves payload during active dispatch', async () => {
  assert.deepEqual(await evaluate(`(() => {
    const e=document.createEvent('TextEvent'),target=new EventTarget(),errors=[];
    try{target.dispatchEvent(e);}catch(error){errors.push(error.name);}
    e.initTextEvent('textInput',true,true,window,'hello');
    target.addEventListener('textInput',event=>{event.initTextEvent('ignored',false,false,null,'changed');event.preventDefault();});
    const accepted=target.dispatchEvent(e),after=[e.type,e.data,e.view===window,e.bubbles,e.cancelable,e.defaultPrevented];
    const readonly=Reflect.set(e,'data','overwritten');e.initTextEvent('new',false,false,null,null);
    const reset=[e.type,e.data,e.view,e.defaultPrevented,e.target];
    e.initTextEvent('empty');
    for(const op of [()=>e.initTextEvent(),()=>e.initTextEvent(Symbol()),()=>e.initTextEvent('x',false,false,{}),
      ()=>e.initTextEvent('x',false,false,null,Symbol())]){try{op();errors.push('none');}catch(error){errors.push(error.name);}}
    const descriptor=Object.getOwnPropertyDescriptor(TextEvent.prototype,'data');
    return [errors,accepted,after,readonly,reset,e.data,descriptor.enumerable,descriptor.set===undefined,
      Object.getOwnPropertyDescriptor(TextEvent.prototype,'initTextEvent').enumerable];
  })()`), [['InvalidStateError',...Array(4).fill('TypeError')],false,['textInput','hello',true,true,true,true],
    false,['new','null',null,false,null],'undefined',true,true,true]);
});
