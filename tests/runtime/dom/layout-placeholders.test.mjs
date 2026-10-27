import test from 'node:test';
import assert from 'node:assert/strict';
import {createSession} from '../../../bindings/node/index.js';

// Deliberate legacy-placeholder contract, not rendered-layout equivalence.
test('layout placeholders let geometry-reading scripts continue on connected styled elements', async () => {
  const session=await createSession();
  try {
    const page=await session.newPage();
    assert.deepEqual(await page.evaluate(`(() => {
      const e=document.createElement('div');
      e.style.cssText='position:absolute;width:120px;height:40px;left:12px;top:8px';
      document.body.appendChild(e);
      const result={area:e.offsetWidth*e.offsetHeight,position:e.offsetLeft+e.offsetTop,parent:e.offsetParent};
      e.remove();return result;
    })()`),{area:0,position:0,parent:null});
  } finally {await session.close();}
});
