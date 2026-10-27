import test from 'node:test';
import assert from 'node:assert/strict';
import {createSession} from '../../../scripts/coverage/session.mjs';

async function evaluate(source) {
  const session = await createSession();
  try { return await (await session.newPage()).evaluate(source); }
  finally { await session.close(); }
}

test('Node text values distinguish containers, character data, attributes and documents', async () => {
  assert.deepEqual(await evaluate(`(() => {
    const doc=document.implementation.createHTMLDocument(),p=doc.createElement('p');
    p.innerHTML='a<b>b</b><!--hidden-->c';doc.body.append(p);
    const attr=doc.createAttribute('title');attr.value='attribute';
    const nodes=[p,p.firstChild,p.childNodes[2],attr,doc,doc.doctype];
    const before=nodes.map(n=>[n.textContent,n.nodeValue]);
    const errors=[];
    for(const name of ['textContent','nodeValue']) for(const node of nodes) {
      try{node[name]=Symbol();errors.push('none');}catch(e){errors.push(e.name);}
    }
    p.nodeValue='ignored';const ignored=p.textContent;
    const text=new Text('old');text.nodeValue=null;const empty=text.data;
    text.nodeValue=undefined;const unset=text.data;text.nodeValue=42;
    const fragment=doc.createDocumentFragment();fragment.textContent='fragment';
    const range=doc.createRange();range.selectNodeContents(p.firstChild);
    doc.textContent='ignored';doc.doctype.nodeValue='ignored';
    const documentRange=[range.startContainer===p.firstChild,range.startOffset,range.endOffset];
    const attrValue=Object.getOwnPropertyDescriptor(Node.prototype,'nodeValue');
    attrValue.set.call(attr,'updated');const borrowed=attrValue.get.call(attr);
    const descriptors=['textContent','nodeValue','normalize','cloneNode'].map(name=>
      Object.getOwnPropertyDescriptor(Node.prototype,name).enumerable);
    return [before,errors,ignored,empty,unset,text.data,fragment.textContent,doc.body.firstChild===p,descriptors,documentRange,borrowed];
  })()`), [[['abc',null],['a','a'],['hidden','hidden'],['attribute','attribute'],[null,null],[null,null]],
    Array(12).fill('TypeError'),'abc','','','42','fragment',true,Array(4).fill(true),[true,0,1],'updated']);
});

test('textContent replacement preserves detached nodes and updates observers and live ranges', async () => {
  assert.deepEqual(await evaluate(`(() => {
    const p=document.createElement('p');p.innerHTML='<b>old</b><!--comment-->';
    const old=p.firstChild,range=document.createRange();range.selectNodeContents(old.firstChild);
    const observer=new MutationObserver(()=>{});observer.observe(p,{childList:true,subtree:true,characterData:true});
    p.textContent='replacement';
    const records=observer.takeRecords().map(r=>[r.type,r.target===p,r.removedNodes.length,
      r.removedNodes[0]===old,r.addedNodes.length,r.addedNodes[0]===p.firstChild]);
    const result=[p.childNodes.length,p.firstChild.nodeType,p.textContent,old.parentNode,old.textContent,
      range.startContainer===p,range.startOffset,range.endContainer===p,range.endOffset,records];
    p.textContent=null;const cleared=p.childNodes.length;p.textContent=undefined;
    observer.disconnect();return [result,cleared,p.childNodes.length];
  })()`), [[1,3,'replacement',null,'old',true,0,true,0,[['childList',true,2,true,1,true]]],0,0]);
});

test('normalize merges adjacent text, respects comment boundaries and moves live range endpoints', async () => {
  assert.deepEqual(await evaluate(`(() => {
    const p=document.createElement('p'),a=new Text('a'),b=new Text('bc'),empty=new Text(''),c=new Text('d');
    const nested=document.createElement('b');nested.append(new Text('x'),new Text('y'));
    p.append(a,b,empty,c,document.createComment('stop'),new Text('e'),nested);
    const range=document.createRange();range.setStart(b,1);range.setEnd(c,1);
    const boundary=document.createRange();boundary.setStart(p,1);boundary.collapse(true);
    const observer=new MutationObserver(()=>{});observer.observe(p,{subtree:true,characterData:true,characterDataOldValue:true,childList:true});
    p.normalize();
    const records=observer.takeRecords();observer.disconnect();
    return [Array.from(p.childNodes,n=>[n.nodeType,n.textContent]),p.firstChild===a,b.parentNode,empty.parentNode,
      range.startContainer===a,range.startOffset,range.endContainer===a,range.endOffset,
      boundary.startContainer===a,boundary.startOffset,nested.childNodes.length,
      records.filter(r=>r.type==='characterData'&&r.target===a).map(r=>r.oldValue)];
  })()`), [[[3,'abcd'],[8,'stop'],[3,'e'],[1,'xy']],true,null,null,true,2,true,4,true,1,1,['a','abc']]);
});

test('cloneNode copies independent subtrees and template content without listeners or ordinary shadow roots', async () => {
  assert.deepEqual(await evaluate(`(() => {
    const p=document.createElement('div');p.innerHTML='<b title="one">text</b><template><i>inside</i></template>';
    const shadow=p.attachShadow({mode:'open'});shadow.textContent='shadow';
    let calls=0;p.addEventListener('custom',()=>calls++);p.extra=42;
    const shallow=p.cloneNode(),copy=p.cloneNode(true);copy.dispatchEvent(new Event('custom'));
    const before=[shallow.childNodes.length,copy.childNodes.length,copy.parentNode,copy.ownerDocument===document,
      copy.firstChild!==p.firstChild,copy.lastChild.content.textContent,copy.shadowRoot,calls,copy.extra===undefined];
    copy.firstChild.setAttribute('title','two');copy.firstChild.firstChild.data='changed';
    const errors=[];for(const deep of [false,true]) {try{shadow.cloneNode(deep);errors.push('none');}catch(e){errors.push(e.name);}}
    const attr=p.firstChild.getAttributeNode('title'),attrCopy=attr.cloneNode();
    return [before,p.firstChild.getAttribute('title'),p.firstChild.textContent,
      attrCopy.value,attrCopy.ownerElement,attrCopy.ownerDocument===document,errors];
  })()`), [[0,2,null,true,true,'inside',null,0,true],'one','text','one',null,true,Array(2).fill('NotSupportedError')]);
});
