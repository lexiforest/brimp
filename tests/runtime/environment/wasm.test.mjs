import test from 'node:test';
import assert from 'node:assert/strict';
import { createSession } from '../../../bindings/node/index.js';

test('WASM compilation errors and streaming body contracts', async () => {
  const session = await createSession();
  try {
    const page = await session.newPage();
    const result = await page.evaluate(`(async()=>{
    const bytes=new Uint8Array([0,97,115,109,1,0,0,0]);
    const error=async fn=>{try{await fn();return 'unexpected success'}catch(e){return e.name}};
    const used=new Response(bytes,{headers:{'Content-Type':'application/wasm'}});await used.arrayBuffer();
    const locked=new Response(bytes,{headers:{'Content-Type':'application/wasm'}});locked.body.getReader();
    return await Promise.all([
      error(()=>WebAssembly.compile(new Uint8Array([1]))),
      error(()=>WebAssembly.compileStreaming(new Response(bytes))),
      error(()=>WebAssembly.compileStreaming(new Response(bytes,{headers:{'Content-Type':'application/wasm; charset=utf-8'}}))),
      error(()=>WebAssembly.compileStreaming(new Response(bytes,{status:404,headers:{'Content-Type':'application/wasm'}}))),
      error(()=>WebAssembly.compileStreaming(used)),error(()=>WebAssembly.compileStreaming(locked)),
      error(()=>WebAssembly.compileStreaming(Promise.reject(new RangeError('source')))),
      error(()=>WebAssembly.compileStreaming({})),
      WebAssembly.instantiate(bytes).then(({instance})=>instance instanceof WebAssembly.Instance),
    ]);
  })()`);
    assert.deepEqual(result, [
      'CompileError',
      ...Array(5).fill('TypeError'),
      'RangeError',
      'TypeError',
      true,
    ]);
  } finally {
    await session.close();
  }
});
