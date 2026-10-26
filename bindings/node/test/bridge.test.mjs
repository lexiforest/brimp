import {test} from 'node:test';
import assert from 'node:assert/strict';
import {createSession, Plugin, JavaScriptError, StaleHandle, Closed, Reentry, Timeout} from '../index.js';

test('rich values, binary copies, references and explicit receivers', async () => {
  const session = await createSession();
  try {
    const page = await session.newPage();
    assert.deepEqual(await page.evaluate('[undefined,null,NaN,Infinity,-Infinity,-0,123n,new Uint8Array([0,255])]'), [undefined,null,NaN,Infinity,-Infinity,-0,123n,Buffer.from([0,255])]);
    const fn = await page.evaluateHandle('(function(data){return [this.tag,data,typeof data.big]})');
    const receiver = await page.evaluateHandle('({tag:"receiver"})');
    const input = {big: 2n**80n, bytes: Buffer.from([0,255]), missing: undefined};
    const result = await fn.invoke([input], {thisArg: receiver});
    assert.deepEqual(await result.value(), ['receiver',input,'bigint']);
    await result.dispose();
    await receiver.setProperty('self', receiver);
    const same = await receiver.call('(value,nested) => value === nested.ref && value.self === value', {ref:receiver});
    assert.equal(await same.value(), true); await same.dispose();
    assert.equal(await receiver.deleteProperty('self'), true);
    assert.deepEqual(await receiver.propertyNames(), ['tag']);
    await receiver.dispose(); await assert.rejects(receiver.value(), StaleHandle);
    await assert.rejects(page.evaluate('new Map()'), JavaScriptError);
    const other = await session.newPage(), foreign = await other.evaluateHandle('({})');
    await assert.rejects(fn.invoke([foreign]), StaleHandle);
    await foreign.dispose(); await fn.dispose();
  } finally { await session.close(); }
});

test('plugin references and registered callable results share the realm', async () => {
  let held;
  const plugin = new Plugin('bridge', `({global,host}) => {
    global.inspect = (obj, fn) => host.call('inspect', host.ref(obj), host.ref(fn));
    global.cycle = () => host.call('cycle', host.ref(() => host.call('increment', 1)));
  }`, {
    async inspect(context, obj, fn) {
      held = obj;
      await obj.setProperty('bytes', Buffer.from([0,255]));
      const result = await fn.invoke([41]); assert.equal(await result.value(),42);
      await result.dispose(); await fn.dispose();
      return {same:obj, increment:context.function('increment'), large:2n**70n};
    },
    increment: (_, n) => n+1,
    async cycle(_, fn) { await assert.rejects(fn.invoke(), Reentry); await fn.dispose(); return 'rejected'; },
  });
  const session = await createSession({plugins:[plugin],timeout:2000});
  try {
    const page = await session.newPage();
    assert.deepEqual(await page.evaluate(`(async () => {
      const obj={};const result=await inspect(obj,n=>n+1);
      return [result.same===obj,obj.bytes,await result.increment(41),result.large];
    })()`), [true,Buffer.from([0,255]),42,2n**70n]);
    assert.equal(await page.evaluate('cycle()'),'rejected');
    assert.deepEqual(await held.value(),{bytes:Buffer.from([0,255])});
    await page.close(); await assert.rejects(held.value(),Closed);
  } finally { await session.close(); }
});

test('concurrent reference operations, timeouts and recovery', async () => {
  const session = await createSession({timeout:200});
  try {
    const page = await session.newPage(), obj = await page.evaluateHandle('({count:0})');
    const values = await Promise.all(Array.from({length:12},async () => {
      const result=await obj.call('obj=>++obj.count');try{return await result.value()}finally{await result.dispose()}
    }));
    assert.deepEqual(values.sort((a,b)=>a-b),Array.from({length:12},(_,i)=>i+1));
    const pending = await page.evaluateHandle('() => new Promise(() => {})');
    await assert.rejects(pending.invoke(),Timeout);
    assert.equal(await page.evaluate('42'),42);
    await pending.dispose(); await obj.dispose();
  } finally { await session.close(); }
});
