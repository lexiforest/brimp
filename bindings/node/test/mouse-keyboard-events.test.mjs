import test from 'node:test';
import assert from 'node:assert/strict';
import {createSession} from '../../../scripts/coverage/session.mjs';
async function evaluate(source) {
  const session=await createSession();
  try { return await (await session.newPage()).evaluate(source); }
  finally { await session.close(); }
}

test('MouseEvent exposes readonly coordinates, buttons and related targets', async () => {
  assert.deepEqual(await evaluate(`(() => {
    const related=document.createElement('b'),e=new MouseEvent('click',{screenX:2.9,screenY:-2.9,clientX:3.5,clientY:-3.5,
      movementX:4.9,movementY:-4.9,button:2,buttons:65537,relatedTarget:related});
    const names=['screenX','screenY','clientX','clientY','movementX','movementY','button','buttons'];
    const readonly=names.concat('relatedTarget').every(n=>{const d=Object.getOwnPropertyDescriptor(MouseEvent.prototype,n);return d?.get&&d.enumerable&&!d.set&&!Reflect.set(e,n,0);});
    return [names.map(n=>e[n]),e.x,e.y,e.which,e.relatedTarget===related,readonly,e.isTrusted,
      Object.getPrototypeOf(MouseEvent.prototype)===UIEvent.prototype,MouseEvent.prototype.constructor===MouseEvent,Object.prototype.toString.call(e)];
  })()`), [[2,-3,3,-4,4.9,-4.9,2,1],3,-4,3,true,true,false,true,true,'[object MouseEvent]']);
});

test('MouseEvent fromElement and toElement follow entering and leaving targets', async () => {
  assert.deepEqual(await evaluate(`(() => {
    const node=document.createElement('i'),other=document.createElement('b');
    return ['mouseover','mouseout','mouseenter','mouseleave','click'].map(type=>{
      const e=new MouseEvent(type,{relatedTarget:other});node.dispatchEvent(e);
      return [e.fromElement===other,e.fromElement===node,e.toElement===other,e.toElement===node];
    });
  })()`), [[true,false,false,true],[false,true,true,false],[true,false,false,true],[false,true,true,false],[true,false,false,true]]);
});

test('KeyboardEvent preserves readonly key payloads and immutable location constants', async () => {
  assert.deepEqual(await evaluate(`(() => {
    const e=new KeyboardEvent('keydown',{key:'A',code:'KeyA',location:1,repeat:true,isComposing:true,keyCode:65,charCode:97});
    const names=['key','code','location','repeat','isComposing','keyCode','charCode'];
    const readonly=names.every(n=>{const d=Object.getOwnPropertyDescriptor(KeyboardEvent.prototype,n);return d?.get&&d.enumerable&&!d.set&&!Reflect.set(e,n,0);});
    const constants=['STANDARD','LEFT','RIGHT','NUMPAD'].every((name,i)=>[KeyboardEvent,KeyboardEvent.prototype].every(o=>{
      const d=Object.getOwnPropertyDescriptor(o,'DOM_KEY_LOCATION_'+name);return d?.value===i&&d.enumerable&&!d.configurable&&!d.writable;
    }));
    return [names.map(n=>e[n]),e.which,readonly,constants,new KeyboardEvent('x',{key:null}).key,
      Object.getPrototypeOf(KeyboardEvent.prototype)===UIEvent.prototype,KeyboardEvent.prototype.constructor===KeyboardEvent,Object.prototype.toString.call(e)];
  })()`), [['A','KeyA',1,true,true,65,97],0,true,true,'null',true,true,'[object KeyboardEvent]']);
});

test('Mouse and keyboard modifier state uses readonly flags and case-sensitive names', async () => {
  assert.deepEqual(await evaluate(`(() => [MouseEvent,KeyboardEvent].map(C=>{
    const e=new C('x',{ctrlKey:true,shiftKey:true,altKey:false,metaKey:true,modifierCapsLock:true});
    const readonly=['ctrlKey','shiftKey','altKey','metaKey'].every(n=>!Reflect.set(e,n,false));
    return [e.ctrlKey,e.shiftKey,e.altKey,e.metaKey,readonly,
      ['Control','Shift','Alt','Meta','CapsLock','AltGraph','control','unknown'].map(n=>e.getModifierState(n))];
  }))()`), Array.from({length:2},()=>[true,true,false,true,true,[true,true,false,true,true,false,false,false]]));
});

test('Legacy mouse and keyboard initialization updates payloads outside dispatch', async () => {
  assert.deepEqual(await evaluate(`(() => {
    const m=document.createEvent('MouseEvent');m.initMouseEvent('click',true,true,window,2,12,-12,3,-3,true,false,true,false,2,null);
    const target=new EventTarget();target.addEventListener('click',e=>e.initMouseEvent('ignored'));target.dispatchEvent(m);
    const k=new KeyboardEvent('x',{key:'A',code:'KeyA',repeat:true,isComposing:true,keyCode:65,charCode:97,modifierCapsLock:true});
    k.initKeyboardEvent('keyup',true,true,window,'B',2,false,true,false,true,true);
    return [[m.type,m.button,m.buttons,m.which,m.screenX,m.clientX,m.ctrlKey,m.shiftKey,m.view===window],
      [k.type,k.key,k.code,k.keyCode,k.charCode,k.which,k.repeat,k.isComposing,k.location,k.altKey,k.metaKey,k.getModifierState('CapsLock')]];
  })()`), [['click',2,0,3,12,3,true,true,true],['keyup','','',0,0,0,false,false,2,true,true,false]]);
});

test('Mouse and keyboard events validate required arguments and nullable dictionaries', async () => {
  assert.deepEqual(await evaluate(`(() => {
    const errors=[];for(const C of [MouseEvent,KeyboardEvent])for(const operation of [()=>new C(),()=>new C(Symbol()),
      ()=>new C('x',1),()=>new C('x').getModifierState(),()=>new C('x').getModifierState(Symbol())]) {
      try{operation();errors.push('none');}catch(e){errors.push(e.name);}
    }
    for(const operation of [()=>new MouseEvent('x',{relatedTarget:{}}),()=>new MouseEvent('x',{clientX:Infinity}),
      ()=>new KeyboardEvent('x',{keyCode:1n}),()=>new MouseEvent('x').initMouseEvent(),()=>new KeyboardEvent('x').initKeyboardEvent()]) {
      try{operation();errors.push('none');}catch(e){errors.push(e.name);}
    }
    return [errors,new MouseEvent('x',null).button,new KeyboardEvent('x',null).key];
  })()`), [[...Array(11).fill('TypeError'),'none',...Array(3).fill('TypeError')],0,'']);
});
