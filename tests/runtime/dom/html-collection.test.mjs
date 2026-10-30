import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import { createSession } from '../../../scripts/coverage/session.mjs';
import {
  htmlCollectionMethodCases,
  htmlCollectionPropertyCases,
  measureHtmlCollectionMethods,
  measureHtmlCollectionProperties,
} from '../../../scripts/coverage/html-collection-cases.mjs';
const reference = JSON.parse(
  fs.readFileSync(new URL('../fixtures/html-collection.safari.json', import.meta.url)),
);
async function evaluate(source) {
  const session = await createSession();
  try {
    return await (await session.newPage()).evaluate(source);
  } finally {
    await session.close();
  }
}
for (const [name, cases, measure] of [
  ['methods', htmlCollectionMethodCases, measureHtmlCollectionMethods],
  ['properties', htmlCollectionPropertyCases, measureHtmlCollectionProperties],
]) {
  test(`HTMLCollection ${name} match Safari conversion and indexed/named property rules`, async () => {
    assert.deepEqual(
      reference[name].map(({ expected, ...c }) => c),
      cases,
    );
    const actual = JSON.parse(
      await evaluate(`JSON.stringify((${measure})(${JSON.stringify(cases)}))`),
    );
    const differences = reference[name].flatMap((c, i) =>
      JSON.stringify(c.expected) === JSON.stringify(actual[i]) ? [] : [{ ...c, actual: actual[i] }],
    );
    assert.deepEqual(
      differences.slice(0, 8),
      [],
      `${differences.length} of ${cases.length} cases differ`,
    );
  });
}

test('HTMLCollection construction, prototype, descriptors and subclass ownership match Safari', async () => {
  assert.deepEqual(
    await evaluate(`(()=>{
    const p=HTMLCollection.prototype,failures=[];
    if(HTMLCollection.length!==0||HTMLCollection.name!=='HTMLCollection'||Object.getPrototypeOf(p)!==Object.prototype||p.constructor!==HTMLCollection)failures.push('constructor/prototype');
    const descriptor=Object.getOwnPropertyDescriptor(HTMLCollection,'prototype');if(descriptor.writable||descriptor.enumerable||descriptor.configurable)failures.push('prototype descriptor');
    for(const construct of [()=>HTMLCollection(),()=>new HTMLCollection(),()=>new HTMLCollection([]),()=>new(class extends HTMLCollection {})(),()=>Reflect.construct(HTMLCollection,[],function Other(){})])try{construct();failures.push('construction accepted')}catch(e){if(e.name!=='TypeError')failures.push(e.name)}
    for(const name of ['length','item','namedItem']){
      const descriptor=Object.getOwnPropertyDescriptor(p,name),fn=descriptor.get??descriptor.value;
      if(!descriptor.enumerable||!descriptor.configurable||fn.length!==(name==='length'?0:1)||fn.name!==(name==='length'?'get length':name)||name==='length'&&descriptor.set!==undefined||name!=='length'&&!descriptor.writable)failures.push([name,'descriptor']);
    }
    const tag=Object.getOwnPropertyDescriptor(p,Symbol.toStringTag),iterator=Object.getOwnPropertyDescriptor(p,Symbol.iterator);
    if(tag.value!=='HTMLCollection'||tag.writable||tag.enumerable||!tag.configurable)failures.push('tag');
    if(iterator.value!==Array.prototype.values||!iterator.writable||iterator.enumerable||!iterator.configurable)failures.push('iterator');
    const d=document.implementation.createHTMLDocument(),form=d.createElement('form');form.innerHTML='<input>';
    if(Object.getPrototypeOf(d.images)!==p||Object.getPrototypeOf(form.elements)!==HTMLFormControlsCollection.prototype||Object.getPrototypeOf(HTMLFormControlsCollection.prototype)!==p||Object.prototype.toString.call(form.elements)!=='[object HTMLFormControlsCollection]'||Object.getOwnPropertyDescriptor(p,'length').get.call(form.elements)!==1)failures.push('subclass');
    return failures;
  })()`),
    [],
  );
});

test('HTMLCollection brands and required arguments are checked before conversion and ignore modified prototypes', async () => {
  assert.deepEqual(
    await evaluate(`(()=>{
    const p=HTMLCollection.prototype,failures=[],d=document.implementation.createHTMLDocument(),list=d.images;
    const length=Object.getOwnPropertyDescriptor(p,'length').get;let conversions=0;
    const value={[Symbol.toPrimitive](){conversions++;return 0}};
    for(const receiver of [null,{},Object.create(p),new Proxy(list,{})])for(const fn of [length,p.item,p.namedItem])try{fn.call(receiver,value);failures.push('accepted receiver')}catch(e){if(e.name!=='TypeError')failures.push(e.name)}
    if(conversions!==0)failures.push('conversion before branding');
    for(const name of ['item','namedItem'])try{p[name].call(list);failures.push('missing argument')}catch(e){if(e.name!=='TypeError')failures.push(e.name)}
    const image=d.createElement('img');image.id='one';d.body.append(image);Object.setPrototypeOf(list,null);
    if(length.call(list)!==1||p.item.call(list,0)!==image||p.namedItem.call(list,'one')!==image||list[0]!==image)failures.push('native identity');
    return failures;
  })()`),
    [],
  );
});

test('HTMLCollection converts arguments before reading live state and keeps indexed operations private', async () => {
  assert.deepEqual(
    await evaluate(`(()=>{
    const d=document.implementation.createHTMLDocument();d.body.innerHTML='<img id="a"><img id="b">';const list=d.images,a=list[0],b=list[1],failures=[],p=HTMLCollection.prototype;
    if(list.item({valueOf(){a.remove();return 0}})!==b)failures.push('item conversion order');
    if(list.namedItem({toString(){b.id='renamed';return 'renamed'}})!==b)failures.push('name conversion order');
    for(const key of ['__items','item','namedItem','length'])Object.defineProperty(list,key,{get(){throw Error('public '+key)},configurable:true});
    for(const key of ['id','name','getAttribute','namespaceURI'])Object.defineProperty(b,key,{get(){throw Error('public '+key)},configurable:true});
    if(list[0]!==b||!('0' in list)||Object.getOwnPropertyDescriptor(list,'0').value!==b||list.renamed!==b||!Reflect.ownKeys(list).includes('renamed')||p.item.call(list,0)!==b||p.namedItem.call(list,'renamed')!==b)failures.push('private operations');
    return failures;
  })()`),
    [],
  );
});

test('HTMLCollection expandos coexist with live names, prototype collisions and ordinary symbols', async () => {
  assert.deepEqual(
    await evaluate(`(()=>{
    const d=document.implementation.createHTMLDocument(),list=d.images,e=d.createElement('img'),symbol=Symbol('extra'),failures=[];
    Object.defineProperty(list,'future',{value:42,writable:false,configurable:false});list[symbol]=7;
    e.id='future';e.name='item';d.body.append(e);
    if(list.future!==42||list.item!==HTMLCollection.prototype.item||list.namedItem('future')!==e||list.namedItem('item')!==e||list[symbol]!==7)failures.push('expando priority');
    const keys=Reflect.ownKeys(list);if(keys.length!==4||keys[0]!=='0'||keys[1]!=='future'||keys[2]!=='item'||keys[3]!==symbol)failures.push('keys');
    if(Object.hasOwn(list,'item')||Object.keys(list).join()!=='0')failures.push('hidden names');
    e.id='renamed';if(list.renamed!==e||list.future!==42)failures.push('renamed');
    e.remove();if(list.length!==0||list.renamed!==undefined||list.future!==42||list[symbol]!==7)failures.push('removed');
    list.later=1;e.id='later';d.body.append(e);
    if(!Reflect.set(list,'later',2)||list.later!==2||!Reflect.defineProperty(list,'later',{value:3})||list.later!==3||!Reflect.deleteProperty(list,'later')||list.later!==e)failures.push('expando reveals live name');
    return failures;
  })()`),
    [],
  );
});

test('HTMLCollection iterator keeps the Safari Array values identity and live generic semantics', async () => {
  assert.deepEqual(
    await evaluate(`(()=>{
    const d=document.implementation.createHTMLDocument(),list=d.images,a=d.createElement('img'),b=d.createElement('img'),c=d.createElement('img'),out=[];
    d.body.append(a,b);const iterator=list[Symbol.iterator]();out.push(iterator.next().value===a);b.remove();d.body.append(c);out.push(iterator.next().value===c,iterator.next().done);d.body.append(b);out.push(iterator.next().done);
    out.push(HTMLCollection.prototype[Symbol.iterator]===Array.prototype.values,[...HTMLCollection.prototype[Symbol.iterator].call({0:'generic',length:1})][0]);
    return out;
  })()`),
    [true, true, true, true, true, 'generic'],
  );
});
