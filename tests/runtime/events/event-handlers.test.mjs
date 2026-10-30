import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import { createSession } from '../../../scripts/coverage/session.mjs';

const reference = JSON.parse(
  fs.readFileSync(
    new URL('../fixtures/global-event-handlers.safari.json', import.meta.url),
    'utf8',
  ),
);
const setup = `
  const interfaces=${JSON.stringify(reference.interfaces)};
  const make=name=>name==='Document'?document.implementation.createHTMLDocument(''):
    name==='SVGElement'?document.createElementNS('http://www.w3.org/2000/svg','g'):document.createElement('div');
  const aliases={onwebkitanimationend:'webkitAnimationEnd',onwebkitanimationiteration:'webkitAnimationIteration',onwebkitanimationstart:'webkitAnimationStart',onwebkittransitionend:'webkitTransitionEnd'};
  const nonEnumerable=['onbeforecopy','onbeforecut','onbeforepaste','onwebkitfullscreenchange','onwebkitfullscreenerror','onwebkitmouseforcechanged','onwebkitmouseforcedown','onwebkitmouseforceup','onwebkitmouseforcewillbegin'];
  const lenient=['onmouseenter','onmouseleave','onreadystatechange'];
`;
async function evaluate(source) {
  const session = await createSession();
  try {
    return await (await session.newPage()).evaluate(source);
  } finally {
    await session.close();
  }
}

test('Safari handler surfaces have correct defaults, descriptors, identity and assignment conversion', async () => {
  assert.deepEqual(
    await evaluate(`(()=>{${setup}
    const failures=[];
    for(const [name,properties]of Object.entries(interfaces)) {
      const e=make(name),other=make(name),prototype=globalThis[name].prototype;
      for(const property of properties) {
        const check=(ok,step)=>{if(!ok)failures.push(name+'.'+property+': '+step);};
        const d=Object.getOwnPropertyDescriptor(prototype,property);
        check(e[property]===null && !Object.hasOwn(e,property),'default');
        check(d?.enumerable===!nonEnumerable.includes(property) && d.configurable && d.get.name==='get '+property && d.set.name==='set '+property && d.get.length===0 && d.set.length===1,'descriptor');
        for(const value of [()=>{}, {}, {handleEvent(){}},null,undefined,false,42,'code',Symbol()]) {
          e[property]=value;
          const expected=value!==null && (typeof value==='object'||typeof value==='function')?value:null;
          check(e[property]===expected,'assignment');check(other[property]===null,'isolated state');
          check(!Object.hasOwn(e,property),'no expando');
        }
        e[property]=null;
        for(const receiver of [null,{},new EventTarget()]) {
          for(const fn of [()=>d.get.call(receiver),()=>d.set.call(receiver,null)]) {
            try{check(fn()===undefined && lenient.includes(property),'accepted receiver');}catch(error){check(error.name==='TypeError' && !lenient.includes(property),'receiver exception');}
          }
        }
      }
    }
    return failures;
  })()`),
    [],
  );
});

test('each supported IDL handler receives its event, target receiver and cancellation result', async () => {
  assert.deepEqual(
    await evaluate(`(()=>{${setup}
    const failures=[];
    for(const [name,properties]of Object.entries(interfaces))for(const property of properties){
      const e=make(name),type=aliases[property]??property.slice(2),event=new Event(type,{cancelable:true});let calls=0;
      e[property]=function(received){calls++;if(this!==e||received!==event||received.currentTarget!==e)failures.push(name+'.'+property+': arguments');return false;};
      const passive=name==='Document' && ['onwheel','onmousewheel'].includes(property);
      if(e.dispatchEvent(event)!==passive || event.defaultPrevented===passive || calls!==1)failures.push(name+'.'+property+': dispatch');
      e[property]=null;e.dispatchEvent(new Event(type));if(calls!==1)failures.push(name+'.'+property+': removal');
      if(aliases[property]) {e[property]=()=>calls++;e.dispatchEvent(new Event(property.slice(2)));if(calls!==1)failures.push(name+'.'+property+': case sensitivity');}
    }
    return failures;
  })()`),
    [],
  );
});

test('handler replacement retains its listener slot and clearing re-registers at the end', async () => {
  assert.deepEqual(
    await evaluate(`(()=>{${setup}
    const results=[];
    for(const name of Object.keys(interfaces)) {
      const e=make(name),log=[],fire=()=>e.dispatchEvent(new Event('click'));
      e.addEventListener('click',()=>log.push('first'));
      const initial=()=>log.push('initial');e.onclick=initial;
      e.addEventListener('click',()=>log.push('last'));fire();
      e.onclick=()=>log.push('replacement');fire();
      e.onclick=null;e.onclick=()=>log.push('readded');fire();
      e.removeEventListener('click',e.onclick);fire();
      results.push(log);
    }
    return results;
  })()`),
    Array(3).fill([
      'first',
      'initial',
      'last',
      'first',
      'replacement',
      'last',
      'first',
      'last',
      'readded',
      'first',
      'last',
      'readded',
    ]),
  );
});

test('listener mutation during dispatch honors removals, additions and immediate stopping', async () => {
  assert.deepEqual(
    await evaluate(`(()=>{${setup}
    const results=[];
    for(const name of Object.keys(interfaces)) {
      const e=make(name),log=[];
      const later=()=>log.push('removed listener');
      e.addEventListener('click',()=>{log.push('first');e.onclick=null;e.onclick=()=>log.push('new handler');e.removeEventListener('click',later);},{once:true});
      e.onclick=()=>log.push('old handler');e.addEventListener('click',later);
      e.dispatchEvent(new Event('click'));log.push('second dispatch');e.dispatchEvent(new Event('click'));
      const s=make(name);s.onpointerdown=event=>{log.push('stop');event.stopImmediatePropagation();};
      s.addEventListener('pointerdown',()=>log.push('not reached'));s.dispatchEvent(new Event('pointerdown'));
      results.push(log);
    }
    return results;
  })()`),
    Array(3).fill(['first', 'second dispatch', 'new handler', 'stop']),
  );
});

test('handler state is independent of public property overrides and addEventListener overrides', async () => {
  assert.deepEqual(
    await evaluate(`(()=>{${setup}
    const failures=[];
    for(const name of Object.keys(interfaces)) {
      const e=make(name);let called=0;
      e.addEventListener=()=>{throw Error('overridden registration')};
      e.removeEventListener=()=>{throw Error('overridden removal')};
      e.onpointerdown=()=>called++;
      Object.defineProperty(e,'onpointerdown',{configurable:true,value:()=>{throw Error('expando called')}});
      e.dispatchEvent(new Event('pointerdown'));
      delete e.onpointerdown;e.onpointerdown=null;e.dispatchEvent(new Event('pointerdown'));
      if(called!==1)failures.push(name);
    }
    return failures;
  })()`),
    [],
  );
});

test('SVG nodes inherit handler properties through creation, parsing, cloning and import', async () => {
  assert.deepEqual(
    await evaluate(`(()=>{
    const host=document.createElement('div');host.innerHTML='<svg><g></g></svg>';
    const created=document.createElementNS('http://www.w3.org/2000/svg','g');
    const nodes=[created,host.firstChild,host.firstChild.firstChild,created.cloneNode(true),document.importNode(created,true)];
    const failures=[];
    for(const e of nodes){
      if(!(e instanceof SVGElement) || e instanceof HTMLElement || e.onpointerdown!==null)failures.push('prototype');
      let calls=0;e.onpointerdown=()=>calls++;e.dispatchEvent(new Event('pointerdown'));if(calls!==1)failures.push('delivery');
    }
    const xml=document.createElementNS('urn:test','node');let calls=0;
    xml.onclick=()=>calls++;xml.dispatchEvent(new Event('click'));
    if(xml instanceof SVGElement || calls!==0)failures.push('generic XML event expando');
    return failures;
  })()`),
    [],
  );
});
