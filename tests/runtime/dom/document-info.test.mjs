import test from'node:test';import assert from'node:assert/strict';import fs from'node:fs';
import{createSession}from'../../../scripts/coverage/session.mjs';
import{documentInfoCases,measureDocumentInfos}from'../../../scripts/coverage/document-info-cases.mjs';
const reference=JSON.parse(fs.readFileSync(new URL('../fixtures/document-info.safari.json',import.meta.url)));

test('document URL, documentURI and contentType match Safari throughout navigation and document lifecycles',async()=>{
  assert.deepEqual(reference.map(({expected,...c})=>c),documentInfoCases);
  const s=await createSession();try{const actual=await measureDocumentInfos(await s.newPage());assert.deepEqual(reference.flatMap((c,i)=>JSON.stringify(c.expected)===JSON.stringify(actual[i])?[]:[{id:c.id,expected:c.expected,actual:actual[i]}]),[]);}finally{await s.close()}
});

test('document metadata has readonly Safari descriptors and canonical native receivers',async()=>{
  const s=await createSession();try{assert.deepEqual(await(await s.newPage()).evaluate(`(()=>{
    const failures=[];
    for(const name of ['URL','documentURI','contentType']){
      const descriptor=Object.getOwnPropertyDescriptor(Document.prototype,name);
      if(!descriptor.enumerable||!descriptor.configurable||descriptor.set!==undefined||descriptor.get.name!=='get '+name||descriptor.get.length!==0)failures.push([name,'descriptor']);
      const forged=document.createElement('div');Object.setPrototypeOf(forged,Document.prototype);
      for(const receiver of [null,{},Object.create(Document.prototype),forged]){
        try{descriptor.get.call(receiver);failures.push([name,'accepted receiver'])}catch(e){if(e.name!=='TypeError')failures.push([name,e.name])}
      }
      const d=document.implementation.createHTMLDocument();const before=d[name];d[name]='spoofed';
      if(d[name]!==before||Object.hasOwn(d,name))failures.push([name,'sloppy assignment']);
      try{(function(){'use strict';d[name]='spoofed'})();failures.push([name,'strict assignment'])}catch(e){if(e.name!=='TypeError')failures.push([name,e.name])}
      Object.setPrototypeOf(d,null);if(descriptor.get.call(d)!==before)failures.push([name,'native identity']);
    }
    return failures;
  })()`),[]);}finally{await s.close()}
});

test('URL and contentType metadata survive adoption and do not dispatch through public document properties',async()=>{
  const s=await createSession();try{assert.deepEqual(await(await s.newPage()).evaluate(`(()=>{
    const failures=[];const read=d=>[d.URL,d.documentURI,d.contentType];
    const a=new DOMParser().parseFromString('<base href="https://base.test/"><a id=link href=child></a>','text/html'),b=document.implementation.createHTMLDocument();
    const before=[read(a),read(b)],link=a.getElementById('link');b.adoptNode(link);
    if(JSON.stringify([read(a),read(b)])!==JSON.stringify(before))failures.push('adoption metadata');
    for(const name of ['URL','documentURI','contentType']){
      const get=Object.getOwnPropertyDescriptor(Document.prototype,name).get;
      Object.defineProperty(a,name,{configurable:true,get(){throw Error('public metadata')}});
      const clone=Node.prototype.cloneNode.call(a,true);
      if(get.call(a)!==get.call(clone))failures.push([name,'clone']);
      delete a[name];
    }
    const xml=new Document();xml.appendChild(xml.createElement('root'));
    try{xml.open();failures.push('XML open')}catch(e){if(e.name!=='InvalidStateError'||!xml.documentElement)failures.push('XML mutation')}
    return failures;
  })()`),[]);}finally{await s.close()}
});

test('iframe data and srcdoc documents retain their own URL and content type when cloned',async()=>{
  const s=await createSession();try{
    const page=await s.newPage();
    const source=`<!doctype html><body><script>parent.postMessage('doc-info:'+JSON.stringify([document.URL,document.documentURI,document.contentType,...(()=>{const d=document.cloneNode(false);return [d.URL,d.documentURI,d.contentType]})()]),'*')</script>`;
    const cases=['text/html','TEXT/HTML;charset=utf-8','text/html;charset=windows-1252'].map(type=>{const url='data:'+type+';base64,'+Buffer.from(source).toString('base64')+'#child';return{url,expected:[url,url,'text/html',url,url,'text/html']}});
    cases.push({srcdoc:source,expected:['about:srcdoc','about:srcdoc','text/html','about:srcdoc','about:srcdoc','text/html']});
    await page.evaluate(`(()=>{globalThis.infoResult=null;addEventListener('message',e=>{if(typeof e.data==='string'&&e.data.startsWith('doc-info:'))infoResult=JSON.parse(e.data.slice(9))});return true;})()`);
    for(const c of cases){
      await page.evaluate(`(()=>{globalThis.infoResult=null;const f=document.createElement('iframe');f.id='info-frame';${c.srcdoc===undefined?`f.src=${JSON.stringify(c.url)}`:`f.srcdoc=${JSON.stringify(c.srcdoc)}`};document.body.appendChild(f);return true;})()`);
      let actual=null;for(let i=0;i<100;i++){actual=await page.evaluate('globalThis.infoResult');if(actual!==null)break;await new Promise(r=>setTimeout(r,20));}
      assert.deepEqual(actual,c.expected);
      await page.evaluate(`(()=>{document.getElementById('info-frame').remove();return true;})()`);
    }
  }finally{await s.close()}
});
