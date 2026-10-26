import test from 'node:test';
import assert from 'node:assert/strict';
import {createSession} from '../index.js';

test('MouseEvent layout coordinates retain explicit legacy zero placeholders', async () => {
  const session=await createSession();
  try {
    assert.deepEqual(await (await session.newPage()).evaluate(`(() => {
      const e=new MouseEvent('mousemove',{clientX:50,clientY:60}),node=document.createElement('div');
      document.body.append(node);node.dispatchEvent(e);
      return ['pageX','pageY','offsetX','offsetY','layerX','layerY'].map(n=>[e[n],Reflect.set(e,n,99)]);
    })()`), Array.from({length:6},()=>[0,false]));
  } finally { await session.close(); }
});
