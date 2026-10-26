import test from 'node:test';
import assert from 'node:assert/strict';
import {createSession} from '../../../scripts/coverage/session.mjs';
async function evaluate(source){const s=await createSession();try{return await(await s.newPage()).evaluate(source);}finally{await s.close();}}

test('button strings, booleans and type have Safari conversion and reflection semantics',async()=>{
  assert.deepEqual(await evaluate(`(() => {
    const failures=[],button=document.createElement('button');
    button.textContent='Label';
    for(const name of ['name','value','formTarget','type']){
      const attribute=name.toLowerCase();
      if(button[name] !== (name==='type'?'submit':''))failures.push([name,'default']);
      for(const value of [null,undefined,0,NaN,123n,'','a\\0b','  a\\n\\rb\\t  ','你好💡',String.fromCharCode(0xd800),
        'submit','SUBMIT','reset','button','BUTTON','auto','select','menu',' submit','reset ','ſubmit',{toString(){return 'object';}}]){
        const raw=String(value),lower=raw.replace(/[A-Z]/g,c=>c.toLowerCase());
        const expected=name==='type'?(['submit','reset','button'].includes(lower)?lower:'submit'):raw;
        button[name]=value;
        if(button[name]!==expected||button.getAttribute(attribute)!==raw)failures.push([name,'conversion']);
      }
      let hints=[];button[name]={[Symbol.toPrimitive](hint){hints.push(hint);return 'button';}};
      if(hints.join()!=='string')failures.push([name,'hint']);
      const before=button.getAttribute(attribute);
      for(const value of [Symbol(),{[Symbol.toPrimitive](){throw new RangeError('conversion');}}]){
        try{button[name]=value;failures.push([name,'throw']);}catch(e){if(!['TypeError','RangeError'].includes(e.name))failures.push([name,e.name]);}
        if(button.getAttribute(attribute)!==before)failures.push([name,'failed conversion']);
      }
    }
    for(const name of ['disabled','formNoValidate']){
      const attribute=name.toLowerCase();if(button[name])failures.push([name,'default']);
      for(const value of [null,undefined,false,0,-0,NaN,0n,'',true,1,1n,'false','0',Symbol(),{[Symbol.toPrimitive](){throw Error('not converted');}}]){
        button[name]=value;
        if(button[name]!==Boolean(value)||button.hasAttribute(attribute)!==Boolean(value)||(value&&button.getAttribute(attribute)!==''))failures.push([name,'boolean']);
      }
      button.setAttribute(attribute,'false');if(!button[name])failures.push([name,'presence']);button[name]=false;
    }
    const fieldset=document.createElement('fieldset');fieldset.disabled=true;fieldset.append(button);
    if(button.disabled)failures.push('own disabled only');
    if(button.textContent!=='Label')failures.push('value changed label');
    return failures;
  })()`),[]);
});

test('button reflection descriptors, namespace identity, cloning and canonical access follow Safari',async()=>{
  assert.deepEqual(await evaluate(`(() => {
    const failures=[];
    for(const name of ['name','value','formTarget','type','disabled','formNoValidate']){
      const button=document.createElement('button'),attribute=name.toLowerCase(),boolean=['disabled','formNoValidate'].includes(name);
      const d=Object.getOwnPropertyDescriptor(HTMLButtonElement.prototype,name);
      if(!d.enumerable||!d.configurable||d.get.name!=='get '+name||d.set.name!=='set '+name||d.get.length!==0||d.set.length!==1)failures.push([name,'descriptor']);
      for(const receiver of [{},document.createElement('input'),Object.create(HTMLButtonElement.prototype)]){
        let converted=false;
        try{d.set.call(receiver,{toString(){converted=true;return 'button';}});failures.push([name,'setter receiver']);}catch(e){if(e.name!=='TypeError'||converted)failures.push([name,'brand conversion']);}
        try{d.get.call(receiver);failures.push([name,'getter receiver']);}catch(e){if(e.name!=='TypeError')failures.push([name,e.name]);}
      }
      button.setAttributeNS('urn:foreign',attribute,'foreign');
      const fallback=boolean?false:name==='type'?'submit':'';
      if(button[name]!==fallback)failures.push([name,'namespace']);
      button[name]=boolean?true:'button';
      if(button.getAttributeNS(null,attribute)!==(boolean?'':'button')||button.getAttributeNS('urn:foreign',attribute)!=='foreign')failures.push([name,'identity']);
      const clone=button.cloneNode(),owner=document.implementation.createHTMLDocument(),imported=owner.importNode(button,true);
      button[name]=boolean?false:'reset';for(const node of [clone,imported])if(node[name]!==(boolean?true:'button'))failures.push([name,'clone/import']);
      button.getAttribute=button.setAttribute=button.hasAttribute=button.removeAttribute=()=>{throw Error('override');};
      button[name]=boolean?true:'button';if(button[name]!==(boolean?true:'button'))failures.push([name,'canonical']);
    }
    return failures;
  })()`),[]);
});

test('button attribute mutation records and reset retain reflected values',async()=>{
  assert.deepEqual(await evaluate(`(() => {
    const failures=[],form=document.createElement('form'),button=document.createElement('button');form.append(button);
    for(const name of ['name','value','formTarget','type','disabled','formNoValidate']){
      const boolean=['disabled','formNoValidate'].includes(name),attribute=name.toLowerCase();
      const observer=new MutationObserver(()=>{});observer.observe(button,{attributes:true,attributeOldValue:true});
      button[name]=boolean?true:'button';button[name]=boolean?true:'button';button[name]=boolean?false:'reset';if(boolean)button[name]=false;
      const records=observer.takeRecords().map(r=>[r.attributeName,r.attributeNamespace,r.oldValue]);observer.disconnect();
      const expected=[[attribute,null,null],[attribute,null,boolean?'':'button'],[attribute,null,boolean?'':'button']];
      if(JSON.stringify(records)!==JSON.stringify(expected))failures.push([name,'records',records]);
      // A button named "reset" shadows form.reset in Safari.
      const before=button[name];HTMLFormElement.prototype.reset.call(form);if(button[name]!==before)failures.push([name,'reset']);
    }
    return failures;
  })()`),[]);
});

test('button auto type follows command presence and direct select parent across live mutations',async()=>{
  assert.deepEqual(await evaluate(`(() => {
    const failures=[];
    for(const attribute of ['command','commandfor','popovertarget'])for(const value of ['', '--foo','x'])
    for(const type of [null,'','auto','invalid','submit','reset','button','SUBMIT','RESET','BUTTON']){
      const b=document.createElement('button');b.setAttribute(attribute,value);if(type!==null)b.type=type;
      const lower=(type??'').toLowerCase(),explicit=['submit','reset','button'].includes(lower);
      if(b.type!==(explicit?lower:attribute==='popovertarget'?'submit':'button'))failures.push([attribute,value,type]);
      b.removeAttribute(attribute);if(b.type!==(explicit?lower:'submit'))failures.push('removed command');
      b.setAttributeNS('urn:foreign',attribute,value);if(b.type!==(explicit?lower:'submit'))failures.push('foreign command');
    }
    for(const tag of ['select','optgroup','option','datalist','div','form'])for(const nested of [false,true])
    for(const type of [null,'','auto','invalid','submit','reset','button']){
      const parent=document.createElement(tag),b=document.createElement('button');if(type!==null)b.type=type;
      if(nested){const div=document.createElement('div');parent.append(div);div.append(b);}else parent.append(b);
      const explicit=['submit','reset','button'].includes(type);
      if(b.type!==(explicit?type:tag==='select'&&!nested?'button':'submit'))failures.push([tag,nested,type]);
      b.remove();if(b.type!==(explicit?type:'submit'))failures.push('detached');
      const select=document.createElement('select');select.append(b);
      if(b.type!==(explicit?type:'button'))failures.push('reparented');
      const clone=b.cloneNode();if(clone.type!==(explicit?type:'submit'))failures.push('clone has no parent');
    }
    const b=document.createElement('button'),select=document.createElement('select');select.append(b);
    Object.defineProperty(b,'parentNode',{value:null});b.getAttribute=b.hasAttribute=()=>{throw Error('override');};
    if(b.type!=='button')failures.push('canonical parent');
    b.type={toString(){b.remove();b.setAttribute('command','');return 'auto';}};if(b.type!=='button')failures.push('conversion mutation');
    b.setAttributeNS(null,'command','x');b.attributes.getNamedItem('command').value='';if(b.type!=='button')failures.push('Attr mutation');
    b.removeAttributeNS(null,'command');if(b.type!=='submit')failures.push('namespace removal');
    return failures;
  })()`),[]);
});

test('button submission eligibility uses canonical dynamic type',async()=>{
  assert.deepEqual(await evaluate(`(() => {
    const failures=[],form=document.createElement('form'),button=document.createElement('button');
    button.name='action';button.value='save';form.append(button);document.body.append(form);
    let submissions=0;form.addEventListener('submit',event=>{event.preventDefault();submissions++;if(event.submitter!==button)failures.push('submitter');});
    for(const attribute of ['command','commandfor']){
      button.setAttribute(attribute,'');
      if(button.willValidate)failures.push('auto button validation');
      for(const call of [()=>form.requestSubmit(button),()=>new FormData(form,button)]){
        try{call();failures.push('non-submitter accepted');}catch(e){if(e.name!=='TypeError')failures.push(e.name);}
      }
      button.removeAttribute(attribute);
    }
    button.type='submit';button.setAttribute('command','');
    Object.defineProperty(button,'type',{value:'reset'});
    if(!button.willValidate)failures.push('canonical validation');
    form.requestSubmit(button);if(submissions!==1)failures.push('canonical requestSubmit');
    if(new FormData(form,button).get('action')!=='save')failures.push('canonical FormData');
    form.remove();return failures;
  })()`),[]);
});
