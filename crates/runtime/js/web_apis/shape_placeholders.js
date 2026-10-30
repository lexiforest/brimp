globalThis.__brimpInstallShapePlaceholders = () => {
  // Firefox-only interfaces measured by the DataDome shape walker. They are
  // intentionally inert: scripts probe their presence and constructor shape.
  const __shapeOnlyConstructors = {
    ViewTransition: class ViewTransition {
      constructor() {
        throw new TypeError('Illegal constructor');
      }
    },
    TextTrackCue: class TextTrackCue {
      constructor() {
        throw new TypeError('Illegal constructor');
      }
    },
    VTTRegion: class VTTRegion {
      constructor() {
        throw new TypeError('Illegal constructor');
      }
    },
    IIRFilterNode: class IIRFilterNode {
      constructor() {
        throw new TypeError('Illegal constructor');
      }
    },
    RTCStatsReport: class RTCStatsReport {
      constructor() {
        throw new TypeError('Illegal constructor');
      }
    },
    PushSubscriptionOptions: class PushSubscriptionOptions {
      constructor() {
        throw new TypeError('Illegal constructor');
      }
    },
    CanvasCaptureMediaStream: class CanvasCaptureMediaStream {
      constructor() {
        throw new TypeError('Illegal constructor');
      }
    },
    SVGFEDropShadowElement: class SVGFEDropShadowElement {
      constructor() {
        throw new TypeError('Illegal constructor');
      }
    },
    SharedWorker: class SharedWorker {
      constructor() {
        throw new TypeError('Illegal constructor');
      }
    },
    PerformanceServerTiming: class PerformanceServerTiming {
      constructor() {
        throw new TypeError('Illegal constructor');
      }
    },
    URLPattern: class URLPattern {
      constructor() {
        throw new TypeError('Illegal constructor');
      }
    },
    VideoFrame: class VideoFrame {
      constructor() {
        throw new TypeError('Illegal constructor');
      }
    },
    CSSCounterStyleRule: class CSSCounterStyleRule {
      constructor() {
        throw new TypeError('Illegal constructor');
      }
    },
    MediaMetadata: class MediaMetadata {
      constructor() {
        throw new TypeError('Illegal constructor');
      }
    },
    VideoPlaybackQuality: class VideoPlaybackQuality {
      constructor() {
        throw new TypeError('Illegal constructor');
      }
    },
    MediaDeviceInfo: class MediaDeviceInfo {
      constructor() {
        throw new TypeError('Illegal constructor');
      }
    },
    RTCPeerConnectionIceErrorEvent: class RTCPeerConnectionIceErrorEvent {
      constructor() {
        throw new TypeError('Illegal constructor');
      }
    },
    RTCEncodedAudioFrame: class RTCEncodedAudioFrame {
      constructor() {
        throw new TypeError('Illegal constructor');
      }
    },
    GeolocationPosition: class GeolocationPosition {
      constructor() {
        throw new TypeError('Illegal constructor');
      }
    },
    WebTransport: class WebTransport {
      constructor() {
        throw new TypeError('Illegal constructor');
      }
    },
    GPUSupportedLimits: class GPUSupportedLimits {
      constructor() {
        throw new TypeError('Illegal constructor');
      }
    },
    ToggleEvent: class ToggleEvent {
      constructor() {
        throw new TypeError('Illegal constructor');
      }
    },
    EventCounts: class EventCounts {
      constructor() {
        throw new TypeError('Illegal constructor');
      }
    },
    RTCRtpTransceiver: class RTCRtpTransceiver {
      constructor() {
        throw new TypeError('Illegal constructor');
      }
      stop() {}
    },
  };
  for (const [name, constructor] of Object.entries(__shapeOnlyConstructors)) {
    if (typeof globalThis[name] === 'function') continue;
    Object.defineProperty(globalThis, name, {
      value: constructor,
      writable: true,
      configurable: true,
    });
  }
  if (
    globalThis.RTCRtpTransceiver &&
    typeof globalThis.RTCRtpTransceiver.prototype.stop !== 'function'
  )
    globalThis.RTCRtpTransceiver.prototype.stop = function stop() {};
  if (globalThis.Controllers === undefined)
    Object.defineProperty(globalThis, 'Controllers', {
      value: Object.freeze({}),
      writable: true,
      configurable: true,
    });

  if (typeof globalThis.Document === 'function') {
    const XMLDocument = function XMLDocument() {
      throw new TypeError('Illegal constructor');
    };
    XMLDocument.prototype = Object.create(Document.prototype);
    Object.defineProperty(XMLDocument.prototype, 'constructor', {
      value: XMLDocument,
      writable: true,
      configurable: true,
    });
    Object.defineProperty(globalThis, 'XMLDocument', {
      value: XMLDocument,
      writable: true,
      configurable: true,
    });
  }

  if (typeof globalThis.Document === 'function' && !('hasStorageAccess' in Document.prototype)) {
    Object.defineProperty(Document.prototype, 'hasStorageAccess', {
      value: function hasStorageAccess() {
        return Promise.resolve(false);
      },
      writable: true,
      enumerable: true,
      configurable: true,
    });
  }
};
