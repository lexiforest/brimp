import test from "node:test";
import assert from "node:assert/strict";
import { createSession } from "../../../bindings/node/index.js";

// These are intentional legacy contracts, not Safari rendering conformance.
test("ignored-result browser UI calls allow a DOM workflow to finish without UI effects", async () => {
  const session = await createSession();
  try {
    const page = await session.newPage();
    assert.deepEqual(
      await page.evaluate(`(() => {
      const button=document.createElement('button');
      button.textContent='Save';document.body.append(button);
      const events=[];
      for(const type of ['scroll','animationstart','gotpointercapture','lostpointercapture','fullscreenchange','pointerlockchange']) {
        button.addEventListener(type,()=>events.push(type));document.addEventListener(type,()=>events.push(type));
      }
      button.addEventListener('click',()=>{
        button.scrollIntoView({behavior:'smooth'});
        button.animate([{opacity:0},{opacity:1}],100);
        button.requestFullscreen();button.requestPointerLock();
        button.setPointerCapture(1);button.releasePointerCapture(1);
        document.execCommand('bold');
        document.exitFullscreen();document.exitPointerLock();document.exitPictureInPicture();
        button.textContent='Saved';button.dataset.saved='true';
      });
      button.click();document.clear();
      return [button.textContent,button.dataset.saved,button.isConnected,button.hasAttribute('style'),events];
    })()`),
      ["Saved", "true", true, false, []],
    );
  } finally {
    await session.close();
  }
});

test("Element geometry answers with a zero rectangle instead of nothing", async () => {
  const session = await createSession();
  try {
    const page = await session.newPage();
    assert.deepEqual(
      await page.evaluate(`(() => {
      const html=document.createElement('div'),svg=document.createElementNS('http://www.w3.org/2000/svg','svg'),fragment=document.createDocumentFragment();
      const rect=html.getBoundingClientRect(),rects=html.getClientRects();
      const brand=[];
      for(const receiver of [document,fragment,{},undefined]) {
        try { Element.prototype.getBoundingClientRect.call(receiver); brand.push('no throw'); }
        catch (error) { brand.push(error.name); }
      }
      return [rect instanceof DOMRect,rect instanceof DOMRectReadOnly,[rect.x,rect.y,rect.width,rect.height,rect.top,rect.right,rect.bottom,rect.left],
        rects instanceof DOMRectList,rects.length,rects.item(0),[...rects].length,
        svg.getBoundingClientRect().width,rect.toJSON().width,Object.prototype.toString.call(rects),brand];
    })()`),
      [
        true,
        true,
        [0, 0, 0, 0, 0, 0, 0, 0],
        true,
        0,
        null,
        0,
        0,
        0,
        "[object DOMRectList]",
        ["TypeError", "TypeError", "TypeError", "TypeError"],
      ],
    );
  } finally {
    await session.close();
  }
});

test("DOM UI placeholders are callable on HTML, SVG and documents and reject unrelated receivers", async () => {
  const session = await createSession();
  try {
    const page = await session.newPage();
    assert.deepEqual(
      await page.evaluate(`(() => {
      const elementMethods=['animate',
        'hasPointerCapture','releasePointerCapture','setPointerCapture','requestFullscreen','requestPointerLock',
        'webkitRequestFullScreen','webkitRequestFullscreen','scroll','scrollBy','scrollTo','scrollIntoView','scrollIntoViewIfNeeded'];
      const documentMethods=['captureEvents','releaseEvents','clear','caretRangeFromPoint','elementFromPoint','elementsFromPoint',
        'execCommand','queryCommandEnabled','queryCommandIndeterm','queryCommandState','queryCommandSupported','queryCommandValue',
        'exitFullscreen','exitPictureInPicture','exitPointerLock','webkitCancelFullScreen','webkitExitFullscreen'];
      const html=document.createElement('div'),svg=document.createElementNS('http://www.w3.org/2000/svg','svg');
      const results=[];
      for(const [receiver,prototype,names] of [[html,Element.prototype,elementMethods],[svg,Element.prototype,elementMethods],
        [document,Document.prototype,documentMethods]]) {
        for(const name of names) {
          results.push(receiver[name](1,2,{behavior:'smooth'})===undefined);
          const d=Object.getOwnPropertyDescriptor(prototype,name);
          results.push(d.enumerable&&d.configurable&&d.writable);
          try {d.value.call({});results.push(false);} catch(e) {results.push(e.name==='TypeError');}
        }
      }
      const animations=[html.getAnimations(),svg.getAnimations(),document.getAnimations()];
      return [results.length,results.every(Boolean),html.childNodes.length,svg.childNodes.length,
        animations.map(a=>Array.isArray(a)&&a.length)];
    })()`),
      [129, true, 0, 0, [0, 0, 0]],
    );
  } finally {
    await session.close();
  }
});
