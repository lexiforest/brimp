import test from 'node:test';
import assert from 'node:assert/strict';
import {createSession,canvasPlugin} from '../index.js';

test('canvas drawing workflow preserves state and configured mock readback without rendering',async()=>{
  const session=await createSession({plugins:[canvasPlugin({rgba:[1,2,3,4]})]});
  try {
    const page=await session.newPage();
    assert.deepEqual(await page.evaluate(`(() => {
      const canvas=document.createElement('canvas');canvas.width=2;canvas.height=2;
      const c=canvas.getContext('2d');c.fillStyle='#ff0000';c.lineWidth=3;c.setLineDash([2,4]);
      c.save();c.translate(4,5);c.rotate(0.5);c.scale(2,2);c.fillStyle='#00ff00';
      c.beginPath();c.moveTo(0,0);c.lineTo(1,1);c.bezierCurveTo(0,0,1,1,2,2);
      c.quadraticCurveTo(1,1,2,2);c.arc(1,1,1,0,Math.PI);c.arcTo(0,0,1,1,1);
      c.ellipse(1,1,1,1,0,0,Math.PI);c.rect(0,0,2,2);c.closePath();c.clip();c.fill();c.stroke();
      const gradient=c.createLinearGradient(0,0,2,2);gradient.addColorStop(0,'red');gradient.addColorStop(1,'blue');
      c.fillStyle=gradient;c.fillRect(0,0,2,2);c.strokeRect(0,0,2,2);c.clearRect(0,0,1,1);
      c.createRadialGradient(0,0,0,1,1,1);c.createPattern(canvas,'repeat');
      c.fillText('text',0,0);c.strokeText('text',0,0);c.drawImage(canvas,0,0);
      const image=c.createImageData(2,2);image.data.fill(255);c.putImageData(image,0,0);
      const focusResult=c.drawFocusIfNeeded(document.createElement('button'));
      c.restore();const matrix=c.getTransform();
      return [c.canvas===canvas,c.fillStyle,c.lineWidth,c.getLineDash(),[matrix.a,matrix.d,matrix.e,matrix.f],
        [...c.getImageData(0,0,1,1).data],c.measureText('abcd').width,c.isPointInPath(0,0),c.isPointInStroke(0,0),
        focusResult===undefined,c.getContextAttributes().alpha];
    })()`),[true,'#ff0000',3,[2,4],[1,1,0,0],[1,2,3,4],20,false,false,true,true]);
  } finally {await session.close();}
});

test('legacy timing is a stable synthetic snapshot rather than measured network phases',async()=>{
  const session=await createSession({clock:{mode:'logical',epochMs:1000}});
  try {
    const page=await session.newPage();
    const snapshot=await page.evaluate(`(() => {
      const t=performance.timing,j=t.toJSON();
      return [t.navigationStart===performance.timeOrigin,Object.keys(j).length,
        Object.values(j).every(v=>v===0||v===performance.timeOrigin),JSON.stringify(t)===JSON.stringify(j),j];
    })()`);
    assert.deepEqual(snapshot.slice(0,4),[true,21,true,true]);
    await page.advanceTime(500);
    assert.deepEqual(await page.evaluate('performance.timing.toJSON()'),snapshot[4]);
  } finally {await session.close();}
});
