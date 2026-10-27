import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {createSession} from '../../../scripts/coverage/session.mjs';
import {documentCollectionNames, documentCollectionCases, measureDocumentCollections} from '../../../scripts/coverage/document-collection-cases.mjs';
const reference=JSON.parse(fs.readFileSync(new URL('../fixtures/document-collections.safari.json',import.meta.url)));
async function evaluate(source){const session=await createSession();try{return await(await session.newPage()).evaluate(source)}finally{await session.close()}}

test('Document collections match Safari membership across namespaces, attributes and tree boundaries',async()=>{
  assert.deepEqual(reference.map(({expected,...c})=>c),documentCollectionCases);
  const actual=[];
  // Isolate independent cases so detached document/shadow trees do not accumulate
  // in one native arena and exhaust the per-evaluation runtime deadline.
  for(let start=0;start<documentCollectionCases.length;start+=250) {
    actual.push(...JSON.parse(await evaluate(`JSON.stringify((${measureDocumentCollections})(${JSON.stringify(documentCollectionCases.slice(start,start+250))}))`)));
  }
  const differences=reference.flatMap((c,i)=>JSON.stringify(c.expected)===JSON.stringify(actual[i])?[]:[{...c,actual:actual[i]}]);
  assert.deepEqual(differences.slice(0,6),[],`${differences.length} of ${reference.length} cases differ`);
});

test('Document collections have readonly Safari descriptors, native receivers and stable per-document identity',async()=>{
  assert.deepEqual(await evaluate(`(()=>{
    const failures=[],names=${JSON.stringify(documentCollectionNames)},d=document.implementation.createHTMLDocument(),other=document.implementation.createHTMLDocument();
    for(const name of names){
      const p=Object.getOwnPropertyDescriptor(Document.prototype,name),c=d[name];
      if(!p.enumerable||!p.configurable||p.set!==undefined||p.get.name!=='get '+name||p.get.length!==0)failures.push([name,'descriptor']);
      if(c!==d[name]||c===other[name]||!(c instanceof HTMLCollection)||Object.prototype.toString.call(c)!=='[object HTMLCollection]')failures.push([name,'identity']);
      const forged=document.createElement('div');Object.setPrototypeOf(forged,Document.prototype);
      for(const receiver of [null,{},Object.create(Document.prototype),forged])try{p.get.call(receiver);failures.push([name,'accepted receiver'])}catch(e){if(e.name!=='TypeError')failures.push([name,e.name])}
      d[name]=null;if(d[name]!==c||Object.hasOwn(d,name))failures.push([name,'readonly']);
      try{(function(){'use strict';d[name]=null})();failures.push([name,'strict assignment'])}catch(e){if(e.name!=='TypeError')failures.push(e.name)}
    }
    const collections=names.map(name=>d[name]);Object.setPrototypeOf(d,null);
    for(let i=0;i<names.length;i++)if(Object.getOwnPropertyDescriptor(Document.prototype,names[i]).get.call(d)!==collections[i]||collections[i].length!==0)failures.push([names[i],'native identity']);
    if(collections[2]!==collections[3])failures.push('embeds/plugins alias');
    return failures;
  })()`),[]);
});

test('Document collections follow root membership, reordering, adoption, cloning and document.write',async()=>{
  assert.deepEqual(await evaluate(`(()=>{
    const names=${JSON.stringify(documentCollectionNames)},tags=['form','img','embed','embed','a','script','a',null],failures=[];
    for(let i=0;i<names.length;i++){
      const name=names[i],tag=tags[i],d=document.implementation.createHTMLDocument(),other=document.implementation.createHTMLDocument(),collection=d[name],next=other[name];
      if(!tag){d.body.innerHTML='<applet></applet><object type="application/x-java-applet"></object>';if(collection.length!==0)failures.push('applets');continue}
      const create=label=>{const e=d.createElement(tag);e.id=label;e.setAttribute('href','');e.setAttribute('name','');return e};
      const a=create('first'),b=create('second');d.body.append(a,b);
      if(collection.length!==2||collection[0]!==a||collection.item(1)!==b)failures.push([name,'insert']);
      d.body.insertBefore(b,a);if(collection[0]!==b||collection[1]!==a)failures.push([name,'reorder']);
      const clone=d.cloneNode(true);if(clone[name]===collection||clone[name].length!==2||clone[name][0]===b||clone[name][0].id!=='second')failures.push([name,'clone']);
      other.body.appendChild(other.adoptNode(b));if(collection.length!==1||next.length!==1||next[0]!==b)failures.push([name,'adopt']);
      d.documentElement.remove();if(collection.length!==0)failures.push([name,'root removal']);
      d.appendChild(a);if(collection.length!==1||collection[0]!==a)failures.push([name,'root membership']);
      d.open();if(collection.length!==0)failures.push([name,'open']);
      d.write('<!doctype html><'+tag+' id="written" href="" name=""></'+tag+'>');d.close();
      if(collection!==d[name]||collection.length!==1||collection[0].id!=='written')failures.push([name,'write']);
    }
    return failures;
  })()`),[]);
});

test('Document links and anchors react to canonical null-namespace attribute changes',async()=>{
  assert.deepEqual(await evaluate(`(()=>{
    const d=document.implementation.createHTMLDocument(),a=d.createElement('a'),area=d.createElement('area'),links=d.links,anchors=d.anchors,states=[];
    d.body.append(a,area);const read=()=>states.push([links.length,anchors.length]);read();
    a.setAttributeNS('urn:other','href','foreign');a.setAttributeNS('urn:other','name','foreign');read();
    a.setAttributeNS(null,'HREF','upper');a.setAttributeNS(null,'NAME','upper');read();
    a.setAttributeNS(null,'href','');a.setAttributeNS(null,'name','');read();
    area.setAttribute('href','');area.setAttribute('name','area');read();
    a.getAttributeNodeNS(null,'href').value='next';a.getAttributeNodeNS(null,'name').value='next';read();
    a.removeAttributeNS(null,'href');a.removeAttributeNS(null,'name');read();
    area.removeAttribute('href');read();return states;
  })()`),[[0,0],[0,0],[0,0],[1,1],[2,1],[2,1],[1,0],[0,0]]);
});

test('Document collection getters, aliases and named lookup ignore overridden public DOM accessors',async()=>{
  assert.deepEqual(await evaluate(`(()=>{
    const names=${JSON.stringify(documentCollectionNames)},d=document.implementation.createHTMLDocument(),failures=[];
    d.body.innerHTML='<form id="form-id"></form><img id="image-id" name="image-name"><embed id="embed-id"><a id="anchor-id" href="" name="anchor-name"></a><script id="script-id" type="application/json"></script>';
    const image=d.images[0],embeds=d.embeds,images=d.images;
    for(const name of ['querySelector','querySelectorAll','getElementsByTagName','getElementsByName','documentElement','embeds'])Object.defineProperty(d,name,{get(){throw Error('public '+name)}});
    for(const name of ['id','name','namespaceURI','localName','getAttribute','getAttributeNS'])Object.defineProperty(image,name,{get(){throw Error('public '+name)}});
    for(const name of names){const collection=Object.getOwnPropertyDescriptor(Document.prototype,name).get.call(d);if(collection.length!==(name==='applets'?0:1))failures.push([name,'canonical members'])}
    if(d.plugins!==embeds||images.namedItem('image-id')!==image||images.namedItem('image-name')!==image||images['image-id']!==image||!Reflect.ownKeys(images).includes('image-name'))failures.push('canonical names/alias');
    return failures;
  })()`),[]);
});

test('Document collection iteration is live and named lookup follows tree order and attribute updates',async()=>{
  assert.deepEqual(await evaluate(`(()=>{
    const d=document.implementation.createHTMLDocument(),a=d.createElement('img'),b=d.createElement('img'),c=d.createElement('img'),failures=[];
    a.name='same';b.id='same';a.id='a';b.name='b';c.id='c';d.body.append(a,b);const list=d.images;
    if(list.namedItem('same')!==b||list.same!==b||list.item(9)!==null||list[9]!==undefined)failures.push('ID before name');
    d.body.insertBefore(b,a);if(list.namedItem('same')!==b)failures.push('reordered names');
    b.id='changed';if(list.namedItem('same')!==a||list.changed!==b)failures.push('live names');
    const it=list[Symbol.iterator]();if(it.next().value!==b)failures.push('first iteration');a.remove();d.body.append(c);
    if(it.next().value!==c||!it.next().done)failures.push('live iteration');
    d.body.append(a);if(!it.next().done)failures.push('completed iterator');
    if(HTMLCollection.prototype[Symbol.iterator]!==Array.prototype.values||[...HTMLCollection.prototype[Symbol.iterator].call({0:'generic',length:1})][0]!=='generic')failures.push('generic array iterator');
    return failures;
  })()`),[]);
});
