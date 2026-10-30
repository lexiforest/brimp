import test from 'node:test';
import assert from 'node:assert/strict';
import { createSession } from '../../../bindings/node/index.js';

test('PointerEvent sampling methods retain callable legacy undefined placeholders', async () => {
  const session = await createSession();
  try {
    assert.deepEqual(
      await (
        await session.newPage()
      ).evaluate(`(() => {
      const e=new PointerEvent('pointermove');
      return ['getCoalescedEvents','getPredictedEvents'].map(name=>{
        let error;try{PointerEvent.prototype[name].call({});}catch(e){error=e.name;}
        const d=Object.getOwnPropertyDescriptor(PointerEvent.prototype,name);
        return [e[name]()===undefined,error,d.enumerable,e[name].length];
      });
    })()`),
      [
        [true, 'TypeError', true, 0],
        [true, 'TypeError', true, 0],
      ],
    );
  } finally {
    await session.close();
  }
});
