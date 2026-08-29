const __legacyEncodingBlocks = new Map();
function __legacyQueryEncode(label, input) {
    if (__asciiLowercase(label) === "iso-2022-jp") {
        return __callHost("legacyQueryEncode", window, label, input);
    }
    let blocks = __legacyEncodingBlocks.get(label);
    if (!blocks) {
        blocks = new Map();
        __legacyEncodingBlocks.set(label, blocks);
    }
    let output = "";
    for (const character of input) {
        const codePoint = character.codePointAt(0);
        if (codePoint < 128) {
            output += character;
            continue;
        }
        const blockStart = codePoint - (codePoint % 256);
        let block = blocks.get(blockStart);
        if (!block) {
            block = JSON.parse(__callHost("legacyQueryEncodeBlock", window, label, blockStart));
            blocks.set(blockStart, block);
        }
        output += block[codePoint - blockStart];
    }
    return output;
}
class HTMLBaseElement extends HTMLElement {
    get href() { return __callHost("elementUrl", this, "href"); }
    set href(value) { this.setAttribute("href", value); }
}

function __illegalHtmlElementConstructor() {
    throw new TypeError("Illegal constructor");
}

function __defineStringReflections(prototype, definitions) {
    for (const [property, attribute = property.toLowerCase()] of definitions) {
        Object.defineProperty(prototype, property, {
            configurable: true, enumerable: true,
            get() { return this.getAttribute(attribute) ?? ""; },
            set(value) { this.setAttribute(attribute, String(value)); },
        });
    }
}

function __defineBooleanReflections(prototype, definitions) {
    for (const [property, attribute = property.toLowerCase()] of definitions) {
        Object.defineProperty(prototype, property, {
            configurable: true, enumerable: true,
            get() { return this.hasAttribute(attribute); },
            set(value) { __setReflectedBoolean(this, attribute, value); },
        });
    }
}

function __parseNonnegativeInteger(value) {
    const match = /^[\t\n\f\r ]*([+-]?\d+)/.exec(value);
    if (!match) return null;
    const number = Number(match[1]);
    if (number === 0) return 0;
    return Number.isSafeInteger(number) && number > 0 && number <= 2147483647 ? number : null;
}

function __defineUnsignedReflections(prototype, definitions) {
    for (const [property, attribute, defaultValue = 0] of definitions) {
        Object.defineProperty(prototype, property, {
            configurable: true, enumerable: true,
            get() {
                const value = this.getAttribute(attribute);
                return value === null ? defaultValue : (__parseNonnegativeInteger(value) ?? defaultValue);
            },
            set(value) {
                const converted = Number(value) >>> 0;
                this.setAttribute(attribute, String(converted <= 2147483647 ? converted : defaultValue));
            },
        });
    }
}

function __defineUrlReflections(prototype, definitions) {
    for (const [property, attribute = property.toLowerCase()] of definitions) {
        Object.defineProperty(prototype, property, {
            configurable: true, enumerable: true,
            get() { return this.hasAttribute(attribute) ? __callHost("elementUrl", this, attribute) : ""; },
            set(value) { this.setAttribute(attribute, __toUSVString(value)); },
        });
    }
}

function __defineActionUrlReflection(prototype, property, attribute) {
    Object.defineProperty(prototype, property, {
        configurable: true, enumerable: true,
        get() { return __callHost("elementUrl", this, attribute); },
        set(value) { this.setAttribute(attribute, __toUSVString(value)); },
    });
}

function __defineEnumReflection(prototype, property, attribute, keywords, defaultValue, invalidValue, aliases = {}, nullable = false) {
    Object.defineProperty(prototype, property, {
        configurable: true, enumerable: true,
        get() {
            const raw = this.getAttribute(attribute);
            if (raw === null) return defaultValue;
            const value = __asciiLowercase(raw);
            if (Object.prototype.hasOwnProperty.call(aliases, value)) return aliases[value];
            return keywords.includes(value) ? value : invalidValue;
        },
        set(value) {
            if (nullable && value == null) this.removeAttribute(attribute);
            else this.setAttribute(attribute, String(value));
        },
    });
}

function __asciiLowercase(value) {
    return String(value).replace(/[A-Z]/g, character => String.fromCharCode(character.charCodeAt(0) + 32));
}

function __defineNullAsEmptyStringReflection(prototype, property, attribute) {
    Object.defineProperty(prototype, property, {
        configurable: true, enumerable: true,
        get() { return this.getAttribute(attribute) ?? ""; },
        set(value) { this.setAttribute(attribute, value === null ? "" : String(value)); },
    });
}

function __defineTokenListReflection(prototype, property, attribute) {
    const lists = new WeakMap();
    Object.defineProperty(prototype, property, {
        configurable: true, enumerable: true,
        get() {
            let list = lists.get(this);
            if (!list) { list = new DOMTokenList(this, attribute); lists.set(this, list); }
            return list;
        },
    });
}

function __parseInteger(value) {
    const match = /^[\t\n\f\r ]*([+-]?\d+)/.exec(value);
    if (!match) return null;
    const number = Number(match[1]);
    if (number === 0) return 0;
    return Number.isSafeInteger(number) && number >= -2147483648 && number <= 2147483647 ? number : null;
}

function __toLong(value) {
    let number = Number(value);
    if (!Number.isFinite(number) || number === 0) return 0;
    number = Math.trunc(number);
    number = ((number % 4294967296) + 4294967296) % 4294967296;
    return number >= 2147483648 ? number - 4294967296 : number;
}

function __defineLongReflections(prototype, definitions) {
    for (const [property, attribute = property.toLowerCase(), defaultValue = 0, nonnegative = false] of definitions) {
        Object.defineProperty(prototype, property, {
            configurable: true, enumerable: true,
            get() {
                const parsed = __parseInteger(this.getAttribute(attribute) ?? "");
                return parsed === null || (nonnegative && parsed < 0) ? defaultValue : parsed;
            },
            set(value) {
                const converted = __toLong(value);
                if (nonnegative && converted < 0) throw new DOMException("value must be non-negative", "IndexSizeError");
                this.setAttribute(attribute, String(converted));
            },
        });
    }
}

function __definePositiveUnsignedReflections(prototype, definitions) {
    for (const [property, attribute = property.toLowerCase(), defaultValue = 1, fallback = false] of definitions) {
        Object.defineProperty(prototype, property, {
            configurable: true, enumerable: true,
            get() {
                const parsed = __parseNonnegativeInteger(this.getAttribute(attribute) ?? "");
                return parsed === null || parsed === 0 ? defaultValue : parsed;
            },
            set(value) {
                const converted = Number(value) >>> 0;
                if (!fallback && converted === 0) throw new DOMException("value must be greater than zero", "IndexSizeError");
                this.setAttribute(attribute, String(converted > 0 && converted <= 2147483647 ? converted : defaultValue));
            },
        });
    }
}

function __defineClampedUnsignedReflections(prototype, definitions) {
    for (const [property, attribute, defaultValue, minimum, maximum] of definitions) {
        Object.defineProperty(prototype, property, {
            configurable: true, enumerable: true,
            get() {
                const match = /^[\t\n\f\r ]*([+-]?\d+)/.exec(this.getAttribute(attribute) ?? "");
                const parsed = match ? Number(match[1]) : NaN;
                if (!Number.isSafeInteger(parsed) || parsed < 0) return defaultValue;
                return Math.min(maximum, Math.max(minimum, parsed));
            },
            set(value) {
                const converted = Number(value) >>> 0;
                this.setAttribute(attribute, String(converted <= 2147483647 ? converted : defaultValue));
            },
        });
    }
}

function __parseFloatingPoint(value) {
    const match = /^[\t\n\f\r ]*([+-]?(?:\d+(?:\.\d*)?|\.\d+)(?:[eE][+-]?\d+)?)/.exec(value);
    if (!match) return null;
    const number = Number(match[1]);
    return Number.isFinite(number) ? (number === 0 ? 0 : number) : null;
}

function __defineDoubleReflections(prototype, definitions) {
    for (const [property, attribute = property.toLowerCase(), defaultValue = 0, positive = false] of definitions) {
        Object.defineProperty(prototype, property, {
            configurable: true, enumerable: true,
            get() {
                const parsed = __parseFloatingPoint(this.getAttribute(attribute) ?? "");
                return parsed === null || (positive && parsed <= 0) ? defaultValue : parsed;
            },
            set(value) {
                const number = Number(value);
                if (!Number.isFinite(number)) throw new DOMException("value must be finite", "NotSupportedError");
                if (!positive || number > 0) this.setAttribute(attribute, String(number === 0 ? 0 : number));
            },
        });
    }
}

function __defineSettableTokenListReflection(prototype, property, attribute) {
    __defineTokenListReflection(prototype, property, attribute);
    const descriptor = Object.getOwnPropertyDescriptor(prototype, property);
    Object.defineProperty(prototype, property, {
        ...descriptor,
        set(value) { this.setAttribute(attribute, String(value)); },
    });
}

function __defineNonceReflection(prototype) {
    const values = new WeakMap();
    Object.defineProperty(prototype, "nonce", {
        configurable: true, enumerable: true,
        get() { return values.has(this) ? values.get(this) : (this.getAttribute("nonce") ?? ""); },
        set(value) { values.set(this, String(value)); },
    });
}

class HTMLPictureElement extends HTMLElement { constructor() { __illegalHtmlElementConstructor(); } }
class HTMLImageElement extends HTMLElement { constructor() { __illegalHtmlElementConstructor(); } }

function __decodeDataUrlDocument(href) {
    const comma = href.indexOf(",");
    if (comma === -1) return "";
    const metadata = href.slice(5, comma);
    const payload = href.slice(comma + 1);
    const label = /(?:^|;)charset=([^;]*)/i.exec(metadata)?.[1] ?? "US-ASCII";
    let byteString;
    if (/(?:^|;)base64(?:;|$)/i.test(metadata)) byteString = atob(payload);
    else byteString = unescape(payload);
    const bytes = new Uint8Array(byteString.length);
    for (let index = 0; index < byteString.length; index++) bytes[index] = byteString.charCodeAt(index);
    return new TextDecoder(label).decode(bytes);
}

function __runInChildWindow(context, source) {
    return Function("window", "document", "location", "parent", "self", "top", source)(
        context.window,
        context.document,
        context.window.location,
        window,
        context.window,
        window,
    );
}

function __childConstructor(context, creator) {
    return function(...arguments_) { return creator(context.document, ...arguments_); };
}

class HTMLIFrameElement extends HTMLElement {
    constructor() { __illegalHtmlElementConstructor(); }
    get contentWindow() {
        let context = __iframeBrowsingContexts.get(this);
        if (!context) {
            let location = new URL("about:blank");
            const childWindow = new EventTarget();
            context = {
                document: new DOMParser().parseFromString("", "text/html"),
                window: childWindow,
                navigate(url) {
                    location = new URL(url, document.URL);
                    const source = location.protocol === "data:"
                        ? __decodeDataUrlDocument(location.href)
                        : "";
                    context.document = new DOMParser().parseFromString(source, "text/html");
                    Object.defineProperty(context.document, "__URL", {
                        value: location.href, configurable: true,
                    });
                    for (const script of context.document.querySelectorAll("script")) {
                        __runInChildWindow(context, script.textContent);
                    }
                    const onload = context.document.body?.getAttribute("onload");
                    if (onload) __runInChildWindow(context, onload);
                },
            };
            Object.defineProperties(childWindow, {
                location: {
                    get() { return location; },
                    set(value) { context.navigate(value); },
                    enumerable: true,
                },
                document: { get() { return context.document; }, enumerable: true },
                frameElement: { value: this, enumerable: true },
                parent: { value: window, enumerable: true },
                top: { value: window, enumerable: true },
            });
            childWindow.window = childWindow;
            childWindow.self = childWindow;
            childWindow.Comment = __childConstructor(context, (doc, data = "") => doc.createComment(String(data)));
            childWindow.Text = __childConstructor(context, (doc, data = "") => doc.createTextNode(String(data)));
            childWindow.DocumentFragment = __childConstructor(context, doc => doc.createDocumentFragment());
            childWindow.postMessage = (message, targetOrigin = "*", transfer = []) => {
                const cloned = __prepareMessage(message, transfer);
                targetOrigin = String(targetOrigin);
                if (targetOrigin !== "*" && targetOrigin !== "/" && targetOrigin !== location.origin) return;
                setTimeout(() => {
                    const event = new MessageEvent("message", {
                        data: cloned,
                        origin: document.location?.origin ?? location.origin,
                        source: window,
                    });
                    event.isTrusted = true;
                    childWindow.dispatchEvent(event);
                }, 0);
            };
            __iframeBrowsingContexts.set(this, context);
        }
        return context.window;
    }
    get contentDocument() {
        this.contentWindow;
        return __iframeBrowsingContexts.get(this).document;
    }
    __connected() {
        if (__iframeBrowsingContexts.get(this)?.connected) return;
        this.contentWindow;
        const context = __iframeBrowsingContexts.get(this);
        context.connected = true;
        context.navigate(this.getAttribute("srcdoc") !== null
            ? "data:text/html;charset=utf-8," + escape(this.getAttribute("srcdoc"))
            : (this.getAttribute("src") || "about:blank"));
        setTimeout(() => this.dispatchEvent(new Event("load")), 0);
    }
    __navigate(url) {
        this.contentWindow;
        const context = __iframeBrowsingContexts.get(this);
        context.navigate(url);
        setTimeout(() => this.dispatchEvent(new Event("load")), 0);
    }
}
class HTMLEmbedElement extends HTMLElement { constructor() { __illegalHtmlElementConstructor(); } }
class HTMLObjectElement extends HTMLElement { constructor() { __illegalHtmlElementConstructor(); } }
class HTMLParamElement extends HTMLElement { constructor() { __illegalHtmlElementConstructor(); } }
const __mediaElementStates = new WeakMap();
function __mediaElementState(element) {
    let state = __mediaElementStates.get(element);
    if (state === undefined) {
        state = {
            currentTime: 0,
            defaultPlaybackRate: 1,
            playbackRate: 1,
            volume: 1,
            muted: false,
            paused: true,
        };
        __mediaElementStates.set(element, state);
    }
    return state;
}
function __mediaElementController(element) {
    return globalThis.__brimpGetMediaElementController?.(element) ?? null;
}
function __trustedMediaEvent(element, type) {
    const event = new Event(type);
    event.isTrusted = true;
    element.dispatchEvent(event);
}
class HTMLMediaElement extends HTMLElement {
    constructor() { __illegalHtmlElementConstructor(); }
    get currentSrc() { return this.src; }
    get currentTime() {
        return __mediaElementController(this)?.currentTime() ?? __mediaElementState(this).currentTime;
    }
    set currentTime(value) {
        value = Number(value);
        if (!Number.isFinite(value) || value < 0) throw new RangeError("currentTime must be finite and non-negative");
        __mediaElementState(this).currentTime = value;
        __mediaElementController(this)?.seek(value);
    }
    get duration() { return __mediaElementController(this)?.duration() ?? NaN; }
    get paused() { return __mediaElementState(this).paused; }
    get ended() { return __mediaElementController(this)?.ended() ?? false; }
    get readyState() { return __mediaElementController(this)?.readyState() ?? 0; }
    get networkState() { return this.currentSrc === "" ? 3 : (this.readyState === 0 ? 2 : 1); }
    get defaultPlaybackRate() { return __mediaElementState(this).defaultPlaybackRate; }
    set defaultPlaybackRate(value) {
        value = Number(value);
        if (!Number.isFinite(value) || value === 0) throw new DOMException("Invalid playback rate", "NotSupportedError");
        __mediaElementState(this).defaultPlaybackRate = value;
    }
    get playbackRate() { return __mediaElementState(this).playbackRate; }
    set playbackRate(value) {
        value = Number(value);
        if (!Number.isFinite(value) || value === 0) throw new DOMException("Invalid playback rate", "NotSupportedError");
        __mediaElementController(this)?.setPlaybackRate(value);
        __mediaElementState(this).playbackRate = value;
        __trustedMediaEvent(this, "ratechange");
    }
    get volume() { return __mediaElementState(this).volume; }
    set volume(value) {
        value = Number(value);
        if (!Number.isFinite(value) || value < 0 || value > 1) throw new DOMException("Volume is outside [0, 1]", "IndexSizeError");
        __mediaElementState(this).volume = value;
        __mediaElementController(this)?.setVolume(value);
        __trustedMediaEvent(this, "volumechange");
    }
    get muted() { return __mediaElementState(this).muted; }
    set muted(value) {
        __mediaElementState(this).muted = Boolean(value);
        __mediaElementController(this)?.setVolume(this.volume);
        __trustedMediaEvent(this, "volumechange");
    }
    play() {
        const controller = __mediaElementController(this);
        if (controller === null) {
            return Promise.reject(new DOMException("Media playback requires a WebAudio media source", "NotSupportedError"));
        }
        const state = __mediaElementState(this);
        state.paused = false;
        __trustedMediaEvent(this, "play");
        return Promise.resolve(controller.play()).then(() => {
            __trustedMediaEvent(this, "playing");
        }, error => {
            state.paused = true;
            throw error;
        });
    }
    pause() {
        const state = __mediaElementState(this);
        if (state.paused) return;
        state.currentTime = this.currentTime;
        state.paused = true;
        __mediaElementController(this)?.pause();
        __trustedMediaEvent(this, "pause");
    }
    load() {
        const state = __mediaElementState(this);
        state.currentTime = 0;
        state.paused = true;
        __mediaElementController(this)?.load();
        __trustedMediaEvent(this, "loadstart");
    }
    canPlayType(type) {
        type = String(type).toLowerCase();
        return /^(audio\/(wav|wave|x-wav|mpeg|mp4|aac|flac|ogg))\b/.test(type) ? "probably" : "";
    }
}
for (const [name, value] of Object.entries({
    NETWORK_EMPTY: 0, NETWORK_IDLE: 1, NETWORK_LOADING: 2, NETWORK_NO_SOURCE: 3,
    HAVE_NOTHING: 0, HAVE_METADATA: 1, HAVE_CURRENT_DATA: 2, HAVE_FUTURE_DATA: 3,
    HAVE_ENOUGH_DATA: 4,
})) {
    Object.defineProperty(HTMLMediaElement, name, { value });
    Object.defineProperty(HTMLMediaElement.prototype, name, { value });
}
class HTMLVideoElement extends HTMLMediaElement { constructor() { __illegalHtmlElementConstructor(); } }
class HTMLAudioElement extends HTMLMediaElement { constructor() { __illegalHtmlElementConstructor(); } }
class HTMLSourceElement extends HTMLElement { constructor() { __illegalHtmlElementConstructor(); } }
class HTMLTrackElement extends HTMLElement { constructor() { __illegalHtmlElementConstructor(); } }
class HTMLCanvasElement extends HTMLElement { constructor() { __illegalHtmlElementConstructor(); } }
class HTMLMapElement extends HTMLElement { constructor() { __illegalHtmlElementConstructor(); } }
class HTMLAreaElement extends HTMLElement { constructor() { __illegalHtmlElementConstructor(); } }
class HTMLFormElement extends HTMLElement {
    constructor() { __illegalHtmlElementConstructor(); }
    submit() {
        if (this.method !== "get") {
            throw new DOMException("Only GET form submission is implemented", "NotSupportedError");
        }
        const charset = this.acceptCharset || document.characterSet;
        const pairs = [];
        for (const control of this.querySelectorAll("input,button,select,textarea")) {
            if (!control.name || control.disabled) continue;
            if (control instanceof HTMLInputElement &&
                ["button", "reset", "submit", "image", "file"].includes(control.type)) continue;
            pairs.push(__callHost("formUrlEncode", window, charset, control.name) + "=" +
                __callHost("formUrlEncode", window, charset, control.value ?? ""));
        }
        const destination = new URL(this.action || document.URL, document.URL);
        destination.search = pairs.join("&");
        const target = this.target;
        let frame = null;
        if (target) {
            for (const candidate of document.getElementsByTagName("iframe")) {
                if (candidate.name === target || candidate.id === target) {
                    frame = candidate;
                    break;
                }
            }
        }
        if (frame) frame.__navigate(destination.href);
        else location.href = destination.href;
    }
}
class HTMLFieldSetElement extends HTMLElement { constructor() { __illegalHtmlElementConstructor(); } }
class HTMLLegendElement extends HTMLElement { constructor() { __illegalHtmlElementConstructor(); } }
class HTMLLabelElement extends HTMLElement { constructor() { __illegalHtmlElementConstructor(); } }
class HTMLInputElement extends HTMLElement {
    constructor() { __illegalHtmlElementConstructor(); }
    get value() {
        return __inputValues.has(this) ? __inputValues.get(this) : (this.getAttribute("value") ?? "");
    }
    set value(value) { __inputValues.set(this, String(value)); }
}
class HTMLButtonElement extends HTMLElement { constructor() { __illegalHtmlElementConstructor(); } }
class HTMLSelectElement extends HTMLElement { constructor() { __illegalHtmlElementConstructor(); } }
class HTMLDataListElement extends HTMLElement { constructor() { __illegalHtmlElementConstructor(); } }
class HTMLOptGroupElement extends HTMLElement { constructor() { __illegalHtmlElementConstructor(); } }
class HTMLOptionElement extends HTMLElement { constructor() { __illegalHtmlElementConstructor(); } }
class HTMLTextAreaElement extends HTMLElement {
    constructor() { __illegalHtmlElementConstructor(); }
    get value() { return __inputValues.has(this) ? __inputValues.get(this) : this.textContent; }
    set value(value) { __inputValues.set(this, String(value)); }
}
class HTMLOutputElement extends HTMLElement { constructor() { __illegalHtmlElementConstructor(); } }
class HTMLProgressElement extends HTMLElement { constructor() { __illegalHtmlElementConstructor(); } }
class HTMLMeterElement extends HTMLElement { constructor() { __illegalHtmlElementConstructor(); } }
class HTMLTableElement extends HTMLElement { constructor() { __illegalHtmlElementConstructor(); } }
class HTMLTableCaptionElement extends HTMLElement { constructor() { __illegalHtmlElementConstructor(); } }
class HTMLTableColElement extends HTMLElement { constructor() { __illegalHtmlElementConstructor(); } }
class HTMLTableSectionElement extends HTMLElement { constructor() { __illegalHtmlElementConstructor(); } }
class HTMLTableRowElement extends HTMLElement { constructor() { __illegalHtmlElementConstructor(); } }
class HTMLTableCellElement extends HTMLElement { constructor() { __illegalHtmlElementConstructor(); } }
class HTMLHeadElement extends HTMLElement { constructor() { __illegalHtmlElementConstructor(); } }
class HTMLTitleElement extends HTMLElement { constructor() { __illegalHtmlElementConstructor(); } }
class HTMLLinkElement extends HTMLElement {
    constructor() { __illegalHtmlElementConstructor(); }
    get sheet() { return __styleSheetForOwner(this); }
    get disabled() { return this.sheet?.disabled ?? false; }
    set disabled(value) {
        const sheet = this.sheet;
        if (sheet !== null) sheet.disabled = Boolean(value);
    }
}
class HTMLMetaElement extends HTMLElement { constructor() { __illegalHtmlElementConstructor(); } }
class HTMLStyleElement extends HTMLElement {
    constructor() { __illegalHtmlElementConstructor(); }
    get sheet() { return __styleSheetForOwner(this); }
    get disabled() { return this.sheet?.disabled ?? false; }
    set disabled(value) {
        const sheet = this.sheet;
        if (sheet !== null) sheet.disabled = Boolean(value);
    }
}
class HTMLParagraphElement extends HTMLElement { constructor() { __illegalHtmlElementConstructor(); } }
class HTMLHRElement extends HTMLElement { constructor() { __illegalHtmlElementConstructor(); } }
class HTMLPreElement extends HTMLElement { constructor() { __illegalHtmlElementConstructor(); } }
class HTMLQuoteElement extends HTMLElement { constructor() { __illegalHtmlElementConstructor(); } }
class HTMLOListElement extends HTMLElement { constructor() { __illegalHtmlElementConstructor(); } }
class HTMLUListElement extends HTMLElement { constructor() { __illegalHtmlElementConstructor(); } }
class HTMLLIElement extends HTMLElement { constructor() { __illegalHtmlElementConstructor(); } }
class HTMLDListElement extends HTMLElement { constructor() { __illegalHtmlElementConstructor(); } }
class HTMLDivElement extends HTMLElement { constructor() { __illegalHtmlElementConstructor(); } }
class HTMLDataElement extends HTMLElement { constructor() { __illegalHtmlElementConstructor(); } }
class HTMLTimeElement extends HTMLElement { constructor() { __illegalHtmlElementConstructor(); } }
class HTMLBRElement extends HTMLElement { constructor() { __illegalHtmlElementConstructor(); } }
class HTMLBodyElement extends HTMLElement { constructor() { __illegalHtmlElementConstructor(); } }
class HTMLHeadingElement extends HTMLElement { constructor() { __illegalHtmlElementConstructor(); } }
class HTMLHtmlElement extends HTMLElement { constructor() { __illegalHtmlElementConstructor(); } }
class HTMLScriptElement extends HTMLElement { constructor() { __illegalHtmlElementConstructor(); } }
class HTMLTemplateElement extends HTMLElement {
    constructor() { __illegalHtmlElementConstructor(); }
    get content() {
        let content = __templateContents.get(this);
        if (!content) {
            content = this.ownerDocument.createDocumentFragment();
            while (this.firstChild) content.appendChild(this.firstChild);
            __templateContents.set(this, content);
        }
        return content;
    }
}
class HTMLSlotElement extends HTMLElement { constructor() { __illegalHtmlElementConstructor(); } }
class HTMLModElement extends HTMLElement { constructor() { __illegalHtmlElementConstructor(); } }
class HTMLDetailsElement extends HTMLElement { constructor() { __illegalHtmlElementConstructor(); } }
class HTMLMenuElement extends HTMLElement { constructor() { __illegalHtmlElementConstructor(); } }
class HTMLDialogElement extends HTMLElement { constructor() { __illegalHtmlElementConstructor(); } }
class HTMLMarqueeElement extends HTMLElement { constructor() { __illegalHtmlElementConstructor(); } }
class HTMLFrameSetElement extends HTMLElement { constructor() { __illegalHtmlElementConstructor(); } }
class HTMLFrameElement extends HTMLElement { constructor() { __illegalHtmlElementConstructor(); } }
class HTMLDirectoryElement extends HTMLElement { constructor() { __illegalHtmlElementConstructor(); } }
class HTMLFontElement extends HTMLElement { constructor() { __illegalHtmlElementConstructor(); } }

__defineStringReflections(HTMLImageElement.prototype, [
    ["alt"], ["srcset"], ["useMap", "usemap"], ["name"], ["align"], ["border"],
]);
__defineUrlReflections(HTMLImageElement.prototype, [["src"], ["lowsrc"], ["longDesc", "longdesc"]]);
__defineBooleanReflections(HTMLImageElement.prototype, [["isMap", "ismap"]]);
__defineUnsignedReflections(HTMLImageElement.prototype, [
    ["width", "width", 0], ["height", "height", 0], ["hspace", "hspace", 0], ["vspace", "vspace", 0],
]);
__defineEnumReflection(HTMLImageElement.prototype, "crossOrigin", "crossorigin", ["anonymous", "use-credentials"], null, "anonymous", { "": "anonymous" }, true);
__defineEnumReflection(HTMLImageElement.prototype, "referrerPolicy", "referrerpolicy", ["", "no-referrer", "no-referrer-when-downgrade", "same-origin", "origin", "strict-origin", "origin-when-cross-origin", "strict-origin-when-cross-origin", "unsafe-url"], "", "");
__defineEnumReflection(HTMLImageElement.prototype, "decoding", "decoding", ["async", "sync", "auto"], "auto", "auto");
__defineNullAsEmptyStringReflection(HTMLImageElement.prototype, "border", "border");
const __imageMetadata = image => JSON.parse(__callHost("imageMetadata", image));
Object.defineProperties(HTMLImageElement.prototype, {
    complete: {
        get() { return (!this.hasAttribute("src") && !this.hasAttribute("srcset")) || __imageMetadata(this).complete; },
        enumerable: true, configurable: true,
    },
    naturalWidth: {
        get() { return __imageMetadata(this).width; },
        enumerable: true, configurable: true,
    },
    naturalHeight: {
        get() { return __imageMetadata(this).height; },
        enumerable: true, configurable: true,
    },
    currentSrc: {
        get() { return this.complete && this.naturalWidth > 0 ? this.src : ""; },
        enumerable: true, configurable: true,
    },
    decode: {
        value() {
            return this.complete && this.naturalWidth > 0
                ? Promise.resolve()
                : Promise.reject(new DOMException("The image could not be decoded", "EncodingError"));
        },
        writable: true, configurable: true,
    },
});

function Image(width = undefined, height = undefined) {
    const image = document.createElement("img");
    if (width !== undefined) image.width = Number(width) >>> 0;
    if (height !== undefined) image.height = Number(height) >>> 0;
    return image;
}

__defineStringReflections(HTMLIFrameElement.prototype, [
    ["srcdoc"], ["name"], ["width"], ["height"], ["align"], ["scrolling"],
    ["frameBorder", "frameborder"], ["marginHeight", "marginheight"], ["marginWidth", "marginwidth"],
]);
__defineUrlReflections(HTMLIFrameElement.prototype, [["src"], ["longDesc", "longdesc"]]);
__defineBooleanReflections(HTMLIFrameElement.prototype, [["allowFullscreen", "allowfullscreen"]]);
__defineEnumReflection(HTMLIFrameElement.prototype, "referrerPolicy", "referrerpolicy", ["", "no-referrer", "no-referrer-when-downgrade", "same-origin", "origin", "strict-origin", "origin-when-cross-origin", "strict-origin-when-cross-origin", "unsafe-url"], "", "");
__defineNullAsEmptyStringReflection(HTMLIFrameElement.prototype, "marginHeight", "marginheight");
__defineNullAsEmptyStringReflection(HTMLIFrameElement.prototype, "marginWidth", "marginwidth");

__defineStringReflections(HTMLEmbedElement.prototype, [["type"], ["width"], ["height"], ["align"], ["name"]]);
__defineUrlReflections(HTMLEmbedElement.prototype, [["src"]]);
__defineStringReflections(HTMLObjectElement.prototype, [
    ["type"], ["name"], ["useMap", "usemap"], ["width"], ["height"], ["align"],
    ["archive"], ["code"], ["standby"], ["codeType", "codetype"], ["border"],
]);
__defineUrlReflections(HTMLObjectElement.prototype, [["data"], ["codeBase", "codebase"]]);
__defineBooleanReflections(HTMLObjectElement.prototype, [["declare"]]);
__defineUnsignedReflections(HTMLObjectElement.prototype, [["hspace", "hspace", 0], ["vspace", "vspace", 0]]);
__defineNullAsEmptyStringReflection(HTMLObjectElement.prototype, "border", "border");
__defineStringReflections(HTMLParamElement.prototype, [["name"], ["value"], ["type"], ["valueType", "valuetype"]]);

__defineUrlReflections(HTMLMediaElement.prototype, [["src"]]);
__defineBooleanReflections(HTMLMediaElement.prototype, [["autoplay"], ["loop"], ["controls"], ["defaultMuted", "muted"]]);
__defineEnumReflection(HTMLMediaElement.prototype, "crossOrigin", "crossorigin", ["anonymous", "use-credentials"], null, "anonymous", { "": "anonymous" }, true);
__defineEnumReflection(HTMLMediaElement.prototype, "preload", "preload", ["none", "metadata", "auto"], "metadata", "metadata", { "": "auto" });
__defineEnumReflection(HTMLMediaElement.prototype, "loading", "loading", ["lazy", "eager"], "eager", "eager");
__defineUnsignedReflections(HTMLVideoElement.prototype, [["width", "width", 0], ["height", "height", 0]]);
__defineUrlReflections(HTMLVideoElement.prototype, [["poster"]]);
__defineBooleanReflections(HTMLVideoElement.prototype, [["playsInline", "playsinline"]]);
__defineTokenListReflection(HTMLVideoElement.prototype, "controlsList", "controlslist");

__defineStringReflections(HTMLSourceElement.prototype, [["type"], ["srcset"], ["sizes"], ["media"]]);
__defineUrlReflections(HTMLSourceElement.prototype, [["src"]]);
__defineStringReflections(HTMLTrackElement.prototype, [["srclang"], ["label"]]);
__defineUrlReflections(HTMLTrackElement.prototype, [["src"]]);
__defineBooleanReflections(HTMLTrackElement.prototype, [["default"]]);
__defineEnumReflection(HTMLTrackElement.prototype, "kind", "kind", ["subtitles", "captions", "descriptions", "chapters", "metadata"], "subtitles", "metadata");
__defineUnsignedReflections(HTMLCanvasElement.prototype, [["width", "width", 300], ["height", "height", 150]]);
__defineStringReflections(HTMLMapElement.prototype, [["name"]]);
__defineStringReflections(HTMLAreaElement.prototype, [["alt"], ["coords"], ["shape"], ["target"], ["download"], ["ping"], ["rel"], ["hreflang"], ["type"]]);
__defineUrlReflections(HTMLAreaElement.prototype, [["href"]]);
__defineBooleanReflections(HTMLAreaElement.prototype, [["noHref", "nohref"]]);
__defineTokenListReflection(HTMLAreaElement.prototype, "relList", "rel");
__defineEnumReflection(HTMLAreaElement.prototype, "referrerPolicy", "referrerpolicy", ["", "no-referrer", "no-referrer-when-downgrade", "same-origin", "origin", "strict-origin", "origin-when-cross-origin", "strict-origin-when-cross-origin", "unsafe-url"], "", "");

__defineStringReflections(HTMLAnchorElement.prototype, [
    ["target"], ["download"], ["ping"], ["rel"], ["hreflang"], ["type"],
    ["coords"], ["charset"], ["name"], ["rev"], ["shape"],
]);
__defineTokenListReflection(HTMLAnchorElement.prototype, "relList", "rel");
__defineEnumReflection(HTMLAnchorElement.prototype, "referrerPolicy", "referrerpolicy", ["", "no-referrer", "no-referrer-when-downgrade", "same-origin", "origin", "strict-origin", "origin-when-cross-origin", "strict-origin-when-cross-origin", "unsafe-url"], "", "");
__defineStringReflections(HTMLBaseElement.prototype, [["target"]]);

__defineStringReflections(HTMLFormElement.prototype, [["acceptCharset", "accept-charset"], ["name"], ["target"]]);
__defineActionUrlReflection(HTMLFormElement.prototype, "action", "action");
__defineBooleanReflections(HTMLFormElement.prototype, [["noValidate", "novalidate"]]);
__defineEnumReflection(HTMLFormElement.prototype, "autocomplete", "autocomplete", ["on", "off"], "on", "on");
__defineEnumReflection(HTMLFormElement.prototype, "enctype", "enctype", ["application/x-www-form-urlencoded", "multipart/form-data", "text/plain"], "application/x-www-form-urlencoded", "application/x-www-form-urlencoded");
__defineEnumReflection(HTMLFormElement.prototype, "encoding", "enctype", ["application/x-www-form-urlencoded", "multipart/form-data", "text/plain"], "application/x-www-form-urlencoded", "application/x-www-form-urlencoded");
__defineEnumReflection(HTMLFormElement.prototype, "method", "method", ["get", "post", "dialog"], "get", "get");
__defineStringReflections(HTMLFieldSetElement.prototype, [["name"]]);
__defineBooleanReflections(HTMLFieldSetElement.prototype, [["disabled"]]);
__defineStringReflections(HTMLLegendElement.prototype, [["align"]]);
__defineStringReflections(HTMLLabelElement.prototype, [["htmlFor", "for"]]);

__defineStringReflections(HTMLInputElement.prototype, [
    ["accept"], ["alt"], ["autocomplete"], ["dirName", "dirname"], ["max"], ["min"],
    ["name"], ["pattern"], ["placeholder"], ["step"], ["defaultValue", "value"],
    ["align"], ["useMap", "usemap"], ["formTarget", "formtarget"],
]);
__defineBooleanReflections(HTMLInputElement.prototype, [
    ["defaultChecked", "checked"], ["disabled"], ["formNoValidate", "formnovalidate"],
    ["multiple"], ["readOnly", "readonly"], ["required"],
]);
__defineUrlReflections(HTMLInputElement.prototype, [["src"]]);
__defineActionUrlReflection(HTMLInputElement.prototype, "formAction", "formaction");
__defineUnsignedReflections(HTMLInputElement.prototype, [["height", "height", 0], ["width", "width", 0]]);
__defineLongReflections(HTMLInputElement.prototype, [["maxLength", "maxlength", -1, true], ["minLength", "minlength", -1, true]]);
__definePositiveUnsignedReflections(HTMLInputElement.prototype, [["size", "size", 20, false]]);
__defineEnumReflection(HTMLInputElement.prototype, "formEnctype", "formenctype", ["application/x-www-form-urlencoded", "multipart/form-data", "text/plain"], "", "application/x-www-form-urlencoded");
__defineEnumReflection(HTMLInputElement.prototype, "formMethod", "formmethod", ["get", "post"], "", "get");
__defineEnumReflection(HTMLInputElement.prototype, "type", "type", ["hidden", "text", "search", "tel", "url", "email", "password", "date", "month", "week", "time", "datetime-local", "number", "range", "color", "checkbox", "radio", "file", "submit", "image", "reset", "button"], "text", "text");

__defineStringReflections(HTMLButtonElement.prototype, [["formTarget", "formtarget"], ["name"], ["value"]]);
__defineBooleanReflections(HTMLButtonElement.prototype, [["disabled"], ["formNoValidate", "formnovalidate"]]);
__defineActionUrlReflection(HTMLButtonElement.prototype, "formAction", "formaction");
__defineEnumReflection(HTMLButtonElement.prototype, "formEnctype", "formenctype", ["application/x-www-form-urlencoded", "multipart/form-data", "text/plain"], "", "application/x-www-form-urlencoded");
__defineEnumReflection(HTMLButtonElement.prototype, "formMethod", "formmethod", ["get", "post", "dialog"], "", "get");
__defineEnumReflection(HTMLButtonElement.prototype, "type", "type", ["submit", "reset", "button"], "submit", "submit");

__defineStringReflections(HTMLSelectElement.prototype, [["autocomplete"], ["name"]]);
__defineBooleanReflections(HTMLSelectElement.prototype, [["disabled"], ["multiple"], ["required"]]);
__defineUnsignedReflections(HTMLSelectElement.prototype, [["size", "size", 0]]);
__defineStringReflections(HTMLOptGroupElement.prototype, [["label"]]);
__defineBooleanReflections(HTMLOptGroupElement.prototype, [["disabled"]]);
__defineStringReflections(HTMLOptionElement.prototype, [["label"], ["value"]]);
__defineBooleanReflections(HTMLOptionElement.prototype, [["disabled"], ["defaultSelected", "selected"]]);
__defineStringReflections(HTMLTextAreaElement.prototype, [
    ["autocomplete"], ["dirName", "dirname"], ["name"], ["placeholder"], ["wrap"],
]);
__defineBooleanReflections(HTMLTextAreaElement.prototype, [["disabled"], ["readOnly", "readonly"], ["required"]]);
__defineLongReflections(HTMLTextAreaElement.prototype, [["maxLength", "maxlength", -1, true], ["minLength", "minlength", -1, true]]);
__definePositiveUnsignedReflections(HTMLTextAreaElement.prototype, [["cols", "cols", 20, true], ["rows", "rows", 2, true]]);
__defineSettableTokenListReflection(HTMLOutputElement.prototype, "htmlFor", "for");
__defineStringReflections(HTMLOutputElement.prototype, [["name"]]);
__defineDoubleReflections(HTMLProgressElement.prototype, [["max", "max", 1, true]]);
__defineDoubleReflections(HTMLMeterElement.prototype, [["value"], ["min"], ["max"], ["low"], ["high"], ["optimum"]]);

__defineStringReflections(HTMLTableElement.prototype, [
    ["align"], ["border"], ["frame"], ["rules"], ["summary"], ["width"],
    ["bgColor", "bgcolor"], ["cellPadding", "cellpadding"], ["cellSpacing", "cellspacing"],
]);
for (const [property, attribute] of [["bgColor", "bgcolor"], ["cellPadding", "cellpadding"], ["cellSpacing", "cellspacing"]]) {
    __defineNullAsEmptyStringReflection(HTMLTableElement.prototype, property, attribute);
}
__defineStringReflections(HTMLTableCaptionElement.prototype, [["align"]]);
__defineClampedUnsignedReflections(HTMLTableColElement.prototype, [["span", "span", 1, 1, 1000]]);
__defineStringReflections(HTMLTableColElement.prototype, [["align"], ["ch", "char"], ["chOff", "charoff"], ["vAlign", "valign"], ["width"]]);
__defineStringReflections(HTMLTableSectionElement.prototype, [["align"], ["ch", "char"], ["chOff", "charoff"], ["vAlign", "valign"]]);
__defineStringReflections(HTMLTableRowElement.prototype, [["align"], ["ch", "char"], ["chOff", "charoff"], ["vAlign", "valign"], ["bgColor", "bgcolor"]]);
__defineNullAsEmptyStringReflection(HTMLTableRowElement.prototype, "bgColor", "bgcolor");
__defineClampedUnsignedReflections(HTMLTableCellElement.prototype, [["colSpan", "colspan", 1, 1, 1000], ["rowSpan", "rowspan", 1, 0, 65534]]);
__defineStringReflections(HTMLTableCellElement.prototype, [
    ["headers"], ["abbr"], ["align"], ["axis"], ["height"], ["width"], ["ch", "char"],
    ["chOff", "charoff"], ["vAlign", "valign"], ["bgColor", "bgcolor"],
]);
__defineBooleanReflections(HTMLTableCellElement.prototype, [["noWrap", "nowrap"]]);
__defineEnumReflection(HTMLTableCellElement.prototype, "scope", "scope", ["row", "col", "rowgroup", "colgroup"], "", "");
__defineNullAsEmptyStringReflection(HTMLTableCellElement.prototype, "bgColor", "bgcolor");

__defineStringReflections(HTMLLinkElement.prototype, [
    ["rel"], ["media"], ["integrity"], ["hreflang"], ["type"], ["charset"], ["rev"], ["target"],
]);
__defineNonceReflection(HTMLLinkElement.prototype);
__defineUrlReflections(HTMLLinkElement.prototype, [["href"]]);
__defineTokenListReflection(HTMLLinkElement.prototype, "relList", "rel");
__defineSettableTokenListReflection(HTMLLinkElement.prototype, "sizes", "sizes");
__defineEnumReflection(HTMLLinkElement.prototype, "crossOrigin", "crossorigin", ["anonymous", "use-credentials"], null, "anonymous", { "": "anonymous" }, true);
__defineEnumReflection(HTMLLinkElement.prototype, "as", "as", ["fetch", "audio", "document", "embed", "font", "image", "manifest", "object", "report", "script", "sharedworker", "style", "track", "video", "worker", "xslt"], "", "");
__defineEnumReflection(HTMLLinkElement.prototype, "referrerPolicy", "referrerpolicy", ["", "no-referrer", "no-referrer-when-downgrade", "same-origin", "origin", "strict-origin", "origin-when-cross-origin", "strict-origin-when-cross-origin", "unsafe-url"], "", "");
__defineStringReflections(HTMLMetaElement.prototype, [["name"], ["httpEquiv", "http-equiv"], ["content"], ["media"], ["scheme"]]);
__defineStringReflections(HTMLStyleElement.prototype, [["media"], ["type"]]);
__defineNonceReflection(HTMLStyleElement.prototype);

__defineStringReflections(HTMLParagraphElement.prototype, [["align"]]);
__defineStringReflections(HTMLHRElement.prototype, [["align"], ["color"], ["size"], ["width"]]);
__defineBooleanReflections(HTMLHRElement.prototype, [["noShade", "noshade"]]);
__defineLongReflections(HTMLPreElement.prototype, [["width", "width", 0, false]]);
__defineUrlReflections(HTMLQuoteElement.prototype, [["cite"]]);
__defineBooleanReflections(HTMLOListElement.prototype, [["reversed"], ["compact"]]);
__defineLongReflections(HTMLOListElement.prototype, [["start", "start", 1, false]]);
__defineStringReflections(HTMLOListElement.prototype, [["type"]]);
__defineBooleanReflections(HTMLUListElement.prototype, [["compact"]]);
__defineStringReflections(HTMLUListElement.prototype, [["type"]]);
__defineLongReflections(HTMLLIElement.prototype, [["value", "value", 0, false]]);
__defineStringReflections(HTMLLIElement.prototype, [["type"]]);
__defineBooleanReflections(HTMLDListElement.prototype, [["compact"]]);
__defineStringReflections(HTMLDivElement.prototype, [["align"]]);
__defineStringReflections(HTMLDataElement.prototype, [["value"]]);
__defineStringReflections(HTMLTimeElement.prototype, [["dateTime", "datetime"]]);
__defineStringReflections(HTMLBRElement.prototype, [["clear"]]);

__defineStringReflections(HTMLBodyElement.prototype, [["text"], ["link"], ["vLink", "vlink"], ["aLink", "alink"], ["bgColor", "bgcolor"], ["background"]]);
for (const [property, attribute] of [["text", "text"], ["link", "link"], ["vLink", "vlink"], ["aLink", "alink"], ["bgColor", "bgcolor"]]) {
    __defineNullAsEmptyStringReflection(HTMLBodyElement.prototype, property, attribute);
}
__defineStringReflections(HTMLHeadingElement.prototype, [["align"]]);
for (const [property, attribute] of [["fgColor", "text"], ["linkColor", "link"], ["vlinkColor", "vlink"], ["alinkColor", "alink"], ["bgColor", "bgcolor"]]) {
    Object.defineProperty(Document.prototype, property, {
        configurable: true, enumerable: true,
        get() { return this.body?.getAttribute(attribute) ?? ""; },
        set(value) { if (this.body) this.body.setAttribute(attribute, value === null ? "" : String(value)); },
    });
}

__defineStringReflections(HTMLHtmlElement.prototype, [["version"]]);
__defineStringReflections(HTMLScriptElement.prototype, [["type"], ["charset"], ["integrity"], ["event"], ["htmlFor", "for"]]);
__defineUrlReflections(HTMLScriptElement.prototype, [["src"]]);
__defineBooleanReflections(HTMLScriptElement.prototype, [["noModule", "nomodule"], ["defer"]]);
__defineEnumReflection(HTMLScriptElement.prototype, "crossOrigin", "crossorigin", ["anonymous", "use-credentials"], null, "anonymous", { "": "anonymous" }, true);
__defineStringReflections(HTMLSlotElement.prototype, [["name"]]);
__defineUrlReflections(HTMLModElement.prototype, [["cite"]]);
__defineStringReflections(HTMLModElement.prototype, [["dateTime", "datetime"]]);
__defineBooleanReflections(HTMLDetailsElement.prototype, [["open"]]);
__defineBooleanReflections(HTMLMenuElement.prototype, [["compact"]]);
__defineBooleanReflections(HTMLDialogElement.prototype, [["open"]]);
__defineEnumReflection(HTMLElement.prototype, "enterKeyHint", "enterkeyhint", ["enter", "done", "go", "next", "previous", "search", "send"], "", "");
__defineEnumReflection(HTMLElement.prototype, "inputMode", "inputmode", ["none", "text", "tel", "url", "email", "numeric", "decimal", "search"], "", "");

__defineStringReflections(HTMLMarqueeElement.prototype, [["bgColor", "bgcolor"], ["height"], ["width"]]);
__defineUnsignedReflections(HTMLMarqueeElement.prototype, [["hspace", "hspace", 0], ["scrollAmount", "scrollamount", 6], ["scrollDelay", "scrolldelay", 85], ["vspace", "vspace", 0]]);
__defineBooleanReflections(HTMLMarqueeElement.prototype, [["trueSpeed", "truespeed"]]);
__defineEnumReflection(HTMLMarqueeElement.prototype, "behavior", "behavior", ["scroll", "slide", "alternate"], "scroll", "scroll");
__defineEnumReflection(HTMLMarqueeElement.prototype, "direction", "direction", ["up", "right", "down", "left"], "left", "left");
__defineStringReflections(HTMLFrameSetElement.prototype, [["cols"], ["rows"]]);
__defineStringReflections(HTMLFrameElement.prototype, [["name"], ["scrolling"], ["frameBorder", "frameborder"], ["marginHeight", "marginheight"], ["marginWidth", "marginwidth"]]);
__defineUrlReflections(HTMLFrameElement.prototype, [["src"], ["longDesc", "longdesc"]]);
__defineBooleanReflections(HTMLFrameElement.prototype, [["noResize", "noresize"]]);
__defineNullAsEmptyStringReflection(HTMLFrameElement.prototype, "marginHeight", "marginheight");
__defineNullAsEmptyStringReflection(HTMLFrameElement.prototype, "marginWidth", "marginwidth");
__defineBooleanReflections(HTMLDirectoryElement.prototype, [["compact"]]);
__defineStringReflections(HTMLFontElement.prototype, [["color"], ["face"], ["size"]]);
__defineNullAsEmptyStringReflection(HTMLFontElement.prototype, "color", "color");

