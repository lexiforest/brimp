import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {createSession} from '../../../scripts/coverage/session.mjs';
import {inputCheckedCases,measureInputChecked} from '../../../scripts/coverage/input-checked-cases.mjs';
import {inputActivationCases,measureInputActivation} from '../../../scripts/coverage/input-activation-cases.mjs';
import {inputRadioGroupCases,measureInputRadioGroups} from '../../../scripts/coverage/input-radio-groups.mjs';
async function evaluate(source){const session=await createSession();try{return await(await session.newPage()).evaluate(source);}finally{await session.close();}}
for(const [name,cases,measure] of [['input-checked',inputCheckedCases,measureInputChecked],['input-activation',inputActivationCases,measureInputActivation],['input-radio-groups',inputRadioGroupCases,measureInputRadioGroups]]) {
  test(`${name}: checked state and activation match measured Safari sequences`,async()=>{
    const reference=JSON.parse(fs.readFileSync(new URL(`./fixtures/${name}.safari.json`,import.meta.url)));
    assert.deepEqual(reference.cases.map(({expected,...c})=>c),cases);
    const actual=JSON.parse(await evaluate(`JSON.stringify((${measure})(${JSON.stringify(cases)}))`));
    assert.deepEqual(cases.flatMap((c,i)=>JSON.stringify(actual[i])===JSON.stringify(reference.cases[i].expected)?[]:
      [{...c,actual:actual[i],expected:reference.cases[i].expected}]),[]);
  });
}

test('checked and defaultChecked have independent dirty state, canonical attributes and Web IDL descriptors',async()=>{
  assert.deepEqual(await evaluate(`(() => {
    const failures=[];
    for(const name of ['checked','defaultChecked']) {
      const i=document.createElement('input'),d=Object.getOwnPropertyDescriptor(HTMLInputElement.prototype,name);
      if(!d.enumerable||!d.configurable||d.get.name!=='get '+name||d.set.name!=='set '+name)failures.push([name,'descriptor']);
      for(const fn of [d.get,d.set])try{fn.call({},true);failures.push([name,'brand']);}catch(e){if(e.name!=='TypeError')failures.push(e.name);}
      for(const value of [undefined,null,false,0,NaN,'',true,1,1n,'false',{},Symbol()]){i[name]=value;if(i[name]!==Boolean(value))failures.push([name,'boolean']);}
    }
    for(const type of ['text','checkbox','radio','file','image','hidden']) {
      const i=document.createElement('input');i.type=type;
      const observer=new MutationObserver(()=>{});observer.observe(i,{attributes:true,attributeOldValue:true});
      i.checked=true;if(observer.takeRecords().length||i.defaultChecked)failures.push([type,'IDL mutation']);
      i.defaultChecked=true;i.defaultChecked=false;if(!i.checked)failures.push([type,'dirty']);
      if(JSON.stringify(observer.takeRecords().map(r=>[r.attributeName,r.oldValue]))!==JSON.stringify([['checked',null],['checked','']]))failures.push([type,'records']);
      const form=document.createElement('form');form.append(i);form.reset();if(i.checked)failures.push([type,'reset']);
      i.setAttributeNS('urn:foreign','checked','foreign');if(i.defaultChecked||i.checked)failures.push([type,'namespace']);
      i.defaultChecked=true;if(!i.checked||i.getAttributeNS(null,'checked')!==''||i.getAttributeNS('urn:foreign','checked')!=='foreign')failures.push([type,'canonical']);
      i.getAttribute=i.hasAttribute=i.setAttribute=i.removeAttribute=i.getRootNode=()=>{throw Error('override');};
      Object.defineProperty(i,'defaultChecked',{value:false});Object.defineProperty(i,'type',{value:'radio'});
      if(!i.checked)failures.push([type,'override']);i.checked=false;if(i.checked)failures.push([type,'write']);
    }
    return failures;
  })()`),[]);
});

test('cloning and importing preserve checkedness and dirty state independently',async()=>{
  assert.deepEqual(await evaluate(`(() => {
    const failures=[];
    for(const type of ['checkbox','radio','text'])for(const checked of [false,true])for(const dirty of [false,true]) {
      const i=document.createElement('input');i.type=type;i.defaultChecked=checked;if(dirty)i.checked=!checked;
      for(const c of [i.cloneNode(),document.importNode(i),document.implementation.createHTMLDocument().importNode(i)]) {
        if(c.checked!==(dirty?!checked:checked)||c.defaultChecked!==checked)failures.push([type,checked,dirty,'copy']);
        c.defaultChecked=!checked;c.defaultChecked=checked;
        if(c.checked!==(dirty?!checked:checked))failures.push([type,checked,dirty,'dirty']);
        c.checked=!c.checked;if(i.checked!==(dirty?!checked:checked))failures.push('independent');
      }
    }
    const form=document.createElement('form');form.innerHTML='<input type=radio name=g checked><input type=radio name=g checked>';
    const copy=form.cloneNode(true),[a,b]=copy.children;
    a.defaultChecked=false;a.defaultChecked=true;if(a.checked||!b.checked)failures.push('group cloning');
    return failures;
  })()`),[]);
});

test('shadow host connection updates radio form ownership and reset requires a browsing context',async()=>{
  assert.deepEqual(await evaluate(`(() => {
    const failures=[],host=document.createElement('div'),root=host.attachShadow({mode:'open'});
    root.innerHTML='<form id=f><input type=radio name=g checked></form><input type=radio name=g form=f checked>';
    const [a,b]=root.querySelectorAll('input');if(!a.checked||!b.checked)failures.push('detached groups');
    document.body.append(host);if(a.checked||!b.checked||a.form!==b.form)failures.push('connected groups');
    host.remove();a.checked=true;if(!a.checked||!b.checked||a.form===b.form)failures.push('disconnected groups');
    const frame=document.createElement('iframe');document.body.append(frame);
    for(const [doc,active] of [[document,true],[frame.contentDocument,true],[document.implementation.createHTMLDocument(),false],
      [new DOMParser().parseFromString('','text/html'),false],[document.createElement('template').content.ownerDocument,false]]) {
      const form=doc.createElement('form'),i=doc.createElement('input');i.type='checkbox';i.checked=true;form.append(i);
      let fired=false;form.addEventListener('reset',()=>fired=true);form.reset();
      if(fired!==active||i.checked===active)failures.push(['reset',active,fired,i.checked]);
    }
    frame.remove();return failures;
  })()`),[]);
});
