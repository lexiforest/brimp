import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {createSession} from '../../../scripts/coverage/session.mjs';
import {tokenListCases, relationTokens, measureTokenLists, measureTokenSupports, measureTokenShape} from '../../../scripts/coverage/token-list-cases.mjs';
const reference = JSON.parse(fs.readFileSync(new URL('../fixtures/token-list.safari.json', import.meta.url)));
async function evaluate(source) {
  const session=await createSession();
  try {return await (await session.newPage()).evaluate(source);} finally {await session.close();}
}

test(`DOMTokenList operations match ${tokenListCases.length} Safari conversion and mutation cases`, async () => {
  assert.deepEqual(reference.cases.map(({expected,...c})=>c),tokenListCases);
  const actual=JSON.parse(await evaluate(`JSON.stringify((${measureTokenLists})(${JSON.stringify(tokenListCases)}))`));
  const diffs=reference.cases.flatMap((c,i)=>JSON.stringify(c.expected)===JSON.stringify(actual[i])?[]:[{...c,actual:actual[i]}]);
  assert.equal(diffs.length,0,JSON.stringify({total:diffs.length,first:diffs.slice(0,10)},null,2));
});

test('DOMTokenList support metadata and interface descriptors match Safari', async () => {
  assert.deepEqual(await evaluate(`(${measureTokenSupports})(${JSON.stringify(relationTokens)})`),reference.supports);
  assert.deepEqual(JSON.parse(await evaluate(`JSON.stringify((${measureTokenShape})())`)),reference.shape);
});

test('token methods validate brands and arguments before conversion, while iteration remains generic', async () => {
  assert.deepEqual(await evaluate(`(() => {
    const failures=[],p=DOMTokenList.prototype;
    for(const name of ['item','contains','add','remove','toggle','replace','supports','toString']) {
      for(const receiver of [{},Object.create(p),new Proxy(document.createElement('a').relList,{})]) {
        let converted=false;
        try{p[name].call(receiver,{toString(){converted=true;return 'a'},valueOf(){converted=true;return 0}},'b');failures.push([name,'brand'])}
        catch(e){if(e.name!=='TypeError'||converted)failures.push([name,'conversion before brand'])}
      }
    }
    const a=document.createElement('a'),list=a.relList;
    a.rel='a b';
    for(const name of ['item','contains','toggle','supports'])try{list[name]();failures.push([name,'missing argument'])}catch(e){if(e.name!=='TypeError')failures.push(e.name)}
    let converted=false;try{list.replace({toString(){converted=true;return 'a'}});failures.push('missing replace arg')}catch(e){if(e.name!=='TypeError'||converted)failures.push('arity before conversion')}
    for(const name of ['entries','keys','values']) {
      const actual=[...p[name].call(['a','b'])], expected=[...Array.prototype[name].call(['a','b'])];
      if(JSON.stringify(actual)!==JSON.stringify(expected))failures.push(['generic',name]);
    }
    const seen=[];p.forEach.call({0:'x',length:1},v=>seen.push(v));if(seen.join()!=='x')failures.push('generic forEach');
    list.add('a');list.value=' a  b ';
    Object.defineProperties(list,{value:{get(){throw Error('override')},set(){throw Error('override')}},contains:{value(){throw Error('override')}},add:{value(){throw Error('override')}},remove:{value(){throw Error('override')}}});
    if(p.toString.call(list)!==' a  b '||list.toggle('a')!==false||list.toggle('z')!==true)failures.push('canonical methods');
    const branded=document.createElement('a').relList;Object.setPrototypeOf(branded,null);
    p.add.call(branded,'still-branded');
    if(p.item.call(branded,0)!=='still-branded')failures.push('brand independent of prototype');
    return failures;
  })()`),[]);
});

test('relList forwards value assignment, preserves identity and uses canonical null-namespace attributes', async () => {
  assert.deepEqual(await evaluate(`(() => {
    const failures=[],a=document.createElement('a'),list=a.relList;
    const d=Object.getOwnPropertyDescriptor(HTMLAnchorElement.prototype,'relList');
    if(!d.enumerable||!d.configurable||d.get.name!=='get relList'||d.set.name!=='set relList'||d.get.length!==0||d.set.length!==1)failures.push('descriptor');
    a.setAttributeNS('urn:other','rel','foreign');a.relList='  a a b ';
    if(list!==a.relList||list.value!=='  a a b '||[...list].join()!=='a,b'||a.getAttributeNS('urn:other','rel')!=='foreign')failures.push('forwarding/namespace');
    const clone=a.cloneNode(),other=document.implementation.createHTMLDocument();other.adoptNode(a);
    if(a.relList!==list||clone.relList===list||clone.relList.value!==list.value)failures.push('clone/adoption');
    a.getAttribute=a.setAttribute=a.hasAttribute=()=>{throw Error('override')};list.add('c');
    if(a.rel!=='a b c')failures.push('canonical attribute');
    let reads=0;const forwardedTarget={};
    Object.defineProperty(a,'relList',{get(){reads++;return forwardedTarget},configurable:true});
    d.set.call(a,'x');if(forwardedTarget.value!=='x'||reads!==1)failures.push('observable forwarding getter');
    delete a.relList;
    let forwarded;Object.defineProperty(list,'value',{set(value){forwarded=value}});
    const value={toString(){throw Error('must not convert before forwarding')}};d.set.call(a,value);
    if(forwarded!==value)failures.push('PutForwards');
    return failures;
  })()`),[]);
});

test('token indexed properties and iterators stay live without using public item or length overrides', async () => {
  assert.deepEqual(await evaluate(`(() => {
    const failures=[],a=document.createElement('a');a.rel='a b';const list=a.relList;
    if(JSON.stringify(Object.keys(list))!=='["0","1"]')failures.push('keys');
    const d=Object.getOwnPropertyDescriptor(list,'0');if(d.value!=='a'||d.writable||!d.enumerable||!d.configurable)failures.push('index descriptor');
    if(Reflect.set(list,'0','x')||Reflect.defineProperty(list,'8',{value:'x'})||Reflect.deleteProperty(list,'0'))failures.push('readonly indices');
    if(!Reflect.deleteProperty(list,'8'))failures.push('missing index deletion');
    list.extra=42;list['4294967295']='expando';list['01']='zero-one';
    if(list.extra!==42||list['4294967295']!=='expando'||list['01']!=='zero-one')failures.push('expandos');
    const iterator=list.values();if(iterator.next().value!=='a')failures.push('iterator start');
    a.rel='c d e';if(iterator.next().value!=='d'||iterator.next().value!=='e'||!iterator.next().done)failures.push('live iterator');
    a.rel='a b c';const seen=[];list.forEach((v,i,self)=>{seen.push([v,i,self===list]);if(i===0)a.rel='x y z extra'});
    if(JSON.stringify(seen)!=='[["a",0,true],["y",1,true],["z",2,true]]')failures.push('forEach live values and fixed length');
    list.item=()=>{throw Error('item override')};Object.defineProperty(list,'length',{value:99});
    if(list[0]!=='x'||list[4]!==undefined||'4' in list||Object.getOwnPropertyDescriptor(list,'4')!==undefined)failures.push('canonical indices');
    return failures;
  })()`),[]);
});

test('token conversions precede validation and read live attributes after reentrant changes', async () => {
  assert.deepEqual(await evaluate(`(() => {
    const a=document.createElement('a'),list=a.relList,failures=[],log=[];
    a.rel='before';
    list.add({toString(){log.push('first');a.rel='intermediate';return 'x'}}, {toString(){log.push('second');a.rel='final';return 'y'}});
    if(a.rel!=='final x y'||log.join()!=='first,second')failures.push('conversion order');
    try{list.add('',{toString(){a.rel='changed';throw new RangeError('conversion')}});failures.push('no error')}
    catch(e){if(e.name!=='RangeError'||a.rel!=='changed')failures.push('conversion before validation')}
    const observer=new MutationObserver(()=>{});observer.observe(a,{attributes:true,attributeOldValue:true});
    try{list.add('would-add','bad token');failures.push('no invalid token error')}catch(e){if(e.name!=='InvalidCharacterError')failures.push(e.name)}
    if(a.rel!=='changed'||observer.takeRecords().length)failures.push('atomic token validation');
    const token=String.fromCharCode(0xd800,0,0xdc00);list.value=token+' '+token;list.add('next');
    if(list[0]!==token||a.rel!==token+' next')failures.push('UTF-16 preservation');
    const records=observer.takeRecords();if(records.length!==2||records[1].oldValue!==token+' '+token)failures.push('UTF-16 mutation oldValue');
    observer.disconnect();return failures;
  })()`),[]);
});
