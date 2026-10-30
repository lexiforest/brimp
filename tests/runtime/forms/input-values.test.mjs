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

test('input default modes reflect attributes and distinguish missing from empty values', async () => {
  assert.deepEqual(
    await evaluate(String.raw`(() => {
    const failures=[];
    for(const type of ['hidden','submit','image','reset','button','checkbox','radio']) {
      const i=document.createElement('input');i.type=type;
      const defaultValue=['checkbox','radio'].includes(type)?'on':'';
      if(i.value!==defaultValue)failures.push([type,'missing',i.value]);
      i.value='edited';if(i.value!=='edited'||i.defaultValue!=='edited')failures.push([type,'write']);
      i.defaultValue='new';if(i.value!=='new')failures.push([type,'default']);
      i.value='';if(i.value!==''||i.getAttribute('value')!=='')failures.push([type,'empty']);
      i.removeAttribute('value');if(i.value!==defaultValue)failures.push([type,'remove']);
      i.setAttributeNS('urn:test','value','foreign');if(i.value!==defaultValue)failures.push([type,'namespace']);
      i.value=null;if(i.value!==''||i.defaultValue!=='')failures.push([type,'null']);
      i.value=undefined;if(i.value!=='undefined')failures.push([type,'undefined']);
      try{i.value=Symbol();failures.push([type,'symbol']);}catch(e){if(e.name!=='TypeError')failures.push([type,e.name]);}
    }return failures;
  })()`),
    [],
  );
});

test('text input sanitization follows Safari whitespace, conversion and dirty-value rules', async () => {
  assert.deepEqual(
    await evaluate(String.raw`(() => {
    const failures=[];
    for(const type of ['text','search','tel','password','email','url','month','week','unknown']) {
      const i=document.createElement('input');i.type=type;
      const expected=['email','url'].includes(type)?'abc':' abc ';
      i.defaultValue=' a\rb\nc ';if(i.value!==expected)failures.push([type,'attribute',i.value]);
      i.value=' a\rb\nc ';if(i.value!==expected||i.defaultValue!==' a\rb\nc ')failures.push([type,'setter']);
      i.defaultValue='later';if(i.value!==expected)failures.push([type,'dirty']);
      i.value=null;if(i.value!=='')failures.push([type,'null']);
      i.value=undefined;if(i.value!=='undefined')failures.push([type,'undefined']);
      try{i.value=Symbol();failures.push([type,'symbol']);}catch(e){if(e.name!=='TypeError')failures.push([type,e.name]);}
    }
    const i=document.createElement('input');i.type='email';i.multiple=true;i.value=' a@b , c@d ';
    if(i.value!=='a@b,c@d')failures.push(['multiple',i.value]);
    i.multiple=false;i.value=' a@b , c@d ';i.multiple=true;
    if(i.value!=='a@b,c@d')failures.push(['multiple mutation',i.value]);
    i.value='\u00a0a@b\u00a0';if(i.value!=='\u00a0a@b\u00a0')failures.push('non-ASCII whitespace');
    return failures;
  })()`),
    [],
  );
});

test('value-mode transitions preserve Safari dirty state across all attribute entry points', async () => {
  assert.deepEqual(
    await evaluate(String.raw`(() => {
    const failures=[],defaults=['hidden','checkbox','radio','button','submit','reset','image'];
    const setters=[(i,t)=>i.type=t,(i,t)=>i.setAttribute('TYPE',t),(i,t)=>i.setAttributeNS(null,'type',t),
      (i,t)=>{const a=document.createAttribute('type');a.value=t;i.setAttributeNode(a);},
      (i,t)=>{if(!i.hasAttribute('type'))i.setAttribute('type','text');i.getAttributeNode('type').value=t;}];
    for(const [index,set] of setters.entries())for(const from of ['text',...defaults,'file'])for(const to of ['text',...defaults,'file']) {
      const i=document.createElement('input');set(i,from);i.defaultValue='attribute';if(from!=='file')i.value='edited';
      set(i,to);const expected=to==='file'?'':from==='file'?'attribute':'edited';
      const attribute=defaults.includes(from)||from==='text'&&defaults.includes(to)?'edited':'attribute';
      if(i.value!==expected||i.defaultValue!==attribute)failures.push([index,from,to,'transition',i.value,i.defaultValue]);
      i.defaultValue='later';const later=to==='file'?'':defaults.includes(to)||from==='file'?'later':'edited';
      if(i.value!==later)failures.push([index,from,to,'dirty',i.value]);
      set(i,'text');const back=defaults.includes(to)||from==='file'?'later':'edited';
      if(i.value!==back)failures.push([index,from,to,'back',i.value]);
    }
    for(const from of ['text',...defaults,'file'])for(const to of ['text','file']) {
      const i=document.createElement('input');i.type=from;i.defaultValue='attribute';i.type=to;i.defaultValue='later';i.type='text';
      const expected=defaults.includes(from)?'attribute':'later';
      if(i.value!==expected)failures.push([from,to,'pristine',i.value]);
    }
    for(const from of ['text',...defaults])for(const middle of [...defaults,'file']) {
      const i=document.createElement('input');i.type=from;i.value='edited';i.defaultValue='attribute';i.type=middle;
      i.defaultValue='later';i.type='file';i.type='text';
      const expected=middle!=='file'?'later':from==='text'?'edited':'attribute';
      if(i.value!==expected)failures.push([from,middle,'consecutive',i.value]);
    }
    // Empty values must not materialize a value attribute on a mode change.
    for(const type of defaults){const i=document.createElement('input');i.value='';i.type=type;
      if(i.hasAttribute('value'))failures.push([type,'empty attribute']);}
    return failures;
  })()`),
    [],
  );
});

test('file value clearing and type changes retain text state but discard selected files', async () => {
  assert.deepEqual(
    await evaluate(String.raw`(() => {
    const failures=[],i=document.createElement('input');i.value='edited';i.type='file';
    const transfer=new DataTransfer();transfer.items.add(new File(['data'],'sample.txt'));i.files=transfer.files;
    if(i.value!=='C:\\fakepath\\sample.txt')failures.push(['filename',i.value]);
    i.type='FILE';if(i.files.length!==1)failures.push('same type');
    for(const value of ['x',undefined]){try{i.value=value;failures.push('nonempty');}catch(e){if(e.name!=='InvalidStateError')failures.push(e.name);}}
    try{i.value=Symbol();failures.push('symbol');}catch(e){if(e.name!=='TypeError')failures.push(e.name);}
    i.value=null;if(i.files.length||i.value!=='')failures.push('clear');
    i.files=transfer.files;i.type='text';if(i.value!=='edited'||i.files!==null)failures.push('text restoration');
    i.type='file';if(i.files.length)failures.push('files restoration');
    return failures;
  })()`),
    [],
  );
});

test('input value operations use canonical attributes and form reset clears dirty state', async () => {
  assert.deepEqual(
    await evaluate(String.raw`(() => {
    const failures=[],form=document.createElement('form');form.innerHTML='<input value="original">';
    const i=form.firstChild;i.value='edited';form.reset();i.defaultValue='reset';if(i.value!=='reset')failures.push('reset');
    i.setAttributeNS('urn:test','type','checkbox');i.value='dirty';if(i.getAttribute('value')!=='reset')failures.push('foreign type');
    Object.defineProperty(i,'type',{value:'file'});
    i.getAttribute=i.setAttribute=i.removeAttribute=()=>{throw Error('public override');};
    i.value=' a\nb ';if(i.value!==' ab ')failures.push('override');
    return failures;
  })()`),
    [],
  );
});

test('defaultValue is a canonical DOMString reflection with Safari descriptors and mutation records', async () => {
  assert.deepEqual(
    await evaluate(String.raw`(() => {
    const i=document.createElement('input'),failures=[];
    const observer=new MutationObserver(()=>{});observer.observe(i,{attributes:true,attributeOldValue:true});
    i.defaultValue=null;i.defaultValue=undefined;i.defaultValue='';
    const records=observer.takeRecords().map(r=>[r.attributeName,r.attributeNamespace,r.oldValue]);
    if(JSON.stringify(records)!==JSON.stringify([['value',null,null],['value',null,'null'],['value',null,'undefined']]))failures.push(['mutations',records]);
    observer.disconnect();
    try{i.defaultValue=Symbol();failures.push('symbol');}catch(e){if(e.name!=='TypeError')failures.push(e.name);}
    i.defaultValue={toString(){return 'converted';}};if(i.defaultValue!=='converted'||i.value!=='converted')failures.push('conversion');
    i.removeAttribute('value');i.setAttributeNS('urn:test','value','foreign');if(i.defaultValue!=='')failures.push('namespace');
    i.getAttribute=i.setAttribute=()=>{throw Error('override');};i.defaultValue='canonical';if(i.defaultValue!=='canonical')failures.push('override');
    for(const name of ['value','defaultValue']){const d=Object.getOwnPropertyDescriptor(HTMLInputElement.prototype,name);
      if(!d.enumerable||!d.configurable||d.get.name!=='get '+name||d.set.name!=='set '+name)failures.push([name,'descriptor']);
      for(const fn of [d.get,d.set]){try{fn.call({},'x');failures.push([name,'brand']);}catch(e){if(e.name!=='TypeError')failures.push([name,e.name]);}}}
    return failures;
  })()`),
    [],
  );
});
