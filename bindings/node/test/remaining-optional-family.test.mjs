import test from 'node:test';
import assert from 'node:assert/strict';
import {createSession, storagePlugin, servicesPlugin, canvasPlugin, graphicsPlugin, schedulingPlugin} from '../index.js';
async function evaluate(source,plugins){const session=await createSession({plugins});try{return await(await session.newPage()).evaluate(source);}finally{await session.close();}}

test('CookieStore and SQLTransaction retain explicit legacy contracts without storage side effects',async()=>{
  assert.deepEqual(await evaluate(`(() => {
    const before=document.cookie;let events=0;cookieStore.onchange=()=>events++;
    const noops=['get','getAll','set','delete'].every(n=>cookieStore[n]('name','value')===undefined);
    const produced=events;cookieStore.dispatchEvent(new Event('change'));cookieStore.onchange=null;cookieStore.dispatchEvent(new Event('change'));
    let illegal=0;for(const C of [CookieStore,SQLTransaction])try{new C()}catch(e){if(e.name==='TypeError')illegal++}
    const fixture=Object.create(SQLTransaction.prototype);let result;
    const returned=fixture.executeSql('ignored',[],(transaction,value)=>{result=[transaction===fixture,value];return 42;});
    return [cookieStore instanceof CookieStore,cookieStore instanceof EventTarget,noops,produced,events,document.cookie===before,
      illegal,returned,result,fixture.executeSql('ignored')===undefined,'openDatabase' in window,openDatabase===undefined];
  })()`,[storagePlugin()]),[true,true,true,0,1,true,2,42,[true,{insertId:'',rows:{value:'0'},rowsAffected:''}],true,true,true]);
});

test('Trusted Types and Worker placeholders accept calls without policies, code execution or threads',async()=>{
  assert.deepEqual(await evaluate(`(() => {
    const worker=new Worker('globalThis.unexpectedWorkerExecution=true');let messages=0,errors=0;
    worker.onmessage=()=>messages++;worker.onerror=()=>errors++;
    const noops=[worker.postMessage('ignored'),worker.terminate(),trustedTypes.createPolicy('name',{createHTML:()=>{throw Error('unused')}}),
      trustedTypes.getAttributeType('script','src'),trustedTypes.getPropertyType('script','src'),trustedTypes.isHTML(''),trustedTypes.isScript(''),trustedTypes.isScriptURL('')];
    const produced=[messages,errors];worker.dispatchEvent(new MessageEvent('message',{data:'local'}));worker.dispatchEvent(new Event('error'));
    worker.onmessage=null;worker.dispatchEvent(new MessageEvent('message'));
    return [worker instanceof EventTarget,noops.every(v=>v===undefined),produced,messages,errors,
      typeof unexpectedWorkerExecution,'_onmsg' in worker,'code_function' in worker,worker.onerror!==null,
      trustedTypes instanceof TrustedTypePolicyFactory,trustedTypes.defaultPolicy,trustedTypes.emptyHTML,trustedTypes.emptyScript,
      'getTypeMapping' in trustedTypes];
  })()`,[servicesPlugin()]),[true,true,[0,0],1,1,'undefined',false,false,true,true,null,'','',false]);
});

test('Canvas and Path2D preserve configured readback and callable paths without rasterization',async()=>{
  assert.deepEqual(await evaluate(`(async () => {
    const canvas=document.createElement('canvas');canvas.width=2;canvas.height=3;const c=canvas.getContext('2d'),p=new Path2D();
    p.moveTo(0,0);p.lineTo(1,1);p.quadraticCurveTo(1,0,2,1);p.bezierCurveTo(0,1,1,2,2,2);
    p.rect(0,0,1,1);p.roundRect(0,0,2,2,0.5);p.arc(1,1,1,0,Math.PI);p.arcTo(1,0,2,1,1);p.ellipse(1,1,1,0.5,0,0,Math.PI);p.closePath();
    const copy=new Path2D(p);copy.addPath(p);c.fill(copy);c.stroke(copy);
    const blob=await new Promise(resolve=>canvas.toBlob(resolve));const encoded=JSON.parse(atob(canvas.toDataURL().split(',')[1]));
    const bitmap=await createImageBitmap(canvas);const size=[bitmap.width,bitmap.height];bitmap.close();
    return [canvas.getContext('2d')===c,c instanceof CanvasRenderingContext2D,copy instanceof Path2D,
      Array.from(c.getImageData(0,0,1,1).data),c.isPointInPath(copy,1,1),encoded,blob instanceof Blob,blob.size>0,size,
      canvas.captureStream()===undefined,canvas.transferControlToOffscreen()===undefined,canvas.width,canvas.height];
  })()`,[canvasPlugin({rgba:[1,2,3,4]})]),[true,true,true,[1,2,3,4],false,{width:2,height:3,rgba:[1,2,3,4]},true,true,[2,3],true,true,2,3]);
});

test('WebGL debug renderer extension reports configured mock identity with Safari constants',async()=>{
  assert.deepEqual(await evaluate(`(() => {
    const gl=document.createElement('canvas').getContext('webgl'),ext=gl.getExtension('WEBGL_debug_renderer_info');
    return [gl instanceof WebGLRenderingContext,gl.getSupportedExtensions(),ext===gl.getExtension('WEBGL_debug_renderer_info'),
      Object.prototype.toString.call(ext),ext.constructor===Object,ext.UNMASKED_VENDOR_WEBGL,ext.UNMASKED_RENDERER_WEBGL,
      gl.getParameter(ext.UNMASKED_VENDOR_WEBGL),gl.getParameter(ext.UNMASKED_RENDERER_WEBGL),gl.getExtension('unknown'),
      typeof WebGLDebugRendererInfo,Object.getOwnPropertyDescriptor(ext,'UNMASKED_VENDOR_WEBGL').writable];
  })()`,[canvasPlugin(),graphicsPlugin({renderer:'configured mock'})]),[true,['WEBGL_debug_renderer_info'],true,'[object WebGLDebugRendererInfo]',true,37445,37446,'Brimp','configured mock',null,'undefined',false]);
});

test('Scheduling retains its existing opt-in task and idle extension despite Safari absence',async()=>{
  assert.deepEqual(await evaluate(`(async () => {
    const order=[],abort=new AbortController();abort.abort('cancelled');let reason;
    try{await scheduler.postTask(()=>order.push('cancelled'),{signal:abort.signal})}catch(e){reason=e}
    const low=scheduler.postTask(()=>order.push('low'),{priority:'background'});
    const high=scheduler.postTask(()=>order.push('high'),{priority:'user-blocking'});await Promise.all([low,high]);
    const id=requestIdleCallback(()=>order.push('cancelled idle'));cancelIdleCallback(id);
    const idle=await new Promise(resolve=>requestIdleCallback(deadline=>resolve([deadline.timeRemaining(),deadline.didTimeout])));
    return [scheduler instanceof Scheduler,order,reason,idle];
  })()`,[schedulingPlugin()]),[true,['high','low'],'cancelled',[0,false]]);
  assert.deepEqual(await evaluate(`['cookieStore','SQLTransaction','Worker','trustedTypes','scheduler','Path2D'].map(n=>typeof globalThis[n])`,[]),Array(6).fill('undefined'));
});
