globalThis.DOMImplementation = DOMImplementation;
globalThis.CustomElementRegistry = CustomElementRegistry;
globalThis.customElements = customElements;
globalThis.DOMException = DOMException;
globalThis.MutationRecord = MutationRecord;
globalThis.MutationObserver = MutationObserver;
globalThis.IntersectionObserverEntry = IntersectionObserverEntry;
globalThis.IntersectionObserver = IntersectionObserver;
globalThis.EventTarget = EventTarget;
globalThis.Event = Event;
globalThis.CustomEvent = CustomEvent;
globalThis.UIEvent = UIEvent;
globalThis.MouseEvent = MouseEvent;
globalThis.KeyboardEvent = KeyboardEvent;
globalThis.PointerEvent = PointerEvent;
globalThis.Touch = Touch;
globalThis.TouchEvent = TouchEvent;
globalThis.MessageEvent = MessageEvent;
globalThis.StorageEvent = StorageEvent;
globalThis.MessagePort = MessagePort;
globalThis.MessageChannel = MessageChannel;
globalThis.postMessage = postMessage.bind(globalThis);
globalThis.AbortSignal = AbortSignal;
globalThis.AbortController = AbortController;
globalThis.DOMTokenList = DOMTokenList;
globalThis.HTMLCollection = HTMLCollection;
globalThis.NodeList = NodeList;
globalThis.NamedNodeMap = NamedNodeMap;
globalThis.Attr = Attr;
globalThis.Node = Node;
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
globalThis.DocumentFragment = DocumentFragment;
globalThis.Range = Range;
globalThis.Selection = Selection;
globalThis.Window = Window;
globalThis.Location = Location;
globalThis.Navigator = Navigator;
globalThis.DOMRect = DOMRect;

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
class Performance extends EventTarget {
    now() { return Math.max(0, Date.now() - __performanceTimeOrigin); }
    get timeOrigin() { return __performanceTimeOrigin; }
    toJSON() { return { timeOrigin: this.timeOrigin }; }
}
globalThis.Performance = Performance;
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
window.location = globalThis.location;
window.navigator = globalThis.navigator;
globalThis.getComputedStyle = function getComputedStyle(element, pseudoElt = null) {
    if (arguments.length === 0) throw new TypeError("getComputedStyle requires an element");
    if (pseudoElt !== null) String(pseudoElt);
    return __styleDeclarationProxy(__callHost("getComputedStyle", element));
};
globalThis.getSelection = () => __selection;
globalThis.setTimeout = (callback, delay = 0) => __callHost("setTimeout", window, callback, delay);
globalThis.clearTimeout = id => __callHost("clearTimeout", window, id);
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
    "setTimeout", "clearTimeout", "queueMicrotask", "requestAnimationFrame",
    "cancelAnimationFrame", "addEventListener", "removeEventListener", "dispatchEvent",
]) {
    __markWebBuiltin(globalThis[name], `function ${name}() { [native code] }`);
}
})();
