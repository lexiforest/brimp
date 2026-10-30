import test from 'node:test';
import assert from 'node:assert/strict';
import { createSession } from '../../../scripts/coverage/session.mjs';

async function evaluate(source) {
  const session = await createSession();
  try {
    return await (await session.newPage()).evaluate(source);
  } finally {
    await session.close();
  }
}

test('CustomEvent retains payload identity with readonly detail and ordinary interface inheritance', async () => {
  assert.deepEqual(
    await evaluate(`(() => {
    const payload={value:1},event=new CustomEvent('custom',{detail:payload,bubbles:true,cancelable:true});
    const target=new EventTarget();let received;target.addEventListener('custom',e=>{received=e.detail;e.detail.value++;});target.dispatchEvent(event);
    const descriptor=Object.getOwnPropertyDescriptor(CustomEvent.prototype,'detail');
    return [event.detail===payload,received===payload,payload.value,Reflect.set(event,'detail',null),
      new CustomEvent('x').detail,new CustomEvent('x',null).detail,new CustomEvent('x',{detail:0}).detail,
      descriptor?.enumerable,descriptor?.set===undefined,event instanceof Event,
      Object.getPrototypeOf(CustomEvent.prototype)===Event.prototype,CustomEvent.prototype.constructor===CustomEvent,
      Object.prototype.toString.call(event)];
  })()`),
    [true, true, 2, false, null, null, 0, true, true, true, true, true, '[object CustomEvent]'],
  );
});

test('CustomEvent legacy initialization updates payload and clears cancellation only outside dispatch', async () => {
  assert.deepEqual(
    await evaluate(`(() => {
    const target=new EventTarget(),event=document.createEvent('CustomEvent'),first={id:1},next={id:2};
    event.initCustomEvent('x',true,true,first);event.preventDefault();
    target.addEventListener('x',e=>e.initCustomEvent('ignored',false,false,next));target.dispatchEvent(event);
    const during=[event.type,event.detail===first,event.defaultPrevented];
    event.initCustomEvent('new',false,true,next);const after=[event.type,event.detail===next,event.defaultPrevented,event.target];
    event.initCustomEvent('empty');
    return [during,after,event.detail,Object.getOwnPropertyDescriptor(CustomEvent.prototype,'initCustomEvent').enumerable];
  })()`),
    [['x', true, true], ['new', true, false, null], null, true],
  );
});

test('UIEvent exposes readonly view, integer detail and legacy defaults without sourceCapabilities', async () => {
  assert.deepEqual(
    await evaluate(`(() => {
    const event=new UIEvent('ui',{view:window,detail:3.9,bubbles:true});
    const initial=[event.view===window,event.detail,event.which,event.bubbles,
      ['view','detail','which'].every(name=>{const d=Object.getOwnPropertyDescriptor(UIEvent.prototype,name);return d?.get&&d.set===undefined&&d.enumerable&&!Reflect.set(event,name,0);})];
    const defaults=new UIEvent('default',null),legacy=document.createEvent('UIEvent');legacy.initUIEvent('legacy',true,true,window,-2.8);
    const target=new EventTarget();target.addEventListener('legacy',e=>e.initUIEvent('ignored',false,false,null,9));target.dispatchEvent(legacy);
    return [initial,defaults.view,defaults.detail,defaults.which,'sourceCapabilities' in event,
      legacy.type,legacy.view===window,legacy.detail,legacy.bubbles,legacy.cancelable,
      Object.getPrototypeOf(UIEvent.prototype)===Event.prototype,UIEvent.prototype.constructor===UIEvent,
      Object.prototype.toString.call(event),Object.getOwnPropertyDescriptor(UIEvent.prototype,'initUIEvent').enumerable];
  })()`),
    [
      [true, 3, 0, true, true],
      null,
      0,
      0,
      false,
      'legacy',
      true,
      -2,
      true,
      true,
      true,
      true,
      '[object UIEvent]',
      true,
    ],
  );
});

test('CustomEvent and UIEvent validate required types, dictionaries and window references', async () => {
  assert.deepEqual(
    await evaluate(`(() => {
    const errors=[],custom=new CustomEvent('x'),ui=new UIEvent('x');
    for(const operation of [()=>new CustomEvent(),()=>new UIEvent(),()=>new CustomEvent(Symbol()),()=>new UIEvent(Symbol()),
      ()=>new CustomEvent('x',1),()=>new UIEvent('x',1),()=>new UIEvent('x',{view:{}}),()=>new UIEvent('x',{detail:Symbol()}),
      ()=>custom.initCustomEvent(),()=>ui.initUIEvent(),()=>custom.initCustomEvent(Symbol()),()=>ui.initUIEvent('x',false,false,{}),
      ()=>ui.initUIEvent('x',false,false,null,1n)]) {
      try{operation();errors.push('none');}catch(e){errors.push(e.name);}
    }
    return [errors,new UIEvent('x',{detail:NaN}).detail,new UIEvent('x',{detail:4294967295}).detail];
  })()`),
    [Array(13).fill('TypeError'), 0, -1],
  );
});
