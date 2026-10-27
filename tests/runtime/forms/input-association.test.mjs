import test from 'node:test';
import assert from 'node:assert/strict';
import {createSession} from '../../../scripts/coverage/session.mjs';

async function evaluate(source) {
  const session=await createSession();
  try {return await (await session.newPage()).evaluate(source);}
  finally {await session.close();}
}

test('input list and input type reflection follow Safari supported states', async () => {
  assert.deepEqual(await evaluate(`(() => {
    const host=document.createElement('div');host.innerHTML='<input list="choices"><datalist id="choices"></datalist>';
    const input=host.firstChild,list=host.lastChild,failures=[];
    const supported=['text','search','tel','url','email','number','range','color'];
    const excluded=['hidden','password','date','time','datetime-local','checkbox','radio','file','submit','image','reset','button'];
    for(const type of [...supported,...excluded,'month','week','datetime','unknown','TEXT',' text','text ','PASSWORD','password ']) {
      for(const set of [()=>input.type=type,()=>input.setAttribute('type',type)]) {
        set();const lower=type.toLowerCase();
        const expectedType=[...supported,...excluded].includes(lower)?lower:'text';
        if(input.type!==expectedType || input.getAttribute('type')!==type)failures.push([type,'reflection',input.type]);
        if(input.list!==(excluded.includes(expectedType)?null:list))failures.push([type,'list']);
      }
    }
    input.removeAttribute('type');input.setAttribute('list','');list.id='';
    if(input.type!=='text' || input.list!==null)failures.push('empty ID');
    return failures;
  })()`),[]);
});

test('datalist association is live, uses first matching ID and stays within its tree', async () => {
  assert.deepEqual(await evaluate(`(() => {
    const host=document.createElement('div');host.innerHTML='<input list="x"><datalist id="x"></datalist>';
    const input=host.firstChild,list=host.lastChild,failures=[];
    if(input.list!==list)failures.push('detached tree');
    const blocker=document.createElement('div');blocker.id='x';host.insertBefore(blocker,list);
    if(input.list!==null)failures.push('wrong type first ID');
    blocker.remove();const other=list.cloneNode();host.insertBefore(other,list);
    if(input.list!==other)failures.push('first duplicate');other.remove();
    list.id='y';if(input.list!==null)failures.push('ID mutation');input.setAttribute('list','y');
    if(input.list!==list)failures.push('list mutation');
    document.body.appendChild(host);
    const shadowHost=document.createElement('div');document.body.appendChild(shadowHost);
    const shadow=shadowHost.attachShadow({mode:'open'});shadow.appendChild(input);
    if(input.list!==null)failures.push('outside shadow');shadow.appendChild(list);
    if(input.list!==list)failures.push('inside shadow');
    const fragment=document.createDocumentFragment();fragment.appendChild(input);fragment.appendChild(list);
    if(input.list!==null)failures.push('plain fragment');
    const root=document.createElement('datalist');root.id='y';root.appendChild(input);
    if(input.list!==null)failures.push('element root must be excluded');
    const foreign=document.implementation.createHTMLDocument();foreign.body.appendChild(fragment);foreign.body.appendChild(input);
    if(input.list!==list || input.ownerDocument!==foreign)failures.push('adoption');
    return failures;
  })()`),[]);
});

test('input associations use canonical null-namespace attributes and ignore public overrides', async () => {
  assert.deepEqual(await evaluate(`(() => {
    const host=document.createElement('div');host.innerHTML='<input><datalist id="x"></datalist>';
    const input=host.firstChild,list=host.lastChild,failures=[];
    input.setAttributeNS('urn:test','list','x');
    if(input.list!==null)failures.push('namespaced list');
    input.setAttributeNS(null,'list','x');
    if(input.list!==list)failures.push('null namespace list');
    list.removeAttribute('id');list.setAttributeNS('urn:test','id','x');
    if(input.list!==null)failures.push('namespaced ID');list.setAttributeNS(null,'id','x');
    const svg=document.createElementNS('http://www.w3.org/2000/svg','datalist');svg.setAttribute('id','x');host.insertBefore(svg,list);
    if(input.list!==null)failures.push('SVG is not a datalist');svg.remove();
    const fail=()=>{throw Error('public override called');};
    input.getRootNode=fail;input.getAttribute=fail;list.getAttribute=fail;host.querySelectorAll=fail;
    Object.defineProperty(input,'type',{get:()=> 'checkbox'});
    if(input.list!==list)failures.push('public override');
    return failures;
  })()`),[]);
});

test('webkitdirectory reflects boolean presence with Safari conversion and mutation records', async () => {
  assert.deepEqual(await evaluate(`(() => {
    const input=document.createElement('input'),failures=[];
    const observer=new MutationObserver(()=>{});observer.observe(input,{attributes:true,attributeOldValue:true});
    input.webkitdirectory=true;input.webkitdirectory=true;input.webkitdirectory=false;
    const records=observer.takeRecords().map(r=>[r.attributeName,r.oldValue]);
    if(JSON.stringify(records)!==JSON.stringify([['webkitdirectory',null],['webkitdirectory',''],['webkitdirectory','']]))failures.push(['records',records]);
    observer.disconnect();
    for(const value of [undefined,null,false,0,NaN,'',true,1,'false',{},Symbol('truthy')]) {
      input.webkitdirectory=value;
      if(input.webkitdirectory!==Boolean(value) || input.getAttribute('webkitdirectory')!==(value?'':null))failures.push('conversion');
    }
    input.removeAttribute('webkitdirectory');input.setAttributeNS('urn:test','webkitdirectory','yes');
    if(input.webkitdirectory)failures.push('namespace');
    input.setAttributeNS(null,'webkitdirectory','false');if(!input.webkitdirectory)failures.push('presence');
    input.getAttribute=input.setAttribute=input.removeAttribute=()=>{throw Error('override');};
    input.webkitdirectory=false;input.webkitdirectory=true;
    if(!input.webkitdirectory)failures.push('override');
    return failures;
  })()`),[]);
});

test('unselected webkitEntries is frozen, stable per input and unaffected by FileList assignment', async () => {
  assert.deepEqual(await evaluate(`(() => {
    const a=document.createElement('input'),b=document.createElement('input'),entries=a.webkitEntries,failures=[];
    if(!Array.isArray(entries) || entries.length || !Object.isFrozen(entries) || entries!==a.webkitEntries || entries===b.webkitEntries)failures.push('default');
    a.type='file';a.webkitdirectory=true;
    const transfer=new DataTransfer();transfer.items.add(new File(['x'],'file.txt'));a.files=transfer.files;
    if(a.webkitEntries!==entries || entries.length)failures.push('FileList');
    a.type='text';if(a.webkitEntries!==entries)failures.push('type change');
    for(const name of ['list','webkitdirectory','webkitEntries']) {
      const d=Object.getOwnPropertyDescriptor(HTMLInputElement.prototype,name);
      if(!d.enumerable || !d.configurable || d.get.name!=='get '+name || (name==='webkitdirectory'?d.set.name!=='set '+name:d.set!==undefined))failures.push([name,'descriptor']);
      try{d.get.call({});failures.push([name,'brand']);}catch(e){if(e.name!=='TypeError')failures.push([name,e.name]);}
    }
    return failures;
  })()`),[]);
});
