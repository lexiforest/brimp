import test from 'node:test';
import assert from 'node:assert/strict';
import {createSession,Plugin} from '../../../bindings/node/index.js';

test('document URL and MIME metadata are installed before plugins and init scripts',async()=>{
  const session=await createSession({plugins:[new Plugin('document-observer','({global})=>{const d=global.document;global.pluginDocumentInfo=[d.URL,d.documentURI,d.contentType];}')]});
  try{
    const page=await session.newPage();
    await page.addInitScript(`globalThis.initDocumentInfo=[document.URL,document.documentURI,document.contentType];Object.defineProperty(document,'URL',{get(){throw Error('public URL')},configurable:true});document.__contentType='spoofed';`);
    for(const type of ['text/html','text/plain','application/json']){
      const url=`data:${type};charset=utf-8,<!doctype html><body>metadata#fragment`;
      await page.get(url);
      assert.deepEqual(await page.evaluate(`(()=>{const clone=document.cloneNode();return [pluginDocumentInfo,initDocumentInfo,[Object.getOwnPropertyDescriptor(Document.prototype,'URL').get.call(document),document.documentURI,document.contentType],[clone.URL,clone.documentURI,clone.contentType]]})()`),Array.from({length:4},()=>[url,url,type]));
    }
  }finally{await session.close();}
});
