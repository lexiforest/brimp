import test from 'node:test';
import assert from 'node:assert/strict';
import { createSession } from '../../../scripts/coverage/session.mjs';

test('input reflection does not upgrade or notify unsupported customized built-ins', async () => {
  const session = await createSession();
  try {
    assert.deepEqual(
      await (
        await session.newPage()
      ).evaluate(`(()=>{
    const log=[];class ReflectedInput extends HTMLInputElement{
      static get observedAttributes(){return ['accept','disabled'];}
      attributeChangedCallback(...args){log.push(args.slice(0,3));}
    }
    customElements.define('reflected-input',ReflectedInput,{extends:'input'});
    const i=document.createElement('input',{is:'reflected-input'});
    i.accept='a';i.accept='a';i.accept='b';i.disabled=true;i.disabled=true;i.disabled=false;i.disabled=false;
    return [log,i instanceof ReflectedInput,i.getAttribute('is'),i.accept,i.disabled];
  })()`),
      [[], false, null, 'b', false],
    );
  } finally {
    await session.close();
  }
});
