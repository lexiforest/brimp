import test from 'node:test';
import assert from 'node:assert/strict';
import {createSession} from '../../../scripts/coverage/session.mjs';

async function evaluate(source) {
  const session=await createSession();
  try { return await (await session.newPage()).evaluate(source); }
  finally { await session.close(); }
}

test('PointerEvent exposes readonly default and supplied device payloads', async () => {
  assert.deepEqual(await evaluate(`(() => {
    const names=['pointerId','width','height','pressure','tangentialPressure','tiltX','tiltY','twist','pointerType','isPrimary','altitudeAngle','azimuthAngle'];
    const defaults=new PointerEvent('pointermove',null);
    const e=new PointerEvent('pointermove',{pointerId:7,width:2.5,height:3.5,pressure:0.25,tangentialPressure:-0.5,
      tiltX:45,tiltY:0,twist:12,pointerType:'pen',isPrimary:true,clientX:10,ctrlKey:true});
    const readonly=names.every(n=>{const d=Object.getOwnPropertyDescriptor(PointerEvent.prototype,n);return d?.get&&d.enumerable&&!d.set&&!Reflect.set(e,n,0);});
    const trust=Object.getOwnPropertyDescriptor(e,'isTrusted');
    const payload=event=>names.map(n=>n.endsWith('Angle')?Number(event[n].toFixed(6)):event[n]);
    return [payload(defaults),payload(e),readonly,e.clientX,e.getModifierState('Control'),
      Object.getPrototypeOf(PointerEvent.prototype)===MouseEvent.prototype,PointerEvent.prototype.constructor===PointerEvent,
      Object.prototype.toString.call(e),e.isTrusted,trust.enumerable,trust.configurable];
  })()`), [[0,1,1,0,0,0,0,0,'',false,1.570796,0],[7,2.5,3.5,0.25,-0.5,45,0,12,'pen',true,0.785398,0],
    true,10,true,true,true,'[object PointerEvent]',false,true,false]);
});

test('PointerEvent orientation follows Safari ordinary tilt and angle initialization', async () => {
  assert.deepEqual(await evaluate(`(() => [{tiltX:0,tiltY:45},
    {altitudeAngle:Math.PI/4,azimuthAngle:Math.PI/2},
    {tiltX:10,tiltY:20,altitudeAngle:0.5,azimuthAngle:1.2}].map(options=>{
      const e=new PointerEvent('pointermove',options);
      return [e.tiltX,e.tiltY,Number(e.altitudeAngle.toFixed(6)),Number(e.azimuthAngle.toFixed(6))];
    }))()`), [[0,45,0.785398,1.570796],[0,45,1.570796,0],[34,60,1.186475,1.119662]]);
});

test('PointerEvent validates required types and numeric payloads and dispatches through MouseEvent', async () => {
  assert.deepEqual(await evaluate(`(() => {
    const errors=[];for(const operation of [()=>new PointerEvent(),()=>new PointerEvent(Symbol()),()=>new PointerEvent('x',1),
      ()=>new PointerEvent('x',{pointerId:1n}),()=>new PointerEvent('x',{pointerType:Symbol()}),()=>new PointerEvent('x',{pressure:Infinity})]) {
      try{operation();errors.push('none');}catch(e){errors.push(e.name);}
    }
    const node=document.createElement('div'),e=new PointerEvent('pointerdown',{pointerId:3,pointerType:'touch',bubbles:true,cancelable:true});
    let received;node.addEventListener('pointerdown',event=>{received=[event===e,event.pointerId,event.target===node,event instanceof MouseEvent];event.preventDefault();});
    return [errors,node.dispatchEvent(e),received,new PointerEvent('x',{pointerType:null}).pointerType];
  })()`), [Array(6).fill('TypeError'),false,[true,3,true,true],'null']);
});
