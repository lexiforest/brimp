import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {createSession} from '../../../scripts/coverage/session.mjs';
import {elementConstructionCases,measureElementConstruction} from '../../../scripts/coverage/element-construction-cases.mjs';
async function evaluate(source) {const s=await createSession();try{return await(await s.newPage()).evaluate(source)}finally{await s.close()}}

test('element construction and ignored customization options match Safari',async()=>{
  const ref=JSON.parse(fs.readFileSync(new URL('./fixtures/element-construction.safari.json',import.meta.url))).cases;
  assert.deepEqual(ref.map(({expected,...c})=>c),elementConstructionCases);
  const actual=JSON.parse(await evaluate(`JSON.stringify((${measureElementConstruction})(${JSON.stringify(elementConstructionCases)}))`));
  assert.deepEqual(ref.flatMap((c,i)=>JSON.stringify(c.expected)===JSON.stringify(actual[i])?[]:[{...c,actual:actual[i]}]),[]);
});

test('failed fresh creation is distinct from failed upgrades and is not retried',async()=>{
  assert.deepEqual(await evaluate(`(()=>{
    const failures=[];
    for(const mode of ['create','upgrade','insert']){
      const name='failed-'+mode;let captured,calls=0,connections=0;
      class X extends HTMLElement{constructor(){super();calls++;captured=this;this.setAttribute('before','yes');throw Error('fail')}
        connectedCallback(){connections++}}
      let e;if(mode!=='create'){e=document.createElement(name);if(mode==='insert')document.body.append(e)}
      customElements.define(name,X);if(mode==='create')e=document.createElement(name);if(mode==='upgrade')customElements.upgrade(e);
      if(mode==='create'){
        if(e===captured||e instanceof X||!(e instanceof HTMLUnknownElement)||e.getAttribute('before')!==null)failures.push('fresh failure');
      }else if(e!==captured||!(e instanceof X)||e.getAttribute('before')!=='yes')failures.push('upgrade failure');
      customElements.upgrade(e);document.body.append(e);e.remove();document.body.append(e);
      if(calls!==1||connections!==0)failures.push('failed retry or callback');
    }
    return failures;
  })()`),[]);
});

test('construction stacks are per definition and cannot be consumed twice',async()=>{
  assert.deepEqual(await evaluate(`(()=>{
    const failures=[],root=document.createElement('div');root.innerHTML='<outer-element></outer-element>';
    class Inner extends HTMLElement{}
    customElements.define('inner-element',Inner);
    class Outer extends HTMLElement{constructor(){const inner=new Inner();super();this.inner=inner}}
    customElements.define('outer-element',Outer);customElements.upgrade(root);
    const e=root.firstChild;
    if(!(e instanceof Outer)||!(e.inner instanceof Inner)||e.inner===e||e.inner.localName!=='inner-element')failures.push('nested different definitions');
    const holder=document.createElement('div');holder.innerHTML='<double-element></double-element>';
    let duplicate;
    class Double extends HTMLElement{constructor(){super();try{Reflect.construct(HTMLElement,[],Double)}catch(e){duplicate=e.name}}}
    customElements.define('double-element',Double);customElements.upgrade(holder);
    if(duplicate!=='TypeError'||!(holder.firstChild instanceof Double))failures.push('duplicate consumption');
    const direct=new Double();if(direct.localName!=='double-element')failures.push('stack cleanup');
    return failures;
  })()`),[]);
});

test('autonomous lifecycle and reflected attribute reactions survive ignored extends options',async()=>{
  assert.deepEqual(await evaluate(`(()=>{
    const log=[];
    class Autonomous extends HTMLElement{
      static get observedAttributes(){return ['title','hidden']}
      constructor(){super();this.marker=true}
      attributeChangedCallback(name,oldValue,value){log.push([name,oldValue,value])}
      connectedCallback(){log.push('connected')}
      disconnectedCallback(){log.push('disconnected')}
    }
    customElements.define('autonomous-reactions',Autonomous,{get extends(){throw Error('must not read')}});
    const e=document.createElement('autonomous-reactions');e.title='a';e.title='a';e.hidden=true;e.hidden=false;
    document.body.append(e);e.remove();
    return [e instanceof Autonomous,e.localName,e.getAttribute('is'),log];
  })()`),[true,'autonomous-reactions',null,[['title',null,'a'],['title','a','a'],['hidden',null,''],['hidden','',null],'connected','disconnected']]);
});

test('implemented HTML constructors match Safari, including the output factory exception',async()=>{
  const {builtinElementInterfaces,measureBuiltinConstructors}=await import('../../../scripts/coverage/element-construction-cases.mjs');
  const expected=JSON.parse(fs.readFileSync(new URL('./fixtures/element-construction.safari.json',import.meta.url))).builtins;
  assert.deepEqual(expected.map(x=>x.name),builtinElementInterfaces);
  assert.deepEqual(await evaluate(`(${measureBuiltinConstructors})(${JSON.stringify(builtinElementInterfaces)})`),expected);
});

test('canonical registration remains usable when public lookup methods are overridden',async()=>{
  assert.deepEqual(await evaluate(`(()=>{
    const lookup=customElements.get,lookupName=customElements.getName,upgrade=customElements.upgrade;
    customElements.get=customElements.getName=customElements.upgrade=()=>{throw Error('public override')};
    try{
      class X extends HTMLElement{}
      customElements.define('canonical-construction',X);
      const a=new X(),b=document.createElement('canonical-construction');
      return [a instanceof X,b instanceof X,a.localName,b.localName];
    }finally{customElements.get=lookup;customElements.getName=lookupName;customElements.upgrade=upgrade}
  })()`),[true,true,'canonical-construction','canonical-construction']);
});
