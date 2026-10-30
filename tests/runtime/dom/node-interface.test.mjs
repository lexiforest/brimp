import test from 'node:test';
import assert from 'node:assert/strict';
import { createSession } from '../../../scripts/coverage/session.mjs';

async function evaluate(source) {
  const session = await createSession();
  try {
    return await (await session.newPage()).evaluate(source);
  } finally {
    await session.close();
  }
}

test('Node exposes standard immutable constants and rejects direct construction', async () => {
  assert.deepEqual(
    await evaluate(`(() => {
    const constants = {ELEMENT_NODE:1, ATTRIBUTE_NODE:2, TEXT_NODE:3, CDATA_SECTION_NODE:4,
      ENTITY_REFERENCE_NODE:5, ENTITY_NODE:6, PROCESSING_INSTRUCTION_NODE:7, COMMENT_NODE:8,
      DOCUMENT_NODE:9, DOCUMENT_TYPE_NODE:10, DOCUMENT_FRAGMENT_NODE:11, NOTATION_NODE:12,
      DOCUMENT_POSITION_DISCONNECTED:1, DOCUMENT_POSITION_PRECEDING:2, DOCUMENT_POSITION_FOLLOWING:4,
      DOCUMENT_POSITION_CONTAINS:8, DOCUMENT_POSITION_CONTAINED_BY:16, DOCUMENT_POSITION_IMPLEMENTATION_SPECIFIC:32};
    const failures = [];
    for (const [name,value] of Object.entries(constants)) for (const target of [Node,Node.prototype]) {
      const d = Object.getOwnPropertyDescriptor(target,name);
      if (d.value !== value || !d.enumerable || d.writable || d.configurable) failures.push(name);
    }
    const errors = [];
    for (const operation of [() => new Node(), () => Node(), () => new (class extends Node {})()]) {
      try { operation(); errors.push('none'); } catch (e) { errors.push(e.name); }
    }
    return [failures, errors, Node.prototype.constructor === Node,
      Object.getPrototypeOf(Node.prototype) === EventTarget.prototype, Node.prototype[Symbol.toStringTag]];
  })()`),
    [[], Array(3).fill('TypeError'), true, true, 'Node'],
  );
});

test('Node kinds, names and owner documents describe ordinary DOM nodes', async () => {
  assert.deepEqual(
    await evaluate(`(() => {
    const doc = new DOMParser().parseFromString('<!doctype html><p>text</p>', 'text/html');
    const nodes = [doc, doc.doctype, doc.documentElement, doc.querySelector('p').firstChild,
      doc.createComment('comment'), doc.createDocumentFragment(), doc.createAttribute('title')];
    return nodes.map(n => [n.nodeType,n.nodeName,n.ownerDocument === (n === doc ? null : doc)]);
  })()`),
    [
      [9, '#document', true],
      [10, 'html', true],
      [1, 'HTML', true],
      [3, '#text', true],
      [8, '#comment', true],
      [11, '#document-fragment', true],
      [2, 'title', true],
    ],
  );
});

test('Node child and sibling reads follow insertion, removal and adoption', async () => {
  assert.deepEqual(
    await evaluate(`(() => {
    const parent = document.createElement('div'), a = new Text('a'), b = document.createElement('b'), c = document.createComment('c');
    parent.append(a,b,c); const list = parent.childNodes;
    const before = [parent.hasChildNodes(), parent.firstChild===a,parent.lastChild===c,
      a.parentNode===parent,a.parentElement===parent,a.previousSibling,a.nextSibling===b,
      b.previousSibling===a,b.nextSibling===c,c.nextSibling,list===parent.childNodes];
    b.remove(); const after = [a.nextSibling===c,c.previousSibling===a,b.parentNode,b.parentElement,list.length];
    const other = document.implementation.createHTMLDocument('other'); other.adoptNode(parent);
    return [before,after,parent.ownerDocument===other,a.ownerDocument===other,
      document.documentElement.parentNode===document,document.documentElement.parentElement,
      document.createDocumentFragment().hasChildNodes()];
  })()`),
    [
      [true, true, true, true, true, null, true, true, true, null, true],
      [true, true, null, null, 2],
      true,
      true,
      true,
      null,
      false,
    ],
  );
});

test('Node connection and roots respect document, fragment and shadow boundaries', async () => {
  assert.deepEqual(
    await evaluate(`(() => {
    const host = document.createElement('div'), child = new Text('shadow'), root = host.attachShadow({mode:'open'});
    root.append(child);
    const detached = [host.isConnected,root.isConnected,child.isConnected,child.getRootNode()===root,
      child.getRootNode({composed:true})===host,child.getRootNode(null)===root];
    document.body.append(host);
    const attached = [host.isConnected,root.isConnected,child.isConnected,child.getRootNode()===root,
      child.getRootNode({composed:true})===document];
    const fragment = document.createDocumentFragment(); fragment.append(host);
    const moved = [child.isConnected,child.getRootNode({composed:true})===fragment];
    const metadata = ['nodeType','nodeName','ownerDocument','parentNode','parentElement','firstChild',
      'lastChild','previousSibling','nextSibling','childNodes','isConnected'];
    const readonly = metadata.every(name => { const d=Object.getOwnPropertyDescriptor(Node.prototype,name); return d.enumerable&&d.configurable&&typeof d.get==='function'&&d.set===undefined; });
    return [detached,attached,moved,readonly,document.isConnected,document.getRootNode()===document];
  })()`),
    [
      [false, false, false, true, true, true],
      [true, true, true, true, true],
      [false, true],
      true,
      true,
      true,
    ],
  );
});
