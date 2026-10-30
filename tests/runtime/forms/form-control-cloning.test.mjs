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

test('input clones and imports preserve edited values, checkedness and dirty flags', async () => {
  assert.deepEqual(
    await evaluate(`(() => {
    const failures=[],foreign=document.implementation.createHTMLDocument();
    const types=['text','search','url','email','password','number','range','date','time','datetime-local','color','checkbox','radio','hidden','button'];
    for(const type of types)for(const dirty of [false,true])for(const imported of [false,true]) {
      const i=document.createElement('input');i.type=type;i.defaultValue='original';i.defaultChecked=true;
      if(dirty){i.value=({number:'42',range:'42',date:'2024-02-29',time:'01:02','datetime-local':'2024-02-29T01:02',color:'#123456'})[type]??'edited';i.checked=false;}
      const expected=i.value,c=imported?foreign.importNode(i,true):i.cloneNode(true);
      if(c.value!==expected||c.checked!==!dirty||c.defaultChecked!==true||c.defaultValue!==i.defaultValue)failures.push([type,dirty,imported,'copy']);
      if(c.ownerDocument!==(imported?foreign:document)||c.parentNode!==null)failures.push([type,'owner']);
      c.defaultValue='later';c.defaultChecked=false;
      const fresh=document.createElement('input');fresh.type=type;fresh.defaultValue='later';
      const attributeMode=['hidden','button','checkbox','radio'].includes(type);
      if(c.value!==(dirty&&!attributeMode?expected:fresh.value)||c.checked!==false)failures.push([type,dirty,'flags',c.value]);
      if(i.checked!==!dirty||i.value!==expected)failures.push([type,'independence']);
    }
    return failures;
  })()`),
    [],
  );
});

test('clones preserve raw range state and pre-file values without copying file lists or custom validity', async () => {
  assert.deepEqual(
    await evaluate(`(() => {
    const failures=[],i=document.createElement('input');i.value='edited';i.type='file';i.indeterminate=true;i.setCustomValidity('invalid');
    const transfer=new DataTransfer();transfer.items.add(new File(['x'],'x.txt'));i.files=transfer.files;
    const c=i.cloneNode();if(c.files.length||c.files===i.files||c.value!==''||c.validationMessage!==''||!c.indeterminate)failures.push('file clone');
    c.type='text';if(c.value!=='edited')failures.push('pre-file value');
    c.indeterminate=false;if(!i.indeterminate)failures.push('independent indeterminate');
    const range=document.createElement('input');range.type='range';range.value='80';range.step='17';
    const r=range.cloneNode();if(r.value!=='80')failures.push(['raw range',r.value]);r.max='60';if(r.value!=='51'||range.value!=='80')failures.push('range independence');
    const form=document.createElement('form');form.appendChild(c);c.defaultValue='reset';form.reset();if(c.value!=='reset')failures.push('reset dirty clone');
    return failures;
  })()`),
    [],
  );
});

test('input and textarea cloning resets selection, omits listeners and retains dirty textarea text', async () => {
  assert.deepEqual(
    await evaluate(`(() => {
    const failures=[];
    for(const tag of ['input','textarea'])for(const deep of [false,true]) {
      const i=document.createElement(tag);i.defaultValue='default';i.value='edited';i.setSelectionRange(1,3,'backward');i.setCustomValidity('invalid');
      let calls=0;i.addEventListener('custom',()=>calls++);i.onchange=()=>calls++;
      const c=i.cloneNode(deep);c.dispatchEvent(new Event('custom'));c.dispatchEvent(new Event('change'));
      if(c.value!=='edited'||c.defaultValue!==(tag==='textarea'&&!deep?'':'default'))failures.push([tag,deep,'value']);
      if(c.selectionStart!==0||c.selectionEnd!==0||c.selectionDirection!=='none'||c.validationMessage!==''||calls)failures.push([tag,deep,'uncopied state']);
      c.setSelectionRange(2,4);if(i.selectionStart!==1||i.selectionEnd!==3)failures.push([tag,'selection independence']);
      c.defaultValue='later';if(c.value!=='edited')failures.push([tag,'dirty flag']);
    }
    const clean=document.createElement('textarea');clean.defaultValue='default';const c=clean.cloneNode(true);c.defaultValue='new';if(c.value!=='new')failures.push('pristine textarea');
    return failures;
  })()`),
    [],
  );
});

test('deep element, fragment, document and template clones copy nested control state using canonical nodes', async () => {
  assert.deepEqual(
    await evaluate(`(() => {
    const failures=[],foreign=document.implementation.createHTMLDocument();
    for(const kind of ['element','fragment','document','template'])for(const imported of [false,true]) {
      if(kind==='document'&&imported)continue;
      const root=kind==='element'?document.createElement('div'):kind==='fragment'?document.createDocumentFragment():kind==='document'?document.implementation.createHTMLDocument():document.createElement('template');
      const container=kind==='document'?root.body:kind==='template'?root.content:root;
      const input=document.createElement('input'),textarea=document.createElement('textarea'),nested=document.createElement('template');
      input.value='input';input.checked=true;input.indeterminate=true;textarea.value='textarea';
      const inside=document.createElement('input');inside.value='nested';nested.content.appendChild(inside);
      container.append(input,textarea,nested);
      const fail=()=>{throw Error('override called');};
      Object.defineProperty(input,'value',{get:fail});Object.defineProperty(container,'childNodes',{get:fail});
      Object.defineProperty(nested,'content',{get:fail});
      const copy=imported?foreign.importNode(root,true):root.cloneNode(true);
      const target=kind==='document'?copy.body:kind==='template'?copy.content:copy;
      const children=Array.from(target.children),c=children[0];
      if(c.value!=='input'||!c.checked||!c.indeterminate||children[1].value!=='textarea'||children[2].content.firstChild.value!=='nested')failures.push([kind,imported,'state']);
      if(kind!=='document'&&copy.ownerDocument!==(imported?foreign:document))failures.push([kind,'owner']);
      const shallow=imported?foreign.importNode(root,false):root.cloneNode(false);
      if(kind!=='document'&&(kind==='template'?shallow.content:shallow).childNodes.length)failures.push([kind,'shallow']);
    }
    return failures;
  })()`),
    [],
  );
});

test('textarea defaults use direct child text, preserve raw line endings and respect the dirty value flag', async () => {
  assert.deepEqual(
    await evaluate(String.raw`(() => {
    const failures=[],t=document.createElement('textarea'),nested=document.createElement('span');nested.textContent='nested';
    t.appendChild(document.createTextNode('a\rb\r\nc'));t.appendChild(nested);t.appendChild(document.createComment('comment'));t.appendChild(document.createTextNode('d'));
    if(t.defaultValue!=='a\rb\r\ncd'||t.value!=='a\nb\ncd')failures.push(['child text',t.defaultValue,t.value]);
    t.value='edited\r\nvalue';t.defaultValue='new\rdefault';
    if(t.defaultValue!=='new\rdefault'||t.value!=='edited\nvalue'||t.childNodes.length!==1)failures.push('dirty default');
    const form=document.createElement('form');form.appendChild(t);form.reset();if(t.value!=='new\ndefault')failures.push('reset');
    t.defaultValue=null;if(t.defaultValue!=='null'||t.value!=='null')failures.push('null default');
    t.value=null;if(t.value!=='')failures.push('null value');
    for(const name of ['defaultValue','value']){try{t[name]=Symbol();failures.push([name,'symbol']);}catch(e){if(e.name!=='TypeError')failures.push([name,e.name]);}}
    Object.defineProperty(t,'textContent',{get(){throw Error('override');},set(){throw Error('override');}});
    Object.defineProperty(t,'childNodes',{get(){throw Error('override');}});
    t.defaultValue='canonical';if(t.defaultValue!=='canonical'||t.value!=='')failures.push('canonical');
    for(const name of ['defaultValue','value']){const d=Object.getOwnPropertyDescriptor(HTMLTextAreaElement.prototype,name);
      if(!d.enumerable||!d.configurable||d.get.name!=='get '+name||d.set.name!=='set '+name)failures.push([name,'descriptor']);
      for(const fn of [d.get,d.set]){try{fn.call({},'x');failures.push([name,'brand']);}catch(e){if(e.name!=='TypeError')failures.push([name,e.name]);}}
    }
    return failures;
  })()`),
    [],
  );
});
