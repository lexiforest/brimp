import test from 'node:test';
import assert from 'node:assert/strict';
import { createSession } from '../../../bindings/node/index.js';
// Attributes drive the IDL/decoded dimensions; the box follows the declared
// CSS size, and there is still no image decoder.
test('image input dimensions use attributes while the box follows its declared size', async () => {
  const s = await createSession();
  try {
    const p = await s.newPage();
    assert.deepEqual(
      await p.evaluate(`(()=>{
 const i=document.createElement('input');i.type='image';i.src='data:image/svg+xml,<svg xmlns="http://www.w3.org/2000/svg" width="60" height="40"/>';
 i.style.cssText='width:120px;height:80px';document.body.append(i);
 const unset=[i.width,i.height];i.width=12;i.height=8;return [unset,[i.width,i.height],i.offsetWidth,i.offsetHeight];
 })()`),
      [[0, 0], [12, 8], 120, 80],
    );
  } finally {
    await s.close();
  }
});
