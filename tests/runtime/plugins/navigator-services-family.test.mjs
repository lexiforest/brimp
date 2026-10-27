import test from 'node:test';
import assert from 'node:assert/strict';
import {createSession, environmentPlugin, permissionsPlugin, devicesPlugin, mediaPlugin, workersPlugin, trustedTypesPlugin} from '../../../bindings/node/index.js';

async function evaluate(source, plugins = []) {
  const session = await createSession({plugins});
  try {
    const page = await session.newPage();
    await page.route('https://core.test/**', route => route.fulfill({body:'<!doctype html><body></body>',headers:{'content-type':'text/html'}}));
    await page.get('https://core.test/');
    return await page.evaluate(source);
  }
  finally { await session.close(); }
}

test('navigator keeps common metadata in core and optional service families out of core', async () => {
  assert.deepEqual(await evaluate(`[
    navigator.appCodeName,navigator.appName,navigator.product,navigator.vendorSub,navigator.javaEnabled(),
    navigator.sendBeacon('https://example.test/unused','ignored')===undefined,
    typeof navigator.userAgent,typeof navigator.appVersion,typeof navigator.platform,typeof navigator.language,
    Array.isArray(navigator.languages),typeof navigator.cookieEnabled,typeof navigator.onLine,
    navigator instanceof Navigator,
    navigator.userActivation===navigator.userActivation,navigator.userActivation.isActive,navigator.userActivation.hasBeenActive,
    ['clipboard','credentials','geolocation','mediaCapabilities','mediaSession','serviceWorker','wakeLock','mediaDevices'].every(n=>!(n in navigator))
  ]`),['Mozilla','Netscape','Gecko','',false,true,'string','string','string','string',true,'boolean','boolean',true,true,false,false,true]);
});

test('service placeholders support ignored-result calls without callbacks, UI or worker startup', async () => {
  assert.deepEqual(await evaluate(`(async () => {
    const families={clipboard:['read','readText','write','writeText'],credentials:['create','get','preventSilentAccess','store'],
      geolocation:['clearWatch','getCurrentPosition','watchPosition'],mediaCapabilities:['decodingInfo'],
      mediaSession:['setActionHandler','setCameraActive','setMicrophoneActive','setPositionState'],
      serviceWorker:['getRegistration','getRegistrations','register','startMessages'],wakeLock:['request'],
      mediaDevices:['getUserMedia','getDisplayMedia']};
    let callbacks=0,ready=false,events=0;const results=[];
    for(const [name,methods] of Object.entries(families)){
      const service=navigator[name];results.push(service===navigator[name]);
      for(const method of methods)results.push(service[method](()=>callbacks++)===undefined);
    }
    for(const method of ['canShare','getGamepads','requestMediaKeySystemAccess','share'])results.push(navigator[method]()===undefined);
    const initial=[navigator.mediaSession.metadata,navigator.mediaSession.playbackState,navigator.serviceWorker.controller];
    const metadata={title:'local'};navigator.mediaSession.metadata=metadata;navigator.mediaSession.playbackState='playing';
    navigator.serviceWorker.ready.then(()=>ready=true);
    navigator.serviceWorker.onmessage=()=>events++;
    navigator.serviceWorker.dispatchEvent(new MessageEvent('message',{data:'local'}));
    navigator.mediaDevices.ondevicechange=()=>events++;navigator.mediaDevices.dispatchEvent(new Event('devicechange'));
    const devices=await navigator.mediaDevices.enumerateDevices();
    return [results.every(Boolean),callbacks,ready,events,initial,navigator.mediaSession.metadata===metadata,
      navigator.mediaSession.playbackState,devices,navigator.mediaDevices.getSupportedConstraints(),
      navigator.clipboard instanceof Clipboard,navigator.mediaDevices instanceof MediaDevices,navigator.serviceWorker instanceof ServiceWorkerContainer,
      navigator.serviceWorker.ready===navigator.serviceWorker.ready];
  })()`,[environmentPlugin(), permissionsPlugin(), devicesPlugin(), mediaPlugin(), workersPlugin(), trustedTypesPlugin()]),[true,0,false,2,[null,'none',null],true,'playing',[],{},true,true,true,true]);
});

test('existing service metadata, plugin collections, permissions, locks and storage remain usable', async () => {
  assert.deepEqual(await evaluate(`(async () => {
    const plugins=navigator.plugins,mimes=navigator.mimeTypes,p=plugins.item(0),m=mimes.namedItem('application/pdf');
    const lock=await navigator.locks.request('local',lock=>[lock.name,lock.mode]);
    const permissions=await navigator.permissions.query({name:'notifications'});
    const estimate=await navigator.storage.estimate();
    return [plugins.length,mimes.length,p.name,p.filename,p.description,p.length,m.type,m.suffixes,m.description,
      plugins[0]===p,plugins[p.name]===p,plugins.namedItem(p.name)===p,[...plugins].length,
      p[0]===p.item(0),p['application/pdf']===p.namedItem('application/pdf'),[...p].length,
      mimes[0]===m,mimes['application/pdf']===m,m.enabledPlugin===p,plugins.refresh()===undefined,
      permissions.state,navigator.permissions instanceof Permissions,lock,(await navigator.locks.query()).held.length,
      typeof estimate.usage,estimate.quota===null,navigator.hardwareConcurrency,navigator.maxTouchPoints,
      navigator.vendor,navigator.productSub,navigator.pdfViewerEnabled,navigator.webdriver,
      navigator.deviceMemory,navigator.connection.rtt,await navigator.bluetooth.getAvailability(),
      ['doNotTrack','getBattery','getUserMedia','registerProtocolHandler','requestMIDIAccess','vibrate','xr','userAgentData'].every(n=>!(n in navigator))];
  })()`,[environmentPlugin(), permissionsPlugin(), devicesPlugin(), mediaPlugin(), workersPlugin(), trustedTypesPlugin()]),
  [2,2,'PDF Viewer','internal-pdf-viewer','Portable Document Format',2,'application/pdf','pdf','Portable Document Format',
    true,true,true,2,true,true,2,true,true,true,true,'prompt',true,['local','exclusive'],0,'number',true,8,0,'Apple Computer, Inc.','20030107',true,false,undefined,50,false,true]);
});
