import test from 'node:test';
import assert from 'node:assert/strict';
import {createSession} from '../../../bindings/node/index.js';

test('Brimp button clicks use canonical dynamic type for submit and reset',async()=>{
  const session=await createSession();try{
    const page=await session.newPage();
    await page.evaluate(`document.body.innerHTML='<form><button id="button" command="">Run</button></form>';
      globalThis.log=[];const form=document.querySelector('form');
      for(const type of ['submit','reset'])form.addEventListener(type,e=>{e.preventDefault();log.push(type);});null`);
    const button=await page.querySelector('#button');try{
      await button.click();assert.deepEqual(await page.evaluate('log'),[]);
      await page.evaluate(`const b=document.getElementById('button');b.removeAttribute('command');Object.defineProperty(b,'type',{value:'button'});null`);
      await button.click();assert.deepEqual(await page.evaluate('log'),['submit']);
      await page.evaluate(`document.getElementById('button').setAttribute('type','reset');null`);
      await button.click();assert.deepEqual(await page.evaluate('log'),['submit','reset']);
    }finally{await button.dispose();}
  }finally{await session.close();}
});

test('Brimp input controller uses the shared checkbox activation exactly once and rolls back cancellation', async () => {
  const session=await createSession();
  try {
    const page=await session.newPage();
    await page.evaluate(`document.body.innerHTML='<input id="check" type="checkbox">';
      globalThis.log=[];const i=document.getElementById('check');i.indeterminate=true;
      for(const name of ['click','input','change'])i.addEventListener(name,e=>log.push([name,i.checked,i.indeterminate,e.isTrusted]));null`);
    const input=await page.querySelector('#check');
    try{
      await input.click();
      assert.deepEqual(await page.evaluate('log'),[['click',true,false,true],['input',true,false,true],['change',true,false,true]]);
      await page.evaluate(`log=[];document.getElementById('check').indeterminate=true;document.getElementById('check').addEventListener('click',e=>e.preventDefault(),{once:true});null`);
      await input.click();
      assert.deepEqual(await page.evaluate(`[document.getElementById('check').checked,document.getElementById('check').indeterminate,log]`),[true,true,[['click',false,false,true]]]);
    }finally{await input.dispose();}
  }finally{await session.close();}
});

test('Brimp radio clicks share group-aware activation and cancellation with script events',async()=>{
  const session=await createSession();try{
    const page=await session.newPage();
    await page.evaluate(`document.body.innerHTML='<form><input type=radio name=g id=a checked><input type=radio name=g id=b></form><form><input type=radio name=g id=c checked></form>';
      globalThis.log=[];for(const n of ['click','input','change'])document.getElementById('b').addEventListener(n,e=>log.push([n,...['a','b','c'].map(id=>document.getElementById(id).checked),e.isTrusted,e.composed]));null`);
    const input=await page.querySelector('#b');try{
      await input.click();
      assert.deepEqual(await page.evaluate('log'),[['click',false,true,true,true,true],['input',false,true,true,true,true],['change',false,true,true,true,false]]);
      await page.evaluate(`document.getElementById('a').checked=true;log=[];document.getElementById('b').addEventListener('click',e=>e.preventDefault(),{once:true});null`);
      await input.click();
      assert.deepEqual(await page.evaluate(`[...['a','b','c'].map(id=>document.getElementById(id).checked),log]`),[true,false,true,[['click',false,true,true,true,true]]]);
    }finally{await input.dispose();}
  }finally{await session.close();}
});
