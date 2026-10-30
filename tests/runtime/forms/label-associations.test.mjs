import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {createSession} from '../../../scripts/coverage/session.mjs';
import {labelAssociationCases as cases,measureLabelAssociations as measure} from '../../../scripts/coverage/label-association-cases.mjs';
async function evaluate(source){const session=await createSession();try{return await(await session.newPage()).evaluate(source);}finally{await session.close();}}
test('label associations and live lists match 588 measured Safari sequences',async()=>{
  const ref=JSON.parse(fs.readFileSync(new URL('../fixtures/label-associations.safari.json',import.meta.url)));
  assert.deepEqual(ref.cases.map(({expected,...c})=>c),cases);
  // Each context is independent. Bound each evaluation so slower CI hosts do
  // not spend the session's entire execution deadline in one 588-case call.
  const actual=[];
  for(let offset=0;offset<cases.length;offset+=84) {
    const batch=cases.slice(offset,offset+84);
    actual.push(...JSON.parse(await evaluate(`JSON.stringify((${measure})(${JSON.stringify(batch)}))`)));
  }
  assert.deepEqual(cases.flatMap((c,i)=>JSON.stringify(actual[i])===JSON.stringify(ref.cases[i].expected)?[]:[{...c,actual:actual[i],expected:ref.cases[i].expected}]),[]);
});

test('label properties use canonical attributes, tree state and readonly Web IDL descriptors',async()=>{
  assert.deepEqual(await evaluate(`(() => {
    const failures=[],host=document.createElement('div');document.body.append(host);
    host.innerHTML='<form><label for=target></label><input id=target></form>';
    const form=host.firstChild,label=form.firstChild,input=form.lastChild;
    for(const [prototype,name,receiver] of [[HTMLLabelElement.prototype,'control',label],[HTMLLabelElement.prototype,'form',label],
      ...[HTMLInputElement,HTMLButtonElement,HTMLSelectElement,HTMLTextAreaElement,HTMLOutputElement,HTMLMeterElement,HTMLProgressElement].map(c=>[c.prototype,'labels',c===HTMLInputElement?input:document.createElement(c.name.slice(4,-7).toLowerCase())])]) {
      const d=Object.getOwnPropertyDescriptor(prototype,name);
      if(!d.enumerable||!d.configurable||d.set!==undefined||d.get.name!=='get '+name)failures.push([prototype.constructor.name,name,'descriptor']);
      try{d.get.call({});failures.push([name,'brand']);}catch(e){if(e.name!=='TypeError')failures.push(e.name);}
    }
    for(const element of [label,input,host,form])for(const name of ['getAttribute','hasAttribute','querySelectorAll','getRootNode','closest'])element[name]=()=>{throw Error('override');};
    Object.defineProperty(label,'htmlFor',{value:'wrong'});Object.defineProperty(input,'id',{value:'wrong'});Object.defineProperty(input,'type',{value:'hidden'});
    Object.defineProperty(input,'form',{value:null});Object.defineProperty(label,'control',{value:null});
    const control=Object.getOwnPropertyDescriptor(HTMLLabelElement.prototype,'control').get;
    if(control.call(label)!==input||label.form!==form||input.labels[0]!==label)failures.push('canonical');
    host.remove();return failures;
  })()`),[]);
});

test('Safari label-list invalidation distinguishes element, text, ID and for/type mutations',async()=>{
  assert.deepEqual(await evaluate(`(() => {
    const failures=[];
    for(const action of ['id','for-same','for-change','type','name','title','text','append','foreign-for','foreign-type','other-type','other-document']) {
      const host=document.createElement('div');host.innerHTML='<label for=i></label><input id=i>';document.body.append(host);
      const [label,input]=host.children,list=input.labels;if(list.length!==1)failures.push([action,'initial']);input.id='new';
      if(list.length!==1||label.control!==null)failures.push([action,'id cache']);
      if(action==='id')label.id='new-label';if(action==='for-same')label.htmlFor='i';if(action==='for-change')label.htmlFor='missing';
      if(action==='type')input.type='text';if(action==='name')input.name='n';if(action==='title')input.title='x';
      if(action==='text')label.textContent='text';if(action==='append')host.append(document.createElement('span'));
      if(action==='foreign-for')label.setAttributeNS('urn:x','for','foreign');if(action==='foreign-type')input.setAttributeNS('urn:x','type','hidden');
      if(action==='other-type')document.createElement('div').setAttribute('type','x');
      if(action==='other-document'){const doc=document.implementation.createHTMLDocument();doc.body.append(doc.createElement('div'));}
      const expected=['for-change','type','append','other-type'].includes(action)?0:1;
      if(list.length!==expected)failures.push([action,list.length]);host.remove();
    }
    return failures;
  })()`),[]);
});

test('form-associated custom definitions participate in label association without public form properties',async()=>{
  assert.deepEqual(await evaluate(`(() => {
    const failures=[],form=document.createElement('form');document.body.append(form);
    form.innerHTML='<label><x-label-upgrade></x-label-upgrade><input></label>';
    const label=form.firstChild,custom=label.firstChild,input=label.lastChild,list=input.labels;
    if(label.control!==input||list.length!==1)failures.push('undefined');
    class Control extends HTMLElement{static formAssociated=true;}
    customElements.define('x-label-upgrade',Control);
    if(label.control!==custom||label.form!==form||list.length!==1)failures.push('upgrade');
    Control.formAssociated=false;Object.defineProperty(custom,'form',{value:null});
    if(label.control!==custom||label.form!==form)failures.push('definition snapshot');
    label.append(document.createElement('span'));if(list.length!==0)failures.push('list invalidation');
    custom.id='custom';label.htmlFor='custom';if(label.control!==custom||label.form!==form)failures.push('explicit');
    class Plain extends HTMLElement{};customElements.define('x-label-plain',Plain);const plain=document.createElement('x-label-plain');
    label.removeAttribute('for');label.prepend(plain);if(label.control!==custom)failures.push('plain');
    form.remove();return failures;
  })()`),[]);
});

test('htmlFor is canonical DOMString reflection and preserves UTF-16, namespaces and mutation records',async()=>{
  assert.deepEqual(await evaluate(`(() => {
    const failures=[],label=document.createElement('label'),d=Object.getOwnPropertyDescriptor(HTMLLabelElement.prototype,'htmlFor');
    if(!d.enumerable||!d.configurable||d.get.name!=='get htmlFor'||d.set.name!=='set htmlFor')failures.push('descriptor');
    for(const fn of [d.get,d.set])try{fn.call({},'x');failures.push('brand');}catch(e){if(e.name!=='TypeError')failures.push(e.name);}
    for(const value of [null,undefined,1n,false,{},' spaced ',String.fromCharCode(0xd800)]){label.htmlFor=value;if(label.htmlFor!==String(value)||label.getAttributeNS(null,'for')!==String(value))failures.push('conversion');}
    try{label.htmlFor=Symbol();failures.push('symbol');}catch(e){if(e.name!=='TypeError')failures.push(e.name);}
    label.removeAttribute('for');label.setAttributeNS('urn:x','for','foreign');if(label.htmlFor!=='')failures.push('namespace');
    const observer=new MutationObserver(()=>{});observer.observe(label,{attributes:true,attributeOldValue:true});label.htmlFor='same';label.htmlFor='same';
    if(JSON.stringify(observer.takeRecords().map(r=>[r.attributeName,r.attributeNamespace,r.oldValue]))!==JSON.stringify([['for',null,null],['for',null,'same']]))failures.push('records');
    label.getAttribute=label.setAttribute=()=>{throw Error('override');};label.htmlFor='canonical';if(label.htmlFor!=='canonical'||label.getAttributeNS('urn:x','for')!=='foreign')failures.push('canonical');
    const host=document.createElement('div'),control=document.createElement('input');control.id=String.fromCharCode(0xd800);label.htmlFor=control.id;host.append(label,control);document.body.append(host);
    if(label.control!==control)failures.push('surrogate lookup');host.remove();return failures;
  })()`),[]);
});

test('explicit labels use the first exact canonical ID and do not cross shadow boundaries',async()=>{
  assert.deepEqual(await evaluate(`(() => {
    const failures=[];
    for(const id of ['', ' spaced ', 'Case',String.fromCharCode(0xd800),String.fromCharCode(0),String.fromCharCode(0xfffd)]) {
      const host=document.createElement('div'),label=document.createElement('label'),input=document.createElement('input');
      input.id=id;label.htmlFor=id;host.append(label,input);document.body.append(host);
      if(label.control!==(id===''?null:input))failures.push([id,'lookup']);
      const blocker=document.createElement('div');blocker.id=id;host.prepend(blocker);if(label.control!==null)failures.push([id,'first ID']);
      blocker.remove();input.type='hidden';if(label.control!==null)failures.push([id,'hidden']);host.remove();
    }
    const host=document.createElement('div'),outer=document.createElement('label');outer.htmlFor='shadow-control';document.body.append(outer,host);
    const shadow=host.attachShadow({mode:'open'});shadow.innerHTML='<input id=shadow-control><label for=outer-control></label>';
    const input=document.createElement('input');input.id='outer-control';document.body.append(input);
    if(outer.control!==null||shadow.lastChild.control!==null)failures.push('boundary');
    const label=document.createElement('label');label.htmlFor='shadow-control';shadow.append(label);
    if(label.control!==shadow.firstChild||shadow.firstChild.labels[0]!==label)failures.push('shadow local');
    const foreign=document.createElementNS('urn:foreign','label');foreign.setAttribute('for','shadow-control');shadow.prepend(foreign);
    if(shadow.querySelector('input').labels.length!==1)failures.push('foreign label');
    outer.remove();host.remove();input.remove();return failures;
  })()`),[]);
});
