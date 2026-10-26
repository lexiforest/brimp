// Fixed profile from the saved Safari coverage fixture, not live display/layout
// measurements. Common window environment surfaces belong in core.
const __windowEnvironmentToken = Symbol('window environment construction');
class BarProp {
    constructor(token) { if (token !== __windowEnvironmentToken) throw new TypeError('Illegal constructor'); }
    get visible() { return true; }
}
class ScreenOrientation extends EventTarget {
    constructor(token) { super(); if (token !== __windowEnvironmentToken) throw new TypeError('Illegal constructor'); }
    get angle() { return 0; }
    get type() { return 'landscape-primary'; }
}
__defineEventHandlers(ScreenOrientation.prototype, ['change']);
const __screenOrientation = new ScreenOrientation(__windowEnvironmentToken);
class Screen extends EventTarget {
    constructor(token) { super(); if (token !== __windowEnvironmentToken) throw new TypeError('Illegal constructor'); }
    get width() { return 1710; }
    get height() { return 1107; }
    get availWidth() { return 1710; }
    get availHeight() { return 1019; }
    get availLeft() { return 0; }
    get availTop() { return 34; }
    get colorDepth() { return 24; }
    get pixelDepth() { return 24; }
    get orientation() { return __screenOrientation; }
}
class VisualViewport extends EventTarget {
    constructor(token) { super(); if (token !== __windowEnvironmentToken) throw new TypeError('Illegal constructor'); }
    get width() { return 800; }
    get height() { return 600; }
    get offsetLeft() { return 0; }
    get offsetTop() { return 0; }
    get pageLeft() { return 0; }
    get pageTop() { return 0; }
    get scale() { return 1; }
}
__defineEventHandlers(VisualViewport.prototype, ['resize', 'scroll']);
for (const [name, value] of Object.entries({
    screen: new Screen(__windowEnvironmentToken),
    visualViewport: new VisualViewport(__windowEnvironmentToken),
    devicePixelRatio: 2, innerWidth: 800, innerHeight: 600,
    outerWidth: 800, outerHeight: 652,
    screenX: 110, screenLeft: 110, screenY: 34, screenTop: 34,
    scrollX: 0, scrollY: 0, pageXOffset: 0, pageYOffset: 0,
    // Legacy capability/obsolete-surface placeholders; no security policy engine.
    isSecureContext: true, styleMedia: undefined,
})) {
    Object.defineProperty(Window.prototype, name, {
        get() { return value; }, enumerable: true, configurable: true,
    });
}
for (const name of ['locationbar', 'menubar', 'personalbar', 'scrollbars', 'statusbar', 'toolbar']) {
    const value = new BarProp(__windowEnvironmentToken);
    Object.defineProperty(Window.prototype, name, {
        get() { return value; }, enumerable: true, configurable: true,
    });
}
Object.assign(globalThis, {BarProp, Screen, ScreenOrientation, VisualViewport});
