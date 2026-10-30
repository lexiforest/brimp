import test from "node:test";
import assert from "node:assert/strict";
import {
  createSession,
  workersPlugin,
  trustedTypesPlugin,
  canvasPlugin,
  graphicsPlugin,
  environmentPlugin,
} from "../../../bindings/node/index.js";
async function evaluate(source, plugins, identity) {
  const session = await createSession({
    plugins,
    ...(identity
      ? { profile: { values: { identity: { user_agent: identity } } } }
      : {}),
  });
  try {
    const page = await session.newPage();
    await page.route("https://core.test/**", (r) =>
      r.fulfill({
        body: "<body></body>",
        headers: { "content-type": "text/html" },
      }),
    );
    await page.get("https://core.test/");
    return await page.evaluate(source);
  } finally {
    await session.close();
  }
}

test("CookieStore shares document cookies and SQL placeholders retain their contract", async () => {
  assert.deepEqual(
    await evaluate(
      `(async () => {
    await cookieStore.set('name','value');
    const value=(await cookieStore.get('name')).value;
    const shared=document.cookie;
    await cookieStore.delete('name');
    return [value,shared,await cookieStore.get('name'),typeof SQLTransaction,typeof openDatabase];
  })()`,
      [],
    ),
    ["value", "name=value", null, "function", "undefined"],
  );
});

test("Trusted Types and Worker placeholders accept calls without policies, code execution or threads", async () => {
  assert.deepEqual(
    await evaluate(
      `(() => {
    const worker=new Worker('globalThis.unexpectedWorkerExecution=true');let messages=0,errors=0;
    worker.onmessage=()=>messages++;worker.onerror=()=>errors++;
    const policy=trustedTypes.createPolicy('name',{createHTML:s=>s,createScript:s=>s,createScriptURL:s=>s});
    const noops=[worker.postMessage('ignored'),worker.terminate(),
      trustedTypes.getAttributeType('script','src'),trustedTypes.getPropertyType('script','src')];
    const produced=[messages,errors];worker.dispatchEvent(new MessageEvent('message',{data:'local'}));worker.dispatchEvent(new Event('error'));
    worker.onmessage=null;worker.dispatchEvent(new MessageEvent('message'));
    return [worker instanceof EventTarget,noops.every(v=>v===undefined),produced,messages,errors,
      typeof unexpectedWorkerExecution,'_onmsg' in worker,'code_function' in worker,worker.onerror!==null,
      trustedTypes instanceof TrustedTypePolicyFactory,trustedTypes.defaultPolicy,trustedTypes.emptyHTML,trustedTypes.emptyScript,
      'getTypeMapping' in trustedTypes,policy.name,policy.createHTML('x'),policy.createScript('x'),policy.createScriptURL('x'),
      trustedTypes.isHTML('x'),trustedTypes.isScript('x'),trustedTypes.isScriptURL('x')];
  })()`,
      [workersPlugin(), trustedTypesPlugin()],
    ),
    [
      true,
      true,
      [0, 0],
      1,
      1,
      "undefined",
      false,
      false,
      true,
      true,
      null,
      "",
      "",
      false,
      "name",
      "x",
      "x",
      "x",
      true,
      true,
      true,
    ],
  );
});

test("Canvas and Path2D preserve configured readback, painted fills and a real PNG", async () => {
  assert.deepEqual(
    await evaluate(
      `(async () => {
    const canvas=document.createElement('canvas');canvas.width=2;canvas.height=3;const c=canvas.getContext('2d'),p=new Path2D();
    p.moveTo(0,0);p.lineTo(1,1);p.quadraticCurveTo(1,0,2,1);p.bezierCurveTo(0,1,1,2,2,2);
    p.rect(0,0,1,1);p.roundRect(0,0,2,2,0.5);p.arc(1,1,1,0,Math.PI);p.arcTo(1,0,2,1,1);p.ellipse(1,1,1,0.5,0,0,Math.PI);p.closePath();
    const copy=new Path2D(p);copy.addPath(p);c.fill(copy);c.stroke(copy);
    c.fillStyle="#ff0000";c.fillRect(0,0,2,3);
    const blob=await new Promise(resolve=>canvas.toBlob(resolve));const dataUrl=canvas.toDataURL(),bytes=atob(dataUrl.split(',')[1]);
    const encoded={prefix:dataUrl.slice(0,22),signature:[bytes.charCodeAt(1),bytes.charCodeAt(2),bytes.charCodeAt(3)],width:(bytes.charCodeAt(16)<<24|bytes.charCodeAt(17)<<16|bytes.charCodeAt(18)<<8|bytes.charCodeAt(19))>>>0,height:(bytes.charCodeAt(20)<<24|bytes.charCodeAt(21)<<16|bytes.charCodeAt(22)<<8|bytes.charCodeAt(23))>>>0};
    const bitmap=await createImageBitmap(canvas);const size=[bitmap.width,bitmap.height];bitmap.close();
    return [canvas.getContext('2d')===c,c instanceof CanvasRenderingContext2D,copy instanceof Path2D,
      Array.from(c.getImageData(0,0,1,1).data),c.isPointInPath(copy,1,1),encoded,blob instanceof Blob,blob.size>0,size,
      canvas.captureStream()===undefined,canvas.transferControlToOffscreen()===undefined,canvas.width,canvas.height];
  })()`,
      [canvasPlugin()],
    ),
    [
      true,
      true,
      true,
      [255, 0, 0, 255],
      false,
      {
        prefix: "data:image/png;base64,",
        signature: [80, 78, 71],
        width: 2,
        height: 3,
      },
      true,
      true,
      [2, 3],
      true,
      true,
      2,
      3,
    ],
  );
});

test("WebGL debug renderer extension reports profile identity with Safari constants", async () => {
  assert.deepEqual(
    await evaluate(
      `(() => {
    const gl=document.createElement('canvas').getContext('webgl'),ext=gl.getExtension('WEBGL_debug_renderer_info');
    return [gl instanceof WebGLRenderingContext,gl.getSupportedExtensions().includes('WEBGL_debug_renderer_info'),ext===gl.getExtension('WEBGL_debug_renderer_info'),
      Object.prototype.toString.call(ext),ext.constructor===Object,ext.UNMASKED_VENDOR_WEBGL,ext.UNMASKED_RENDERER_WEBGL,
      gl.getParameter(ext.UNMASKED_VENDOR_WEBGL),gl.getParameter(ext.UNMASKED_RENDERER_WEBGL),gl.getExtension('unknown'),
      typeof WebGLDebugRendererInfo,Object.getOwnPropertyDescriptor(ext,'UNMASKED_VENDOR_WEBGL').writable];
  })()`,
      [canvasPlugin(), graphicsPlugin({ renderer: "configured mock" })],
    ),
    [
      true,
      true,
      true,
      "[object WebGLDebugRendererInfo]",
      true,
      37445,
      37446,
      "Apple Inc.",
      "ANGLE (Apple, ANGLE Metal Renderer: Apple M1, Unspecified Version)",
      null,
      "undefined",
      false,
    ],
  );
});

const CHROME_AGENT =
  "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/156.0.0.0 Safari/537.36";

test("Core scheduling runs prioritized tasks and idle callbacks under Chromium", async () => {
  assert.deepEqual(
    await evaluate(
      `(async () => {
    const order=[],abort=new AbortController();abort.abort('cancelled');let reason;
    try{await scheduler.postTask(()=>order.push('cancelled'),{signal:abort.signal})}catch(e){reason=e}
    const low=scheduler.postTask(()=>order.push('low'),{priority:'background'});
    const high=scheduler.postTask(()=>order.push('high'),{priority:'user-blocking'});await Promise.all([low,high]);
    const id=requestIdleCallback(()=>order.push('cancelled idle'));cancelIdleCallback(id);
    const idle=await new Promise(resolve=>requestIdleCallback(deadline=>resolve([deadline.timeRemaining(),deadline.didTimeout])));
    return [scheduler instanceof Scheduler,order,reason,idle];
  })()`,
      [],
      CHROME_AGENT,
    ),
    [true, ["high", "low"], "cancelled", [0, false]],
  );
  assert.deepEqual(
    await evaluate(
      `['cookieStore','SQLTransaction','Worker','trustedTypes','scheduler','Path2D'].map(n=>typeof globalThis[n])`,
      [],
      CHROME_AGENT,
    ),
    ["object", "function", "undefined", "undefined", "object", "undefined"],
  );
});

test("WebKit exposes no scheduling API, idle callbacks or network information", async () => {
  // Measured against Safari 27: none of these names exist, and the search
  // challenge reads requestIdleCallback and navigator.connection while probing.
  const names = [
    "Scheduler",
    "TaskController",
    "TaskSignal",
    "TaskPriorityChangeEvent",
    "scheduler",
    "requestIdleCallback",
    "cancelIdleCallback",
  ];
  assert.deepEqual(
    await evaluate(`${JSON.stringify(names)}.map(n=>typeof globalThis[n])`, []),
    names.map(() => "undefined"),
  );
  assert.equal(
    await evaluate(`typeof navigator.connection`, [environmentPlugin()]),
    "undefined",
  );
  // The same surfaces stay in place for a Chromium identity.
  assert.deepEqual(
    await evaluate(
      `${JSON.stringify(names)}.map(n=>typeof globalThis[n])`,
      [],
      CHROME_AGENT,
    ),
    [
      "function",
      "function",
      "function",
      "function",
      "object",
      "function",
      "function",
    ],
  );
  assert.equal(
    await evaluate(
      `typeof navigator.connection`,
      [environmentPlugin()],
      CHROME_AGENT,
    ),
    "object",
  );
});

test("core SQLTransaction preserves synthetic callbacks without executing SQL", async () => {
  assert.deepEqual(
    await evaluate(
      `(() => {
    let illegal=false;try{new SQLTransaction()}catch(error){illegal=error instanceof TypeError;}
    const transaction=Object.create(SQLTransaction.prototype);let result;
    const returned=transaction.executeSql('ignored',[],(receiver,value)=>{result=[receiver===transaction,value];return 42;});
    return [illegal,returned,result,transaction.executeSql('ignored')===undefined,
      Object.hasOwn(globalThis,'openDatabase'),openDatabase===undefined];
  })()`,
      [],
    ),
    [
      true,
      42,
      [true, { insertId: "", rows: { value: "0" }, rowsAffected: "" }],
      true,
      true,
      true,
    ],
  );
});
