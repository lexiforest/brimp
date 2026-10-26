import test from 'node:test';
import assert from 'node:assert/strict';
import {createSession} from '../index.js';

test('token lists reject non-extensibility while retaining live indexed properties', async () => {
  const session = await createSession();
  try {
    const page = await session.newPage();
    assert.deepEqual(await page.evaluate(`(() => {
      const a=document.createElement('a');a.rel='a';const list=a.relList;
      const prevented=Reflect.preventExtensions(list);
      a.rel='b c';
      return [prevented,Object.isExtensible(list),Object.keys(list),list[1],list.value];
    })()`), [false,true,['0','1'],'c','b c']);
  } finally { await session.close(); }
});
