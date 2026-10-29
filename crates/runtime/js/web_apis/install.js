Object.assign(globalThis, {
  ElementInternals,
  CustomStateSet,
  CSSStyleSheet,
  StyleSheet,
  StyleSheetList,
  CSSRuleList,
  MediaList,
  CSSRule,
  CSSStyleRule,
  CSSGroupingRule,
  CSSConditionRule,
  CSSMediaRule,
  CSSSupportsRule,
  CSSFontFaceRule,
});
globalThis.DOMImplementation = DOMImplementation;
globalThis.CustomElementRegistry = CustomElementRegistry;
globalThis.customElements = customElements;
globalThis.DOMException = DOMException;
globalThis.MutationRecord = MutationRecord;
globalThis.MutationObserver = MutationObserver;
globalThis.EventTarget = EventTarget;
globalThis.Event = Event;
globalThis.CustomEvent = CustomEvent;
globalThis.ErrorEvent = ErrorEvent;
globalThis.BeforeUnloadEvent = BeforeUnloadEvent;
globalThis.UIEvent = UIEvent;
globalThis.TextEvent = TextEvent;
globalThis.MouseEvent = MouseEvent;
globalThis.WheelEvent = WheelEvent;
globalThis.FocusEvent = FocusEvent;
globalThis.ProgressEvent = ProgressEvent;
globalThis.SubmitEvent = SubmitEvent;
globalThis.KeyboardEvent = KeyboardEvent;
globalThis.PointerEvent = PointerEvent;
globalThis.Touch = Touch;
globalThis.TouchEvent = TouchEvent;
globalThis.MessageEvent = MessageEvent;
globalThis.StorageEvent = StorageEvent;
globalThis.PopStateEvent = PopStateEvent;
globalThis.HashChangeEvent = HashChangeEvent;
globalThis.MessagePort = MessagePort;
globalThis.MessageChannel = MessageChannel;
globalThis.postMessage = postMessage.bind(globalThis);
globalThis.AbortSignal = AbortSignal;
globalThis.AbortController = AbortController;
globalThis.DOMTokenList = DOMTokenList;
globalThis.DOMStringMap = DOMStringMap;
globalThis.HTMLCollection = HTMLCollection;
globalThis.HTMLFormControlsCollection = HTMLFormControlsCollection;
globalThis.NodeList = NodeList;
globalThis.NamedNodeMap = NamedNodeMap;
globalThis.Attr = Attr;
globalThis.Node = Node;
// Global event-handler surfaces measured in Safari 27.0. Event production is
// independent of property registration (for example optional media engines).
const __globalEventHandlerTypes = [
  'abort',
  'animationcancel',
  'animationend',
  'animationiteration',
  'animationstart',
  'auxclick',
  'beforecopy',
  'beforecut',
  'beforeinput',
  'beforematch',
  'beforepaste',
  'beforetoggle',
  'blur',
  'cancel',
  'canplay',
  'canplaythrough',
  'change',
  'click',
  'close',
  'command',
  'contentvisibilityautostatechange',
  'contextmenu',
  'copy',
  'cuechange',
  'cut',
  'dblclick',
  'drag',
  'dragend',
  'dragenter',
  'dragleave',
  'dragover',
  'dragstart',
  'drop',
  'durationchange',
  'emptied',
  'ended',
  'error',
  'focus',
  'formdata',
  'gotpointercapture',
  'input',
  'invalid',
  'keydown',
  'keypress',
  'keyup',
  'load',
  'loadeddata',
  'loadedmetadata',
  'loadstart',
  'lostpointercapture',
  'mousedown',
  'mouseenter',
  'mouseleave',
  'mousemove',
  'mouseout',
  'mouseover',
  'mouseup',
  'mousewheel',
  'paste',
  'pause',
  'play',
  'playing',
  'pointercancel',
  'pointerdown',
  'pointerenter',
  'pointerleave',
  'pointermove',
  'pointerout',
  'pointerover',
  'pointerup',
  'progress',
  'ratechange',
  'reset',
  'resize',
  'scroll',
  'scrollend',
  'securitypolicyviolation',
  'seeked',
  'seeking',
  'select',
  'selectionchange',
  'selectstart',
  'slotchange',
  'stalled',
  'submit',
  'suspend',
  'timeupdate',
  'toggle',
  'transitioncancel',
  'transitionend',
  'transitionrun',
  'transitionstart',
  'volumechange',
  'waiting',
  'webkitanimationend',
  'webkitanimationiteration',
  'webkitanimationstart',
  'webkitmouseforcechanged',
  'webkitmouseforcedown',
  'webkitmouseforceup',
  'webkitmouseforcewillbegin',
  'webkittransitionend',
  'wheel',
];
for (const prototype of [HTMLElement.prototype, SVGElement.prototype, Document.prototype]) {
  __defineEventHandlers(prototype, __globalEventHandlerTypes);
}
__defineEventHandlers(Document.prototype, [
  'fullscreenchange',
  'fullscreenerror',
  'pointerlockchange',
  'pointerlockerror',
  'readystatechange',
  'visibilitychange',
  'webkitfullscreenchange',
  'webkitfullscreenerror',
]);
__defineEventHandlers(Element.prototype, [
  'fullscreenchange',
  'fullscreenerror',
  'webkitfullscreenchange',
  'webkitfullscreenerror',
]);
__defineEventHandlers(HTMLMediaElement.prototype, ['encrypted', 'waitingforkey']);
__defineEventHandlers(HTMLVideoElement.prototype, [
  'enterpictureinpicture',
  'leavepictureinpicture',
]);
__defineTokenListReflection(Element.prototype, 'part', 'part');
__defineStringReflections(HTMLElement.prototype, [['nonce']]);
__defineStringReflections(SVGElement.prototype, [['nonce']]);
__defineBooleanReflections(SVGElement.prototype, [['autofocus']]);
Object.defineProperty(
  SVGElement.prototype,
  'tabIndex',
  Object.getOwnPropertyDescriptor(HTMLElement.prototype, 'tabIndex'),
);
__makeWebIdlMembersEnumerable(Element.prototype, [
  'querySelector',
  'querySelectorAll',
  'matches',
  'webkitMatchesSelector',
  'closest',
]);
__makeWebIdlMembersEnumerable(Document.prototype, ['querySelector', 'querySelectorAll']);
// Window callbacks share registration with ordinary DOM handlers; error and
// beforeunload use their event-specific argument and cancellation algorithms.
__defineEventHandlers(Window.prototype, __globalEventHandlerTypes);
__defineEventHandlers(Window.prototype, [
  'devicemotion',
  'deviceorientation',
  'focus',
  'blur',
  'focusin',
  'focusout',
  'readystatechange',
]);
__defineEventHandlers(Window.prototype, __bodyWindowEventTypes);
__defineEventHandlers(HTMLBodyElement.prototype, __bodyWindowEventTypes, true);
__defineEventHandlers(HTMLFrameSetElement.prototype, __bodyWindowEventTypes, true);
globalThis.NodeFilter = NodeFilter;
globalThis.TreeWalker = TreeWalker;
globalThis.NodeIterator = NodeIterator;
globalThis.Document = Document;
globalThis.DocumentTimeline = DocumentTimeline;
globalThis.HTMLDocument = Document;
globalThis.XMLDocument = Document;
globalThis.DOMParser = DOMParser;
globalThis.Element = Element;
globalThis.SVGElement = SVGElement;
globalThis.SVGGraphicsElement = SVGGraphicsElement;
globalThis.SVGGeometryElement = SVGGeometryElement;
globalThis.SVGSVGElement = SVGSVGElement;
globalThis.SVGTextPositioningElement = SVGTextPositioningElement;
globalThis.SVGTextElement = SVGTextElement;
globalThis.SVGTSpanElement = SVGTSpanElement;
globalThis.SVGTextPathElement = SVGTextPathElement;
globalThis.MathMLElement = MathMLElement;
globalThis.StaticRange = StaticRange;
globalThis.FragmentDirective = FragmentDirective;
globalThis.CSSFontPaletteValuesRule = CSSFontPaletteValuesRule;
globalThis.ContentVisibilityAutoStateChangeEvent = ContentVisibilityAutoStateChangeEvent;
globalThis.SVGAnimationElement = SVGAnimationElement;
globalThis.SVGAnimateElement = SVGAnimateElement;
globalThis.SVGAnimateMotionElement = SVGAnimateMotionElement;
globalThis.SVGAnimateTransformElement = SVGAnimateTransformElement;
globalThis.SVGSetElement = SVGSetElement;
globalThis.SVGMPathElement = SVGMPathElement;
globalThis.HTMLElement = HTMLElement;
globalThis.HTMLAnchorElement = HTMLAnchorElement;
globalThis.HTMLUnknownElement = HTMLUnknownElement;
globalThis.HTMLBaseElement = HTMLBaseElement;
globalThis.HTMLPictureElement = HTMLPictureElement;
globalThis.HTMLImageElement = HTMLImageElement;
globalThis.Image = Image;
globalThis.HTMLIFrameElement = HTMLIFrameElement;
globalThis.HTMLEmbedElement = HTMLEmbedElement;
globalThis.HTMLObjectElement = HTMLObjectElement;
globalThis.HTMLParamElement = HTMLParamElement;
globalThis.HTMLMediaElement = HTMLMediaElement;
globalThis.HTMLVideoElement = HTMLVideoElement;
globalThis.HTMLAudioElement = HTMLAudioElement;
globalThis.HTMLSourceElement = HTMLSourceElement;
globalThis.HTMLTrackElement = HTMLTrackElement;
globalThis.HTMLCanvasElement = HTMLCanvasElement;
globalThis.HTMLMapElement = HTMLMapElement;
globalThis.HTMLAreaElement = HTMLAreaElement;
globalThis.ValidityState = ValidityState;
globalThis.HTMLFormElement = HTMLFormElement;
globalThis.HTMLFieldSetElement = HTMLFieldSetElement;
globalThis.HTMLLegendElement = HTMLLegendElement;
globalThis.HTMLLabelElement = HTMLLabelElement;
globalThis.HTMLInputElement = HTMLInputElement;
globalThis.HTMLButtonElement = HTMLButtonElement;
globalThis.HTMLSelectElement = HTMLSelectElement;
globalThis.HTMLDataListElement = HTMLDataListElement;
globalThis.HTMLOptGroupElement = HTMLOptGroupElement;
globalThis.HTMLOptionElement = HTMLOptionElement;
globalThis.HTMLTextAreaElement = HTMLTextAreaElement;
globalThis.HTMLOutputElement = HTMLOutputElement;
globalThis.HTMLProgressElement = HTMLProgressElement;
globalThis.HTMLMeterElement = HTMLMeterElement;
globalThis.HTMLTableElement = HTMLTableElement;
globalThis.HTMLTableCaptionElement = HTMLTableCaptionElement;
globalThis.HTMLTableColElement = HTMLTableColElement;
globalThis.HTMLTableSectionElement = HTMLTableSectionElement;
globalThis.HTMLTableRowElement = HTMLTableRowElement;
globalThis.HTMLTableCellElement = HTMLTableCellElement;
globalThis.HTMLHeadElement = HTMLHeadElement;
globalThis.HTMLTitleElement = HTMLTitleElement;
globalThis.HTMLLinkElement = HTMLLinkElement;
globalThis.HTMLMetaElement = HTMLMetaElement;
globalThis.HTMLStyleElement = HTMLStyleElement;
globalThis.HTMLParagraphElement = HTMLParagraphElement;
globalThis.HTMLHRElement = HTMLHRElement;
globalThis.HTMLPreElement = HTMLPreElement;
globalThis.HTMLQuoteElement = HTMLQuoteElement;
globalThis.HTMLOListElement = HTMLOListElement;
globalThis.HTMLUListElement = HTMLUListElement;
globalThis.HTMLLIElement = HTMLLIElement;
globalThis.HTMLDListElement = HTMLDListElement;
globalThis.HTMLDivElement = HTMLDivElement;
globalThis.HTMLSpanElement = HTMLSpanElement;
globalThis.HTMLDataElement = HTMLDataElement;
globalThis.HTMLTimeElement = HTMLTimeElement;
globalThis.HTMLBRElement = HTMLBRElement;
globalThis.HTMLBodyElement = HTMLBodyElement;
globalThis.HTMLHeadingElement = HTMLHeadingElement;
globalThis.HTMLHtmlElement = HTMLHtmlElement;
globalThis.HTMLScriptElement = HTMLScriptElement;
__makeWebIdlMembersEnumerable(HTMLScriptElement.prototype, ['text']);
globalThis.HTMLTemplateElement = HTMLTemplateElement;
globalThis.HTMLSlotElement = HTMLSlotElement;
globalThis.HTMLModElement = HTMLModElement;
globalThis.HTMLDetailsElement = HTMLDetailsElement;
globalThis.HTMLMenuElement = HTMLMenuElement;
globalThis.HTMLDialogElement = HTMLDialogElement;
globalThis.HTMLMarqueeElement = HTMLMarqueeElement;
globalThis.HTMLFrameSetElement = HTMLFrameSetElement;
globalThis.HTMLFrameElement = HTMLFrameElement;
globalThis.HTMLDirectoryElement = HTMLDirectoryElement;
globalThis.HTMLFontElement = HTMLFontElement;
globalThis.CharacterData = CharacterData;
globalThis.Text = Text;
globalThis.Comment = Comment;
globalThis.DocumentType = DocumentType;
globalThis.CDATASection = CDATASection;
globalThis.ProcessingInstruction = ProcessingInstruction;
globalThis.DocumentFragment = DocumentFragment;
globalThis.Range = Range;
globalThis.Selection = Selection;
globalThis.WindowProperties = WindowProperties;
globalThis.Window = Window;
globalThis.Location = Location;
globalThis.Navigator = Navigator;
globalThis.History = History;
globalThis.UserActivation = UserActivation;
globalThis.Crypto = Crypto;

function __makeWebIdlMembersEnumerable(prototype, names) {
  for (const name of names) {
    const descriptor = Object.getOwnPropertyDescriptor(prototype, name);
    if (descriptor !== undefined && !descriptor.enumerable) {
      Object.defineProperty(prototype, name, { ...descriptor, enumerable: true });
    }
  }
}

function __tagWebIdlPrototype(constructor, name) {
  Object.defineProperty(constructor.prototype, Symbol.toStringTag, {
    value: name,
    writable: false,
    enumerable: false,
    configurable: true,
  });
}

for (const constructor of [Element, CharacterData, DocumentType]) {
  __makeWebIdlMembersEnumerable(constructor.prototype, [
    'before',
    'after',
    'replaceWith',
    'remove',
  ]);
}
__makeWebIdlMembersEnumerable(DOMTokenList.prototype, [
  'length',
  'value',
  'item',
  'contains',
  'add',
  'remove',
  'toggle',
  'replace',
  'supports',
  'toString',
]);
__makeWebIdlMembersEnumerable(HTMLCollection.prototype, ['length', 'item', 'namedItem']);
__makeWebIdlMembersEnumerable(NodeList.prototype, ['length', 'item']);
__makeWebIdlMembersEnumerable(Node.prototype, [
  'nodeType',
  'nodeName',
  'ownerDocument',
  'parentNode',
  'parentElement',
  'firstChild',
  'lastChild',
  'previousSibling',
  'nextSibling',
  'childNodes',
  'isConnected',
  'hasChildNodes',
  'getRootNode',
]);
__makeWebIdlMembersEnumerable(Node.prototype, ['baseURI', 'contains', 'isSameNode', 'isEqualNode']);
__makeWebIdlMembersEnumerable(Node.prototype, [
  'textContent',
  'nodeValue',
  'normalize',
  'cloneNode',
]);
__makeWebIdlMembersEnumerable(Node.prototype, [
  'appendChild',
  'insertBefore',
  'replaceChild',
  'removeChild',
]);
__makeWebIdlMembersEnumerable(Node.prototype, ['compareDocumentPosition']);
for (const constructor of [Document, Element, DocumentFragment]) {
  __makeWebIdlMembersEnumerable(constructor.prototype, [
    'children',
    'childElementCount',
    'firstElementChild',
    'lastElementChild',
  ]);
}
for (const constructor of [Element, CharacterData]) {
  __makeWebIdlMembersEnumerable(constructor.prototype, [
    'previousElementSibling',
    'nextElementSibling',
  ]);
}
__makeWebIdlMembersEnumerable(CharacterData.prototype, [
  'data',
  'length',
  'substringData',
  'appendData',
  'insertData',
  'deleteData',
  'replaceData',
]);
__makeWebIdlMembersEnumerable(Text.prototype, ['wholeText', 'splitText']);
__makeWebIdlMembersEnumerable(Attr.prototype, [
  'name',
  'localName',
  'prefix',
  'namespaceURI',
  'specified',
  'ownerElement',
  'value',
]);
__makeWebIdlMembersEnumerable(Document.prototype, [
  'doctype',
  'compatMode',
  'characterSet',
  'charset',
  'inputEncoding',
  'URL',
  'documentURI',
  'contentType',
  'title',
  'dir',
  'documentElement',
  'head',
  'body',
]);
__makeWebIdlMembersEnumerable(Document.prototype, [
  'forms',
  'images',
  'embeds',
  'plugins',
  'links',
  'scripts',
  'anchors',
  'applets',
]);
__makeWebIdlMembersEnumerable(HTMLInputElement.prototype, [
  'files',
  'list',
  'webkitdirectory',
  'webkitEntries',
  'value',
  'defaultValue',
  'alpha',
  'autocomplete',
  'min',
  'max',
  'step',
  'indeterminate',
  'checked',
]);
__makeWebIdlMembersEnumerable(HTMLTextAreaElement.prototype, ['value', 'defaultValue']);
__makeWebIdlMembersEnumerable(HTMLAnchorElement.prototype, [
  'text',
  'href',
  'origin',
  'protocol',
  'username',
  'password',
  'host',
  'hostname',
  'port',
  'pathname',
  'search',
  'hash',
  'toString',
]);
__makeWebIdlMembersEnumerable(HTMLTitleElement.prototype, ['text']);
__makeWebIdlMembersEnumerable(HTMLIFrameElement.prototype, ['contentDocument', 'contentWindow']);
__makeWebIdlMembersEnumerable(HTMLImageElement.prototype, [
  'complete',
  'currentSrc',
  'naturalWidth',
  'naturalHeight',
  'x',
  'y',
  'decode',
]);
__makeWebIdlMembersEnumerable(ErrorEvent.prototype, [
  'message',
  'filename',
  'lineno',
  'colno',
  'error',
]);
__makeWebIdlMembersEnumerable(Event.prototype, [
  'type',
  'bubbles',
  'cancelable',
  'composed',
  'target',
  'srcElement',
  'currentTarget',
  'eventPhase',
  'defaultPrevented',
  'timeStamp',
  'preventDefault',
  'stopPropagation',
  'stopImmediatePropagation',
  'composedPath',
  'initEvent',
  'cancelBubble',
  'returnValue',
]);
__makeWebIdlMembersEnumerable(EventTarget.prototype, [
  'addEventListener',
  'removeEventListener',
  'dispatchEvent',
]);
__makeWebIdlMembersEnumerable(CustomEvent.prototype, ['detail', 'initCustomEvent']);
__makeWebIdlMembersEnumerable(UIEvent.prototype, ['detail', 'view', 'which', 'initUIEvent']);
__makeWebIdlMembersEnumerable(TextEvent.prototype, ['data', 'initTextEvent']);
__makeWebIdlMembersEnumerable(MouseEvent.prototype, [
  'screenX',
  'screenY',
  'clientX',
  'clientY',
  'button',
  'buttons',
  'relatedTarget',
  'movementX',
  'movementY',
  'ctrlKey',
  'shiftKey',
  'altKey',
  'metaKey',
  'fromElement',
  'toElement',
  'x',
  'y',
  'pageX',
  'pageY',
  'offsetX',
  'offsetY',
  'layerX',
  'layerY',
  'getModifierState',
  'initMouseEvent',
]);
__makeWebIdlMembersEnumerable(KeyboardEvent.prototype, [
  'key',
  'code',
  'location',
  'repeat',
  'isComposing',
  'charCode',
  'keyCode',
  'ctrlKey',
  'shiftKey',
  'altKey',
  'metaKey',
  'getModifierState',
  'initKeyboardEvent',
]);
__makeWebIdlMembersEnumerable(PointerEvent.prototype, [
  'pointerId',
  'width',
  'height',
  'pressure',
  'tangentialPressure',
  'tiltX',
  'tiltY',
  'twist',
  'pointerType',
  'isPrimary',
  'altitudeAngle',
  'azimuthAngle',
  'getCoalescedEvents',
  'getPredictedEvents',
]);
__makeWebIdlMembersEnumerable(BeforeUnloadEvent.prototype, ['returnValue']);
__makeWebIdlMembersEnumerable(HTMLElement.prototype, [
  'offsetHeight',
  'offsetLeft',
  'offsetParent',
  'offsetTop',
  'offsetWidth',
  'title',
  'lang',
  'accessKey',
  'dir',
  'autofocus',
  'hidden',
  'contentEditable',
  'draggable',
  'spellcheck',
  'translate',
]);
for (const [constructor, names] of [
  [
    Element,
    ['slot', 'before', 'after', 'replaceWith', 'remove', 'prepend', 'append', 'replaceChildren'],
  ],
  [CharacterData, ['before', 'after', 'replaceWith', 'remove']],
  [DocumentType, ['before', 'after', 'replaceWith', 'remove']],
  [Document, ['prepend', 'append', 'replaceChildren']],
  [DocumentFragment, ['prepend', 'append', 'replaceChildren']],
]) {
  Object.defineProperty(constructor.prototype, Symbol.unscopables, {
    value: Object.assign(
      Object.create(null),
      Object.fromEntries(names.map((name) => [name, true])),
    ),
    configurable: true,
  });
}

function __exposeWebIdl(name, value) {
  Object.defineProperty(globalThis, name, {
    value,
    writable: true,
    enumerable: false,
    configurable: true,
  });
}

for (const [constructor, name, members] of [
  [DOMStringMap, 'DOMStringMap', []],
  [DOMImplementation, 'DOMImplementation', ['hasFeature', 'createHTMLDocument']],
  [
    DocumentType,
    'DocumentType',
    ['name', 'publicId', 'systemId', 'before', 'after', 'replaceWith', 'remove'],
  ],
  [
    FormData,
    'FormData',
    ['append', 'delete', 'get', 'getAll', 'has', 'set', 'entries', 'keys', 'values', 'forEach'],
  ],
  [
    History,
    'History',
    ['length', 'state', 'scrollRestoration', 'pushState', 'replaceState', 'go', 'back', 'forward'],
  ],
  [PopStateEvent, 'PopStateEvent', ['state', 'hasUAVisualTransition']],
  [HashChangeEvent, 'HashChangeEvent', ['oldURL', 'newURL']],
  [
    CSSStyleDeclaration,
    'CSSStyleDeclaration',
    [
      'cssText',
      'length',
      'item',
      'getPropertyValue',
      'getPropertyPriority',
      'setProperty',
      'removeProperty',
      'parentRule',
    ],
  ],
  [CSSStyleProperties, 'CSSStyleProperties', ['cssFloat']],
]) {
  __makeWebIdlMembersEnumerable(constructor.prototype, members);
  __tagWebIdlPrototype(constructor, name);
  __exposeWebIdl(name, constructor);
}

__exposeWebIdl('CSS', CSS);

const __performanceTimeOrigin = __nativeHost('clockOrigin');
let __performanceOffsets = { navigation: 0, domContentLoaded: 0, load: 0 };
const __performanceEntries = [];
const __performanceObservers = new Set();
const __performanceEntryToken = {};
class PerformanceEntry {
  constructor(token, name, entryType, startTime, duration) {
    if (token !== __performanceEntryToken) throw new TypeError('Illegal constructor');
    this.__name = name;
    this.__entryType = entryType;
    this.__startTime = startTime;
    this.__duration = duration;
  }
  get name() {
    return this.__name;
  }
  get entryType() {
    return this.__entryType;
  }
  get startTime() {
    return this.__startTime;
  }
  get duration() {
    return this.__duration;
  }
  toJSON() {
    return {
      name: this.name,
      entryType: this.entryType,
      startTime: this.startTime,
      duration: this.duration,
    };
  }
}
class PerformanceMark extends PerformanceEntry {
  constructor(token, name, startTime, detail) {
    if (token !== __performanceEntryToken) throw new TypeError('Illegal constructor');
    super(token, name, 'mark', startTime, 0);
    this.__detail = detail;
  }
  get detail() {
    return this.__detail;
  }
  toJSON() {
    return { ...super.toJSON(), detail: this.detail };
  }
}
class PerformanceMeasure extends PerformanceEntry {
  constructor(token, name, startTime, duration, detail) {
    if (token !== __performanceEntryToken) throw new TypeError('Illegal constructor');
    super(token, name, 'measure', startTime, duration);
    this.__detail = detail;
  }
  get detail() {
    return this.__detail;
  }
  toJSON() {
    return { ...super.toJSON(), detail: this.detail };
  }
}
class PerformanceObserverEntryList {
  constructor(entries) {
    this.__entries = entries;
  }
  getEntries() {
    return this.__entries.slice();
  }
  getEntriesByType(type) {
    type = String(type);
    return this.__entries.filter((entry) => entry.entryType === type);
  }
  getEntriesByName(name, type) {
    name = String(name);
    return this.__entries.filter(
      (entry) => entry.name === name && (type === undefined || entry.entryType === String(type)),
    );
  }
}
class PerformanceObserver {
  constructor(callback) {
    if (typeof callback !== 'function') throw new TypeError('callback must be a function');
    this.__callback = callback;
    this.__entryTypes = new Set();
    this.__records = [];
    this.__scheduled = false;
  }
  observe(options = {}) {
    if (options === null || typeof options !== 'object')
      throw new TypeError('options must be an object');
    const types =
      options.entryTypes === undefined
        ? options.type === undefined
          ? []
          : [String(options.type)]
        : Array.from(options.entryTypes, String);
    if (!types.length) throw new TypeError('an entry type is required');
    this.__entryTypes = new Set(types);
    __performanceObservers.add(this);
    if (options.buffered) {
      for (const entry of __performanceEntries) {
        if (this.__entryTypes.has(entry.entryType)) this.__records.push(entry);
      }
      this.__schedule();
    }
  }
  disconnect() {
    __performanceObservers.delete(this);
    this.__entryTypes.clear();
    this.__records.length = 0;
    this.__scheduled = false;
  }
  takeRecords() {
    return this.__records.splice(0);
  }
  __enqueue(entry) {
    if (!this.__entryTypes.has(entry.entryType)) return;
    this.__records.push(entry);
    this.__schedule();
  }
  __schedule() {
    if (this.__scheduled || !this.__records.length) return;
    this.__scheduled = true;
    queueMicrotask(() => {
      this.__scheduled = false;
      const records = this.takeRecords();
      if (records.length) this.__callback(new PerformanceObserverEntryList(records), this);
    });
  }
  static get supportedEntryTypes() {
    return ['mark', 'measure'];
  }
}
const __queuePerformanceEntry = (entry) => {
  __performanceEntries.push(entry);
  for (const observer of __performanceObservers) observer.__enqueue(entry);
  return entry;
};
class PerformanceTiming {
  // Offsets are profile values relative to navigation start; this is not a
  // resource-timing backend and no request measurements are produced.
  get navigationStart() {
    return __performanceTimeOrigin;
  }
  get unloadEventStart() {
    return 0;
  }
  get unloadEventEnd() {
    return 0;
  }
  get redirectStart() {
    return 0;
  }
  get redirectEnd() {
    return 0;
  }
  get fetchStart() {
    return __performanceTimeOrigin + __performanceOffsets.navigation;
  }
  get domainLookupStart() {
    return __performanceTimeOrigin + __performanceOffsets.navigation;
  }
  get domainLookupEnd() {
    return __performanceTimeOrigin + __performanceOffsets.navigation;
  }
  get connectStart() {
    return __performanceTimeOrigin + __performanceOffsets.navigation;
  }
  get connectEnd() {
    return __performanceTimeOrigin + __performanceOffsets.navigation;
  }
  get secureConnectionStart() {
    return __performanceTimeOrigin + __performanceOffsets.navigation;
  }
  get requestStart() {
    return __performanceTimeOrigin + __performanceOffsets.navigation;
  }
  get responseStart() {
    return __performanceTimeOrigin + __performanceOffsets.navigation;
  }
  get responseEnd() {
    return __performanceTimeOrigin + __performanceOffsets.navigation;
  }
  get domLoading() {
    return __performanceTimeOrigin + __performanceOffsets.navigation;
  }
  get domInteractive() {
    return __performanceTimeOrigin + __performanceOffsets.domContentLoaded;
  }
  get domContentLoadedEventStart() {
    return __performanceTimeOrigin + __performanceOffsets.domContentLoaded;
  }
  get domContentLoadedEventEnd() {
    return __performanceTimeOrigin + __performanceOffsets.domContentLoaded;
  }
  get domComplete() {
    return __performanceTimeOrigin + __performanceOffsets.load;
  }
  get loadEventStart() {
    return __performanceTimeOrigin + __performanceOffsets.load;
  }
  get loadEventEnd() {
    return __performanceTimeOrigin + __performanceOffsets.load;
  }
  toJSON() {
    const result = {};
    for (const name of Object.getOwnPropertyNames(PerformanceTiming.prototype)) {
      if (name !== 'constructor' && name !== 'toJSON') result[name] = this[name];
    }
    return result;
  }
}
const __performanceTiming = new PerformanceTiming();
globalThis.__brimpInstallPerformance = (profile) => {
  __performanceOffsets = {
    navigation: profile.timing.navigation_start_offset_ms,
    domContentLoaded: profile.timing.dom_content_loaded_ms,
    load: profile.timing.load_event_ms,
  };
};
class PerformanceNavigation {
  constructor(token) {
    if (token !== __performanceEntryToken) throw new TypeError('Illegal constructor');
  }
  // Legacy initial-navigation placeholders; no redirect/history timing backend.
  get redirectCount() {
    return 0;
  }
  get type() {
    return 0;
  }
  toJSON() {
    return { type: this.type, redirectCount: this.redirectCount };
  }
}
for (const [name, value] of Object.entries({
  TYPE_NAVIGATE: 0,
  TYPE_RELOAD: 1,
  TYPE_BACK_FORWARD: 2,
  TYPE_RESERVED: 255,
})) {
  Object.defineProperty(PerformanceNavigation, name, { value, enumerable: true });
  Object.defineProperty(PerformanceNavigation.prototype, name, { value, enumerable: true });
}
const __performanceNavigation = new PerformanceNavigation(__performanceEntryToken);
class PerformancePaintTiming extends PerformanceEntry {}
class Performance extends EventTarget {
  now() {
    return __nativeHost('clockNow');
  }
  get timeOrigin() {
    return __performanceTimeOrigin;
  }
  get timing() {
    return __performanceTiming;
  }
  get navigation() {
    return __performanceNavigation;
  }
  // Legacy placeholders: no event counting or resource timing production.
  get eventCounts() {
    return '';
  }
  clearResourceTimings() {}
  setResourceTimingBufferSize() {}
  getEntries() {
    return __performanceEntries.slice();
  }
  getEntriesByType(type) {
    type = String(type);
    return __performanceEntries.filter((entry) => entry.entryType === type);
  }
  getEntriesByName(name, type) {
    name = String(name);
    return __performanceEntries.filter(
      (entry) => entry.name === name && (type === undefined || entry.entryType === String(type)),
    );
  }
  mark(name, options = {}) {
    name = String(name);
    const startTime = options.startTime === undefined ? this.now() : Number(options.startTime);
    if (!Number.isFinite(startTime) || startTime < 0)
      throw new TypeError('startTime must be non-negative');
    const entry = new PerformanceMark(
      __performanceEntryToken,
      name,
      startTime,
      options.detail ?? null,
    );
    return __queuePerformanceEntry(entry);
  }
  clearMarks(name) {
    const selected = name === undefined ? null : String(name);
    for (let index = __performanceEntries.length - 1; index >= 0; index--) {
      const entry = __performanceEntries[index];
      if (entry.entryType === 'mark' && (selected === null || entry.name === selected)) {
        __performanceEntries.splice(index, 1);
      }
    }
  }
  measure(name, startOrOptions, endMark) {
    name = String(name);
    const resolve = (value) => {
      if (value === undefined) return 0;
      if (typeof value === 'number') return value;
      const entries = this.getEntriesByName(String(value), 'mark');
      if (!entries.length)
        throw new DOMException(`The mark '${value}' does not exist`, 'SyntaxError');
      return entries[entries.length - 1].startTime;
    };
    let startTime;
    let endTime;
    let detail = null;
    if (startOrOptions !== null && typeof startOrOptions === 'object') {
      startTime = resolve(startOrOptions.start);
      endTime =
        startOrOptions.duration === undefined
          ? startOrOptions.end === undefined
            ? this.now()
            : resolve(startOrOptions.end)
          : startTime + Number(startOrOptions.duration);
      detail = startOrOptions.detail ?? null;
    } else {
      startTime = resolve(startOrOptions);
      endTime = endMark === undefined ? this.now() : resolve(endMark);
    }
    if (!Number.isFinite(startTime) || !Number.isFinite(endTime) || endTime < startTime) {
      throw new TypeError('invalid performance measure range');
    }
    const entry = new PerformanceMeasure(
      __performanceEntryToken,
      name,
      startTime,
      endTime - startTime,
      detail,
    );
    return __queuePerformanceEntry(entry);
  }
  clearMeasures(name) {
    const selected = name === undefined ? null : String(name);
    for (let index = __performanceEntries.length - 1; index >= 0; index--) {
      const entry = __performanceEntries[index];
      if (entry.entryType === 'measure' && (selected === null || entry.name === selected)) {
        __performanceEntries.splice(index, 1);
      }
    }
  }
  toJSON() {
    return { timeOrigin: this.timeOrigin };
  }
}
for (const [constructor, name, members] of [
  [PerformanceEntry, 'PerformanceEntry', ['name', 'entryType', 'startTime', 'duration', 'toJSON']],
  [PerformancePaintTiming, 'PerformancePaintTiming', []],
  [PerformanceNavigation, 'PerformanceNavigation', ['redirectCount', 'type', 'toJSON']],
  [PerformanceMark, 'PerformanceMark', ['detail', 'toJSON']],
  [PerformanceMeasure, 'PerformanceMeasure', ['detail', 'toJSON']],
  [
    PerformanceObserverEntryList,
    'PerformanceObserverEntryList',
    ['getEntries', 'getEntriesByType', 'getEntriesByName'],
  ],
  [PerformanceObserver, 'PerformanceObserver', ['observe', 'disconnect', 'takeRecords']],
  [
    PerformanceTiming,
    'PerformanceTiming',
    [
      'navigationStart',
      'unloadEventStart',
      'unloadEventEnd',
      'redirectStart',
      'redirectEnd',
      'fetchStart',
      'domainLookupStart',
      'domainLookupEnd',
      'connectStart',
      'connectEnd',
      'secureConnectionStart',
      'requestStart',
      'responseStart',
      'responseEnd',
      'domLoading',
      'domInteractive',
      'domContentLoadedEventStart',
      'domContentLoadedEventEnd',
      'domComplete',
      'loadEventStart',
      'loadEventEnd',
      'toJSON',
    ],
  ],
  [
    Performance,
    'Performance',
    [
      'now',
      'timeOrigin',
      'timing',
      'navigation',
      'eventCounts',
      'clearResourceTimings',
      'setResourceTimingBufferSize',
      'getEntries',
      'getEntriesByType',
      'getEntriesByName',
      'mark',
      'clearMarks',
      'measure',
      'clearMeasures',
      'toJSON',
    ],
  ],
]) {
  __makeWebIdlMembersEnumerable(constructor.prototype, members);
  __tagWebIdlPrototype(constructor, name);
  __exposeWebIdl(name, constructor);
}
__defineEventHandlers(Performance.prototype, ['resourcetimingbufferfull']);
globalThis.performance = new Performance();

globalThis.window = globalThis;
globalThis.self = globalThis;
globalThis.parent = globalThis;
globalThis.top = globalThis;
globalThis.opener = null;
globalThis.addEventListener = EventTarget.prototype.addEventListener.bind(globalThis);
globalThis.removeEventListener = EventTarget.prototype.removeEventListener.bind(globalThis);
globalThis.dispatchEvent = EventTarget.prototype.dispatchEvent.bind(globalThis);
Object.defineProperties(globalThis, {});
window.fetch = globalThis.fetch;
const __windowLocation = Object.create(Location.prototype);
Object.defineProperty(__windowLocation, Symbol.toPrimitive, { value: undefined });
Object.defineProperty(globalThis, 'location', {
  get() {
    return __windowLocation;
  },
  set(value) {
    __windowLocation.href = value;
  },
  enumerable: true,
  configurable: true,
});
globalThis.navigator = Object.create(Navigator.prototype);
globalThis.history = new History(__historyConstructorToken);
window.navigator = globalThis.navigator;
window.history = globalThis.history;

globalThis.getSelection = () => __selection;
globalThis.setTimeout = (callback, delay = 0) => __callHost('setTimeout', window, callback, delay);
globalThis.clearTimeout = (id) => __callHost('clearTimeout', window, id);
globalThis.setInterval = (callback, delay = 0) =>
  __callHost('setInterval', window, callback, delay);
globalThis.clearInterval = (id) => __callHost('clearInterval', window, id);
// Use the engine's Promise job queue, shared with async/await. Capture the
// intrinsics before page code can replace them. Callback errors are reported as
// task errors rather than becoming unhandled promise rejections.
const __enqueueJob = Function.prototype.call.bind(Promise.prototype.then, Promise.resolve());
globalThis.queueMicrotask = (callback) => {
  if (typeof callback !== 'function') throw new TypeError('callback must be callable');
  __enqueueJob(() => {
    try {
      callback();
    } catch (error) {
      setTimeout(() => {
        throw error;
      }, 0);
    }
  });
};
let __nextAnimationFrameId = 1;
let __animationFrameScheduled = false;
const __animationFrameCallbacks = new Map();
function __runAnimationFrame() {
  __animationFrameScheduled = false;
  const callbacks = [...__animationFrameCallbacks];
  __animationFrameCallbacks.clear();
  const timestamp = performance.now();
  for (const [, callback] of callbacks) callback(timestamp);
}
globalThis.requestAnimationFrame = (callback) => {
  if (typeof callback !== 'function') throw new TypeError('callback must be a function');
  const id = __nextAnimationFrameId++;
  __animationFrameCallbacks.set(id, callback);
  if (!__animationFrameScheduled) {
    __animationFrameScheduled = true;
    setTimeout(__runAnimationFrame, 16);
  }
  return id;
};
globalThis.cancelAnimationFrame = (id) => {
  __animationFrameCallbacks.delete(Number(id));
};
const __isFirefox = /Firefox\//.test(navigator.userAgent);
if (!__isFirefox) {
  globalThis.webkitRequestAnimationFrame = globalThis.requestAnimationFrame;
  globalThis.webkitCancelAnimationFrame = globalThis.cancelAnimationFrame;
}
window.setTimeout = globalThis.setTimeout;
window.clearTimeout = globalThis.clearTimeout;
window.setInterval = globalThis.setInterval;
window.clearInterval = globalThis.clearInterval;
window.queueMicrotask = globalThis.queueMicrotask;
window.requestAnimationFrame = globalThis.requestAnimationFrame;
window.cancelAnimationFrame = globalThis.cancelAnimationFrame;

for (const name of Object.getOwnPropertyNames(globalThis)) {
  if (__initialGlobalNames.has(name)) continue;
  const value = globalThis[name];
  if (typeof value !== 'function') continue;
  __markWebBuiltinInterface(value);
  if (/^[A-Z]/.test(name)) {
    Object.defineProperty(globalThis, name, {
      value,
      writable: true,
      enumerable: false,
      configurable: true,
    });
  }
}
for (const name of [
  'atob',
  'btoa',
  'fetch',
  'postMessage',
  'getSelection',
  'setTimeout',
  'clearTimeout',
  'setInterval',
  'clearInterval',
  'queueMicrotask',
  'requestAnimationFrame',
  'cancelAnimationFrame',
  'addEventListener',
  'removeEventListener',
  'dispatchEvent',
]) {
  __markWebBuiltin(globalThis[name], name, 'value');
}
