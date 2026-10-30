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

test('Node mutations return node identities, move children and drain adopted fragments', async () => {
  assert.deepEqual(
    await evaluate(`(() => {
    const p=document.createElement('p'),a=new Text('a'),b=new Text('b'),c=new Text('c');
    const returned=[p.appendChild(a)===a,p.insertBefore(b,a)===b,p.insertBefore(c,null)===c];
    p.appendChild(b);const moved=p.textContent;
    p.insertBefore(a,a);const same=p.textContent;
    const foreign=document.implementation.createHTMLDocument(),f=foreign.createDocumentFragment();
    const x=foreign.createElement('i');x.textContent='x';f.append(x,foreign.createTextNode('y'));
    returned.push(p.replaceChild(f,c)===c,p.removeChild(b)===b);
    return [returned,moved,same,p.textContent,f.childNodes.length,c.parentNode,b.parentNode,
      x.ownerDocument===document,x.firstChild.ownerDocument===document,f.ownerDocument===document,
      ['appendChild','insertBefore','replaceChild','removeChild'].map(n=>Object.getOwnPropertyDescriptor(Node.prototype,n).enumerable)];
  })()`),
    [
      Array(5).fill(true),
      'acb',
      'acb',
      'axy',
      0,
      null,
      null,
      true,
      true,
      false,
      Array(4).fill(true),
    ],
  );
});

test('Node mutation argument and hierarchy errors leave trees unchanged', async () => {
  assert.deepEqual(
    await evaluate(`(() => {
    const p=document.createElement('p'),a=new Text('a'),outside=new Text('outside');p.append(a);
    const attr=document.createAttribute('title'),errors=[];
    const operations=[()=>p.appendChild(),()=>p.appendChild(null),()=>p.insertBefore(new Text('b')),
      ()=>p.replaceChild(new Text('b'),null),()=>p.removeChild({}),()=>p.removeChild(outside),
      ()=>p.insertBefore(new Text('b'),outside),()=>p.replaceChild(new Text('b'),outside),
      ()=>p.appendChild(p),()=>a.appendChild(p),()=>p.appendChild(document),()=>p.appendChild(attr),()=>p.removeChild(attr)];
    for(const operation of operations){try{operation();errors.push('none');}catch(e){errors.push(e.name);}}
    const doc=document.implementation.createHTMLDocument(),fragment=document.createDocumentFragment();
    fragment.append(document.createElement('i'),document.createElement('b'));
    for(const operation of [()=>doc.appendChild(new Text('bad')),()=>doc.appendChild(document.createElement('html')),
      ()=>doc.replaceChild(fragment,doc.documentElement),()=>p.appendChild(doc.doctype),()=>doc.appendChild(doc.doctype)]) {
      try{operation();errors.push('none');}catch(e){errors.push(e.name);}
    }
    return [errors,p.textContent,fragment.childNodes.length,doc.doctype.nextSibling===doc.documentElement];
  })()`),
    [
      [
        ...Array(5).fill('TypeError'),
        ...Array(3).fill('NotFoundError'),
        ...Array(4).fill('HierarchyRequestError'),
        'NotFoundError',
        ...Array(5).fill('HierarchyRequestError'),
      ],
      'a',
      2,
      true,
    ],
  );
});

test('Node replacement and removal expose mutation records and collapse live ranges', async () => {
  assert.deepEqual(
    await evaluate(`(() => {
    const p=document.createElement('p'),a=new Text('a'),b=new Text('bc'),c=new Text('d');p.append(a,b,c);
    const range=document.createRange();range.selectNodeContents(b);
    const boundary=document.createRange();boundary.setStart(p,3);boundary.collapse(true);
    const observer=new MutationObserver(()=>{});observer.observe(p,{childList:true});
    const f=document.createDocumentFragment();f.append(new Text('x'),new Text('y'));
    p.replaceChild(f,b);
    const record=observer.takeRecords()[0];
    const replacement=[p.textContent,record.type,Array.from(record.addedNodes,n=>n.data),record.removedNodes[0]===b,
      record.previousSibling===a,record.nextSibling===c,range.startContainer===p,range.startOffset,range.endOffset,boundary.startOffset];
    p.removeChild(a);const removal=observer.takeRecords()[0];observer.disconnect();
    return [replacement,removal.removedNodes[0]===a,removal.addedNodes.length,removal.previousSibling,
      removal.nextSibling===p.firstChild,range.startOffset,boundary.startOffset];
  })()`),
    [['axyd', 'childList', ['x', 'y'], true, true, true, true, 1, 1, 4], true, 0, null, true, 0, 3],
  );
});

test('Node insertion rejects cycles through shadow hosts', async () => {
  assert.deepEqual(
    await evaluate(`(() => {
    const host=document.createElement('div'),root=host.attachShadow({mode:'open'}),inside=document.createElement('i');
    root.append(inside);const errors=[];
    for(const operation of [()=>root.appendChild(host),()=>inside.insertBefore(host,null)]) {
      try{operation();errors.push('none');}catch(e){errors.push(e.name);}
    }
    return [errors,host.parentNode,root.firstChild===inside];
  })()`),
    [Array(2).fill('HierarchyRequestError'), null, true],
  );
});

test('Node fragment insertion and connected moves deliver custom element lifecycle callbacks', async () => {
  assert.deepEqual(
    await evaluate(`(() => {
    const events=[];customElements.define('mutation-item',class extends HTMLElement {
      connectedCallback(){events.push('connected');}
      disconnectedCallback(){events.push('disconnected');}
    });
    const left=document.createElement('div'),right=document.createElement('div');document.body.append(left,right);
    const fragment=document.createDocumentFragment(),item=document.createElement('mutation-item');fragment.appendChild(item);
    left.appendChild(fragment);right.insertBefore(item,null);right.removeChild(item);
    left.remove();right.remove();return [events,fragment.childNodes.length,item.parentNode];
  })()`),
    [['connected', 'disconnected', 'connected', 'disconnected'], 0, null],
  );
});
