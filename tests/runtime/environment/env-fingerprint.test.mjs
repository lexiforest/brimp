import test from 'node:test';
import assert from 'node:assert/strict';
import {
  createSession,
  canvasPlugin,
  graphicsPlugin,
  audioPlugin,
  rtcPlugin,
} from '../../../bindings/node/index.js';

const plugins = () => [canvasPlugin(), graphicsPlugin(), audioPlugin(), rtcPlugin()];

test('canvas paints solid rectangles, clears them, and encodes a real PNG', async () => {
  const session = await createSession({ plugins: plugins() });
  try {
    const page = await session.newPage();
    assert.deepEqual(
      await page.evaluate(`(() => {
      const canvas = document.createElement('canvas');
      canvas.width = 4; canvas.height = 3;
      const context = canvas.getContext('2d');
      const background = Array.from(context.getImageData(0, 0, 1, 1).data);
      context.fillStyle = '#ff0000';
      context.fillRect(1, 1, 2, 1);
      const painted = Array.from(context.getImageData(1, 1, 1, 1).data);
      const untouched = Array.from(context.getImageData(0, 0, 1, 1).data);
      context.clearRect(1, 1, 2, 1);
      const cleared = Array.from(context.getImageData(1, 1, 1, 1).data);
      context.fillStyle = '#00ff00';
      context.fillRect(0, 0, 4, 3);
      const url = canvas.toDataURL();
      const bytes = atob(url.split(',')[1]);
      return [background, painted, untouched, cleared,
        url.startsWith('data:image/png;base64,'),
        [bytes.charCodeAt(1), bytes.charCodeAt(2), bytes.charCodeAt(3)],
        (bytes.charCodeAt(16) << 24 | bytes.charCodeAt(17) << 16 | bytes.charCodeAt(18) << 8 | bytes.charCodeAt(19)) >>> 0,
        (bytes.charCodeAt(20) << 24 | bytes.charCodeAt(21) << 16 | bytes.charCodeAt(22) << 8 | bytes.charCodeAt(23)) >>> 0,
        canvas.toDataURL('image/jpeg')];
    })()`),
      [
        [0, 0, 0, 0],
        [255, 0, 0, 255],
        [0, 0, 0, 0],
        [0, 0, 0, 0],
        true,
        [80, 78, 71],
        4,
        3,
        'data:,',
      ],
    );
  } finally {
    await session.close();
  }
});

test('canvas text advance, WebGL identity, audio and RTC values come from the profile', async () => {
  const session = await createSession({ plugins: plugins() });
  try {
    const page = await session.newPage();
    assert.deepEqual(
      await page.evaluate(`(() => {
      const canvas = document.createElement('canvas');
      const context = canvas.getContext('2d');
      const widths = ['10px Helvetica', '10px PingFang SC', '10px sans-serif'].map(font => {
        context.font = font; return context.measureText('abcd').width;
      });
      const gl = document.createElement('canvas').getContext('webgl');
      const debug = gl.getExtension('WEBGL_debug_renderer_info');
      const audio = new AudioContext();
      const peer = new RTCPeerConnection();
      return [widths,
        gl.getParameter(gl.VERSION), gl.getParameter(gl.VENDOR), gl.getParameter(gl.RENDERER),
        gl.getParameter(debug.UNMASKED_VENDOR_WEBGL), gl.getParameter(debug.UNMASKED_RENDERER_WEBGL),
        gl.getParameter(gl.MAX_TEXTURE_SIZE), gl.getSupportedExtensions().length,
        gl.getExtension('EXT_blend_minmax') !== null,
        audio.sampleRate, audio.baseLatency, audio.outputLatency,
        peer.getConfiguration()];
    })()`),
      [
        [20, 40, 20],
        'WebGL 1.0',
        'Apple Inc.',
        'Apple GPU',
        'Apple Inc.',
        'ANGLE (Apple, ANGLE Metal Renderer: Apple M1, Unspecified Version)',
        16384,
        44,
        true,
        44100,
        0,
        0.01,
        { iceServers: [] },
      ],
    );
    const visible = await page.evaluate(`[
      document.createElement('canvas').getContext('webgl').getParameter(7936),
      document.createElement('canvas').getContext('webgl').getParameter(7937),
      document.createElement('canvas').toDataURL(),
    ]`);
    assert.ok(!JSON.stringify(visible).match(/brimp|mock|deterministic/i), JSON.stringify(visible));
  } finally {
    await session.close();
  }
});

test('a supplied profile changes canvas metrics, WebGL limits, audio rate and ICE servers', async () => {
  const custom = { values: { fonts: {}, webgl: {}, audio: {}, webrtc: {} } };
  custom.values.fonts.metrics = { Helvetica: 0.25 };
  custom.values.webgl.vendor = 'Profile Vendor';
  custom.values.webgl.renderer = 'Profile Renderer';
  custom.values.webgl.unmasked_vendor = 'Profile Unmasked Vendor';
  custom.values.webgl.unmasked_renderer = 'Profile Unmasked Renderer';
  custom.values.webgl.extensions = ['EXT_blend_minmax', 'WEBGL_debug_renderer_info'];
  custom.values.webgl.parameters = { MAX_TEXTURE_SIZE: 8192 };
  custom.values.audio.sample_rate = 48000;
  custom.values.audio.output_latency = 0.02;
  custom.values.webrtc.ice_servers = ['stun:example.test'];
  const session = await createSession({ profile: custom, plugins: plugins() });
  try {
    const page = await session.newPage();
    assert.deepEqual(
      await page.evaluate(`(() => {
      const canvas = document.createElement('canvas');
      const context = canvas.getContext('2d');
      context.font = '10px Helvetica';
      const gl = document.createElement('canvas').getContext('webgl');
      const debug = gl.getExtension('WEBGL_debug_renderer_info');
      const audio = new AudioContext();
      return [context.measureText('abcd').width,
        gl.getParameter(gl.VENDOR), gl.getParameter(gl.RENDERER),
        gl.getParameter(debug.UNMASKED_VENDOR_WEBGL), gl.getParameter(debug.UNMASKED_RENDERER_WEBGL),
        gl.getParameter(gl.MAX_TEXTURE_SIZE), gl.getSupportedExtensions(),
        gl.getExtension('EXT_blend_minmax') !== null, gl.getExtension('WEBGL_debug_renderer_info') !== null,
        audio.sampleRate, audio.outputLatency,
        new RTCPeerConnection().getConfiguration()];
    })()`),
      [
        10,
        'Profile Vendor',
        'Profile Renderer',
        'Profile Unmasked Vendor',
        'Profile Unmasked Renderer',
        8192,
        ['EXT_blend_minmax', 'WEBGL_debug_renderer_info'],
        true,
        true,
        48000,
        0.02,
        { iceServers: [{ urls: 'stun:example.test' }] },
      ],
    );
  } finally {
    await session.close();
  }
});
