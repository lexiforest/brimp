import test from 'node:test';
import assert from 'node:assert/strict';
import { createSession, audioPlugin } from '../../../bindings/node/index.js';

test('audio graph workflow retains buffers, listener/compressor state and logical time', async () => {
  const session = await createSession({ plugins: [audioPlugin()], clock: { mode: 'logical' } });
  try {
    const page = await session.newPage();
    assert.deepEqual(
      await page.evaluate(`(async()=>{
      const c=globalThis.audio=new AudioContext(),osc=c.createOscillator(),gain=c.createGain(),compressor=c.createDynamicsCompressor();
      osc.connect(gain).connect(compressor).connect(c.destination);
      c.listener.setPosition(1,2,3);c.listener.setOrientation(0,0,-1,0,1,0);
      const buffer=c.createBuffer(1,4,48000);buffer.copyToChannel(new Float32Array([1,2]),0,1);
      gain.gain.setValueAtTime(0.5,0);osc.start();await c.resume();
      return [c instanceof BaseAudioContext,osc instanceof AudioScheduledSourceNode,c.destination instanceof AudioDestinationNode,
        c.listener.positionX.value,c.listener.positionY.value,c.listener.upY.value,compressor.threshold.value,
        compressor.ratio.value,compressor.reduction,gain.gain.value,[...buffer.getChannelData(0)],c.state];
    })()`),
      [true, true, true, 1, 2, 1, -24, 12, 0, 0.5, [0, 1, 2, 0], 'running'],
    );
    await page.advanceTime(500);
    assert.equal(await page.evaluate('audio.currentTime'), 0.5);
    await page.evaluate('audio.close().then(()=>null)');
  } finally {
    await session.close();
  }
});

test('audio placeholders let ignored-result DSP and worklet calls complete without changing parameter state', async () => {
  const session = await createSession({ plugins: [audioPlugin()] });
  try {
    const page = await session.newPage();
    assert.deepEqual(
      await page.evaluate(`(() => {
      const c=new AudioContext(),param=c.createGain().gain,results=[];
      for(const name of ['createBiquadFilter','createBufferSource','createChannelMerger','createChannelSplitter','createConstantSource',
        'createConvolver','createDelay','createIIRFilter','createPanner','createPeriodicWave','createScriptProcessor','createStereoPanner','createWaveShaper'])results.push(c[name]()===undefined);
      for(const name of ['linearRampToValueAtTime','exponentialRampToValueAtTime','setTargetAtTime','setValueCurveAtTime'])results.push(param[name](0,1)===undefined);
      results.push(c.audioWorklet instanceof Worklet,c.audioWorklet instanceof AudioWorklet,
        c.audioWorklet.addModule('unused.js')===undefined,c.createOscillator().setPeriodicWave({})===undefined);
      return [results.every(Boolean),param.value];
    })()`),
      [true, 1],
    );
  } finally {
    await session.close();
  }
});

test('offline audio returns a silent buffer placeholder and retains legacy pending controls', async () => {
  const session = await createSession({ plugins: [audioPlugin()] });
  try {
    const page = await session.newPage();
    assert.deepEqual(
      await page.evaluate(`(async()=>{
      const c=new OfflineAudioContext(2,8,44100);let completed=false;
      c.suspend(0).then(()=>completed=true);c.resume().then(()=>completed=true);
      const b=await c.startRendering();
      return [c instanceof BaseAudioContext,c.length,c.sampleRate,'numberOfChannels' in c,
        b instanceof AudioBuffer,b.numberOfChannels,b.length,b.getChannelData(0).every(v=>v===0),completed,c.oncomplete];
    })()`),
      [true, 8, 44100, false, true, 2, 8, true, false, null],
    );
  } finally {
    await session.close();
  }
});

test('core-only selection omits the audio family', async () => {
  const session = await createSession({ plugins: [] });
  try {
    const page = await session.newPage();
    assert.deepEqual(
      await page.evaluate(
        '[typeof BaseAudioContext,typeof OfflineAudioContext,typeof AudioListener,typeof Worklet]',
      ),
      Array(4).fill('undefined'),
    );
  } finally {
    await session.close();
  }
});
