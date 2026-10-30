import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import { createSession } from '../../../scripts/coverage/session.mjs';
import {
  bodyWindowEventTypes,
  measureBodyHandlers,
} from '../../../scripts/coverage/body-handler-cases.mjs';
const forwardingTypes = [...bodyWindowEventTypes, 'error', 'beforeunload'];
async function evaluate(source) {
  const s = await createSession();
  try {
    return await (await s.newPage()).evaluate(source);
  } finally {
    await s.close();
  }
}

test('body and frameset handlers match Safari forwarding, conversion, descriptors and dispatch', async () => {
  const ref = JSON.parse(
    fs.readFileSync(new URL('../fixtures/body-handlers.safari.json', import.meta.url)),
  );
  assert.deepEqual(ref.types, bodyWindowEventTypes);
  assert.deepEqual(
    await evaluate(`(${measureBodyHandlers})(${JSON.stringify(bodyWindowEventTypes)})`),
    ref.cases,
  );
});

test('forwarded handlers follow owner-document windows across detachment, adoption and cloning', async () => {
  assert.deepEqual(
    await evaluate(`(()=>{
    const failures=[];
    for(const tag of ['body','frameset'])for(const type of ${JSON.stringify(forwardingTypes)}){
      const property='on'+type,prior=window[property],e=document.createElement(tag),inert=document.implementation.createHTMLDocument();
      const one=()=>{},two=()=>{};
      try{
        e[property]=one;
        const clone=e.cloneNode();if(clone[property]!==one)failures.push('active clone');
        document.body.append(e);e.remove();if(e[property]!==one)failures.push('detached');
        inert.adoptNode(e);if(e[property]!==null||window[property]!==one)failures.push('adopt inert');
        e[property]=two;if(e[property]!==null||window[property]!==one)failures.push('inert setter');
        document.adoptNode(e);if(e[property]!==one)failures.push('return active');
        window[property]=two;if(e[property]!==two||clone[property]!==two)failures.push('live window');
        // Ordinary element handlers remain local even for the same event type.
        if(type==='load'||type==='focus'||type==='blur'||type==='resize'||type==='scroll'||type==='error'){
          const div=document.createElement('div');div[property]=one;if(e[property]!==two||div[property]!==one)failures.push('ordinary local');
          const d=Object.getOwnPropertyDescriptor(HTMLElement.prototype,property);
          // Safari's HTMLElement descriptor stores local state, even on a body receiver.
          d.set.call(e,one);if(window[property]!==two||e[property]!==two||d.get.call(e)!==one)failures.push('explicit base descriptor');
        }
      }finally{window[property]=prior;}
    }
    return failures;
  })()`),
    [],
  );
});

test('forwarded handlers retain window listener order and obey cancellation and propagation', async () => {
  assert.deepEqual(
    await evaluate(`(()=>{
    const failures=[];
    for(const tag of ['body','frameset']){
      const e=document.createElement(tag),log=[],first=()=>log.push('first'),last=()=>log.push('last'),prior=window.onafterprint;
      window.onafterprint=null;
      window.addEventListener('afterprint',first);e.onafterprint=()=>log.push('old');window.addEventListener('afterprint',last);
      window.onafterprint=()=>log.push('new');window.dispatchEvent(new Event('afterprint'));
      e.onafterprint=null;e.onafterprint=event=>{log.push('again');event.stopImmediatePropagation()};window.dispatchEvent(new Event('afterprint'));
      if(JSON.stringify(log)!==JSON.stringify(['first','new','last','first','last','again']))failures.push(log);
      window.removeEventListener('afterprint',first);window.removeEventListener('afterprint',last);window.onafterprint=prior;
    }
    return failures;
  })()`),
    [],
  );
});

test('forwarded handlers use the owning iframe window and change targets on adoption', async () => {
  assert.deepEqual(
    await evaluate(`(()=>{
    const failures=[],frame=document.createElement('iframe');document.body.append(frame);
    const child=frame.contentWindow;
    try{
      for(const tag of ['body','frameset'])for(const type of ${JSON.stringify(forwardingTypes)}){
        const property='on'+type,prior=window[property],e=document.createElement(tag),calls=[];
        const outer=()=>calls.push('outer'),inner=function(event){calls.push(this===child&&event.currentTarget===child?'inner':'wrong receiver')};
        try{
          e[property]=outer;child[property]=inner;child.document.adoptNode(e);
          if(e[property]!==inner||window[property]!==outer)failures.push('adopt child');
          const clone=e.cloneNode();if(clone[property]!==inner)failures.push('child clone');
          child.dispatchEvent(new Event(type));window.dispatchEvent(new Event(type));
          if(JSON.stringify(calls)!==JSON.stringify(['inner','outer']))failures.push(calls);
          e[property]=null;if(child[property]!==null||window[property]!==outer)failures.push('child setter');
          document.adoptNode(e);if(e[property]!==outer)failures.push('adopt outer');
        }finally{window[property]=prior;child[property]=null;}
      }
    }finally{frame.remove();}
    return failures;
  })()`),
    [],
  );
});

test('forwarding uses canonical targets and checks body/frameset receiver brands', async () => {
  assert.deepEqual(
    await evaluate(`(()=>{
    const failures=[];
    for(const tag of ['body','frameset'])for(const type of ${JSON.stringify(forwardingTypes)}){
      const property='on'+type,e=document.createElement(tag),d=Object.getOwnPropertyDescriptor(Object.getPrototypeOf(e),property);
      for(const receiver of [null,{},document.createElement('div'),Object.create(Object.getPrototypeOf(e))]){
        for(const invoke of [()=>d.get.call(receiver),()=>d.set.call(receiver,()=>{})])try{invoke();failures.push('accepted receiver')}catch(err){if(err.name!=='TypeError')failures.push(err.name)}
      }
      const prior=window[property],own=Object.getOwnPropertyDescriptor(window,property),fn=()=>{};
      try{
        Object.defineProperty(e,'ownerDocument',{get(){throw Error('owner override')}});
        Object.defineProperty(document,'defaultView',{configurable:true,get(){throw Error('window override')}});
        Object.defineProperty(window,property,{configurable:true,get(){throw Error('public window getter')},set(){throw Error('public window setter')}});
        e.addEventListener=e.removeEventListener=()=>{throw Error('listener override')};
        e[property]=fn;if(e[property]!==fn)failures.push('canonical target');e[property]=null;
      }finally{
        delete document.defaultView;
        if(own)Object.defineProperty(window,property,own);else delete window[property];
        window[property]=prior;
      }
    }
    return failures;
  })()`),
    [],
  );
});
