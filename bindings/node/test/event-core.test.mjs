import test from 'node:test';
import assert from 'node:assert/strict';
import {createSession} from '../../../scripts/coverage/session.mjs';

async function evaluate(source) {
  const session=await createSession();
  try { return await (await session.newPage()).evaluate(source); }
  finally { await session.close(); }
}

test('Event exposes readonly state, phase constants and a per-instance trust property', async () => {
  assert.deepEqual(await evaluate(`(() => {
    const start=performance.now(),event=new Event('sample',{bubbles:true,cancelable:true,composed:true});
    const trust=Object.getOwnPropertyDescriptor(event,'isTrusted');
    const values=[event.type,event.bubbles,event.cancelable,event.composed,event.target,event.srcElement,
      event.currentTarget,event.eventPhase,event.defaultPrevented,event.returnValue,event.cancelBubble,event.isTrusted,
      event.timeStamp>=start&&event.timeStamp<=performance.now(),event.composedPath().length];
    const readonly=['type','bubbles','cancelable','composed','target','srcElement','currentTarget','eventPhase','defaultPrevented','timeStamp'];
    const shape=readonly.every(name=>{const d=Object.getOwnPropertyDescriptor(Event.prototype,name);return d?.get&&d.set===undefined&&d.enumerable&&d.configurable&&!Reflect.set(event,name,'changed');});
    const constants=['NONE','CAPTURING_PHASE','AT_TARGET','BUBBLING_PHASE'].map((name,i)=>
      [Event,Event.prototype].every(o=>{const d=Object.getOwnPropertyDescriptor(o,name);return d?.value===i&&d.enumerable&&!d.writable&&!d.configurable;}));
    return [values,shape,constants,!!trust?.get,trust?.enumerable,trust?.configurable,
      Event.prototype.constructor===Event,Event.prototype[Symbol.toStringTag],Object.hasOwn(Event.prototype,'path'),
      EventTarget.prototype.constructor===EventTarget,Object.prototype.toString.call(new EventTarget())];
  })()`), [['sample',true,true,true,null,null,null,0,false,true,false,false,true,0],true,Array(4).fill(true),
    true,true,false,true,'Event',false,true,'[object EventTarget]']);
});

test('EventTarget handles duplicates, capture matching, once, objects and AbortSignal removal', async () => {
  assert.deepEqual(await evaluate(`(() => {
    const target=new EventTarget(),log=[],controller=new AbortController();
    function duplicate(){log.push(this===target?'duplicate':'wrong this');}
    const object={handleEvent(e){log.push(this===object&&e.currentTarget===target?'object':'wrong object');}};
    target.addEventListener('x',duplicate);target.addEventListener('x',duplicate,{once:true});
    target.addEventListener('x',()=>log.push('once'),{once:true});target.addEventListener('x',object,{signal:controller.signal});
    target.removeEventListener('x',duplicate,true);target.dispatchEvent(new Event('x'));
    controller.abort();target.dispatchEvent(new Event('x'));target.removeEventListener('x',duplicate);
    target.addEventListener('x',object,{signal:controller.signal});target.dispatchEvent(new Event('x'));
    return [log,['addEventListener','removeEventListener','dispatchEvent'].map(n=>Object.getOwnPropertyDescriptor(EventTarget.prototype,n).enumerable)];
  })()`), [['duplicate','once','object','duplicate'],[true,true,true]]);
});

test('Event dispatch follows capture, target and bubble phases and cancellation', async () => {
  assert.deepEqual(await evaluate(`(() => {
    const parent=document.createElement('div'),child=document.createElement('i');parent.append(child);const log=[];
    const record=name=>e=>log.push([name,e.eventPhase,e.target===child,e.currentTarget===parent,e.composedPath().length]);
    parent.addEventListener('x',record('capture'),true);child.addEventListener('x',record('target'));
    parent.addEventListener('x',e=>{record('bubble')(e);e.preventDefault();});
    const event=new Event('x',{bubbles:true,cancelable:true}),result=child.dispatchEvent(event);
    const passive=new Event('passive',{cancelable:true});child.addEventListener('passive',e=>e.preventDefault(),{passive:true});
    return [log,result,event.defaultPrevented,event.returnValue,event.target===child,event.srcElement===child,
      event.currentTarget,event.eventPhase,event.composedPath().length,child.dispatchEvent(passive),passive.defaultPrevented];
  })()`), [[['capture',1,true,true,2],['target',2,true,false,2],['bubble',3,true,true,2]],false,true,false,true,true,null,0,0,true,false]);
});

test('Event propagation flags reset after dispatch and cancellation aliases are one-way', async () => {
  assert.deepEqual(await evaluate(`(() => {
    const target=new EventTarget(),log=[],event=new Event('x',{cancelable:true});
    target.addEventListener('x',e=>{log.push('first');e.stopImmediatePropagation();},{once:true});
    target.addEventListener('x',()=>log.push('second'));
    target.dispatchEvent(event);const reset=event.cancelBubble;target.dispatchEvent(event);
    event.returnValue=false;event.returnValue=true;const canceled=event.defaultPrevented;
    const plain=new Event('plain');plain.preventDefault();
    const stopped=new Event('stopped');stopped.cancelBubble=true;stopped.cancelBubble=false;
    return [log,reset,canceled,event.returnValue,plain.defaultPrevented,stopped.cancelBubble];
  })()`), [['first','second'],false,true,false,false,true]);
});

test('Event initialization clears dispatch state and redispatching an active event throws', async () => {
  assert.deepEqual(await evaluate(`(() => {
    const target=new EventTarget(),event=new Event('x',{bubbles:true,cancelable:true}),errors=[];
    const legacy=document.createEvent('Event');
    try{target.dispatchEvent(legacy);}catch(error){errors.push(error.name);}
    legacy.initEvent('legacy');const initialized=target.dispatchEvent(legacy);
    target.addEventListener('x',e=>{try{target.dispatchEvent(e);}catch(error){errors.push(error.name);}e.initEvent('ignored');e.preventDefault();});
    target.dispatchEvent(event);const after=event.type;event.initEvent('new',false,true);
    return [errors,after,event.type,event.target,event.srcElement,event.bubbles,event.cancelable,event.defaultPrevented,event.cancelBubble,initialized];
  })()`), [['InvalidStateError','InvalidStateError'],'x','new',null,null,false,true,false,false,true]);
});

test('Event and EventTarget reject invalid arguments before dispatch', async () => {
  assert.deepEqual(await evaluate(`(() => {
    const target=new EventTarget(),errors=[];
    const operations=[()=>new Event(),()=>new Event(Symbol()),()=>new Event('x',1),()=>new Event('x').initEvent(Symbol()),
      ()=>target.addEventListener('x'),()=>target.removeEventListener('x'),()=>target.addEventListener(Symbol(),null),
      ()=>target.removeEventListener(Symbol(),null),()=>target.addEventListener('x',42),()=>target.addEventListener('x',()=>{},{signal:{}}),
      ()=>target.dispatchEvent({}),()=>target.dispatchEvent()];
    for(const operation of operations){try{operation();errors.push('none');}catch(e){errors.push(e.name);}}
    return [errors,new Event('null',null).type];
  })()`), [Array(12).fill('TypeError'),'null']);
});

test('Event composed paths and retargeting respect shadow boundaries', async () => {
  assert.deepEqual(await evaluate(`(() => {
    const host=document.createElement('div'),shadow=host.attachShadow({mode:'open'}),child=document.createElement('i');shadow.append(child);
    const log=[];shadow.addEventListener('x',e=>log.push(['shadow',e.target===child,e.composedPath().length]));
    host.addEventListener('x',e=>log.push(['host',e.target===host,e.composedPath().length]));
    child.dispatchEvent(new Event('x',{bubbles:true}));child.dispatchEvent(new Event('x',{bubbles:true,composed:true}));
    return log;
  })()`), [['shadow',true,2],['shadow',true,3],['host',true,3]]);
});

test('Event trust is readonly and a script redispatch clears trusted status', async () => {
  assert.deepEqual(await evaluate(`(() => {
    const controller=new AbortController(),log=[];let captured;
    controller.signal.addEventListener('abort',e=>{captured=e;log.push(e.isTrusted);});controller.abort();
    const after=captured.isTrusted,readonly=Reflect.set(captured,'isTrusted',false);
    controller.signal.dispatchEvent(captured);
    return [log,after,readonly,captured.isTrusted];
  })()`), [[true,false],true,false,false]);
});
