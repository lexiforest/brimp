import test from 'node:test';
import assert from 'node:assert/strict';
import { createSession } from '../../../bindings/node/index.js';
import { canvasPlugin } from '../../../bindings/node/plugins.js';

for (const canvas of [false, true])
  test(`image resource and geometry placeholders remain core with canvas ${canvas ? 'enabled' : 'disabled'}`, async () => {
    const s = await createSession({ plugins: canvas ? [canvasPlugin()] : [] });
    try {
      const p = await s.newPage();
      assert.deepEqual(
        await p.evaluate(`(()=>{
    const image=new Image(12,8),events=[];image.onload=()=>events.push('load');image.onerror=()=>events.push('error');
    image.src='data:image/svg+xml,<svg xmlns="http://www.w3.org/2000/svg" width="60" height="40"/>';
    image.srcset='large.png 2x';image.style.cssText='position:absolute;left:30px;top:40px;width:120px;height:80px';document.body.append(image);
    const observer=new MutationObserver(()=>{});observer.observe(image,{attributes:true,childList:true});
    const poison={[Symbol.toPrimitive](){throw Error('unexpected conversion')}};
    const result=image.decode(poison,Symbol()),snapshot=e=>[e.complete,e.currentSrc,e.naturalWidth,e.naturalHeight,e.x,e.y,e.width,e.height];
    const before=snapshot(image),clone=image.cloneNode();document.implementation.createHTMLDocument().adoptNode(image);
    const after=snapshot(image);const records=observer.takeRecords().length;observer.disconnect();
    clone.removeAttribute('width');clone.removeAttribute('height');const unrendered=snapshot(clone);
    image.removeAttribute('src');image.removeAttribute('srcset');
    return [result===undefined,before,after,unrendered,events,records,snapshot(image)];
  })()`),
        [
          true,
          [true, '', 0, 0, 0, 0, 12, 8],
          [true, '', 0, 0, 0, 0, 12, 8],
          [true, '', 0, 0, 0, 0, 0, 0],
          [],
          0,
          [true, '', 0, 0, 0, 0, 12, 8],
        ],
      );
    } finally {
      await s.close();
    }
  });
