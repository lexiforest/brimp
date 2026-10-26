import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {createSession} from '../../../scripts/coverage/session.mjs';
import {elementSiblingCases,measureElementSiblings} from '../../../scripts/coverage/element-sibling-cases.mjs';
const reference=JSON.parse(fs.readFileSync(new URL('./fixtures/element-siblings.safari.json',import.meta.url)));
async function evaluate(source){const session=await createSession();try{return await(await session.newPage()).evaluate(source)}finally{await session.close()}}

test('Element and CharacterData sibling getters match Safari mixed-node mutation and adoption sequences',async()=>{
  assert.deepEqual(reference.map(({expected,...c})=>c),elementSiblingCases);
  const actual=JSON.parse(await evaluate(`JSON.stringify((${measureElementSiblings})(${JSON.stringify(elementSiblingCases)}))`));
  assert.deepEqual(reference.flatMap((c,i)=>JSON.stringify(c.expected)===JSON.stringify(actual[i])?[]:[{...c,actual:actual[i]}]),[]);
});

test('Element-sibling getters have readonly Safari descriptors and canonical native receiver brands',async()=>{
  assert.deepEqual(await evaluate(`(()=>{
    const failures=[],names=['previousElementSibling','nextElementSibling'];
    for(const [constructor,make,wrong] of [[Element,()=>document.createElement('div'),()=>document.createTextNode('text')],[CharacterData,()=>document.createTextNode('text'),()=>document.createElement('div')],[CharacterData,()=>document.createComment('comment'),()=>document.createDocumentFragment()]]){
      const parent=document.createElement('div'),before=document.createElementNS('urn:foreign','before'),node=make(),after=document.createElementNS('http://www.w3.org/2000/svg','svg');parent.append(before,node,after);
      for(const name of names){
        const descriptor=Object.getOwnPropertyDescriptor(constructor.prototype,name),get=descriptor.get,expected=name==='previousElementSibling'?before:after;
        if(!descriptor.enumerable||!descriptor.configurable||descriptor.set!==undefined||get.name!=='get '+name||get.length!==0)failures.push([constructor.name,name,'descriptor']);
        const forged=wrong();Object.setPrototypeOf(forged,constructor.prototype);
        for(const receiver of [null,{},Object.create(constructor.prototype),forged,new Proxy(node,{})])try{get.call(receiver);failures.push([name,'accepted receiver'])}catch(e){if(e.name!=='TypeError')failures.push(e.name)}
        node[name]=null;if(get.call(node)!==expected||Object.hasOwn(node,name))failures.push([name,'readonly']);
        try{(function(){'use strict';node[name]=null})();failures.push([name,'strict'])}catch(e){if(e.name!=='TypeError')failures.push(e.name)}
      }
      Object.setPrototypeOf(node,null);
      if(Object.getOwnPropertyDescriptor(constructor.prototype,names[0]).get.call(node)!==before||Object.getOwnPropertyDescriptor(constructor.prototype,names[1]).get.call(node)!==after)failures.push('native identity');
    }
    return failures;
  })()`),[]);
});

test('Element-sibling selection ignores overridden DOM properties and JavaScript prototypes',async()=>{
  assert.deepEqual(await evaluate(`(()=>{
    const failures=[];
    for(const [prototype,node] of [[Element.prototype,document.createElement('span')],[CharacterData.prototype,document.createTextNode('t')],[CharacterData.prototype,document.createComment('c')]]){
      const parent=document.createElement('div'),before=document.createElement('a'),after=document.createElementNS('urn:other','b'),text=document.createTextNode('skip'),comment=document.createComment('skip');parent.append(before,text,node,comment,after);
      Object.setPrototypeOf(before,null);Object.setPrototypeOf(after,null);Object.setPrototypeOf(text,Element.prototype);Object.setPrototypeOf(comment,Element.prototype);
      for(const object of [parent,before,after,text,comment,node])for(const key of ['parentNode','childNodes','nodeType','previousSibling','nextSibling'])Object.defineProperty(object,key,{get(){throw Error('public '+key)}});
      const previous=Object.getOwnPropertyDescriptor(prototype,'previousElementSibling').get,next=Object.getOwnPropertyDescriptor(prototype,'nextElementSibling').get;
      if(previous.call(node)!==before||next.call(node)!==after)failures.push('canonical neighbors');
    }
    return failures;
  })()`),[]);
});

test('Element-sibling traversal skips doctypes and respects template, shadow and slot boundaries',async()=>{
  assert.deepEqual(await evaluate(`(()=>{
    const d=new Document(),type=document.implementation.createHTMLDocument().doctype.cloneNode(),before=d.createComment('before'),after=d.createComment('after'),root=d.createElementNS('urn:root','root'),out=[];
    d.append(before,type,root,after);
    out.push(before.previousElementSibling===null,before.nextElementSibling===root,after.previousElementSibling===root,after.nextElementSibling===null,root.previousElementSibling===null,root.nextElementSibling===null,!('nextElementSibling' in type));
    const template=document.createElement('template');template.innerHTML='<i></i>text<b></b>';const direct=document.createElement('em');template.appendChild(direct);
    out.push(direct.previousElementSibling===null,direct.nextElementSibling===null,template.content.firstChild.nextElementSibling===template.content.lastChild);
    const host=document.createElement('div'),light=document.createElement('a'),shadow=host.attachShadow({mode:'open'}),slot=document.createElement('slot'),svg=document.createElementNS('http://www.w3.org/2000/svg','svg');host.appendChild(light);shadow.append(slot,document.createTextNode('x'),svg);
    out.push(light.previousElementSibling===null,light.nextElementSibling===null,slot.nextElementSibling===svg,svg.previousElementSibling===slot);
    return out;
  })()`),Array(14).fill(true));
});

test('CharacterData sibling getters stay live through splitText, normalize and detached cloning',async()=>{
  assert.deepEqual(await evaluate(`(()=>{
    const parent=document.createElement('div'),a=document.createElement('a'),b=document.createElement('b'),text=document.createTextNode('ab');parent.append(a,text,b);
    const split=text.splitText(1),out=[text.previousElementSibling===a,text.nextElementSibling===b,split.previousElementSibling===a,split.nextElementSibling===b];
    const clone=text.cloneNode();out.push(clone.previousElementSibling===null,clone.nextElementSibling===null);
    parent.normalize();out.push(text.nextElementSibling===b,split.previousElementSibling===null,split.nextElementSibling===null);
    b.remove();out.push(text.nextElementSibling===null,text.previousElementSibling===a);return out;
  })()`),Array(11).fill(true));
});
