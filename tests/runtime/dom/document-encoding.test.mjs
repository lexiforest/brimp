import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {createSession} from '../../../scripts/coverage/session.mjs';
import {documentEncodingCases,documentEncodingBody,measureDocumentEncodings} from '../../../scripts/coverage/document-encoding-cases.mjs';
const reference=JSON.parse(fs.readFileSync(new URL('../fixtures/document-encoding.safari.json',import.meta.url)));

test('document encoding selection, byte decoding and metadata lifetime match Safari',async()=>{
  assert.deepEqual(reference.map(({expected,...c})=>c),documentEncodingCases);
  const session=await createSession();
  try{
    const results=await measureDocumentEncodings(await session.newPage());
    assert.deepEqual(reference.flatMap((c,i)=>JSON.stringify(c.expected)===JSON.stringify(results[i])?[]:[{id:c.id,expected:c.expected,actual:results[i]}]),[]);
  }finally{await session.close();}
});

test('document encoding aliases have Safari readonly descriptors and native receivers',async()=>{
  const session=await createSession();
  try{assert.deepEqual(await(await session.newPage()).evaluate(`(()=>{
    const failures=[];
    for(const name of ['characterSet','charset','inputEncoding']){
      const d=Object.getOwnPropertyDescriptor(Document.prototype,name);
      if(!d.enumerable||!d.configurable||d.set!==undefined||d.get.name!=='get '+name||d.get.length!==0)failures.push([name,'descriptor']);
      const forged=document.createElement('div');Object.setPrototypeOf(forged,Document.prototype);
      for(const receiver of [{},null,document.createElement('div'),Object.create(Document.prototype),forged]){
        try{d.get.call(receiver);failures.push([name,'accepted receiver'])}catch(e){if(e.name!=='TypeError')failures.push([name,e.name])}
      }
      const doc=document.implementation.createHTMLDocument();doc[name]='Big5';
      if(doc[name]!=='UTF-8'||Object.hasOwn(doc,name))failures.push([name,'readonly']);
      try{(function(){'use strict';doc[name]='Big5'})();failures.push([name,'strict write'])}catch(e){if(e.name!=='TypeError')failures.push([name,e.name])}
      Object.setPrototypeOf(doc,null);if(d.get.call(doc)!=='UTF-8')failures.push([name,'canonical brand']);
    }
    const d=document.implementation.createHTMLDocument();d.open();d.write('<meta charset=Big5><p>é</p>');d.close();
    if(d.characterSet!=='UTF-8'||d.body.textContent!=='é')failures.push('string document.write');
    return failures;
  })()`),[]);}finally{await session.close();}
});

test('document.open/write retains navigation encoding and string-created documents stay UTF-8',async()=>{
  const session=await createSession();
  try{
    const page=await session.newPage();await page.get('data:text/html;charset=windows-1252,<!doctype html><body>');
    assert.deepEqual(await page.evaluate(`(()=>{
      const read=d=>[d.characterSet,d.charset,d.inputEncoding];const result=[];
      const frame=document.createElement('iframe');document.body.appendChild(frame);result.push(read(frame.contentDocument));
      for(const d of [document.cloneNode(true),document.implementation.createHTMLDocument(),document]){
        result.push(read(d));d.open();d.write('<meta charset=Big5><p>é</p>');d.close();result.push(read(d),d.body.textContent);
      }
      return result;
    })()`),[['UTF-8','UTF-8','UTF-8'],['windows-1252','windows-1252','windows-1252'],['windows-1252','windows-1252','windows-1252'],'é',['UTF-8','UTF-8','UTF-8'],['UTF-8','UTF-8','UTF-8'],'é',['windows-1252','windows-1252','windows-1252'],['windows-1252','windows-1252','windows-1252'],'é']);
  }finally{await session.close();}
});

test('iframe data bytes use the same encoding selection; srcdoc stays a Unicode string',async()=>{
  const session=await createSession();
  try{
    const page=await session.newPage();
    const report='<script>parent.postMessage("encoding:"+JSON.stringify([document.characterSet,document.charset,document.inputEncoding,document.getElementById("sample").textContent]),"*")</script>';
    const cases=reference.filter(c=>c.id.startsWith('decoded ')||c.id.startsWith('BOM ')).map(c=>({
      id:c.id,url:`data:${c.header};base64,${documentEncodingBody(c,report).toString('base64')}`,
      expected:[...c.expected.before,c.expected.text],
    }));
    cases.push({id:'srcdoc',srcdoc:'<meta charset=Big5><p id=sample>é☃𠮷</p>'+report,expected:['UTF-8','UTF-8','UTF-8','é☃𠮷']});
    await page.evaluate(`(()=>{globalThis.frameResult=null;addEventListener('message',event=>{if(typeof event.data==='string'&&event.data.startsWith('encoding:'))globalThis.frameResult=JSON.parse(event.data.slice(9));});return true;})()`);
    for(const c of cases){
      await page.evaluate(`(()=>{globalThis.frameResult=null;const f=document.createElement('iframe');f.id='encoding-frame';${c.srcdoc===undefined?`f.src=${JSON.stringify(c.url)}`:`f.srcdoc=${JSON.stringify(c.srcdoc)}`};document.body.appendChild(f);return true;})()`);
      let actual=null;
      for(let i=0;i<100;i++){
        actual=await page.evaluate('globalThis.frameResult');if(actual!==null)break;
        await new Promise(resolve=>setTimeout(resolve,20));
      }
      assert.deepEqual(actual,c.expected,c.id);
      await page.evaluate(`(()=>{document.getElementById('encoding-frame').remove();return true;})()`);
    }
  }finally{await session.close();}
});
