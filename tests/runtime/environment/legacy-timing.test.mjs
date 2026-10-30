import test from 'node:test';
import assert from 'node:assert/strict';
import { createSession } from '../../../bindings/node/index.js';

test('legacy timing is a stable synthetic snapshot rather than measured network phases', async () => {
  const session = await createSession({ clock: { mode: 'logical', epochMs: 1000 } });
  try {
    const page = await session.newPage();
    const snapshot = await page.evaluate(`(() => {
      const t=performance.timing,j=t.toJSON();
      return [t.navigationStart===performance.timeOrigin,Object.keys(j).length,
        Object.values(j).every(v=>v===0||v===performance.timeOrigin||v>performance.timeOrigin),JSON.stringify(t)===JSON.stringify(j),j];
    })()`);
    assert.deepEqual(snapshot.slice(0, 4), [true, 21, true, true]);
    await page.advanceTime(500);
    assert.deepEqual(await page.evaluate('performance.timing.toJSON()'), snapshot[4]);
  } finally {
    await session.close();
  }
});
