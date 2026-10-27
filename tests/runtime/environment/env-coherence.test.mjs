import test from 'node:test';
import assert from 'node:assert/strict';
import {createSession, environmentPlugin} from '../../../bindings/node/index.js';


test('default profile drives identity, screen, viewport and Intl coherently', async () => {
  const session = await createSession();
  try {
    const page = await session.newPage();
    assert.deepEqual(await page.evaluate(`(() => ({
      userAgent: navigator.userAgent,
      appCodeName: navigator.appCodeName,
      appName: navigator.appName,
      appVersion: navigator.appVersion,
      product: navigator.product,
      productSub: navigator.productSub,
      vendor: navigator.vendor,
      vendorSub: navigator.vendorSub,
      platform: navigator.platform,
      language: navigator.language,
      languages: navigator.languages,
      hardwareConcurrency: navigator.hardwareConcurrency,
      maxTouchPoints: navigator.maxTouchPoints,
      pdfViewerEnabled: navigator.pdfViewerEnabled,
      screen: [screen.width, screen.height, screen.availWidth, screen.availHeight,
        screen.availLeft, screen.availTop, screen.colorDepth, screen.orientation.type],
      viewport: [innerWidth, innerHeight, outerWidth, outerHeight,
        screenX, screenY, devicePixelRatio, visualViewport.width, visualViewport.height],
      intl: [new Intl.DateTimeFormat().resolvedOptions().locale,
        new Intl.DateTimeFormat().resolvedOptions().timeZone,
        new Intl.NumberFormat().resolvedOptions().locale],
      offset: new Date('2026-01-15T12:00:00Z').getTimezoneOffset(),
      internal: Object.getOwnPropertyNames(globalThis).filter(name => name.startsWith('__brimp')),
    }))()`), {
      userAgent: 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/27.0 Safari/605.1.15',
      appCodeName: 'Mozilla',
      appName: 'Netscape',
      appVersion: '5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/27.0 Safari/605.1.15',
      product: 'Gecko',
      productSub: '20030107',
      vendor: 'Apple Computer, Inc.',
      vendorSub: '',
      platform: 'MacIntel',
      language: 'en-US',
      languages: ['en-US'],
      hardwareConcurrency: 8,
      maxTouchPoints: 0,
      pdfViewerEnabled: true,
      screen: [1710, 1107, 1710, 1019, 0, 34, 24, 'landscape-primary'],
      viewport: [800, 600, 800, 652, 110, 34, 2, 800, 600],
      intl: ['en-US', 'Asia/Taipei', 'en-US'],
      offset: -480,
      internal: [],
    });
  } finally { await session.close(); }
});

test('a supplied profile changes every derived surface and locale/timezone default', async () => {
  const custom = {values: {identity: {}, device: {}, screen: {}, window: {}, navigatorPlugins: {}}};
  custom.values.identity.languages = ['fr-CA', 'fr'];
  custom.values.identity.locale = 'fr-CA';
  custom.values.identity.timezone = 'America/New_York';
  custom.values.identity.platform = 'Win32';
  custom.values.identity.user_agent = 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/27.0 Safari/605.1.15';
  custom.values.identity.app_version = custom.values.identity.user_agent.slice('Mozilla/'.length);
  custom.values.device.hardware_concurrency = 16;
  custom.values.screen.width = 1920;
  custom.values.screen.height = 1080;
  custom.values.screen.avail_width = 1920;
  custom.values.screen.avail_height = 1040;
  custom.values.window.inner_width = 1280;
  custom.values.window.inner_height = 720;
  custom.values.window.outer_width = 1280;
  custom.values.window.outer_height = 800;
  custom.values.navigatorPlugins = {entries: [], pdfViewerEnabled: false};
  const session = await createSession({profile: custom});
  try {
    const page = await session.newPage();
    assert.deepEqual(await page.evaluate(`(() => ({
      identity: [navigator.userAgent, navigator.platform, navigator.language, navigator.languages,
        navigator.hardwareConcurrency, navigator.pdfViewerEnabled],
      geometry: [screen.width, screen.height, screen.availHeight, innerWidth, innerHeight,
        outerWidth, outerHeight],
      intl: [new Intl.DateTimeFormat().resolvedOptions().locale,
        new Intl.DateTimeFormat().resolvedOptions().timeZone],
      winter: [new Date('2026-01-15T12:00:00Z').getTimezoneOffset(),
        new Date('2026-01-15T12:00:00Z').toLocaleString() ===
          new Date('2026-01-15T12:00:00Z').toLocaleString('fr-CA', {timeZone: 'America/New_York'})],
      summer: [new Date('2026-07-15T12:00:00Z').getTimezoneOffset(),
        new Date('2026-07-15T12:00:00Z').toLocaleString() ===
          new Date('2026-07-15T12:00:00Z').toLocaleString('fr-CA', {timeZone: 'America/New_York'})],
    }))()`), {
      identity: [custom.values.identity.user_agent, 'Win32', 'fr-CA', ['fr-CA', 'fr'], 16, false],
      geometry: [1920, 1080, 1040, 1280, 720, 1280, 800],
      intl: ['fr-CA', 'America/New_York'],
      winter: [300, true],
      summer: [240, true],
    });
  } finally { await session.close(); }
});

test('services plugin keeps profile identity instead of overriding it', async () => {
  const session = await createSession({plugins: [environmentPlugin()]});
  try {
    const page = await session.newPage();
    assert.deepEqual(await page.evaluate(`(() => ({
      vendor: navigator.vendor,
      hardwareConcurrency: navigator.hardwareConcurrency,
      deviceMemory: 'deviceMemory' in navigator,
      pdfViewerEnabled: navigator.pdfViewerEnabled,
      internal: Object.getOwnPropertyNames(globalThis).filter(name => name.startsWith('__brimp')),
    }))()`), {
      vendor: 'Apple Computer, Inc.',
      hardwareConcurrency: 8,
      deviceMemory: false,
      pdfViewerEnabled: true,
      internal: [],
    });
  } finally { await session.close(); }
});



test('services options override core navigator state and preserve optional exposure', async () => {
  const custom = {values: {identity: {}, device: {}, screen: {}, window: {}, navigatorPlugins: {}}}; custom.values.navigatorPlugins = {entries: [], pdfViewerEnabled: false};
  const options = {hardware_concurrency:0,max_touch_points:2,vendor:'',product_sub:'custom',
    pdf_viewer_enabled:true,device_memory_gb:16,expose_global_privacy_control:true,global_privacy_control:false};
  const session = await createSession({profile:custom,plugins:[environmentPlugin(options)]});
  try {
    const page = await session.newPage();
    await page.route('https://override.test/**', r=>r.fulfill({body:'<body></body>',headers:{'content-type':'text/html'}}));
    const check = async () => assert.deepEqual(await page.evaluate(`[
      navigator.hardwareConcurrency,navigator.maxTouchPoints,navigator.vendor,navigator.productSub,
      navigator.pdfViewerEnabled,navigator.plugins.length,navigator.mimeTypes.length,
      'deviceMemory' in navigator,'globalPrivacyControl' in navigator,navigator.globalPrivacyControl,
      navigator.plugins===navigator.plugins,typeof __brimpConfigureNavigator]`),
      [0,2,'','custom',true,5,2,true,true,false,true,'undefined']);
    await check(); await page.get('https://override.test/'); await check();
  } finally {await session.close();}
  const disabled = await createSession({plugins:[environmentPlugin({pdf_viewer_enabled:false,expose_global_privacy_control:false,global_privacy_control:true})]});
  try {assert.deepEqual(await (await disabled.newPage()).evaluate('[navigator.pdfViewerEnabled,"globalPrivacyControl" in navigator]'),[false,false]);}
  finally {await disabled.close();}
});
