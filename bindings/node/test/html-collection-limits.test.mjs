import test from 'node:test';
import assert from 'node:assert/strict';
import {createSession} from '../index.js';

test('HTMLCollection rejects non-extensibility while retaining live indexed and named properties',async()=>{
  const session=await createSession();try{
    assert.deepEqual(await(await session.newPage()).evaluate(`(()=>{
      const d=document.implementation.createHTMLDocument(),list=d.images;
      const results=[Reflect.preventExtensions(list),Object.isExtensible(list)];
      for(const operation of [Object.preventExtensions,Object.seal,Object.freeze])try{operation(list);results.push('accepted')}catch(e){results.push(e.name)}
      const image=d.createElement('img');image.id='live';d.body.append(image);results.push(list[0]===image,list.live===image,Reflect.ownKeys(list));return results;
    })()`),[false,true,'TypeError','TypeError','TypeError',true,true,['0','live']]);
  }finally{await session.close()}
});
