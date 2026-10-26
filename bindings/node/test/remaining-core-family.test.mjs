import test from 'node:test';
import assert from 'node:assert/strict';
import {createSession} from '../index.js';
async function evaluate(source){const session=await createSession();try{return await(await session.newPage()).evaluate(source);}finally{await session.close();}}

test('Script async state distinguishes created and parsed elements without changing the loader',async()=>{
  assert.deepEqual(await evaluate(`(() => {
    const script=document.createElement('script'),ns=document.createElementNS('http://www.w3.org/1999/xhtml','script');
    const parsed=new DOMParser().parseFromString('<script></script><script async></script>','text/html');
    const values=[script.async,ns.async,parsed.scripts[0].async,parsed.scripts[1].async];
    script.async=false;values.push(script.async,script.hasAttribute('async'));
    script.async=true;values.push(script.async,script.hasAttribute('async'));script.removeAttribute('async');values.push(script.async);
    ns.setAttribute('async','');ns.removeAttribute('async');values.push(ns.async);
    const host=document.createElement('div');host.innerHTML='<script></script>';values.push(host.firstChild.async);
    return values;
  })()`),[true,true,false,true,false,false,true,true,false,false,false]);
});

test('Frame accessors and DOM interface remnants share existing tree, sheet and slot behavior',async()=>{
  assert.deepEqual(await evaluate(`(() => {
    const frame=document.createElement('frame'),iframe=document.createElement('iframe');document.body.append(iframe);
    const child=iframe.contentDocument,win=iframe.contentWindow;const htmlSvg=iframe.getSVGDocument();
    const svg=child.createElementNS('http://www.w3.org/2000/svg','svg');child.replaceChildren(svg);
    const host=document.createElement('div'),text=document.createTextNode('assigned');host.append(text);document.body.append(host);
    const shadow=host.attachShadow({mode:'open'});shadow.innerHTML='<slot></slot>';
    const link=document.createElement('link');link.disabled=true;
    const image=new Image(3,4),label=document.createElement('label');
    return [frame.contentDocument,frame.contentWindow,child===iframe.contentDocument,win===iframe.contentWindow,win.document===child,
      htmlSvg,iframe.getSVGDocument()===child,text.assignedSlot===shadow.firstChild,
      link instanceof HTMLLinkElement,link.disabled,link.sheet,label instanceof HTMLLabelElement,
      Object.getPrototypeOf(image)===Image.prototype,image.width,image.height];
  })()`),[null,null,true,true,true,null,true,true,true,true,null,true,true,3,4]);
});

test('Custom element registry retains definition, upgrade and lifecycle workflows',async()=>{
  assert.deepEqual(await evaluate(`(async () => {
    const element=document.createElement('family-widget');let connected=0;
    const ready=customElements.whenDefined('family-widget');
    class Widget extends HTMLElement {constructor(){super();this.answer=42;}connectedCallback(){connected++;}}
    customElements.define('family-widget',Widget);const definition=await ready;
    customElements.upgrade(element);document.body.append(element);
    return [customElements instanceof CustomElementRegistry,customElements.get('family-widget')===Widget,
      definition===Widget,element instanceof Widget,element.answer,connected];
  })()`),[true,true,true,true,42,1]);
});

test('Blob and Crypto preserve binary state, randomness validation and real digest behavior',async()=>{
  assert.deepEqual(await evaluate(`(async () => {
    const blob=new Blob([new Uint8Array([0,255]),'A'],{type:'APPLICATION/OCTET-STREAM'});
    const random=new Uint8Array(16),same=crypto.getRandomValues(random)===random,uuid=crypto.randomUUID();let invalid;
    try{crypto.getRandomValues(new Float32Array(1))}catch(e){invalid=e.name}
    const hash=await crypto.subtle.digest('SHA-256',new TextEncoder().encode('abc'));
    return [blob.size,blob.type,Array.from(new Uint8Array(await blob.arrayBuffer())),'code' in blob,
      crypto instanceof Crypto,same,/^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/.test(uuid),invalid,
      Array.from(new Uint8Array(hash),b=>b.toString(16).padStart(2,'0')).join('')];
  })()`),[3,'application/octet-stream',[0,255,65],false,true,true,true,'TypeMismatchError',
    'ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad']);
});

test('XHR event target interfaces and constants preserve manual upload handler dispatch',async()=>{
  assert.deepEqual(await evaluate(`(() => {
    const xhr=new XMLHttpRequest(),upload=xhr.upload;let calls=0;const initial=[];
    for(const name of ['abort','error','load','loadend','loadstart','progress','timeout']){
      initial.push(upload['on'+name]);upload['on'+name]=()=>calls++;upload.dispatchEvent(new ProgressEvent(name));
      upload['on'+name]=null;upload.dispatchEvent(new ProgressEvent(name));
    }
    return [[XMLHttpRequest.UNSENT,XMLHttpRequest.OPENED,XMLHttpRequest.HEADERS_RECEIVED,XMLHttpRequest.LOADING,XMLHttpRequest.DONE],
      xhr instanceof XMLHttpRequestEventTarget,upload instanceof XMLHttpRequestEventTarget,upload instanceof XMLHttpRequestUpload,
      initial,calls,Object.prototype.toString.call(upload)];
  })()`),[[0,1,2,3,4],true,true,true,Array(7).fill(null),7,'[object XMLHttpRequestUpload]']);
});
