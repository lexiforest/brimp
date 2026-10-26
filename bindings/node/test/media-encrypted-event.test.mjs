import test from 'node:test';
import assert from 'node:assert/strict';
import {createSession} from '../../../scripts/coverage/session.mjs';
import {servicesPlugin} from '../index.js';

async function evaluate(source, services=true) {
  const session=await createSession({plugins:services?[servicesPlugin()]:[]});
  try { return await (await session.newPage()).evaluate(source); }
  finally { await session.close(); }
}

test('MediaEncryptedEvent retains ArrayBuffer identity behind readonly payload accessors', async () => {
  assert.deepEqual(await evaluate(`(() => {
    const buffer=new Uint8Array([1,2]).buffer,e=new MediaEncryptedEvent('encrypted',{initData:buffer,initDataType:'cenc'});
    new Uint8Array(buffer)[0]=9;
    const target=new EventTarget();let received;target.addEventListener('encrypted',event=>received=event.initData===buffer);target.dispatchEvent(e);
    const defaults=new MediaEncryptedEvent('encrypted',null);
    return [e.initData===buffer,Array.from(new Uint8Array(e.initData)),e.initDataType,received,
      Reflect.set(e,'initData',null),Reflect.set(e,'initDataType','other'),defaults.initData,defaults.initDataType,
      ['initData','initDataType'].every(n=>{const d=Object.getOwnPropertyDescriptor(MediaEncryptedEvent.prototype,n);return d?.get&&d.enumerable&&!d.set;}),
      Object.getPrototypeOf(MediaEncryptedEvent.prototype)===Event.prototype,MediaEncryptedEvent.prototype.constructor===MediaEncryptedEvent,Object.prototype.toString.call(e)];
  })()`), [true,[9,2],'cenc',true,false,false,null,'',true,true,true,'[object MediaEncryptedEvent]']);
});

test('MediaEncryptedEvent validates dictionaries and nullable ArrayBuffer payloads', async () => {
  assert.deepEqual(await evaluate(`(() => {
    const errors=[];for(const op of [()=>new MediaEncryptedEvent(),()=>new MediaEncryptedEvent(Symbol()),
      ()=>new MediaEncryptedEvent('x',1),()=>new MediaEncryptedEvent('x',{initData:{}}),
      ()=>new MediaEncryptedEvent('x',{initData:new Uint8Array([1])}),()=>new MediaEncryptedEvent('x',{initDataType:Symbol()})]) {
      try{op();errors.push('none');}catch(e){errors.push(e.name);}
    }
    return [errors,new MediaEncryptedEvent('x',{initData:null,initDataType:null}).initDataType];
  })()`), [Array(6).fill('TypeError'),'null']);
});

test('MediaEncryptedEvent belongs to the optional services plugin', {skip:process.env.COVERAGE_BROWSER==='safari'}, async () => {
  assert.equal(await evaluate('typeof MediaEncryptedEvent',false),'undefined');
  assert.equal(await evaluate('typeof MediaEncryptedEvent'),'function');
});
