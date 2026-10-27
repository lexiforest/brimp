import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import YAML from 'yaml';
import {createSession} from '../../../bindings/node/index.js';

const fields = Object.values(YAML.parse(fs.readFileSync(new URL('../../../COVERAGE.yaml', import.meta.url), 'utf8')).fields);
const handlers = fields.filter(f => f.target === 'window' && f.name.startsWith('on') && f.safari.presence === 'present').map(f => f.name);
const constructors = fields.filter(f => f.target === 'window' && f.legacy.kinds.includes('constructor') && f.brimp.core.presence === 'present').map(f => f.name);

test('window handler family reuses ordinary DOM dispatch and supports clearing', async () => {
  const session = await createSession();
  try {
    const page = await session.newPage();
    assert.deepEqual(await page.evaluate(`(() => {
      const failed=[];
      const aliases={onwebkitanimationend:'webkitAnimationEnd',onwebkitanimationiteration:'webkitAnimationIteration',
        onwebkitanimationstart:'webkitAnimationStart',onwebkittransitionend:'webkitTransitionEnd'};
      for(const name of ${JSON.stringify(handlers)}) {
        let calls=0;if(window[name]!==null)failed.push(name+':default');
        window[name]=function(){if(this===window)calls++;};
        window.dispatchEvent(new Event(aliases[name]??name.slice(2)));
        window[name]=null;window.dispatchEvent(new Event(aliases[name]??name.slice(2)));
        if(calls!==1)failed.push(name+':dispatch');
      }
      return failed;
    })()`),[]);
  } finally { await session.close(); }
});

test('window identity and UI placeholders leave DOM and runtime state usable', async () => {
  const session = await createSession();
  try {
    const page = await session.newPage();
    assert.deepEqual(await page.evaluate(`(() => {
      const before=document.documentElement.outerHTML;
      const names=['alert','blur','captureEvents','close','confirm','find','focus','getComputedStyle',
        'moveBy','moveTo','open','print','prompt','releaseEvents','reportError','resizeBy','resizeTo','scroll','scrollBy','scrollTo','stop'];
      const noops=names.every(name=>window[name]('ignored')===undefined)&&alert('ignored')===undefined&&getComputedStyle(document.body)===undefined&&typeof matchMedia('all')==='object';
      window.name=42;window.status='ready';
      const frame=document.createElement('iframe');document.body.appendChild(frame);
      const frameIdentity=frame.contentWindow.frameElement===frame;frame.remove();
      return [noops,window===self,window===parent,window===top,window===frames,window.window===window,
        clientInformation===navigator,origin===location.origin,frameElement,closed,window.name,window.status,
        frameIdentity,before===document.documentElement.outerHTML,window.length,
        ${JSON.stringify(constructors)}.every(name=>typeof window[name]==='function'&&window[name].prototype),
        document instanceof Document,document.body instanceof HTMLBodyElement,
        document.createElement('input') instanceof HTMLInputElement];
    })()`),[true,true,true,true,true,true,true,true,null,false,'42','ready',true,true,0,true,true,true,true]);
  } finally { await session.close(); }
});

test('window timer aliases retain realm-clock scheduling and cancellation', async () => {
  const session = await createSession({clock:{mode:'logical'}});
  try {
    const page = await session.newPage();
    await page.evaluate(`globalThis.work=[];
      queueMicrotask(()=>work.push('microtask'));
      const cancelled=setTimeout(()=>work.push('cancelled'),5);clearTimeout(cancelled);
      const interval=setInterval(()=>{work.push('interval');clearInterval(interval);},5);
      const cancelledFrame=webkitRequestAnimationFrame(()=>work.push('cancelled-frame'));webkitCancelAnimationFrame(cancelledFrame);
      requestAnimationFrame(time=>work.push(['frame',time]));
      webkitRequestAnimationFrame(time=>work.push(['webkit-frame',time]));null`);
    await page.advanceTime(20);
    assert.deepEqual(await page.evaluate('work'),['microtask','interval',['frame',16],['webkit-frame',16]]);
  } finally { await session.close(); }
});
