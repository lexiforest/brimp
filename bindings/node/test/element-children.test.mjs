import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {createSession} from '../../../scripts/coverage/session.mjs';
import {elementChildrenCases,measureElementChildren} from '../../../scripts/coverage/element-children-cases.mjs';
const reference=JSON.parse(fs.readFileSync(new URL('./fixtures/element-children.safari.json',import.meta.url)));
async function evaluate(source){const session=await createSession();try{return await(await session.newPage()).evaluate(source)}finally{await session.close()}}

test('ParentNode element-child getters match Safari mixed-node mutation sequences',async()=>{
  assert.deepEqual(reference.map(({expected,...c})=>c),elementChildrenCases);
  const actual=JSON.parse(await evaluate(`JSON.stringify((${measureElementChildren})(${JSON.stringify(elementChildrenCases)}))`));
  assert.deepEqual(reference.flatMap((c,i)=>JSON.stringify(c.expected)===JSON.stringify(actual[i])?[]:[{...c,actual:actual[i]}]),[]);
});

test('ParentNode element-child getters have Safari readonly descriptors and native interface brands',async()=>{
  assert.deepEqual(await evaluate(`(()=>{
    const failures=[],names=['children','childElementCount','firstElementChild','lastElementChild'];
    for(const [constructor,make,wrong] of [[Document,()=>new Document(),()=>document.createElement('div')],[Element,()=>document.createElement('div'),()=>new Document()],[DocumentFragment,()=>document.createDocumentFragment(),()=>document.createTextNode('text')]]){
      const prototype=constructor.prototype,parent=make();
      const child=document.createElement('span');parent.appendChild(child);
      for(const name of names){
        const descriptor=Object.getOwnPropertyDescriptor(prototype,name),get=descriptor.get,before=get.call(parent);
        if(!descriptor.enumerable||!descriptor.configurable||descriptor.set!==undefined||get.name!=='get '+name||get.length!==0)failures.push([constructor.name,name,'descriptor']);
        const forged=wrong();Object.setPrototypeOf(forged,prototype);
        for(const receiver of [null,{},Object.create(prototype),forged,new Proxy(parent,{})])try{get.call(receiver);failures.push([constructor.name,name,'accepted receiver'])}catch(e){if(e.name!=='TypeError')failures.push(e.name)}
        parent[name]=null;if(get.call(parent)!==before||Object.hasOwn(parent,name))failures.push([name,'readonly']);
        try{(function(){'use strict';parent[name]=null})();failures.push([name,'strict'])}catch(e){if(e.name!=='TypeError')failures.push(e.name)}
      }
      const children=parent.children;Object.setPrototypeOf(parent,null);
      for(const name of names){const value=Object.getOwnPropertyDescriptor(prototype,name).get.call(parent);if(value!==(name==='children'?children:name==='childElementCount'?1:child))failures.push([constructor.name,name,'native identity'])}
    }
    return failures;
  })()`),[]);
});

test('Element-child selection uses native child identity and ignores public parent and collection properties',async()=>{
  assert.deepEqual(await evaluate(`(()=>{
    const failures=[],names=['children','childElementCount','firstElementChild','lastElementChild'];
    for(const [prototype,parent] of [[Document.prototype,new Document()],[Element.prototype,document.createElement('div')],[DocumentFragment.prototype,document.createDocumentFragment()]]){
      const list=parent.children,child=document.createElementNS('urn:child','node');parent.appendChild(child);
      Object.setPrototypeOf(child,null);
      for(const key of ['nodeType','namespaceURI','localName'])Object.defineProperty(child,key,{get(){throw Error('child '+key)}});
      for(const key of ['childNodes','children','firstChild','lastChild','querySelectorAll'])Object.defineProperty(parent,key,{get(){throw Error('parent '+key)}});
      for(const key of ['length','item'])Object.defineProperty(list,key,{get(){throw Error('collection '+key)}});
      for(const name of names){const result=Object.getOwnPropertyDescriptor(prototype,name).get.call(parent);if(result!==(name==='children'?list:name==='childElementCount'?1:child))failures.push(name)}
      if(list[0]!==child||HTMLCollection.prototype.item.call(list,0)!==child||Object.getOwnPropertyDescriptor(HTMLCollection.prototype,'length').get.call(list)!==1)failures.push('collection');
    }
    return failures;
  })()`),[]);
});

test('Element-child collections remain live through adoption, fragment insertion and document replacement',async()=>{
  assert.deepEqual(await evaluate(`(()=>{
    const failures=[],d=document.implementation.createHTMLDocument(),other=document.implementation.createHTMLDocument(),fragment=d.createDocumentFragment(),a=d.createElement('a'),b=d.createElementNS('urn:b','b');
    a.id='named';fragment.append(d.createTextNode('x'),a,d.createComment('c'),b);
    const source=fragment.children,target=d.body.children;
    d.body.appendChild(fragment);
    if(source.length!==0||target.length!==2||target[0]!==a||target[1]!==b||target.named!==a)failures.push('fragment insertion');
    const cloned=d.body.cloneNode(true);if(cloned.children===target||cloned.children.length!==2||cloned.firstElementChild===a)failures.push('clone');
    other.body.appendChild(other.adoptNode(a));if(target.length!==1||other.body.children[0]!==a||source.length!==0)failures.push('adopt');
    const rootList=d.children,root=d.documentElement;root.remove();if(rootList.length!==0||d.childElementCount!==0||d.firstElementChild!==null||d.lastElementChild!==null)failures.push('empty document');
    d.appendChild(root);if(rootList.length!==1||rootList[0]!==root)failures.push('root reinsertion');
    d.open();if(rootList.length!==0)failures.push('open');d.write('<!doctype html><html><body><main></main></body></html>');d.close();
    if(rootList!==d.children||rootList.length!==1||rootList[0]===root||d.childElementCount!==1)failures.push('write');
    const type=document.implementation.createHTMLDocument().doctype.cloneNode(),empty=new Document();empty.appendChild(type);empty.appendChild(empty.createComment('only comment'));
    if(empty.children.length!==0||empty.childElementCount!==0||empty.firstElementChild!==null||empty.lastElementChild!==null)failures.push('doctype/comment');
    return failures;
  })()`),[]);
});

test('Element-child getters keep parsed template content and shadow trees separate from ordinary children',async()=>{
  assert.deepEqual(await evaluate(`(()=>{
    const d=document.implementation.createHTMLDocument(),template=d.createElement('template'),host=d.createElement('div'),out=[];
    template.innerHTML='<span></span><!--comment--><b></b>';
    const parsed=template.content.children,direct=template.children;
    out.push(direct.length,parsed.length,template.firstElementChild===null);
    const child=d.createElement('em');template.appendChild(child);out.push(direct.length,parsed.length,template.firstElementChild===child);
    const shadow=host.attachShadow({mode:'open'}),light=d.createElement('a'),slot=d.createElement('slot');host.appendChild(light);shadow.appendChild(slot);
    out.push(host.children.length,host.firstElementChild===light,shadow.children.length,shadow.firstElementChild===slot);
    light.slot='missing';out.push(host.children.length,shadow.children.length);
    return out;
  })()`),[0,2,true,1,2,true,1,true,1,true,1,1]);
});
