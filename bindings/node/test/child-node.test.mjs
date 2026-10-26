import test from 'node:test';
import assert from 'node:assert/strict';
import {createSession} from '../../../scripts/coverage/session.mjs';

async function evaluate(source) {
  const session = await createSession();
  try { return await (await session.newPage()).evaluate(source); }
  finally { await session.close(); }
}

test('ChildNode inserts, moves, replaces and removes elements, text and comments', async () => {
  assert.deepEqual(await evaluate(`(() => {
    const failures=[];
    for(const kind of ['element','text','comment']) {
      const make=()=>kind==='element'?document.createElement('i'):kind==='text'?document.createTextNode('x'):document.createComment('x');
      for(const method of ['before','after','replaceWith']) {
        const p=document.createElement('div'), n=make(); p.appendChild(n);
        const f=document.createDocumentFragment(); f.appendChild(document.createTextNode('F'));
        const result=n[method]('A',f,'B');
        const expected=method==='before'?['A','F','B',n]:method==='after'?[n,'A','F','B']:['A','F','B'];
        const actual=Array.from(p.childNodes);
        if(result!==undefined || f.childNodes.length || actual.length!==expected.length || actual.some((v,i)=>typeof expected[i]==='string'?v.data!==expected[i]:v!==expected[i])) failures.push([kind,method,'basic']);
      }
      for(const method of ['before','after','replaceWith']) {
        const p=document.createElement('div'), a=document.createTextNode('A'), n=make(), b=document.createTextNode('B'), c=document.createTextNode('C');
        for(const v of [a,n,b,c])p.appendChild(v);
        n[method](b,n,a,b);
        const expected=[n,a,b,c];
        if(Array.from(p.childNodes).some((v,i)=>v!==expected[i]) || p.childNodes.length!==4)failures.push([kind,method,'moves/duplicates']);
        n[method]();
        if(p.childNodes.length!==(method==='replaceWith'?3:4))failures.push([kind,method,'empty']);
      }
      const p=document.createElement('div'),n=make();p.appendChild(n);
      if(n.remove()!==undefined || n.parentNode!==null || p.childNodes.length)failures.push([kind,'remove']);
      n.remove();
    }
    return failures;
  })()`), []);
});

test('ChildNode converts before method steps and preserves conversion exceptions', async () => {
  assert.deepEqual(await evaluate(`(() => {
    const failures=[];
    for(const method of ['before','after','replaceWith']) for(const attached of [false,true]) {
      const p=document.createElement('div'), n=document.createElement('i');
      if(attached)p.appendChild(n);
      const log=[],sentinel=new Error('sentinel');
      try{n[method]({[Symbol.toPrimitive](hint){log.push(hint);return 'ok';}},Symbol('bad'));failures.push('symbol accepted');}
      catch(e){if(e.name!=='TypeError')failures.push('symbol exception');}
      if(log.join()!=='string' || p.childNodes.length!==(attached?1:0))failures.push('conversion ordering');
      try{n[method]({toString(){throw sentinel;}});failures.push('exception swallowed');}
      catch(e){if(e!==sentinel)failures.push('exception identity');}
      let count=0;
      n[method]({toString(){count++;if(n.parentNode)n.remove();return 'x';}});
      if(count!==1 || p.childNodes.length)failures.push('parent read before conversion');
    }
    const p=document.createElement('div'),n=document.createElement('i');p.appendChild(n);
    n.before(null,undefined,false,42);
    if(Array.from(p.childNodes).slice(0,4).map(n=>n.data).join('|')!=='null|undefined|false|42')failures.push('DOMString values');
    return failures;
  })()`), []);
});

test('ChildNode validates hierarchy, adopts nodes and bypasses overridden mutation methods', async () => {
  assert.deepEqual(await evaluate(`(() => {
    const failures=[];
    for(const method of ['before','after','replaceWith']) {
      const p=document.createElement('div'),n=document.createElement('i');p.appendChild(n);
      for(const invalid of [p,document]) {
        try{n[method](invalid);failures.push('invalid insertion accepted');}
        catch(e){if(e.name!=='HierarchyRequestError')failures.push('wrong hierarchy exception');}
        if(n.parentNode!==p || p.childNodes.length!==1)failures.push('invalid insertion mutated parent');
      }
      const other=document.implementation.createHTMLDocument('other'), foreign=other.createElement('b');
      n[method](foreign,'text');
      if(foreign.ownerDocument!==document || foreign.nextSibling.ownerDocument!==document)failures.push('adoption');
      const anchor=other.createElement('i');other.body.appendChild(anchor);anchor[method]('local');
      if(Array.from(other.body.childNodes).some(c=>c.ownerDocument!==other))failures.push('owner document');
    }
    const p=document.createElement('div'),n=document.createElement('i');p.appendChild(n);
    for(const name of ['insertBefore','appendChild','replaceChild','removeChild'])p[name]=()=>{throw Error('overridden '+name);};
    n.before('a');n.after('b');n.replaceWith('c');p.firstChild.remove();
    if(p.textContent!=='cb')failures.push('internal mutation');
    return failures;
  })()`), []);
});

test('remove reads the native parent independently of a shadowed parentNode property',async()=>{
  assert.deepEqual(await evaluate(`(() => {
    const failures=[];
    for(const node of [document.createElement('i'),document.createTextNode('text'),document.createComment('comment')]){
      const parent=document.createElement('div');parent.append(node);
      const observer=new MutationObserver(()=>{});observer.observe(parent,{childList:true});
      const range=document.createRange();range.selectNodeContents(node);
      const iterator=document.createNodeIterator(parent);iterator.nextNode();iterator.nextNode();
      Object.defineProperty(node,'parentNode',{get(){throw Error('public parent');}});
      parent.removeChild=()=>{throw Error('public removal');};
      node.remove();node.remove();if(parent.childNodes.length)failures.push(node.nodeName);
      const records=observer.takeRecords();observer.disconnect();
      if(records.length!==1||records[0].removedNodes[0]!==node)failures.push('observer');
      if(range.startContainer!==parent||range.startOffset!==0||range.endContainer!==parent||range.endOffset!==0)failures.push('range');
      if(iterator.referenceNode!==parent||iterator.nextNode()!==null)failures.push('iterator');
    }
    return failures;
  })()`),[]);
});

test('ChildNode Web IDL descriptors and unscopables exclude only standard members', async () => {
  assert.deepEqual(await evaluate(`(() => {
    const failures=[];
    for(const ctor of [Element,CharacterData])for(const name of ['before','after','replaceWith','remove']) {
      const d=Object.getOwnPropertyDescriptor(ctor.prototype,name);
      if(!d?.enumerable || !d.writable || !d.configurable || d.value.name!==name || d.value.length!==0)failures.push(ctor.name+'.'+name+' descriptor');
      for(const receiver of [{},null,document]) {
        try{d.value.call(receiver);failures.push('illegal receiver');}catch(e){if(e.name!=='TypeError')failures.push('receiver exception');}
      }
    }
    for(const [ctor,names] of [
      [Element,['slot','before','after','replaceWith','remove','prepend','append','replaceChildren']],
      [CharacterData,['before','after','replaceWith','remove']],
      [Document,['prepend','append','replaceChildren']],
      [DocumentFragment,['prepend','append','replaceChildren']],
    ]) {
      const d=Object.getOwnPropertyDescriptor(ctor.prototype,Symbol.unscopables);
      if(!d || d.writable || d.enumerable || !d.configurable || Object.getPrototypeOf(d.value)!==null)failures.push(ctor.name+' unscopables descriptor');
      for(const name of names)if(d.value[name]!==true)failures.push(ctor.name+'.'+name+' unscopable');
      if(ctor===Document || ctor===DocumentFragment)for(const name of ['before','after','replaceWith','remove'])if(Object.hasOwn(d.value,name))failures.push('legacy nonstandard entry');
    }
    const resolve=Function('node','with(node) { return typeof before; }');
    if(resolve(document.createElement('i'))!=='undefined')failures.push('with scope');
    if(Object.hasOwn(Text.prototype,'remove'))failures.push('redundant Text.remove');
    return failures;
  })()`), []);
});
