import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {createSession} from '../../../scripts/coverage/session.mjs';
import {graphicsPlugin} from '../../../bindings/node/plugins.js';

const reference = JSON.parse(fs.readFileSync(new URL('../fixtures/webgl.safari.json', import.meta.url)));
async function evaluate(source) {
  const session = await createSession({plugins: [graphicsPlugin()]});
  try { return await (await session.newPage()).evaluate(source); }
  finally { await session.close(); }
}

test('WebGL constants match installed Safari values, ownership and readonly descriptors', async () => {
  assert.deepEqual(await evaluate(`(() => {
    const expected=${JSON.stringify(reference.constants)}, failures=[];
    const context=document.createElement('canvas').getContext('webgl');
    if(!context)throw Error('WebGL context unavailable');
    for(const [name,record] of Object.entries(expected)) {
      for(const [target,expectedDescriptor] of [[WebGLRenderingContext,record.constructor],[WebGLRenderingContext.prototype,record]]) {
        const d=Object.getOwnPropertyDescriptor(target,name);
        for(const key of ['value','enumerable','configurable','writable']) {
          if(!d || d[key]!==expectedDescriptor[key]) failures.push([name,key,d?.[key]]);
        }
        if(Reflect.set(target,name,-123))failures.push([name,'writable']);
        if(Reflect.deleteProperty(target,name))failures.push([name,'configurable']);
      }
      if(context[name]!==record.value)failures.push([name,'instance']);
    }
    return failures;
  })()`), []);
});

test('placeholder method surfaces and receiver checks match installed Safari', async () => {
  assert.deepEqual(await evaluate(`(() => {
    const expected=${JSON.stringify(reference.placeholderMethods)}, failures=[];
    if('makeXRCompatible' in WebGLRenderingContext.prototype)failures.push(['makeXRCompatible','not exposed by reference Safari']);
    for(const [name,record] of Object.entries(expected)) {
      const d=Object.getOwnPropertyDescriptor(WebGLRenderingContext.prototype,name);
      if(typeof d?.value!=='function'){failures.push([name,'missing']);continue;}
      for(const key of ['enumerable','configurable','writable'])if(d[key]!==record[key])failures.push([name,key]);
      if(d.value.name!==record.name || d.value.length!==record.length)failures.push([name,'signature']);
      for(const receiver of [{},null,Object.create(WebGLRenderingContext.prototype)]) {
        try{d.value.apply(receiver,Array(record.length).fill(0));failures.push([name,'receiver']);}
        catch(e){if(e.name!=='TypeError')failures.push([name,e.name]);}
      }
    }
    return failures;
  })()`), []);
});
