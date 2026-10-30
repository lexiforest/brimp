import test from 'node:test';
import assert from 'node:assert/strict';
import { createSession } from '../../../bindings/node/index.js';

async function evaluate(source) {
  const session = await createSession();
  try {
    return await (await session.newPage()).evaluate(source);
  } finally {
    await session.close();
  }
}

test('media metadata and state use the existing core DOM without a playback backend', async () => {
  assert.deepEqual(
    await evaluate(`(() => {
    const v=document.createElement('video'),a=document.createElement('audio');
    const defaults=[v.currentSrc,v.networkState,v.preload,v.readyState,v.paused,v.preservesPitch,v.srcObject,v.error,v.seeking];
    for(const property of ['autoplay','controls','loop','defaultMuted','disableRemotePlayback','playsInline','disablePictureInPicture'])v[property]=true;
    v.src='https://example.test/video.mp4';v.poster='https://example.test/poster.png';v.width=320;v.height=180;
    v.currentTime=2;v.playbackRate=1.5;v.defaultPlaybackRate=1.25;v.volume=0.5;v.muted=true;v.preservesPitch=false;
    const stream={};v.srcObject=stream;
    const clone=v.cloneNode();
    return [defaults,v instanceof HTMLMediaElement,v instanceof HTMLVideoElement,a instanceof HTMLAudioElement,
      v.src,v.currentSrc,v.poster,clone.width,clone.height,clone.controls,clone.defaultMuted,clone.disableRemotePlayback,
      clone.playsInline,clone.disablePictureInPicture,v.currentTime,v.playbackRate,v.defaultPlaybackRate,v.volume,v.muted,
      v.preservesPitch,v.srcObject===stream,HTMLMediaElement.HAVE_ENOUGH_DATA,v.NETWORK_NO_SOURCE];
  })()`),
    [
      ['', 0, 'auto', 0, true, true, null, null, false],
      true,
      true,
      true,
      'https://example.test/video.mp4',
      '',
      'https://example.test/poster.png',
      320,
      180,
      true,
      true,
      true,
      true,
      true,
      2,
      1.5,
      1.25,
      0.5,
      true,
      false,
      true,
      4,
      3,
    ],
  );
});

test('media placeholders allow ignored-result UI calls and keep their limited return values explicit', async () => {
  assert.deepEqual(
    await evaluate(`(async()=>{
    const v=document.createElement('video'),results=[];let callbacks=0;
    for(const name of ['addTextTrack','setSinkId','cancelVideoFrameCallback','getVideoPlaybackQuality','requestPictureInPicture',
      'webkitEnterFullscreen','webkitEnterFullScreen','webkitExitFullscreen','webkitExitFullScreen'])results.push(v[name]('ignored')===undefined);
    results.push(v.requestVideoFrameCallback(()=>callbacks++)===undefined);
    v.load();v.pause();let playError;try{await v.play();}catch(e){playError=e.name;}
    return [results.every(Boolean),callbacks,[v.buffered,v.played,v.seekable,v.textTracks,v.remote],
      v.videoWidth,v.videoHeight,v.webkitDisplayingFullscreen,v.webkitSupportsFullscreen,v.sinkId,v.paused,v.currentTime,
      Number.isNaN(v.duration),playError,['captureStream','controlsList','webkitAudioDecodedByteCount','webkitVideoDecodedByteCount',
      'webkitDecodedFrameCount','webkitDroppedFrameCount'].every(name=>!(name in v))];
  })()`),
    [
      true,
      0,
      ['', '', '', '', ''],
      0,
      0,
      false,
      false,
      '',
      true,
      0,
      true,
      'NotSupportedError',
      true,
    ],
  );
});

test('media event handler properties use the existing event dispatcher', async () => {
  assert.equal(
    await evaluate(`(() => {
    const v=document.createElement('video');let count=0;
    for(const name of ['encrypted','waitingforkey','enterpictureinpicture','leavepictureinpicture']){
      if(v['on'+name]!==null)return false;v['on'+name]=function(e){if(this===v&&e.target===v)count++;};
      v.dispatchEvent(new Event(name));v['on'+name]=null;v.dispatchEvent(new Event(name));
    }
    return count===4;
  })()`),
    true,
  );
});
