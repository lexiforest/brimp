import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {createSession} from '../../../bindings/node/index.js';
import {canvasPlugin, graphicsPlugin} from '../../../bindings/node/plugins.js';

const reference = JSON.parse(fs.readFileSync(new URL('../fixtures/webgl.safari.json', import.meta.url)));
async function evaluate(source, plugins = [graphicsPlugin()]) {
  const session = await createSession({plugins});
  try { return await (await session.newPage()).evaluate(source); }
  finally { await session.close(); }
}

// These deliberate legacy contracts do not claim Safari execution semantics.
test('every added WebGL placeholder returns undefined without processing arguments or GL state', async () => {
  assert.deepEqual(await evaluate(`(() => {
    const methods=${JSON.stringify(reference.placeholderMethods)}, failures=[];
    for(const type of ['webgl','webgl2']) {
      const gl=document.createElement('canvas').getContext(type);
      gl.viewport(1,2,3,4);gl.clearColor(0.25,0.5,0.75,1);
      const poison={ [Symbol.toPrimitive](){throw Error('unexpected conversion');} };
      for(const [name,{length}] of Object.entries(methods)) {
        const args=Array(length).fill(poison);
        if(gl[name](...args)!==undefined || gl[name]()!==undefined)failures.push([type,name,'return']);
      }
      if(gl.getError()!==gl.NO_ERROR)failures.push([type,'error state']);
      if(String(gl.getParameter(gl.VIEWPORT))!=='1,2,3,4')failures.push([type,'viewport']);
      if(String(gl.getParameter(gl.COLOR_CLEAR_VALUE))!=='0.25,0.5,0.75,1')failures.push([type,'clear color']);
    }
    return failures;
  })()`), []);
});

test('ignored-result graphics workload continues while existing buffers and readback still work', async () => {
  assert.deepEqual(await evaluate(`(() => {
    const canvas=document.createElement('canvas');canvas.width=2;canvas.height=1;
    const gl=canvas.getContext('webgl');
    const texture=gl.createTexture();gl.activeTexture(gl.TEXTURE0);gl.bindTexture(gl.TEXTURE_2D,texture);
    gl.texParameteri(gl.TEXTURE_2D,gl.TEXTURE_MIN_FILTER,gl.LINEAR);
    gl.texImage2D(gl.TEXTURE_2D,0,gl.RGBA,1,1,0,gl.RGBA,gl.UNSIGNED_BYTE,new Uint8Array(4));
    const framebuffer=gl.createFramebuffer();gl.bindFramebuffer(gl.FRAMEBUFFER,framebuffer);
    gl.framebufferTexture2D(gl.FRAMEBUFFER,gl.COLOR_ATTACHMENT0,gl.TEXTURE_2D,texture,0);
    gl.checkFramebufferStatus(gl.FRAMEBUFFER);
    const buffer=gl.createBuffer();gl.bindBuffer(gl.ARRAY_BUFFER,buffer);
    gl.bufferData(gl.ARRAY_BUFFER,new Float32Array([0,0,1,1]),gl.STATIC_DRAW);
    gl.bufferSubData(gl.ARRAY_BUFFER,0,new Float32Array([1,0]));
    gl.enable(gl.BLEND);gl.blendFunc(gl.SRC_ALPHA,gl.ONE_MINUS_SRC_ALPHA);
    gl.vertexAttribPointer(0,2,gl.FLOAT,false,0,0);gl.enableVertexAttribArray(0);
    gl.uniform1f(undefined,1);gl.drawElements(gl.TRIANGLES,3,gl.UNSIGNED_SHORT,0);
    gl.deleteTexture(texture);gl.deleteFramebuffer(framebuffer);
    const out=new Uint8Array(8);gl.readPixels(0,0,2,1,gl.RGBA,gl.UNSIGNED_BYTE,out);
    const shader=gl.createShader(gl.VERTEX_SHADER);gl.shaderSource(shader,'void main(){}');gl.compileShader(shader);
    return {completed:true,error:gl.getError(),pixels:[...out],
      compiled:gl.getShaderParameter(shader,gl.COMPILE_STATUS),
      same:canvas.getContext('webgl')===gl,other:canvas.getContext('2d')};
  })()` , [canvasPlugin(),graphicsPlugin({rgba:[1,2,3,4]})]),
  {completed:true,error:0,pixels:[1,2,3,4,1,2,3,4],compiled:false,same:true,other:null});
});

test('WebGL remains an optional graphics plugin surface', async () => {
  assert.deepEqual(await evaluate(`(() => ({
    webgl:typeof WebGLRenderingContext,webgl2:typeof WebGL2RenderingContext,
    getContext:typeof document.createElement('canvas').getContext
  }))()`, []), {webgl:'undefined',webgl2:'undefined',getContext:'function'});
  assert.deepEqual(await evaluate(`(() => ({
    webgl:typeof WebGLRenderingContext,webgl2:typeof WebGL2RenderingContext,
    context:document.createElement('canvas').getContext('webgl')
  }))()`, [canvasPlugin()]), {webgl:'undefined',webgl2:'undefined',context:null});
});
