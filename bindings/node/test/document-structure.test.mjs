import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {createSession} from '../../../scripts/coverage/session.mjs';
import {documentStructureReadCases, documentBodyWriteCases, measureDocumentStructure} from '../../../scripts/coverage/document-structure-cases.mjs';
const reference=JSON.parse(fs.readFileSync(new URL('./fixtures/document-structure.safari.json',import.meta.url)));

for(const [mode,cases] of [['read',documentStructureReadCases],['write',documentBodyWriteCases]]) {
  test(`Document structure ${mode} matches Safari roots, namespaces, replacement, adoption and mutation records`,async()=>{
    assert.deepEqual(reference[mode].map(({expected,...c})=>c),cases);
    const session=await createSession();try{
      const page=await session.newPage();
      const actual=JSON.parse(await page.evaluate(`JSON.stringify((${measureDocumentStructure})(${JSON.stringify(cases)},${mode==='write'}))`));
      const differences=reference[mode].flatMap((c,i)=>JSON.stringify(c.expected)===JSON.stringify(actual[i])?[]:[{...c,actual:actual[i]}]);
      assert.deepEqual(differences.slice(0,6),[],`${differences.length} of ${cases.length} cases differ`);
    }finally{await session.close()}
  });
}

test('Document structure descriptors and receiver checks use native document and element identities',async()=>{
  const session=await createSession();try{assert.deepEqual(await(await session.newPage()).evaluate(`(()=>{
    const failures=[];
    for(const name of ['documentElement','head','body']){
      const p=Object.getOwnPropertyDescriptor(Document.prototype,name),writable=name==='body';
      if(!p.enumerable||!p.configurable||p.get.name!=='get '+name||p.get.length!==0||!!p.set!==writable||writable&&(p.set.name!=='set body'||p.set.length!==1))failures.push([name,'descriptor']);
      const forged=document.createElement('div');Object.setPrototypeOf(forged,Document.prototype);
      for(const receiver of [null,{},Object.create(Document.prototype),forged])for(const method of [p.get,...p.set?[p.set]:[]]){
        try{method.call(receiver,null);failures.push([name,'accepted receiver'])}catch(e){if(e.name!=='TypeError')failures.push([name,e.name])}
      }
      const d=document.implementation.createHTMLDocument(),expected=d[name];
      if(!writable){d[name]=null;if(d[name]!==expected||Object.hasOwn(d,name))failures.push([name,'readonly']);try{(function(){'use strict';d[name]=null})();failures.push([name,'strict'])}catch(e){if(e.name!=='TypeError')failures.push(e.name)}}
      Object.setPrototypeOf(d,null);if(p.get.call(d)!==expected)failures.push([name,'native document']);
      if(writable){const incoming=document.createElement('body');Object.setPrototypeOf(incoming,null);p.set.call(d,incoming);if(p.get.call(d)!==incoming)failures.push('native body')}
    }
    const d=document.implementation.createHTMLDocument(),set=Object.getOwnPropertyDescriptor(Document.prototype,'body').set;
    for(const value of [Object.create(HTMLBodyElement.prototype),new Proxy(document.createElement('body'),{}),Symbol()]){
      try{set.call(d,value);failures.push('forged body')}catch(e){if(e.name!=='TypeError')failures.push(e.name)}
    }
    return failures;
  })()`),[])}finally{await session.close()}
});

test('Document structure stays live across root replacement, cloning, import and adoption',async()=>{
  const session=await createSession();try{assert.deepEqual(await(await session.newPage()).evaluate(`(()=>{
    const failures=[],d=document.implementation.createHTMLDocument(),root=d.documentElement,head=d.head,body=d.body;
    d.insertBefore(d.createComment('before'),root);
    if(d.documentElement!==root)failures.push('comment');
    root.remove();if(d.documentElement!==null||d.head!==null||d.body!==null)failures.push('removed');
    d.appendChild(root);if(d.head!==head||d.body!==body)failures.push('reinserted');
    const second=d.createElement('head');root.insertBefore(second,head);if(d.head!==second)failures.push('first head');second.remove();
    const f=d.createElement('frameset');root.insertBefore(f,body);if(d.body!==f)failures.push('first body');f.remove();
    const clone=d.cloneNode(true);if(clone.documentElement===root||clone.head===head||clone.body===body||clone.head.localName!=='head'||clone.body.localName!=='body')failures.push('clone');
    const empty=d.cloneNode(false);if(empty.documentElement!==null||empty.head!==null||empty.body!==null)failures.push('shallow clone');
    const adopted=empty.adoptNode(root);empty.appendChild(adopted);if(d.documentElement!==null||empty.head!==head||empty.body!==body||body.ownerDocument!==empty)failures.push('adopted');
    d.appendChild(d.importNode(root,true));if(d.head===head||d.head.localName!=='head'||d.body.ownerDocument!==d)failures.push('imported');
    return failures;
  })()`),[])}finally{await session.close()}
});

test('Document body writes ignore public selectors and replacement methods and update live ranges',async()=>{
  const session=await createSession();try{assert.deepEqual(await(await session.newPage()).evaluate(`(()=>{
    const d=document.implementation.createHTMLDocument(),root=d.documentElement,old=d.body,incoming=d.createElement('body');old.textContent='old';
    const range=d.createRange();range.setStart(old.firstChild,1);range.collapse(true);
    const observer=new MutationObserver(()=>{});observer.observe(root,{childList:true});
    for(const [object,names] of [[d,['documentElement','head','querySelector','querySelectorAll']], [root,['namespaceURI','localName','replaceChild','appendChild']], [incoming,['namespaceURI','localName']]])
      for(const name of names)Object.defineProperty(object,name,{configurable:true,get(){throw Error('public '+name)}});
    const get=Object.getOwnPropertyDescriptor(Document.prototype,'body').get,set=Object.getOwnPropertyDescriptor(Document.prototype,'body').set;
    Object.defineProperty(d,'body',{get(){throw Error('public body')}});
    set.call(d,incoming);
    const records=observer.takeRecords();observer.disconnect();
    return [get.call(d)===incoming,range.startContainer===root,range.startOffset,records.length,records[0]?.addedNodes[0]===incoming,records[0]?.removedNodes[0]===old];
  })()`),[true,true,1,1,true,true])}finally{await session.close()}
});

test('Document body replacement runs custom-element disconnection, adoption and connection reactions',async()=>{
  const session=await createSession();try{assert.deepEqual(await(await session.newPage()).evaluate(`(()=>{
    const log=[],a=document.implementation.createHTMLDocument(),b=document.implementation.createHTMLDocument();
    const owner=d=>d===document?'main':d===a?'a':'b';
    customElements.define('body-lifecycle',class extends HTMLElement{
      connectedCallback(){log.push(['connected',this.id,owner(this.ownerDocument)])}
      disconnectedCallback(){log.push(['disconnected',this.id,owner(this.ownerDocument)])}
      adoptedCallback(old,next){log.push(['adopted',this.id,owner(old),owner(next)])}
    });
    const old=document.body,one=document.createElement('body-lifecycle');one.id='old';old.appendChild(one);
    const incoming=document.createElement('body'),two=document.createElement('body-lifecycle');two.id='new';incoming.appendChild(two);
    log.length=0;document.body=incoming;const first=log.splice(0);
    a.body=incoming;const second=log.splice(0);b.body=incoming;const third=log.splice(0);document.body=old;
    return [first,second,third];
  })()`),[[['disconnected','old','main'],['connected','new','main']],[['disconnected','new','a'],['adopted','new','main','a'],['connected','new','a']],[['disconnected','new','b'],['adopted','new','a','b'],['connected','new','b']]])}finally{await session.close()}
});
