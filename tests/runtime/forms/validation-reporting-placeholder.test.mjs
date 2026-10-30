import test from 'node:test';
import assert from 'node:assert/strict';
import { createSession } from '../../../bindings/node/index.js';
// Deliberate no-UI presentation contract; Safari focuses invalid controls.
test('validation reporting retains focus and uses a no-op visual presentation placeholder', async () => {
  const session = await createSession();
  try {
    const page = await session.newPage();
    assert.deepEqual(
      await page.evaluate(`(()=>{
  document.body.innerHTML='<input id="invalid" required><button id="focused">other</button>';
  const input=document.getElementById('invalid'),focused=document.getElementById('focused');focused.focus();
  const events=[];input.addEventListener('invalid',e=>events.push(e.isTrusted));
  const observer=new MutationObserver(()=>{});observer.observe(document.body,{childList:true,subtree:true,attributes:true});
  const result=input.reportValidity();return [result,events,document.activeElement===focused,observer.takeRecords().length];
 })()`),
      [false, [true], true, 0],
    );
  } finally {
    await session.close();
  }
});
