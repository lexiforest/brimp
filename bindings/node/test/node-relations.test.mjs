import test from 'node:test';
import assert from 'node:assert/strict';
import {createSession} from '../../../scripts/coverage/session.mjs';

async function evaluate(source) {
  const session = await createSession();
  try { return await (await session.newPage()).evaluate(source); }
  finally { await session.close(); }
}

test('Node identity and containment distinguish descendants, attributes and shadow trees', async () => {
  assert.deepEqual(await evaluate(`(() => {
    const parent = document.createElement('div'), child = new Text('child'); parent.append(child);
    parent.setAttribute('title','value'); const attr=parent.getAttributeNode('title');
    const shadow=parent.attachShadow({mode:'open'}), shadowChild=new Text('shadow'); shadow.append(shadowChild);
    const ordinary=[parent.isSameNode(parent),parent.isSameNode(parent.cloneNode(true)),parent.isSameNode(null),
      parent.contains(parent),parent.contains(child),child.contains(parent),parent.contains(null),
      parent.contains(attr),attr.contains(attr),attr.contains(child),parent.contains(shadowChild),shadow.contains(shadowChild)];
    child.remove(); return [ordinary,parent.contains(child),child.isSameNode(child)];
  })()`), [[true,false,false,true,true,false,false,false,true,false,false,true],false,true]);
});

test('Node relation methods reject invalid arguments and compare node content', async () => {
  assert.deepEqual(await evaluate(`(() => {
    const element=document.createElement('div'),errors=[];
    for(const name of ['contains','isSameNode','isEqualNode']) for(const args of [[],[{}]]) {
      try { element[name](...args); errors.push('none'); } catch(e) { errors.push(e.name); }
    }
    element.innerHTML='<b title="one">text</b><!--comment-->';
    const clone=element.cloneNode(true),equal=element.isEqualNode(clone);
    clone.firstChild.setAttribute('title','two'); const changed=element.isEqualNode(clone);
    const a=document.createElement('i'),b=document.createElement('i');
    a.setAttribute('x','1');a.setAttribute('y','2');b.setAttribute('y','2');b.setAttribute('x','1');
    const attr=document.createAttributeNS('urn:test','p:value');attr.value='same';const attrCopy=attr.cloneNode();
    const attrEqual=attr.isEqualNode(attrCopy);attrCopy.value='different';
    return [errors,equal,changed,a.isEqualNode(b),new Text('a').isEqualNode(new Text('a')),
      new Text('a').isEqualNode(document.createComment('a')),attrEqual,attr.isEqualNode(attrCopy),element.isEqualNode(null)];
  })()`), [Array(6).fill('TypeError'),true,false,true,true,false,true,false,false]);
});

test('Node baseURI follows the first HTML base element and live document ownership', async () => {
  assert.deepEqual(await evaluate(`(() => {
    const doc=document.implementation.createHTMLDocument('base');
    const base=doc.createElement('base');base.href='https://example.test/one/';doc.head.append(base);
    const second=doc.createElement('base');second.href='https://example.test/two/';doc.head.append(second);
    const element=doc.createElement('p');element.textContent='text';element.setAttribute('title','a');doc.body.append(element);
    const attr=element.getAttributeNode('title'),text=element.firstChild;
    const first=[doc.baseURI,element.baseURI,text.baseURI,attr.baseURI,base.baseURI];
    base.href='https://example.test/changed/';const changed=text.baseURI;
    base.remove();const next=element.baseURI;element.remove();const detached=element.baseURI;
    const other=document.implementation.createHTMLDocument('other');
    other.head.innerHTML='<base href="https://other.test/root/">';other.adoptNode(element);
    return [first,changed,next,detached,element.baseURI,text.baseURI,attr.baseURI];
  })()`), [Array(5).fill('https://example.test/one/'),'https://example.test/changed/',
    'https://example.test/two/','https://example.test/two/',...Array(3).fill('https://other.test/root/')]);
});

test('Node baseURI ignores base elements in templates and shadow trees and falls back for invalid bases', async () => {
  assert.deepEqual(await evaluate(`(() => {
    const doc=document.implementation.createHTMLDocument('base');const original=doc.baseURI;
    const template=doc.createElement('template');template.innerHTML='<base href="https://ignored.test/">';doc.head.append(template);
    const host=doc.createElement('div');doc.body.append(host);host.attachShadow({mode:'open'}).innerHTML='<base href="https://shadow.test/">';
    const ignored=doc.baseURI===original;
    const base=doc.createElement('base');base.href='https://example.test/root/';doc.head.append(base);
    const relative=doc.createElement('base');relative.setAttribute('href','child/');doc.head.prepend(relative);
    const relativeFallback=doc.baseURI===original;
    relative.setAttribute('href','http://[');const invalid=doc.baseURI===original;
    relative.remove();const restored=doc.baseURI;
    return [ignored,relativeFallback,invalid,restored];
  })()`), [true,true,true,'https://example.test/root/']);
});
