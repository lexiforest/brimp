globalThis.DOMImplementation = DOMImplementation;
globalThis.CustomElementRegistry = CustomElementRegistry;
globalThis.customElements = customElements;
globalThis.DOMException = DOMException;
globalThis.MutationRecord = MutationRecord;
globalThis.MutationObserver = MutationObserver;
globalThis.IntersectionObserverEntry = IntersectionObserverEntry;
globalThis.IntersectionObserver = IntersectionObserver;
globalThis.ResizeObserverSize = ResizeObserverSize;
globalThis.ResizeObserverEntry = ResizeObserverEntry;
globalThis.ResizeObserver = ResizeObserver;
globalThis.EventTarget = EventTarget;
globalThis.Event = Event;
globalThis.CustomEvent = CustomEvent;
globalThis.UIEvent = UIEvent;
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
globalThis.NodeFilter = NodeFilter;
globalThis.TreeWalker = TreeWalker;
globalThis.Document = Document;
globalThis.HTMLDocument = Document;
globalThis.XMLDocument = XMLDocument;
globalThis.DOMParser = DOMParser;
globalThis.Element = Element;
globalThis.SVGElement = SVGElement;
globalThis.HTMLElement = HTMLElement;
globalThis.HTMLAnchorElement = HTMLAnchorElement;
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
globalThis.HTMLDataElement = HTMLDataElement;
globalThis.HTMLTimeElement = HTMLTimeElement;
globalThis.HTMLBRElement = HTMLBRElement;
globalThis.HTMLBodyElement = HTMLBodyElement;
globalThis.HTMLHeadingElement = HTMLHeadingElement;
globalThis.HTMLHtmlElement = HTMLHtmlElement;
globalThis.HTMLScriptElement = HTMLScriptElement;
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
globalThis.Lock = Lock;
globalThis.LockManager = LockManager;
globalThis.Crypto = Crypto;
globalThis.DOMRect = DOMRect;
globalThis.MediaQueryList = MediaQueryList;
globalThis.MediaQueryListEvent = MediaQueryListEvent;

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

function __exposeWebIdl(name, value) {
    Object.defineProperty(globalThis, name, {
        value,
        writable: true,
        enumerable: false,
        configurable: true,
    });
}

for (const [constructor, name, members] of [
    [DOMStringMap, "DOMStringMap", []],
    [DOMImplementation, "DOMImplementation", ["hasFeature", "createHTMLDocument"]],
    [FormData, "FormData", ["append", "delete", "get", "getAll", "has", "set", "entries", "keys", "values", "forEach"]],
    [History, "History", ["length", "state", "scrollRestoration", "pushState", "replaceState", "go", "back", "forward"]],
    [PopStateEvent, "PopStateEvent", ["state", "hasUAVisualTransition"]],
    [HashChangeEvent, "HashChangeEvent", ["oldURL", "newURL"]],
    [MediaQueryList, "MediaQueryList", ["media", "matches", "onchange", "addListener", "removeListener", "addEventListener", "removeEventListener", "dispatchEvent"]],
    [MediaQueryListEvent, "MediaQueryListEvent", ["media", "matches"]],
    [MediaList, "MediaList", ["mediaText", "length", "item", "appendMedium", "deleteMedium", "toString"]],
    [StyleSheet, "StyleSheet", ["type", "href", "ownerNode", "parentStyleSheet", "title", "media", "disabled"]],
    [CSSStyleSheet, "CSSStyleSheet", ["ownerRule", "cssRules", "insertRule", "deleteRule", "replace", "replaceSync", "rules", "addRule", "removeRule"]],
    [StyleSheetList, "StyleSheetList", ["item", "length"]],
    [CSSRuleList, "CSSRuleList", ["item", "length"]],
    [CSSRule, "CSSRule", ["cssText", "parentRule", "parentStyleSheet", "type"]],
    [CSSStyleRule, "CSSStyleRule", ["selectorText", "style"]],
    [CSSImportRule, "CSSImportRule", ["href", "media", "styleSheet", "layerName", "supportsText"]],
    [CSSGroupingRule, "CSSGroupingRule", ["cssRules", "insertRule", "deleteRule"]],
    [CSSConditionRule, "CSSConditionRule", ["conditionText"]],
    [CSSMediaRule, "CSSMediaRule", ["media"]],
    [CSSFontFaceRule, "CSSFontFaceRule", []],
    [CSSPageRule, "CSSPageRule", []],
    [CSSKeyframesRule, "CSSKeyframesRule", ["name", "cssRules", "appendRule", "deleteRule", "findRule", "length"]],
    [CSSKeyframeRule, "CSSKeyframeRule", ["keyText", "style"]],
    [CSSNamespaceRule, "CSSNamespaceRule", ["namespaceURI", "prefix"]],
    [CSSSupportsRule, "CSSSupportsRule", []],
    [CSSStyleDeclaration, "CSSStyleDeclaration", ["cssText", "length", "item", "getPropertyValue", "getPropertyPriority", "setProperty", "removeProperty", "parentRule"]],
    [CSSStyleProperties, "CSSStyleProperties", ["cssFloat"]],
]) {
    __makeWebIdlMembersEnumerable(constructor.prototype, members);
    __tagWebIdlPrototype(constructor, name);
    __exposeWebIdl(name, constructor);
}

__exposeWebIdl("CSS", CSS);

const __performanceTimeOrigin = Date.now();
const __performanceEntries = [];
const __performanceObservers = new Set();
const __performanceEntryToken = {};
class PerformanceEntry {
    constructor(token, name, entryType, startTime, duration) {
        if (token !== __performanceEntryToken) throw new TypeError("Illegal constructor");
        this.__name = name;
        this.__entryType = entryType;
        this.__startTime = startTime;
        this.__duration = duration;
    }
    get name() { return this.__name; }
    get entryType() { return this.__entryType; }
    get startTime() { return this.__startTime; }
    get duration() { return this.__duration; }
    toJSON() {
        return { name: this.name, entryType: this.entryType, startTime: this.startTime, duration: this.duration };
    }
}
class PerformanceMark extends PerformanceEntry {
    constructor(token, name, startTime, detail) {
        if (token !== __performanceEntryToken) throw new TypeError("Illegal constructor");
        super(token, name, "mark", startTime, 0);
        this.__detail = detail;
    }
    get detail() { return this.__detail; }
    toJSON() { return { ...super.toJSON(), detail: this.detail }; }
}
class PerformanceMeasure extends PerformanceEntry {
    constructor(token, name, startTime, duration, detail) {
        if (token !== __performanceEntryToken) throw new TypeError("Illegal constructor");
        super(token, name, "measure", startTime, duration);
        this.__detail = detail;
    }
    get detail() { return this.__detail; }
    toJSON() { return { ...super.toJSON(), detail: this.detail }; }
}
class PerformanceObserverEntryList {
    constructor(entries) { this.__entries = entries; }
    getEntries() { return this.__entries.slice(); }
    getEntriesByType(type) {
        type = String(type);
        return this.__entries.filter(entry => entry.entryType === type);
    }
    getEntriesByName(name, type) {
        name = String(name);
        return this.__entries.filter(entry => entry.name === name &&
            (type === undefined || entry.entryType === String(type)));
    }
}
class PerformanceObserver {
    constructor(callback) {
        if (typeof callback !== "function") throw new TypeError("callback must be a function");
        this.__callback = callback;
        this.__entryTypes = new Set();
        this.__records = [];
        this.__scheduled = false;
    }
    observe(options = {}) {
        if (options === null || typeof options !== "object") throw new TypeError("options must be an object");
        const types = options.entryTypes === undefined
            ? (options.type === undefined ? [] : [String(options.type)])
            : Array.from(options.entryTypes, String);
        if (!types.length) throw new TypeError("an entry type is required");
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
    takeRecords() { return this.__records.splice(0); }
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
    static get supportedEntryTypes() { return ["mark", "measure"]; }
}
const __queuePerformanceEntry = entry => {
    __performanceEntries.push(entry);
    for (const observer of __performanceObservers) observer.__enqueue(entry);
    return entry;
};
class PerformanceTiming {
    get navigationStart() { return __performanceTimeOrigin; }
    get unloadEventStart() { return 0; }
    get unloadEventEnd() { return 0; }
    get redirectStart() { return 0; }
    get redirectEnd() { return 0; }
    get fetchStart() { return __performanceTimeOrigin; }
    get domainLookupStart() { return __performanceTimeOrigin; }
    get domainLookupEnd() { return __performanceTimeOrigin; }
    get connectStart() { return __performanceTimeOrigin; }
    get connectEnd() { return __performanceTimeOrigin; }
    get secureConnectionStart() { return __performanceTimeOrigin; }
    get requestStart() { return __performanceTimeOrigin; }
    get responseStart() { return __performanceTimeOrigin; }
    get responseEnd() { return __performanceTimeOrigin; }
    get domLoading() { return __performanceTimeOrigin; }
    get domInteractive() { return __performanceTimeOrigin; }
    get domContentLoadedEventStart() { return __performanceTimeOrigin; }
    get domContentLoadedEventEnd() { return __performanceTimeOrigin; }
    get domComplete() { return __performanceTimeOrigin; }
    get loadEventStart() { return __performanceTimeOrigin; }
    get loadEventEnd() { return __performanceTimeOrigin; }
    toJSON() {
        const result = {};
        for (const name of Object.getOwnPropertyNames(PerformanceTiming.prototype)) {
            if (name !== "constructor" && name !== "toJSON") result[name] = this[name];
        }
        return result;
    }
}
const __performanceTiming = new PerformanceTiming();
class Performance extends EventTarget {
    now() { return Math.max(0, Date.now() - __performanceTimeOrigin); }
    get timeOrigin() { return __performanceTimeOrigin; }
    get timing() { return __performanceTiming; }
    getEntries() { return __performanceEntries.slice(); }
    getEntriesByType(type) {
        type = String(type);
        return __performanceEntries.filter(entry => entry.entryType === type);
    }
    getEntriesByName(name, type) {
        name = String(name);
        return __performanceEntries.filter(entry => entry.name === name &&
            (type === undefined || entry.entryType === String(type)));
    }
    mark(name, options = {}) {
        name = String(name);
        const startTime = options.startTime === undefined ? this.now() : Number(options.startTime);
        if (!Number.isFinite(startTime) || startTime < 0) throw new TypeError("startTime must be non-negative");
        const entry = new PerformanceMark(__performanceEntryToken, name, startTime, options.detail ?? null);
        return __queuePerformanceEntry(entry);
    }
    clearMarks(name) {
        const selected = name === undefined ? null : String(name);
        for (let index = __performanceEntries.length - 1; index >= 0; index--) {
            const entry = __performanceEntries[index];
            if (entry.entryType === "mark" && (selected === null || entry.name === selected)) {
                __performanceEntries.splice(index, 1);
            }
        }
    }
    measure(name, startOrOptions, endMark) {
        name = String(name);
        const resolve = value => {
            if (value === undefined) return 0;
            if (typeof value === "number") return value;
            const entries = this.getEntriesByName(String(value), "mark");
            if (!entries.length) throw new DOMException(`The mark '${value}' does not exist`, "SyntaxError");
            return entries[entries.length - 1].startTime;
        };
        let startTime;
        let endTime;
        let detail = null;
        if (startOrOptions !== null && typeof startOrOptions === "object") {
            startTime = resolve(startOrOptions.start);
            endTime = startOrOptions.duration === undefined
                ? (startOrOptions.end === undefined ? this.now() : resolve(startOrOptions.end))
                : startTime + Number(startOrOptions.duration);
            detail = startOrOptions.detail ?? null;
        } else {
            startTime = resolve(startOrOptions);
            endTime = endMark === undefined ? this.now() : resolve(endMark);
        }
        if (!Number.isFinite(startTime) || !Number.isFinite(endTime) || endTime < startTime) {
            throw new TypeError("invalid performance measure range");
        }
        const entry = new PerformanceMeasure(
            __performanceEntryToken, name, startTime, endTime - startTime, detail,
        );
        return __queuePerformanceEntry(entry);
    }
    clearMeasures(name) {
        const selected = name === undefined ? null : String(name);
        for (let index = __performanceEntries.length - 1; index >= 0; index--) {
            const entry = __performanceEntries[index];
            if (entry.entryType === "measure" && (selected === null || entry.name === selected)) {
                __performanceEntries.splice(index, 1);
            }
        }
    }
    toJSON() { return { timeOrigin: this.timeOrigin }; }
}
for (const [constructor, name, members] of [
    [PerformanceEntry, "PerformanceEntry", ["name", "entryType", "startTime", "duration", "toJSON"]],
    [PerformanceMark, "PerformanceMark", ["detail", "toJSON"]],
    [PerformanceMeasure, "PerformanceMeasure", ["detail", "toJSON"]],
    [PerformanceObserverEntryList, "PerformanceObserverEntryList", [
        "getEntries", "getEntriesByType", "getEntriesByName",
    ]],
    [PerformanceObserver, "PerformanceObserver", ["observe", "disconnect", "takeRecords"]],
    [PerformanceTiming, "PerformanceTiming", [
        "navigationStart", "unloadEventStart", "unloadEventEnd", "redirectStart", "redirectEnd",
        "fetchStart", "domainLookupStart", "domainLookupEnd", "connectStart", "connectEnd",
        "secureConnectionStart", "requestStart", "responseStart", "responseEnd", "domLoading",
        "domInteractive", "domContentLoadedEventStart", "domContentLoadedEventEnd", "domComplete",
        "loadEventStart", "loadEventEnd", "toJSON",
    ]],
    [Performance, "Performance", [
        "now", "timeOrigin", "timing", "getEntries", "getEntriesByType", "getEntriesByName",
        "mark", "clearMarks", "measure", "clearMeasures", "toJSON",
    ]],
]) {
    __makeWebIdlMembersEnumerable(constructor.prototype, members);
    __tagWebIdlPrototype(constructor, name);
    __exposeWebIdl(name, constructor);
}
globalThis.performance = new Performance();

globalThis.window = globalThis;
globalThis.self = globalThis;
globalThis.parent = globalThis;
globalThis.top = globalThis;
globalThis.opener = null;
globalThis.addEventListener = EventTarget.prototype.addEventListener.bind(globalThis);
globalThis.removeEventListener = EventTarget.prototype.removeEventListener.bind(globalThis);
globalThis.dispatchEvent = EventTarget.prototype.dispatchEvent.bind(globalThis);
Object.defineProperties(globalThis, {
    innerWidth: { get: Object.getOwnPropertyDescriptor(Window.prototype, "innerWidth").get },
    innerHeight: { get: Object.getOwnPropertyDescriptor(Window.prototype, "innerHeight").get },
    devicePixelRatio: { get: Object.getOwnPropertyDescriptor(Window.prototype, "devicePixelRatio").get },
});
window.fetch = globalThis.fetch;
globalThis.location = Object.create(Location.prototype);
globalThis.navigator = Object.create(Navigator.prototype);
globalThis.history = new History(__historyConstructorToken);
window.location = globalThis.location;
window.navigator = globalThis.navigator;
window.history = globalThis.history;
globalThis.getComputedStyle = function getComputedStyle(element, pseudoElt = null) {
    if (arguments.length === 0) throw new TypeError("getComputedStyle requires an element");
    if (pseudoElt !== null) String(pseudoElt);
    return __styleDeclarationProxy(__callHost("getComputedStyle", element));
};
globalThis.getSelection = () => __selection;
globalThis.matchMedia = matchMedia;
globalThis.setTimeout = (callback, delay = 0) => __callHost("setTimeout", window, callback, delay);
globalThis.clearTimeout = id => __callHost("clearTimeout", window, id);
globalThis.setInterval = (callback, delay = 0) => __callHost("setInterval", window, callback, delay);
globalThis.clearInterval = id => __callHost("clearInterval", window, id);
globalThis.queueMicrotask = callback => __callHost("queueMicrotask", window, callback);
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
globalThis.requestAnimationFrame = callback => {
    if (typeof callback !== "function") throw new TypeError("callback must be a function");
    const id = __nextAnimationFrameId++;
    __animationFrameCallbacks.set(id, callback);
    if (!__animationFrameScheduled) {
        __animationFrameScheduled = true;
        setTimeout(__runAnimationFrame, 16);
    }
    return id;
};
globalThis.cancelAnimationFrame = id => { __animationFrameCallbacks.delete(Number(id)); };
window.setTimeout = globalThis.setTimeout;
window.clearTimeout = globalThis.clearTimeout;
window.setInterval = globalThis.setInterval;
window.clearInterval = globalThis.clearInterval;
window.queueMicrotask = globalThis.queueMicrotask;
window.requestAnimationFrame = globalThis.requestAnimationFrame;
window.cancelAnimationFrame = globalThis.cancelAnimationFrame;

for (const name of Object.getOwnPropertyNames(globalThis)) {
    if (name === "__brimpMarkWebBuiltin") continue;
    const value = globalThis[name];
    if (typeof value === "function" && (!__initialGlobalNames.has(name) || /^[A-Z]/.test(name))) {
        __markWebBuiltinInterface(value);
        if (!__initialGlobalNames.has(name) && /^[A-Z]/.test(name)) {
            Object.defineProperty(globalThis, name, {
                value,
                writable: true,
                enumerable: false,
                configurable: true,
            });
        }
    }
}
for (const name of [
    "atob", "btoa", "fetch", "postMessage", "getComputedStyle", "getSelection",
    "setTimeout", "clearTimeout", "setInterval", "clearInterval", "queueMicrotask", "requestAnimationFrame", "matchMedia",
    "cancelAnimationFrame", "addEventListener", "removeEventListener", "dispatchEvent",
]) {
    __markWebBuiltin(globalThis[name], `function ${name}() { [native code] }`);
}
})();
