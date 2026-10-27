import test from 'node:test';
import assert from 'node:assert/strict';
import {createSession, rtcPlugin, pluginCatalog} from '../../../bindings/node/index.js';

async function evaluate(source, plugins = [rtcPlugin()]) {
  const session = await createSession({plugins});
  try { return await (await session.newPage()).evaluate(source); }
  finally { await session.close(); }
}

test('RTC local peer/channel workflow exposes defaults and explicit legacy no-ops', async () => {
  assert.deepEqual(await evaluate(`(async () => {
    const p = new RTCPeerConnection(), c = p.createDataChannel('workload');
    const defaults = [p.connectionState,p.iceConnectionState,p.iceGatheringState,p.signalingState,
      p.canTrickleIceCandidates,p.currentLocalDescription,p.currentRemoteDescription,p.localDescription,
      p.pendingLocalDescription,p.pendingRemoteDescription,p.remoteDescription,p.sctp,
      c.label,c.binaryType,c.bufferedAmount,c.bufferedAmountLowThreshold,c.id,c.maxRetransmits,
      c.negotiated,c.ordered,c.protocol,c.readyState];
    const methods = ['addIceCandidate','addTrack','addTransceiver','close','createAnswer',
      'getReceivers','getSenders','getStats','getTransceivers','removeTrack','restartIce','setConfiguration','setRemoteDescription'];
    const noops = methods.every(name => p[name]({}) === undefined) && c.send('ignored') === undefined && c.close() === undefined;
    let settled = false; p.createOffer().then(() => settled = true);
    const description = new RTCSessionDescription({type:'offer',sdp:'local payload'});
    await p.setLocalDescription(description);
    c.binaryType = 'blob'; c.bufferedAmountLowThreshold = 32;
    return [defaults,noops,settled,p.localDescription===description,p.pendingLocalDescription===description,p.getConfiguration(),
      p.connectionState,c.readyState,c.binaryType,c.bufferedAmountLowThreshold,
      ['addStream','createDTMFSender','getLocalStreams','getRemoteStreams','onaddstream','onremovestream','removeStream'].every(n=>!(n in p)),
      !('reliable' in c)&&!('maxRetransmitTime' in c),p instanceof EventTarget,c instanceof EventTarget];
  })()`), [['new','new','new','stable',null,null,null,null,null,null,null,null,
    'workload','arraybuffer',0,0,null,null,false,true,'','connecting'],true,false,true,true,{iceServers:[]},'new','connecting','blob',32,true,true,true,true]);
});

test('RTC payload serialization and manual event dispatch work without a transport', async () => {
  assert.deepEqual(await evaluate(`(() => {
    const candidate = new RTCIceCandidate({candidate:'opaque candidate',sdpMid:'audio',sdpMLineIndex:0,usernameFragment:'user'});
    const description = new RTCSessionDescription({type:'offer',sdp:'opaque SDP'});
    description.type = 'answer';
    const p = new RTCPeerConnection(),c=p.createDataChannel('events'); let calls = 0;
    p.onicecandidate = e => { if(e.candidate===candidate) calls++; };
    p.dispatchEvent(new RTCPeerConnectionIceEvent('icecandidate',{candidate}));
    p.onicecandidate = null;
    p.dispatchEvent(new RTCPeerConnectionIceEvent('icecandidate',{candidate}));
    c.onmessage = () => calls++; c.dispatchEvent(new MessageEvent('message',{data:'local'}));
    return [candidate.toJSON(),description.toJSON(),calls,new RTCPeerConnectionIceEvent('icecandidate').candidate,
      ['address','component','foundation','port','priority','protocol','relatedAddress','relatedPort','tcpType','type'].every(n=>candidate[n]===null),
      ['maxChannels','maxMessageSize','state','transport'].every(n=>typeof Object.getOwnPropertyDescriptor(RTCSctpTransport.prototype,n).get==='function'),
      Object.prototype.toString.call(c)];
  })()`), [{candidate:'opaque candidate',sdpMid:'audio',sdpMLineIndex:0,usernameFragment:'user'},
    {type:'answer',sdp:'opaque SDP'},2,null,true,true,'[object RTCDataChannel]']);
});

test('RTC is separately opt-in and exposes only constructible entry points', async () => {
  assert.deepEqual(await evaluate('[typeof RTCPeerConnection,typeof RTCIceCandidate,typeof RTCDataChannel]', []),Array(3).fill('undefined'));
  assert.deepEqual(await evaluate(`[RTCDataChannel,RTCSctpTransport].map(C=>{try{new C();return false;}catch(e){return e instanceof TypeError;}})`),[true,true]);
  assert.equal(pluginCatalog().rtc.installers.node,'rtcPlugin');
});
