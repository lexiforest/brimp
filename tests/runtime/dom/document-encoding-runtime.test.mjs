import test from 'node:test';
import assert from 'node:assert/strict';
import {createSession,Plugin} from '../../../bindings/node/index.js';

test('encoding is installed before plugins and init scripts and reset on each navigation',async()=>{
  const plugin=new Plugin('encoding-observer','({global})=>{global.pluginEncoding=global.document.characterSet;}');
  const session=await createSession({plugins:[plugin]});
  try{
    const page=await session.newPage();
    await page.addInitScript(`globalThis.initEncoding=document.characterSet;Object.defineProperty(document,'characterSet',{value:'spoofed',configurable:true});`);
    for(const[label,bytes,query]of [['windows-1252','%E9','%E9'],['UTF-8','%C3%A9','%C3%A9']]){
      await page.get(`data:text/html;charset=${label},<!doctype html><p id=sample>${bytes}</p>`);
      assert.deepEqual(await page.evaluate(`(()=>{const a=document.createElement('a');a.href='https://example.test/?é';return [pluginEncoding,initEncoding,document.charset,document.inputEncoding,document.getElementById('sample').textContent,a.search];})()`),[label,label,label,label,'é','?'+query]);
    }
  }finally{await session.close();}
});
