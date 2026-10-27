function installRtc({ global, profile: settings }) {
  const rtcProfile = settings.values(
    'webrtc',
    {
      ice_servers: [],
    },
    {
      version: 1,
      fields: {
        ice_servers: {
          items: {
            type: 'string',
          },
        },
      },
      capture: ({ read }) => {
        if (!globalThis.RTCPeerConnection) return { ice_servers: read(() => undefined) };
        const rtc = new RTCPeerConnection();
        try {
          return {
            ice_servers: read(() =>
              rtc
                .getConfiguration()
                .iceServers.flatMap((s) => (typeof s.urls === 'string' ? [s.urls] : s.urls)),
            ),
          };
        } finally {
          rtc.close();
        }
      },
    },
  );

  // Local API shapes only: no ICE, SDP negotiation, sockets or media transport.
  const constructionToken = Symbol('RTC internal construction');

  class RTCDataChannel extends EventTarget {
    #label;
    #binaryType = 'arraybuffer';
    #threshold = 0;
    constructor(token, label) {
      super();
      if (token !== constructionToken) throw new TypeError('Illegal constructor');
      this.#label = label;
    }
    get label() {
      return this.#label;
    }
    get binaryType() {
      return this.#binaryType;
    }
    set binaryType(value) {
      this.#binaryType = String(value);
    }
    get bufferedAmountLowThreshold() {
      return this.#threshold;
    }
    set bufferedAmountLowThreshold(value) {
      this.#threshold = Number(value) >>> 0;
    }
    get bufferedAmount() {
      return 0;
    }
    get id() {
      return null;
    }
    get maxRetransmits() {
      return null;
    }
    get negotiated() {
      return false;
    }
    get ordered() {
      return true;
    }
    get protocol() {
      return '';
    }
    get readyState() {
      return 'connecting';
    }
    // Legacy ignored-result placeholders; even close does not transition state.
    close() {}
    send() {}
  }

  class RTCPeerConnection extends EventTarget {
    #localDescription = null;
    get canTrickleIceCandidates() {
      return null;
    }
    get connectionState() {
      return 'new';
    }
    get iceConnectionState() {
      return 'new';
    }
    get iceGatheringState() {
      return 'new';
    }
    get signalingState() {
      return 'stable';
    }
    get currentLocalDescription() {
      return null;
    }
    get currentRemoteDescription() {
      return null;
    }
    get localDescription() {
      return this.#localDescription;
    }
    get pendingLocalDescription() {
      return this.#localDescription;
    }
    get pendingRemoteDescription() {
      return null;
    }
    get remoteDescription() {
      return null;
    }
    get sctp() {
      return null;
    }
    createDataChannel(label) {
      if (!arguments.length) throw new TypeError('A data channel label is required');
      return new RTCDataChannel(constructionToken, String(label));
    }
    // Preserve the legacy pending offer and local-description storage contract.
    createOffer() {
      return new Promise(() => {});
    }
    setLocalDescription(description) {
      this.#localDescription = description;
      return Promise.resolve();
    }
    addIceCandidate() {}
    addTrack() {}
    addTransceiver() {}
    close() {}
    createAnswer() {}
    getConfiguration() {
      return { iceServers: (rtcProfile.ice_servers ?? []).map((urls) => ({ urls })) };
    }
    getReceivers() {}
    getSenders() {}
    getStats() {}
    getTransceivers() {}
    removeTrack() {}
    restartIce() {}
    setConfiguration() {}
    setRemoteDescription() {}
  }

  class RTCSessionDescription {
    #type;
    #sdp;
    constructor(init) {
      if (!init) throw new TypeError('A session description is required');
      this.type = init.type;
      this.sdp = init.sdp ?? '';
    }
    get type() {
      return this.#type;
    }
    set type(value) {
      this.#type = String(value);
    }
    get sdp() {
      return this.#sdp;
    }
    set sdp(value) {
      this.#sdp = String(value);
    }
    toJSON() {
      return { type: this.type, sdp: this.sdp };
    }
  }

  class RTCIceCandidate {
    #init;
    constructor(init = {}) {
      this.#init = {
        candidate: String(init.candidate ?? ''),
        sdpMid: init.sdpMid == null ? null : String(init.sdpMid),
        sdpMLineIndex: init.sdpMLineIndex == null ? null : Number(init.sdpMLineIndex) & 65535,
        usernameFragment: init.usernameFragment == null ? null : String(init.usernameFragment),
      };
    }
    get candidate() {
      return this.#init.candidate;
    }
    get sdpMid() {
      return this.#init.sdpMid;
    }
    get sdpMLineIndex() {
      return this.#init.sdpMLineIndex;
    }
    get usernameFragment() {
      return this.#init.usernameFragment;
    }
    // Deliberate placeholders: no candidate parser or fabricated local IP.
    get address() {
      return null;
    }
    get component() {
      return null;
    }
    get foundation() {
      return null;
    }
    get port() {
      return null;
    }
    get priority() {
      return null;
    }
    get protocol() {
      return null;
    }
    get relatedAddress() {
      return null;
    }
    get relatedPort() {
      return null;
    }
    get tcpType() {
      return null;
    }
    get type() {
      return null;
    }
    toJSON() {
      return { ...this.#init };
    }
  }

  class RTCPeerConnectionIceEvent extends Event {
    #candidate;
    constructor(type, init = {}) {
      super(type, init);
      this.#candidate = init.candidate ?? null;
    }
    get candidate() {
      return this.#candidate;
    }
  }

  class RTCSctpTransport extends EventTarget {
    constructor() {
      super();
      throw new TypeError('Illegal constructor');
    }
    // Legacy shape only: no transport is created by this plugin.
    get maxChannels() {
      return '';
    }
    get maxMessageSize() {
      return '';
    }
    get state() {
      return '';
    }
    get transport() {
      return '';
    }
  }

  function eventHandlers(prototype, names) {
    for (const name of names) {
      const handlers = new WeakMap();
      Object.defineProperty(prototype, 'on' + name, {
        configurable: true,
        enumerable: true,
        get() {
          return handlers.get(this) ?? null;
        },
        set(value) {
          // Brimp's EventTarget dispatches plugin on* properties.
          handlers.set(this, typeof value === 'function' ? value : null);
        },
      });
    }
  }
  eventHandlers(RTCPeerConnection.prototype, [
    'connectionstatechange',
    'datachannel',
    'icecandidate',
    'icecandidateerror',
    'iceconnectionstatechange',
    'icegatheringstatechange',
    'negotiationneeded',
    'signalingstatechange',
    'track',
  ]);
  eventHandlers(RTCDataChannel.prototype, [
    'bufferedamountlow',
    'close',
    'error',
    'message',
    'open',
  ]);
  eventHandlers(RTCSctpTransport.prototype, ['statechange']);

  for (const constructor of [
    RTCPeerConnection,
    RTCDataChannel,
    RTCIceCandidate,
    RTCSessionDescription,
    RTCPeerConnectionIceEvent,
    RTCSctpTransport,
  ]) {
    Object.defineProperty(constructor.prototype, Symbol.toStringTag, {
      value: constructor.name,
      configurable: true,
    });
    Object.defineProperty(global, constructor.name, {
      value: constructor,
      writable: true,
      configurable: true,
    });
  }
}
