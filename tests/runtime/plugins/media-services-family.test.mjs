import test from 'node:test';
import assert from 'node:assert/strict';
import {createSession, mediaPlugin, environmentPlugin, devicesPlugin, audioPlugin} from '../../../bindings/node/index.js';

async function evaluate(source, plugins = []) {
  const session = await createSession({plugins});
  try { return await (await session.newPage()).evaluate(source); }
  finally { await session.close(); }
}

test('MediaStream supplies stable identity and legacy calls without tracks or capture', async () => {
  assert.deepEqual(await evaluate(`(() => {
    const stream=new MediaStream(),other=new MediaStream();let events=0;
    const defaults=[stream.active,stream.onaddtrack,stream.onremovetrack];
    stream.onaddtrack=()=>events++;stream.onremovetrack=()=>events++;
    const noops=['addTrack','removeTrack','clone','getAudioTracks','getVideoTracks','getTracks','getTrackById']
      .every(n=>stream[n]({id:'unused'})===undefined);
    const produced=events;stream.dispatchEvent(new Event('addtrack'));stream.dispatchEvent(new Event('removetrack'));
    stream.onaddtrack=null;stream.dispatchEvent(new Event('addtrack'));
    return [defaults,noops,produced,events,stream instanceof EventTarget,stream.id===stream.id,
      stream.id!==other.id,/^[0-9a-f-]{36}$/.test(stream.id),!('onactive' in stream),!('oninactive' in stream)];
  })()`,[mediaPlugin(), environmentPlugin(), devicesPlugin()]),[[false,null,null],true,0,2,true,true,true,true,true,true]);
  assert.deepEqual(await evaluate(`['MediaStream','SourceBuffer','SpeechSynthesisUtterance','speechSynthesis'].map(n=>typeof globalThis[n])`),
    ['undefined','undefined','undefined','undefined']);
});

test('SourceBuffer exposes legacy prototype contracts without constructing a media backend', async () => {
  assert.deepEqual(await evaluate(`(() => {
    let error;try{new SourceBuffer()}catch(e){error=e.name}
    // A prototype fixture checks the shape only; there is no MediaSource factory.
    const fixture=Object.setPrototypeOf(new EventTarget(),SourceBuffer.prototype),other=Object.setPrototypeOf(new EventTarget(),SourceBuffer.prototype);
    const defaults=['appendWindowEnd','appendWindowStart','buffered','mode','timestampOffset','updating'].map(n=>fixture[n]);
    fixture.mode='sequence';fixture.timestampOffset=2;
    const noops=['abort','appendBuffer','changeType','remove'].every(n=>fixture[n](new Uint8Array([1,2]),3)===undefined);
    const handlers=['onabort','onerror','onupdate','onupdateend','onupdatestart'];
    const initial=handlers.every(n=>fixture[n]===null);let events=0;
    for(const name of handlers){fixture[name]=()=>events++;fixture.dispatchEvent(new Event(name.slice(2)));fixture[name]=null;}
    return [error,defaults,noops,initial,events,fixture.mode,fixture.timestampOffset,other.mode,typeof MediaSource];
  })()`,[mediaPlugin(), environmentPlugin(), devicesPlugin()]),['TypeError',['','','','','',''],true,true,5,'sequence','2','','undefined']);
});

test('Speech utterances retain payloads and manual handlers while synthesis stays a legacy placeholder', async () => {
  assert.deepEqual(await evaluate(`(() => {
    const utterance=new SpeechSynthesisUtterance(),other=new SpeechSynthesisUtterance('hello');
    const defaults=[utterance.text,utterance.lang,utterance.pitch,utterance.rate,utterance.volume,utterance.voice];
    utterance.text='local';utterance.lang='en';utterance.pitch=1.5;utterance.rate=2;utterance.volume=0.5;
    const voice={name:'local payload'};utterance.voice=voice;
    const handlers=['onboundary','onend','onerror','onmark','onpause','onresume','onstart'];
    const initial=handlers.every(n=>utterance[n]===null);let events=0;
    for(const name of handlers){utterance[name]=()=>events++;utterance.dispatchEvent(new Event(name.slice(2)));utterance[name]=null;utterance.dispatchEvent(new Event(name.slice(2)));}
    return [defaults,[utterance.text,utterance.lang,utterance.pitch,utterance.rate,utterance.volume],utterance.voice===voice,
      other.text,other.lang,initial,events,utterance instanceof EventTarget,Object.keys(speechSynthesis),typeof speechSynthesis.speak];
  })()`,[audioPlugin()]),[['','',1,1,1,null],['local','en',1.5,2,0.5],true,'hello','',true,7,true,[],'undefined']);
});

test('Existing network and Bluetooth mocks remain opt-in Safari-absent extensions', async () => {
  assert.deepEqual(await evaluate(`(async () => {
    const connection=navigator.connection;let changes=0;connection.addEventListener('change',()=>changes++);connection.dispatchEvent(new Event('change'));
    return [connection===navigator.connection,Object.prototype.toString.call(connection),connection.constructor.name,
      connection.downlink,connection.effectiveType,connection.rtt,connection.saveData,changes,
      await navigator.bluetooth.getAvailability(),navigator.bluetooth.constructor===Object,
      typeof Bluetooth,typeof NetworkInformation,'requestDevice' in navigator.bluetooth,'onchange' in connection];
  })()`,[environmentPlugin({connection_downlink_mbps:12,connection_rtt_ms:8,connection_effective_type:'3g',connection_save_data:true}), devicesPlugin({bluetooth_available:true})]),
    [true,'[object EventTarget]','NetworkInformation',12,'3g',8,true,1,true,true,'undefined','undefined',false,false]);
});

test('UserActivation follows controller input and expires transient state without clearing history', async () => {
  const session=await createSession();
  try {
    const page=await session.newPage();
    await page.evaluate(`document.body.innerHTML='<button id="activate">Activate</button>';null`);
    assert.deepEqual(await page.evaluate(`[navigator.userActivation instanceof UserActivation,navigator.userActivation.isActive,navigator.userActivation.hasBeenActive]`),[true,false,false]);
    await page.evaluate(`document.getElementById('activate').click();null`);
    assert.equal(await page.evaluate('navigator.userActivation.isActive'),false);
    await page.click('#activate');
    assert.deepEqual(await page.evaluate(`[navigator.userActivation.isActive,navigator.userActivation.hasBeenActive]`),[true,true]);
    assert.deepEqual(await page.evaluate(`new Promise(resolve=>setTimeout(()=>resolve([navigator.userActivation.isActive,navigator.userActivation.hasBeenActive]),5001))`),[false,true]);
  } finally { await session.close(); }
});
