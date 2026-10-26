import test from 'node:test';
import assert from 'node:assert/strict';
import {createSession} from '../../../scripts/coverage/session.mjs';

async function evaluate(source) {
  const session = await createSession();
  try { return await (await session.newPage()).evaluate(source); }
  finally { await session.close(); }
}

test('Node namespace lookup follows element namespaces and inherited declarations', async () => {
  assert.deepEqual(await evaluate(`(() => {
    const xmlns='http://www.w3.org/2000/xmlns/',root=document.createElementNS('urn:root','r:root');
    root.setAttributeNS(xmlns,'xmlns:p','urn:outer');root.setAttributeNS(xmlns,'xmlns','urn:default');
    const child=document.createElementNS(null,'child'),text=new Text('text');root.append(child);child.append(text);
    child.setAttribute('title','value');const attr=child.getAttributeNode('title');
    const before=[child.lookupNamespaceURI('r'),child.lookupNamespaceURI('p'),text.lookupNamespaceURI(null),
      attr.lookupPrefix('urn:outer'),child.isDefaultNamespace('urn:default'),root.lookupPrefix('urn:root')];
    child.setAttributeNS(xmlns,'xmlns:p','urn:inner');
    const shadowed=[child.lookupNamespaceURI('p'),child.lookupPrefix('urn:outer'),child.lookupPrefix('urn:inner')];
    child.setAttributeNS(xmlns,'xmlns','');
    return [before,shadowed,text.lookupNamespaceURI(''),text.isDefaultNamespace(null),document.lookupNamespaceURI(null),
      document.documentElement.lookupPrefix('http://www.w3.org/1999/xhtml')];
  })()`), [['urn:root','urn:outer','urn:default','p',true,'r'],['urn:inner','p','p'],null,true,
    'http://www.w3.org/1999/xhtml',null]);
});

test('Node namespace lookup respects detached, document, fragment and shadow boundaries', async () => {
  assert.deepEqual(await evaluate(`(() => {
    const xmlns='http://www.w3.org/2000/xmlns/',host=document.createElement('div');
    host.setAttributeNS(xmlns,'xmlns:p','urn:host');const shadow=host.attachShadow({mode:'open'});
    const child=document.createElementNS(null,'child');shadow.append(child);
    const attr=document.createAttributeNS('urn:attr','a:name'),fragment=document.createDocumentFragment();
    const doc=document.implementation.createHTMLDocument();doc.documentElement.setAttributeNS(xmlns,'xmlns:p','urn:document');
    const comment=doc.createComment('comment');doc.appendChild(comment);
    return [shadow.lookupNamespaceURI('p'),child.lookupNamespaceURI('p'),attr.lookupNamespaceURI('a'),
      fragment.lookupNamespaceURI(null),doc.lookupNamespaceURI('p'),doc.doctype.lookupNamespaceURI('p'),
      comment.lookupNamespaceURI('p'),new Text('detached').lookupPrefix('urn:host')];
  })()`), [null,null,null,null,'urn:document',null,null,null]);
});

test('Node namespace methods require nullable strings and reject Symbols', async () => {
  assert.deepEqual(await evaluate(`(() => {
    const node=document.createElement('div'),errors=[];
    for(const name of ['lookupNamespaceURI','lookupPrefix','isDefaultNamespace']) {
      for(const args of [[],[Symbol()]]) {try{node[name](...args);errors.push('none');}catch(e){errors.push(e.name);}}
    }
    return [errors,node.lookupNamespaceURI(undefined),node.lookupPrefix(undefined),node.isDefaultNamespace(undefined),
      node.lookupNamespaceURI({toString(){return '';}})];
  })()`), [Array(6).fill('TypeError'),'http://www.w3.org/1999/xhtml',null,false,'http://www.w3.org/1999/xhtml']);
});

test('compareDocumentPosition follows ordinary tree and attribute order after edits', async () => {
  assert.deepEqual(await evaluate(`(() => {
    const p=document.createElement('div'),a=document.createElement('i'),b=document.createElement('b');p.append(a,b);
    p.setAttribute('x','one');p.setAttribute('y','two');const x=p.getAttributeNode('x'),y=p.getAttributeNode('y');
    const before=[p.compareDocumentPosition(p),p.compareDocumentPosition(a),a.compareDocumentPosition(p),
      a.compareDocumentPosition(b),b.compareDocumentPosition(a),p.compareDocumentPosition(x),x.compareDocumentPosition(p),
      x.compareDocumentPosition(y),y.compareDocumentPosition(x),x.compareDocumentPosition(a),a.compareDocumentPosition(x)];
    p.insertBefore(b,a);
    return [before,a.compareDocumentPosition(b),Object.getOwnPropertyDescriptor(Node.prototype,'compareDocumentPosition').enumerable];
  })()`), [[0,20,10,4,2,20,10,36,34,4,2],2,true]);
});

test('compareDocumentPosition validates Nodes and gives stable reciprocal disconnected order', async () => {
  assert.deepEqual(await evaluate(`(() => {
    const a=document.createElement('div'),b=document.createElement('div'),shadow=a.attachShadow({mode:'open'});
    const attr=document.createAttribute('title'),errors=[];
    for(const args of [[],[null],[{}]]) {try{a.compareDocumentPosition(...args);errors.push('none');}catch(e){errors.push(e.name);}}
    const orders=[[a,b],[a,shadow],[a,attr]].map(([left,right])=>{
      const forward=left.compareDocumentPosition(right),back=right.compareDocumentPosition(left);
      return [(forward&33)===33,(forward&6)===2||(forward&6)===4,(forward^back)===6,
        left.compareDocumentPosition(right)===forward];
    });return [errors,orders];
  })()`), [Array(3).fill('TypeError'),Array.from({length:3},()=>[true,true,true,true])]);
});
