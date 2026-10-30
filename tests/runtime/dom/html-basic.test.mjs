import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import { createSession } from '../../../scripts/coverage/session.mjs';
import {
  basicInterfaces,
  basicReflections,
  basicAttributeCases,
  measureBasicAttributes,
} from '../../../scripts/coverage/html-basic-cases.mjs';
import { measureControlIdentity } from '../../../scripts/coverage/control-identity-cases.mjs';
const reference = JSON.parse(
  fs.readFileSync(new URL('../fixtures/html-basic.safari.json', import.meta.url)),
);
async function evaluate(source) {
  const s = await createSession();
  try {
    return await (await s.newPage()).evaluate(source);
  } finally {
    await s.close();
  }
}

test('basic HTML attributes match Safari conversions, integer parsing and mutation records', async () => {
  assert.deepEqual(
    reference.cases.map(({ expected, ...c }) => c),
    basicAttributeCases,
  );
  const actual = JSON.parse(
    await evaluate(
      `JSON.stringify((${measureBasicAttributes})(${JSON.stringify(basicAttributeCases)}))`,
    ),
  );
  assert.deepEqual(
    reference.cases.flatMap((c, i) =>
      JSON.stringify(c.expected) === JSON.stringify(actual[i]) ? [] : [{ ...c, actual: actual[i] }],
    ),
    [],
  );
});

test('basic HTML interfaces have Safari identity, descriptors and construction policy', async () => {
  assert.deepEqual(reference.interfaces, basicInterfaces);
  assert.deepEqual(
    await evaluate(`(${measureControlIdentity})(${JSON.stringify(basicInterfaces)})`),
    reference.identity,
  );
  assert.deepEqual(
    await evaluate(`(()=>{
    const failures=[];
    for(const [tag,name] of ${JSON.stringify(basicInterfaces)}){
      const C=globalThis[name],element=document.createElement(tag);
      const clone=element.cloneNode();document.implementation.createHTMLDocument().adoptNode(clone);
      if(Object.getPrototypeOf(clone)!==C.prototype)failures.push(name+' clone/adopt');
      const root=document.createElement('template');root.innerHTML='<'+tag+'></'+tag+'>';
      // Fragment parsing treats html/head/frameset specially; span is the new native mapping.
      if(tag==='span'&&Object.getPrototypeOf(root.content.firstChild)!==C.prototype)failures.push('parsed span');
    }
    return failures;
  })()`),
    [],
  );
});

test('basic HTML reflections are branded, canonical and live through cloning and adoption', async () => {
  assert.deepEqual(
    await evaluate(`(()=>{
    const failures=[];
    for(const [tag,property,attribute] of ${JSON.stringify([...basicReflections, ['li', 'value', 'value']])}){
      const e=document.createElement(tag),d=Object.getOwnPropertyDescriptor(Object.getPrototypeOf(e),property),integer=property==='value';
      if(e[property]!== (integer?0:''))failures.push(property+' default');
      if(!d.enumerable||!d.configurable||d.get.name!=='get '+property||d.set.name!=='set '+property||d.get.length!==0||d.set.length!==1)failures.push(property+' descriptor');
      for(const receiver of [{},document.createElement('section'),Object.create(Object.getPrototypeOf(e))]){
        let converted=false;try{d.set.call(receiver,{[Symbol.toPrimitive](){converted=true;return 1}});failures.push('receiver accepted')}catch(err){if(err.name!=='TypeError'||converted)failures.push('brand before conversion')}
        try{d.get.call(receiver);failures.push('getter accepted')}catch(err){if(err.name!=='TypeError')failures.push(err.name)}
      }
      e.setAttributeNS('urn:other',attribute,'foreign');if(e[property]!== (integer?0:''))failures.push('namespace');
      const text='a'+String.fromCharCode(0xd800)+'b'+String.fromCharCode(0xdc00),value=integer?-31:text;
      e[property]=value;
      if(e.getAttributeNS(null,attribute)!==String(value)||e.getAttributeNS('urn:other',attribute)!=='foreign')failures.push('attribute');
      const clone=e.cloneNode();e[property]=integer?22:'changed';
      document.implementation.createHTMLDocument().adoptNode(clone);if(clone[property]!==value)failures.push('clone/adopt');
      e.getAttribute=e.setAttribute=e.hasAttribute=()=>{throw Error('public override')};
      e[property]=value;if(e[property]!==value)failures.push('canonical');
    }
    return failures;
  })()`),
    [],
  );
});

test('title text reads direct text children and canonically replaces children', async () => {
  assert.deepEqual(
    await evaluate(`(()=>{
    const failures=[],title=document.createElement('title'),span=document.createElement('span');span.textContent='nested';
    title.append('first',span,document.createComment('ignored'),'last');
    if(title.text!=='firstlast')failures.push('direct text');
    Object.defineProperty(title,'childNodes',{get(){throw Error('children override')}});
    Object.defineProperty(title,'textContent',{get(){throw Error('text getter override')},set(){throw Error('text setter override')}});
    Object.defineProperty(title.firstChild,'nodeType',{get(){throw Error('type override')}});
    Object.defineProperty(title.firstChild,'data',{get(){throw Error('data override')}});
    if(title.text!=='firstlast')failures.push('canonical read');
    title.text='plain';if(title.text!=='plain'||title.firstChild!==title.lastChild||title.firstChild.nodeValue!=='plain')failures.push('replace');
    title.text=null;if(title.text!=='null')failures.push('null');
    title.text='';if(title.text!==''||title.firstChild!==null)failures.push('empty');
    title.text=undefined;if(title.text!=='undefined')failures.push('undefined');
    const d=Object.getOwnPropertyDescriptor(HTMLTitleElement.prototype,'text');
    if(!d.enumerable||!d.configurable||d.get.name!=='get text'||d.set.name!=='set text'||d.get.length!==0||d.set.length!==1)failures.push('descriptor');
    for(const receiver of [{},document.createElement('div'),Object.create(HTMLTitleElement.prototype)]){
      let converted=false;try{d.set.call(receiver,{toString(){converted=true;return 'x'}});failures.push('receiver')}catch(e){if(e.name!=='TypeError'||converted)failures.push('brand before conversion')}
      try{d.get.call(receiver);failures.push('getter receiver')}catch(e){if(e.name!=='TypeError')failures.push(e.name)}
    }
    return failures;
  })()`),
    [],
  );
});

test('title text stays live and conversion precedes canonical replacement', async () => {
  assert.deepEqual(
    await evaluate(`(()=>{
    const failures=[],title=document.createElement('title');
    title.text='before';title.firstChild.data='changed';if(title.text!=='changed')failures.push('live text');
    const observer=new MutationObserver(()=>{});observer.observe(title,{childList:true,subtree:true});
    title.text={toString(){title.append('during');return 'after'}};
    const records=observer.takeRecords().map(r=>[r.type,r.addedNodes.length,r.removedNodes.length]);
    if(JSON.stringify(records)!==JSON.stringify([['childList',1,0],['childList',1,2]]))failures.push(records);
    title.text='after';const same=observer.takeRecords();
    if(same.length!==1||same[0].removedNodes.length!==1||same[0].addedNodes.length!==1)failures.push('same-value replacement');
    let error;try{title.text={toString(){title.append('retained');throw new RangeError('conversion')}}}catch(e){error=e.name}
    if(error!=='RangeError'||title.text!=='afterretained'||observer.takeRecords().length!==1)failures.push('failed conversion');
    observer.disconnect();
    const value=String.fromCharCode(0xd800)+'x'+String.fromCharCode(0xdc00);title.text=value;
    const clone=title.cloneNode(true);document.implementation.createHTMLDocument().adoptNode(clone);
    title.text='changed';if(clone.text!==value)failures.push('clone/adopt UTF-16');
    return failures;
  })()`),
    [],
  );
});
