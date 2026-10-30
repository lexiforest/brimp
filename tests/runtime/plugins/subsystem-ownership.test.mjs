import test from 'node:test';
import assert from 'node:assert/strict';
import {
  createSession,
  environmentPlugin,
  permissionsPlugin,
  devicesPlugin,
  mediaPlugin,
  workersPlugin,
  trustedTypesPlugin,
  audioPlugin,
  pluginCatalog,
} from '../../../bindings/node/index.js';

// navigator.connection is a Chromium-only surface, so that family runs under a
// Chromium identity; every other family is identity-neutral.
const CHROME_AGENT =
  'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/156.0.0.0 Safari/537.36';
const families = [
  [
    environmentPlugin,
    'navigator.connection',
    `navigator.connection.addEventListener('change',()=>{});`,
    { connection_rtt_ms: 7 },
    true,
  ],
  [
    permissionsPlugin,
    'Notification',
    `await Notification.requestPermission();await navigator.permissions.query({name:'notifications'});navigator.credentials.get();`,
    { notification_permission: 'denied' },
  ],
  [
    devicesPlugin,
    'navigator.clipboard',
    `navigator.clipboard.readText();navigator.geolocation.getCurrentPosition();navigator.wakeLock.request();await navigator.bluetooth.getAvailability();`,
    { bluetooth_available: true },
  ],
  [
    mediaPlugin,
    'MediaStream',
    `new MediaStream().getTracks();new MediaEncryptedEvent('encrypted');navigator.mediaSession.setActionHandler();await navigator.mediaDevices.enumerateDevices();`,
    {},
  ],
  [
    workersPlugin,
    'Worker',
    `const w=new Worker('ignored.js');w.postMessage('ignored');w.terminate();navigator.serviceWorker.register();`,
    {},
  ],
  [trustedTypesPlugin, 'trustedTypes', `trustedTypes.createPolicy();trustedTypes.isHTML();`, {}],
  [
    audioPlugin,
    'AudioContext',
    `const a=new AudioContext();a.createGain().connect(a.destination);new SpeechSynthesisUtterance('hello');await a.close();`,
    {},
  ],
];
for (const [installer, surface, calls, options, chromium] of families) {
  test(`${installer.name} owns its surface independently and reinstalls on navigation`, async () => {
    const session = await createSession({
      plugins: [installer(options)],
      ...(chromium ? { profile: { values: { identity: { user_agent: CHROME_AGENT } } } } : {}),
    });
    try {
      const page = await session.newPage();
      for (let i = 0; i < 2; i++) {
        if (i) await page.get('data:text/html,<body>navigation</body>');
        assert.deepEqual(
          await page.evaluate(`(async()=>{
          ${calls}
          document.body.textContent='continued';
          return [${families.map(([, s]) => `typeof ${s} !== 'undefined'`).join(',')},document.body.textContent];
        })()`),
          [...families.map(([f]) => f === installer), 'continued'],
        );
      }
    } finally {
      await session.close();
    }
  });
}
test('all former services options have an owning plugin', () => {
  const catalog = pluginCatalog();
  assert.equal('services' in catalog, false);
  assert.deepEqual(Object.keys(catalog.environment.options), [
    'hardware_concurrency',
    'max_touch_points',
    'vendor',
    'product_sub',
    'pdf_viewer_enabled',
    'device_memory_gb',
    'expose_global_privacy_control',
    'global_privacy_control',
    'connection_type',
    'connection_rtt_ms',
    'connection_downlink_mbps',
    'connection_effective_type',
    'connection_save_data',
  ]);
  assert.deepEqual(Object.keys(catalog.permissions.options), [
    'notification_permission',
    'permissions_enabled',
  ]);
  assert.deepEqual(Object.keys(catalog.devices.options), [
    'bluetooth_enabled',
    'bluetooth_available',
  ]);
  assert.deepEqual(Object.keys(catalog.media.options), ['media_devices_enabled']);
});

test('default browser plugins preserve ignored-result script continuation; explicit lists replace them', async () => {
  const { browserPlugins, Plugin } = await import('../../../bindings/node/index.js');
  assert.deepEqual(
    new Set(browserPlugins().map((p) => p.name)),
    new Set(
      Object.entries(pluginCatalog())
        .filter(([, p]) => p.defaultEnabled)
        .map(([name]) => name),
    ),
  );
  for (const options of [undefined, { plugins: [] }, { plugins: [workersPlugin()] }]) {
    const session = await createSession(options);
    try {
      const page = await session.newPage();
      const present =
        await page.evaluate(`[typeof Worker,typeof AudioContext,typeof RTCPeerConnection,
        !!document.createElement('canvas').getContext('2d')]`);
      assert.deepEqual(
        present,
        options === undefined
          ? ['function', 'function', 'function', true]
          : options.plugins.length
            ? ['function', 'undefined', 'undefined', false]
            : ['undefined', 'undefined', 'undefined', false],
      );
      if (options === undefined)
        assert.equal(
          await page.evaluate(`(async()=>{
        new Worker('ignored').postMessage('ignored');navigator.clipboard.readText();trustedTypes.createPolicy();
        new MediaStream().getTracks();new SpeechSynthesisUtterance('hello');
        new RTCPeerConnection().createDataChannel('ignored').send('ignored');
        const canvas=document.createElement('canvas');canvas.getContext('2d').fillRect(0,0,1,1);
        const audio=new AudioContext();audio.createGain().connect(audio.destination);await audio.close();
        document.body.textContent='continued';return document.body.textContent;
      })()`),
          'continued',
        );
    } finally {
      await session.close();
    }
  }
  const session = await createSession({
    plugins: [
      ...browserPlugins(),
      new Plugin(
        'user-audio',
        `({global})=>{global.AudioContext=class AudioContext {get source(){return 'user'}}}`,
      ),
    ],
  });
  try {
    const page = await session.newPage();
    assert.deepEqual(await page.evaluate(`[new AudioContext().source,typeof Worker]`), [
      'user',
      'function',
    ]);
  } finally {
    await session.close();
  }
});
