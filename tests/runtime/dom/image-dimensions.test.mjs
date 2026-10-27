import test from 'node:test';import assert from 'node:assert/strict';import fs from 'node:fs';
import {createSession} from '../../../scripts/coverage/session.mjs';
import {imageDimensionCases,measureImageDimensions,imageFactoryCases,measureImageFactory} from '../../../scripts/coverage/image-dimension-cases.mjs';
const ref=JSON.parse(fs.readFileSync(new URL('../fixtures/image-dimensions.safari.json',import.meta.url)));
async function evaluate(source){const s=await createSession();try{return await(await s.newPage()).evaluate(source);}finally{await s.close();}}
for(const[name,cases,measure]of [['dimensions',imageDimensionCases,measureImageDimensions],['factory',imageFactoryCases,measureImageFactory]])
test(`image ${name} follow Safari numeric conversion and attributes`,async()=>{
  assert.deepEqual(ref[name].map(({expected,...c})=>name==='factory'?c.sources:c),cases);
  const actual=await evaluate(`(${measure})(${JSON.stringify(cases)})`);
  assert.deepEqual(ref[name].flatMap((c,i)=>JSON.stringify(c.expected)===JSON.stringify(actual[i])?[]:[{...c,actual:actual[i]}]),[]);
});

test('image dimensions use canonical attributes, branded descriptors and reentrant number conversion',async()=>{
  assert.deepEqual(await evaluate(`(()=>{
    const failures=[];
    for(const name of ['width','height','hspace','vspace']){
      const e=document.createElement('img'),d=Object.getOwnPropertyDescriptor(HTMLImageElement.prototype,name);
      if(e[name]!==0||!d.enumerable||!d.configurable||d.get.name!=='get '+name||d.set.name!=='set '+name||d.get.length!==0||d.set.length!==1)failures.push('descriptor/default');
      for(const receiver of [{},document.createElement('div'),Object.create(HTMLImageElement.prototype)]){
        let converted=false;try{d.set.call(receiver,{valueOf(){converted=true;return 12}});failures.push('receiver')}catch(err){if(err.name!=='TypeError'||converted)failures.push('brand conversion')}
      }
      e.setAttributeNS('urn:other',name,'99');if(e[name]!==0)failures.push('namespace');
      const observer=new MutationObserver(()=>{});observer.observe(e,{attributes:true,attributeOldValue:true});
      e[name]={valueOf(){e.setAttributeNS(null,name,'21');return 17.9}};
      if(e[name]!==17||JSON.stringify(observer.takeRecords().map(r=>r.oldValue))!==JSON.stringify([null,'21']))failures.push('conversion ordering');
      observer.disconnect();const clone=e.cloneNode();document.implementation.createHTMLDocument().adoptNode(clone);e[name]=22;
      if(clone[name]!==17||clone.getAttributeNS('urn:other',name)!=='99')failures.push('clone/adopt');
      e.getAttribute=e.setAttribute=()=>{throw Error('override')};e[name]=8;if(e[name]!==8)failures.push('canonical');
    }
    return failures;
  })()`),[]);
});

test('Image factory follows Safari prototype, subclass and conversion order without public overrides',async()=>{
  assert.deepEqual(await evaluate(`(()=>{
    const failures=[],d=Object.getOwnPropertyDescriptor(Image,'prototype');
    if(Image.name!=='Image'||Image.length!==0||d.value!==HTMLImageElement.prototype||d.writable||d.enumerable||d.configurable||Object.getPrototypeOf(Image)!==Function.prototype)failures.push('shape');
    try{Image();failures.push('call')}catch(e){if(e.name!=='TypeError')failures.push(e.name)}
    class X extends Image{};const x=new X(7,8);if(!(x instanceof X)||!(x instanceof HTMLImageElement)||x.width!==7||x.height!==8)failures.push('subclass');
    const order=[],Fake=new Proxy(function(){},{get(t,k,r){if(k==='prototype')order.push('prototype');return Reflect.get(t,k,r)}});
    const e=Reflect.construct(Image,[{valueOf(){order.push('width');return 1}},{valueOf(){order.push('height');return 2}}],Fake);
    if(order.join()!=='width,height,prototype'||Object.getPrototypeOf(e)!==Fake.prototype)failures.push('order/newTarget');
    function Primitive(){}Primitive.prototype=1;if(Object.getPrototypeOf(Reflect.construct(Image,[],Primitive))!==HTMLImageElement.prototype)failures.push('fallback prototype');
    const create=document.createElement,width=Object.getOwnPropertyDescriptor(HTMLImageElement.prototype,'width');
    try{document.createElement=()=>{throw Error('public create')};Object.defineProperty(HTMLImageElement.prototype,'width',{configurable:true,set(){throw Error('public width')}});
      const e=new Image(19,12);if(e.getAttribute('width')!=='19'||e.height!==12)failures.push('canonical factory');
    }finally{document.createElement=create;Object.defineProperty(HTMLImageElement.prototype,'width',width)}
    return failures;
  })()`),[]);
});

test('image resource placeholder surfaces and empty-image defaults match Safari',async()=>{
  assert.deepEqual(await evaluate(`(()=>{
    const failures=[],e=document.createElement('img');
    for(const[name,value]of Object.entries({complete:true,currentSrc:'',naturalWidth:0,naturalHeight:0,x:0,y:0})){
      const d=Object.getOwnPropertyDescriptor(HTMLImageElement.prototype,name);
      if(e[name]!==value||!d.enumerable||!d.configurable||d.set!==undefined||d.get.name!=='get '+name||d.get.length!==0)failures.push(name);
      for(const receiver of [{},document.createElement('div'),Object.create(HTMLImageElement.prototype)])try{d.get.call(receiver);failures.push('getter receiver')}catch(err){if(err.name!=='TypeError')failures.push(err.name)}
    }
    const decode=Object.getOwnPropertyDescriptor(HTMLImageElement.prototype,'decode');
    if(!decode.enumerable||!decode.configurable||!decode.writable||decode.value.name!=='decode'||decode.value.length!==0)failures.push('decode shape');
    return failures;
  })()`),[]);
});
