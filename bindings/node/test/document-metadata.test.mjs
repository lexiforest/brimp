import test from 'node:test';
import assert from 'node:assert/strict';
import {createSession} from '../../../scripts/coverage/session.mjs';

async function evaluate(source) {
  const session = await createSession();
  try { return await (await session.newPage()).evaluate(source); }
  finally { await session.close(); }
}

test('parsed doctypes expose canonical names and identifiers through stable DocumentType nodes', async () => {
  assert.deepEqual(await evaluate(`(() => {
    const cases=[
      ['',null], ['<div>x</div>',null], ['<!--x--><!DoCtYpE HTML>', ['html','','']],
      ['<!doctype potato>', ['potato','','']], ['<!doctype>', ['','','']],
      ['<!doctype html PUBLIC "public &amp; ID" "system ID">',['html','public &amp; ID','system ID']],
      ['<!doctype html SYSTEM "about:legacy-compat">',['html','','about:legacy-compat']],
      ['<!doctype html><!doctype other>',['html','','']], ['<p>x<!doctype html>',null],
    ];
    const failures=[];
    for(const [html,expected] of cases) {
      const d=new DOMParser().parseFromString(html,'text/html'),t=d.doctype;
      if(expected===null){if(t!==null)failures.push([html,'absence']);continue;}
      if(JSON.stringify([t.name,t.publicId,t.systemId])!==JSON.stringify(expected))failures.push([html,'identifiers']);
      if(t.nodeType!==10 || t.nodeName!==t.name || t.nodeValue!==null || t.textContent!==null)failures.push([html,'node metadata']);
      if(!(t instanceof DocumentType) || !(t instanceof Node) || Object.getPrototypeOf(t)!==DocumentType.prototype || Object.prototype.toString.call(t)!=='[object DocumentType]')failures.push([html,'prototype']);
      if(d.doctype!==t || ![...d.childNodes].includes(t) || t.ownerDocument!==d || t.parentNode!==d || t.parentElement!==null || !t.isConnected)failures.push([html,'identity/ownership']);
      t.nodeValue='ignored';t.textContent='ignored';
      try{t.textContent=Symbol('text');failures.push([html,'symbol conversion']);}catch(e){if(e.name!=='TypeError')failures.push([html,e.name]);}
      if(t.nodeValue!==null || t.textContent!==null || t.name!==expected[0] || t.childNodes.length)failures.push([html,'null node data']);
      const clone=t.cloneNode(true);
      if(clone===t || clone.parentNode!==null || clone.ownerDocument!==d || !(clone instanceof DocumentType) || JSON.stringify([clone.name,clone.publicId,clone.systemId])!==JSON.stringify(expected))failures.push([html,'clone']);
    }
    return failures;
  })()`), []);
});

test('document doctype stays live across removal, replacement, adoption and import', async () => {
  assert.deepEqual(await evaluate(`(() => {
    const a=new DOMParser().parseFromString('<!doctype html PUBLIC "A" "B">','text/html');
    const b=new DOMParser().parseFromString('<!doctype other>','text/html');
    const t=a.doctype, failures=[];
    for(const deep of [false,true]) {
      try{b.importNode(t,deep);failures.push('import accepted');}catch(e){if(e.name!=='NotSupportedError')failures.push(e.name);}
    }
    const imported=t.cloneNode(true);
    t.before(a.createComment('before'));t.after(a.createComment('after'));
    if(t.previousSibling.data!=='before' || t.nextSibling.data!=='after')failures.push('siblings');
    t.remove();t.remove();
    if(a.doctype!==null || t.parentNode!==null || t.ownerDocument!==a || t.isConnected)failures.push('remove');
    a.insertBefore(t,a.documentElement);
    if(a.doctype!==t)failures.push('reinsert');
    b.doctype.replaceWith(imported);
    if(b.doctype!==imported)failures.push('replace');
    imported.remove();b.adoptNode(t);b.insertBefore(t,b.documentElement);
    if(a.doctype!==null || b.doctype!==t || t.ownerDocument!==b || t.publicId!=='A')failures.push('adopt');
    const cloned=b.cloneNode(true);
    if(cloned.doctype===t || cloned.doctype.ownerDocument!==cloned || cloned.doctype.systemId!=='B')failures.push('document clone');
    for(const mutate of [()=>b.body.appendChild(t),()=>b.appendChild(t.cloneNode()),()=>t.after(t.cloneNode()),()=>t.before('text')]) {
      try{mutate();failures.push('missing hierarchy rejection');}catch(e){if(e.name!=='HierarchyRequestError')failures.push(e.name);}
    }
    return failures;
  })()`), []);
});

test('DocumentType shape and readonly document metadata match Safari', async () => {
  assert.deepEqual(await evaluate(`(() => {
    const failures=[];
    try{new DocumentType();failures.push('constructor');}catch(e){if(e.name!=='TypeError')failures.push(e.name);}
    for(const [C,names] of [[Document,['doctype','compatMode']],[DocumentType,['name','publicId','systemId']]]) {
      for(const name of names) {
        const d=Object.getOwnPropertyDescriptor(C.prototype,name);
        if(!d.enumerable || !d.configurable || d.set!==undefined || d.get.name!=='get '+name || d.get.length!==0)failures.push([C.name,name,'descriptor']);
        try{d.get.call({});failures.push([name,'brand']);}catch(e){if(e.name!=='TypeError')failures.push([name,e.name]);}
      }
    }
    const u=DocumentType.prototype[Symbol.unscopables];
    if(Object.getPrototypeOf(u)!==null || Object.keys(u).sort().join()!=='after,before,remove,replaceWith')failures.push('unscopables');
    for(const name of ['before','after','replaceWith','remove']) {
      const d=Object.getOwnPropertyDescriptor(DocumentType.prototype,name);
      if(!d.enumerable || !d.writable || !d.configurable || d.value.name!==name || d.value.length!==0 || u[name]!==true)failures.push([name,'method']);
    }
    return failures;
  })()`), []);
});

test('full HTML document parsing selects and retains quirks, limited-quirks and standards modes', async () => {
  assert.deepEqual(await evaluate(`(() => {
    const cases=[
      ['<html><body>x</body></html>','BackCompat'],
      ['<!--x--><html><body>x</body></html>','BackCompat'],
      ['<!doctype html>','CSS1Compat'], ['<!--x--><!doctype html>','CSS1Compat'],
      ['<?xml x?><!doctype html>','CSS1Compat'], ['<!doctype potato>','BackCompat'],
      ['<!doctype html PUBLIC "-//W3C//DTD HTML 4.01 Transitional//EN">','BackCompat'],
      ['<!doctype html PUBLIC "-//W3C//DTD HTML 4.01 Transitional//EN" "http://www.w3.org/TR/html4/loose.dtd">','CSS1Compat'],
      ['<!doctype html SYSTEM "about:legacy-compat">','CSS1Compat'],
    ];
    const failures=[];
    for(const [html,mode] of cases) {
      const d=new DOMParser().parseFromString(html,'text/html');
      if(d.compatMode!==mode)failures.push([html,d.compatMode]);
      for(const deep of [false,true])if(d.cloneNode(deep).compatMode!==mode)failures.push([html,'clone',deep]);
      d.doctype?.remove();
      if(d.compatMode!==mode)failures.push([html,'changed after mutation']);
    }
    return failures;
  })()`), []);
});

test('document.write resets parser mode and exposes the replacement doctype', async () => {
  assert.deepEqual(await evaluate(`(() => {
    const d=document.implementation.createHTMLDocument(),failure=[];
    const inputs=[['<p>quirks</p>','BackCompat',null],['<!doctype html><p>standards</p>','CSS1Compat','html'],['<!doctype potato><p>quirks</p>','BackCompat','potato'],['<!--x--><!doctype html><p>standards</p>','CSS1Compat','html']];
    for(const [html,mode,name] of inputs) {
      const previous=d.doctype;d.open();
      if(d.doctype!==null)failure.push('open retained doctype');
      d.write(html);d.close();
      if(d.compatMode!==mode || (d.doctype?.name??null)!==name || (previous&&previous.parentNode!==null))failure.push([html,d.compatMode,d.doctype?.name]);
    }
    return failure;
  })()`), []);
});
