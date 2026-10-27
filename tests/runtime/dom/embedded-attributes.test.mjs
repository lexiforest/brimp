import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {createSession} from '../../../scripts/coverage/session.mjs';
import {embeddedInterfaces,embeddedFields,embeddedAttributeCases,measureEmbeddedAttributes,sandboxTokens,measureSandboxSupport} from '../../../scripts/coverage/embedded-attribute-cases.mjs';
import {measureControlIdentity} from '../../../scripts/coverage/control-identity-cases.mjs';
const reference=JSON.parse(fs.readFileSync(new URL('../fixtures/embedded-attributes.safari.json',import.meta.url)));
async function evaluate(source){const s=await createSession();try{return await(await s.newPage()).evaluate(source);}finally{await s.close();}}

test('embedded/body attributes match Safari conversion, enum and mutation cases',async()=>{
  assert.deepEqual(reference.cases.map(({expected,...c})=>c),embeddedAttributeCases);
  const actual=JSON.parse(await evaluate(`JSON.stringify((${measureEmbeddedAttributes})(${JSON.stringify(embeddedAttributeCases)}))`));
  assert.deepEqual(reference.cases.flatMap((c,i)=>JSON.stringify(c.expected)===JSON.stringify(actual[i])?[]:[{...c,actual:actual[i]}]),[]);
});

test('embedded and body interfaces retain Safari prototype identity and construction policy',async()=>{
  assert.deepEqual(reference.interfaces,embeddedInterfaces);
  assert.deepEqual(await evaluate(`(${measureControlIdentity})(${JSON.stringify(embeddedInterfaces)})`),reference.identity);
});

test('embedded reflections are canonical, branded and live through cloning and adoption',async()=>{
  assert.deepEqual(await evaluate(`(()=>{
    const failures=[];
    for(const [tag,name,attribute,kind] of ${JSON.stringify(embeddedFields.filter(x=>x[3]!=='tokens'))}){
      const e=document.createElement(tag),d=Object.getOwnPropertyDescriptor(Object.getPrototypeOf(e),name),read=()=>e[name];
      const initial=read(),defaultValue=kind==='boolean'?false:name==='crossOrigin'?null:name==='loading'?'eager':name==='decoding'?'auto':'';
      if(initial!==defaultValue)failures.push(name+' default');
      e.setAttributeNS('urn:other',attribute,'foreign');if(read()!==initial)failures.push(name+' namespace');
      if(!d?.enumerable||!d.configurable||d.get.name!=='get '+name||d.set.name!=='set '+name||d.get.length!==0||d.set.length!==1)failures.push(name+' descriptor');
      for(const receiver of [{},document.createElement('section'),Object.create(Object.getPrototypeOf(e))]){
        let converted=false;try{d.set.call(receiver,{toString(){converted=true;return 'value'}});failures.push('receiver')}catch(err){if(err.name!=='TypeError'||converted)failures.push('brand before conversion')}
        try{d.get.call(receiver);failures.push('getter receiver')}catch(err){if(err.name!=='TypeError')failures.push(err.name)}
      }
      const value=kind==='boolean'?true:'a'+String.fromCharCode(0xd800)+'b'+String.fromCharCode(0xdc00);
      e[name]=value;const expected=read(),raw=kind==='boolean'?'':name==='srcset'?'a�b�':value;
      if(e.getAttributeNS(null,attribute)!==raw||e.getAttributeNS('urn:other',attribute)!=='foreign')failures.push(name+' raw');
      const clone=e.cloneNode();e[name]=kind==='boolean'?false:'changed';document.implementation.createHTMLDocument().adoptNode(clone);
      if(clone[name]!==expected||clone.getAttributeNS(null,attribute)!==raw)failures.push(name+' clone/adopt');
      if(['string','null-empty','usv-string'].includes(kind)){
        e.setAttributeNS(null,attribute,value);if(read()!==value)failures.push(name+' raw getter UTF-16');
      }
      e.getAttribute=e.setAttribute=e.hasAttribute=()=>{throw Error('public override')};
      e[name]=value;if(read()!==expected)failures.push(name+' canonical');
    }
    const frame=document.createElement('iframe');for(const name of ['allowPaymentRequest','csp','featurePolicy'])if(name in frame)failures.push('Safari absent '+name);
    return failures;
  })()`),[]);
});

test('iframe sandbox retains token identity and forwards value assignments like Safari',async()=>{
  assert.deepEqual(await evaluate(`(${measureSandboxSupport})(${JSON.stringify(sandboxTokens)})`),reference.sandboxSupport);
  assert.deepEqual(await evaluate(`(()=>{
    const failures=[],e=document.createElement('iframe'),list=e.sandbox,d=Object.getOwnPropertyDescriptor(HTMLIFrameElement.prototype,'sandbox');
    if(!(list instanceof DOMTokenList)||list!==e.sandbox||list.value!==''||list.length!==0)failures.push('default');
    if(!d.enumerable||!d.configurable||d.get.name!=='get sandbox'||d.set.name!=='set sandbox'||d.get.length!==0||d.set.length!==1)failures.push('descriptor');
    e.setAttributeNS('urn:other','sandbox','foreign');if(list.value!=='')failures.push('namespace');
    e.sandbox='allow-scripts allow-scripts allow-forms';
    if(list.value!=='allow-scripts allow-scripts allow-forms'||JSON.stringify([...list])!==JSON.stringify(['allow-scripts','allow-forms']))failures.push('tokens');
    list.remove('allow-forms');list.add('allow-popups');if(e.getAttributeNS(null,'sandbox')!=='allow-scripts allow-popups')failures.push('mutation');
    document.implementation.createHTMLDocument().adoptNode(e);if(e.sandbox!==list)failures.push('adopt identity');
    const clone=e.cloneNode();if(clone.sandbox===list||clone.sandbox.value!==list.value)failures.push('clone');
    e.getAttribute=e.setAttribute=()=>{throw Error('override')};list.add('allow-forms');if(!list.contains('allow-forms'))failures.push('canonical');
    let received,gets=0;const supplied={};Object.defineProperty(e,'sandbox',{get(){gets++;return {set value(value){received=value}}}});
    d.set.call(e,supplied);if(received!==supplied||gets!==1)failures.push('forward unconverted value');
    for(const receiver of [{},document.createElement('div'),Object.create(HTMLIFrameElement.prototype)])try{d.get.call(receiver);failures.push('getter brand')}catch(err){if(err.name!=='TypeError')failures.push(err.name)}
    return failures;
  })()`),[]);
});
