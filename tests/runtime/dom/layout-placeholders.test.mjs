import test from 'node:test';
import assert from 'node:assert/strict';
import { createSession } from '../../../bindings/node/index.js';

test('an absolutely positioned box reports its containing-block offset and size', async () => {
  const session = await createSession();
  try {
    const page = await session.newPage();
    assert.deepEqual(
      await page.evaluate(`(() => {
      const e=document.createElement('div');
      e.style.cssText='position:absolute;width:120px;height:40px;left:12px;top:8px';
      document.body.appendChild(e);
      const result={area:e.offsetWidth*e.offsetHeight,position:e.offsetLeft+e.offsetTop,
        parent:e.offsetParent===document.body,rect:e.getBoundingClientRect().toJSON()};
      e.remove();return result;
    })()`),
      {
        area: 4800,
        position: 20,
        parent: true,
        rect: { x: 20, y: 16, width: 120, height: 40, top: 16, right: 140, bottom: 56, left: 20 },
      },
    );
  } finally {
    await session.close();
  }
});

test('a detached element stays in flow-less state and reports no box', async () => {
  const session = await createSession();
  try {
    const page = await session.newPage();
    assert.deepEqual(
      await page.evaluate(`(() => {
      const e=document.createElement('div');
      e.style.cssText='width:120px;height:40px;padding:5px';
      const detached={rect:e.getBoundingClientRect().toJSON(),offsetWidth:e.offsetWidth,
        clientWidth:e.clientWidth,parent:e.offsetParent,rects:e.getClientRects().length};
      document.body.appendChild(e);
      const connected={offsetWidth:e.offsetWidth,offsetHeight:e.offsetHeight,
        clientWidth:e.clientWidth,scrollWidth:e.scrollWidth,parent:e.offsetParent===document.body};
      e.remove();return {detached,connected};
    })()`),
      {
        detached: {
          rect: { x: 0, y: 0, width: 0, height: 0, top: 0, right: 0, bottom: 0, left: 0 },
          offsetWidth: 0,
          clientWidth: 0,
          parent: null,
          rects: 0,
        },
        connected: {
          offsetWidth: 130,
          offsetHeight: 50,
          clientWidth: 130,
          scrollWidth: 130,
          parent: true,
        },
      },
    );
  } finally {
    await session.close();
  }
});
