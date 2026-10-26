import test from 'node:test';import assert from 'node:assert/strict';import {createSession}from'../index.js';
// The JavaScript dimension contract has no CSS layout or image decoder.
test('image input dimensions use attributes while intrinsic and styled sizing remain no-layout placeholders',async()=>{
 const s=await createSession();try{const p=await s.newPage();assert.deepEqual(await p.evaluate(`(()=>{
 const i=document.createElement('input');i.type='image';i.src='data:image/svg+xml,<svg xmlns="http://www.w3.org/2000/svg" width="60" height="40"/>';
 i.style.cssText='width:120px;height:80px';document.body.append(i);
 const unset=[i.width,i.height];i.width=12;i.height=8;return [unset,[i.width,i.height],i.offsetWidth,i.offsetHeight];
 })()`),[[0,0],[12,8],0,0]);}finally{await s.close()}
});
