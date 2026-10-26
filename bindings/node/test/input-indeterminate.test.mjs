import test from 'node:test';
import assert from 'node:assert/strict';
import {createSession} from '../../../scripts/coverage/session.mjs';
async function evaluate(source){const session=await createSession();try{return await(await session.newPage()).evaluate(source);}finally{await session.close();}}

test('indeterminate is independent boolean state retained by reset, type changes, cloning and import', async () => {
  assert.deepEqual(await evaluate(`(() => {
    const failures=[],i=document.createElement('input'),observer=new MutationObserver(()=>{});observer.observe(i,{attributes:true});
    if(i.indeterminate!==false)failures.push('default');
    for(const value of [undefined,null,false,0,NaN,'',true,1,'false',{},Symbol()]){i.indeterminate=value;if(i.indeterminate!==Boolean(value))failures.push('conversion');}
    if(observer.takeRecords().length)failures.push('attribute mutation');observer.disconnect();
    i.setAttribute('indeterminate','');i.indeterminate=false;if(i.indeterminate)failures.push('attribute reflection');
    i.indeterminate=true;i.checked=true;i.checked=false;i.value='text';
    for(const type of ['text','checkbox','radio','file','number']){i.type=type;if(!i.indeterminate)failures.push([type,'type change']);}
    const form=document.createElement('form');form.appendChild(i);form.reset();if(!i.indeterminate)failures.push('reset');
    const doc=document.implementation.createHTMLDocument();const c=doc.importNode(i);if(!c.indeterminate)failures.push('import');
    doc.adoptNode(i);if(!i.indeterminate)failures.push('adopt');c.indeterminate=false;if(!i.indeterminate)failures.push('independence');
    const d=Object.getOwnPropertyDescriptor(HTMLInputElement.prototype,'indeterminate');
    if(!d.enumerable||!d.configurable||d.get.name!=='get indeterminate'||d.set.name!=='set indeterminate')failures.push('descriptor');
    for(const fn of [d.get,d.set]){try{fn.call({},true);failures.push('brand');}catch(e){if(e.name!=='TypeError')failures.push(e.name);}}
    return failures;
  })()`),[]);
});

test('checkbox pre-activation and follow-up events match Safari for click and dispatched events', async () => {
  assert.deepEqual(await evaluate(`(() => {
    const failures=[];
    for(const attached of [false,true])for(const disabled of [false,true])for(const action of ['click','Event','MouseEvent','PointerEvent']){
      const i=document.createElement('input');i.type='checkbox';i.disabled=disabled;i.indeterminate=true;if(attached)document.body.appendChild(i);
      const log=[];for(const name of ['click','input','change'])i.addEventListener(name,e=>log.push([name,i.checked,i.indeterminate,e.isTrusted,e.composed,e.constructor.name]));
      if(action==='click')i.click();else i.dispatchEvent(new globalThis[action]('click',{bubbles:true,cancelable:true}));
      const active=action!=='Event'&&!(action==='click'&&disabled), expected=[];
      if(!(action==='click'&&disabled))expected.push(['click',active,!active,false,action==='click',action==='click'?'PointerEvent':action]);
      if(active&&attached)expected.push(['input',true,false,true,true,'Event'],['change',true,false,true,false,'Event']);
      if(i.checked!==active||i.indeterminate!==!active||JSON.stringify(log)!==JSON.stringify(expected))failures.push([attached,disabled,action,i.checked,i.indeterminate,log]);
      i.remove();
    }return failures;
  })()`),[]);
});

test('checkbox cancellation, reentrancy and click-time mutations follow Safari', async () => {
  assert.deepEqual(await evaluate(`(() => {
    const failures=[];
    for(const action of ['cancel','remove','text','radio-cancel','mutate','nested','reset']){
      const i=document.createElement('input');i.type='checkbox';i.indeterminate=true;
      const form=document.createElement('form');document.body.appendChild(form);form.appendChild(i);const log=[];
      for(const n of ['input','change'])i.addEventListener(n,()=>log.push(n));
      i.addEventListener('click',e=>{log.push(['click',i.checked,i.indeterminate]);
        if(action==='cancel')e.preventDefault();if(action==='remove')i.remove();if(action==='text')i.type='text';
        if(action==='radio-cancel'){i.type='radio';e.preventDefault();}
        if(action==='mutate'){i.checked=false;i.indeterminate=true;}if(action==='reset')form.reset();
        if(action==='nested'&&log.length===1)i.click();
      });i.click();
      const checked=!['cancel','radio-cancel','mutate','reset'].includes(action),indeterminate=['cancel','mutate'].includes(action);
      const expected=[['click',true,false]];if(['mutate','nested','reset'].includes(action))expected.push('input','change');
      if(i.checked!==checked||i.indeterminate!==indeterminate||JSON.stringify(log)!==JSON.stringify(expected))failures.push([action,i.checked,i.indeterminate,log]);
      form.remove();
    }return failures;
  })()`),[]);
});

test('checkbox activation respects fieldsets and shadow boundaries while bypassing public dispatch overrides', async () => {
  assert.deepEqual(await evaluate(`(() => {
    const failures=[],fieldset=document.createElement('fieldset');fieldset.disabled=true;
    fieldset.innerHTML='<legend><input type="checkbox"></legend><input type="checkbox">';document.body.appendChild(fieldset);
    const a=fieldset.querySelector('legend input'),b=fieldset.lastChild;a.indeterminate=b.indeterminate=true;a.click();b.click();
    if(!a.checked||a.indeterminate||b.checked||!b.indeterminate)failures.push('fieldset');
    const host=document.createElement('div');document.body.appendChild(host);const shadow=host.attachShadow({mode:'open'});
    const i=document.createElement('input');i.type='checkbox';i.indeterminate=true;shadow.appendChild(i);
    const log=[];for(const name of ['input','change']){host.addEventListener(name,()=>log.push('host '+name));i.addEventListener(name,()=>log.push(name));}
    i.dispatchEvent=()=>{throw Error('override');};i.click();
    if(i.indeterminate||!i.checked||JSON.stringify(log)!==JSON.stringify(['input','host input','change']))failures.push(['shadow',log]);
    return failures;
  })()`),[]);
});
