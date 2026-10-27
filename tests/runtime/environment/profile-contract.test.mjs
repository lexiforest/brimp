import test from 'node:test';
import assert from 'node:assert/strict';
import {createSession, Plugin, audioPlugin, environmentPlugin, graphicsPlugin, canvasPlugin} from '../../../bindings/node/index.js';

test('custom plugins own profile defaults, validation and exported recipes', async () => {
  const plugin = new Plugin('custom', `({global, profile}) => {
    const values = profile.values('customValues', {answer: 42, nested: {a: 1, b: 2}}, {
      version: 3,
      fields: {answer: {integer: true, minimum: 0}},
      capture: ({read}) => ({answer: read(() => 7)})
    });
    global.answer = () => values.answer;
  }`);
  const session = await createSession({plugins: [plugin], profile: {values: {customValues: {answer: 7, nested: {a: 9}}}}});
  try {
    const page = await session.newPage();
    const contract = await page.profileContract();
    const custom = contract.interfaces.find(entry => entry.scope === 'customValues');
    assert.equal(custom.owner, 'custom');
    assert.equal(custom.version, 3);
    assert.equal(custom.fields.answer.default, 42);
    assert.match(custom.probe, /read\(\(\) => 7\)/);
    assert.doesNotMatch(custom.probe, /global.answer/);
    assert.equal(contract.interfaces.some(entry => entry.scope === 'audio'), false);
    assert.ok((await page.profileReport()).defaultsUsed.includes('customValues.nested.b'));
    assert.equal(await page.evaluate('answer()'), 7);
    assert.deepEqual(await page.evaluate('Reflect.ownKeys(globalThis).filter(k=>typeof k==="string"&&k.startsWith("__brimp"))'), []);
  } finally { await session.close(); }
  for (const values of [{customValues: {answer: -1}}, {customValues: {unknown: 1}}, {unloaded: {answer: 1}}]) {
    const session = await createSession({plugins: [plugin], profile: {values}});
    try { await assert.rejects(session.newPage(), /Profile/); }
    finally { await session.close(); }
  }
});

test('previously ineffective profile fields reach their runtime consumers', async () => {
  const session = await createSession({plugins: [audioPlugin(), environmentPlugin(), graphicsPlugin(), canvasPlugin()], profile: {
    values: {
      audio: {sample_rate: 192000, max_channel_count: 8},
      identity: {doNotTrack: '1'},
      privacy: {device_memory_gb: 16},
      webgl: {parameters: {'3379': 8192}},
      canvas: {fallbackAdvance: 0.75},
    },
  }});
  try {
    const page = await session.newPage();
    assert.deepEqual(await page.evaluate(`(() => {
      const audio = new AudioContext();
      const gl = document.createElement('canvas').getContext('webgl');
      const ctx = document.createElement('canvas').getContext('2d');
      ctx.font = '10px sans-serif';
      return [audio.sampleRate, audio.destination.maxChannelCount, navigator.doNotTrack,
        navigator.deviceMemory, gl.getParameter(3379), ctx.measureText('abcd').width];
    })()`), [192000, 8, '1', 16, 8192, 30]);
  } finally { await session.close(); }
});
