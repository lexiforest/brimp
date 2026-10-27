// Window environment surfaces driven by the active browser profile.
// Classes are defined once per realm; __brimpInstallWindowEnvironment applies
// the profile values and is removed by the WebIDL finalizer.
const __windowEnvironmentToken = Symbol('window environment construction');
let __windowProfile = null;

class BarProp {
  constructor(token) {
    if (token !== __windowEnvironmentToken) throw new TypeError('Illegal constructor');
  }
  get visible() {
    return true;
  }
}
class ScreenOrientation extends EventTarget {
  constructor(token) {
    super();
    if (token !== __windowEnvironmentToken) throw new TypeError('Illegal constructor');
  }
  get angle() {
    return 0;
  }
  get type() {
    return __windowProfile.screen.orientation;
  }
}
__defineEventHandlers(ScreenOrientation.prototype, ['change']);
class Screen extends EventTarget {
  constructor(token) {
    super();
    if (token !== __windowEnvironmentToken) throw new TypeError('Illegal constructor');
  }
  get width() {
    return __windowProfile.screen.width;
  }
  get height() {
    return __windowProfile.screen.height;
  }
  get availWidth() {
    return __windowProfile.screen.avail_width;
  }
  get availHeight() {
    return __windowProfile.screen.avail_height;
  }
  get availLeft() {
    return __windowProfile.screen.avail_left;
  }
  get availTop() {
    return __windowProfile.screen.avail_top;
  }
  get colorDepth() {
    return __windowProfile.screen.color_depth;
  }
  get pixelDepth() {
    return __windowProfile.screen.pixel_depth;
  }
  get orientation() {
    return __screenOrientation;
  }
}
class VisualViewport extends EventTarget {
  constructor(token) {
    super();
    if (token !== __windowEnvironmentToken) throw new TypeError('Illegal constructor');
  }
  get width() {
    return __windowProfile.window.inner_width;
  }
  get height() {
    return __windowProfile.window.inner_height;
  }
  get offsetLeft() {
    return 0;
  }
  get offsetTop() {
    return 0;
  }
  get pageLeft() {
    return 0;
  }
  get pageTop() {
    return 0;
  }
  get scale() {
    return 1;
  }
}
__defineEventHandlers(VisualViewport.prototype, ['resize', 'scroll']);
let __screenOrientation = null;
let __screen = null;
let __visualViewport = null;

globalThis.__brimpInstallWindowEnvironment = function (profile) {
  __windowProfile = profile;
  if (__screenOrientation === null) {
    __screenOrientation = new ScreenOrientation(__windowEnvironmentToken);
    __screen = new Screen(__windowEnvironmentToken);
    __visualViewport = new VisualViewport(__windowEnvironmentToken);
  }
  for (const [name, value] of Object.entries({
    screen: __screen,
    visualViewport: __visualViewport,
    devicePixelRatio: profile.screen.device_pixel_ratio,
    innerWidth: profile.window.inner_width,
    innerHeight: profile.window.inner_height,
    outerWidth: profile.window.outer_width,
    outerHeight: profile.window.outer_height,
    screenX: profile.window.screen_x,
    screenLeft: profile.window.screen_x,
    screenY: profile.window.screen_y,
    screenTop: profile.window.screen_y,
    scrollX: 0,
    scrollY: 0,
    pageXOffset: 0,
    pageYOffset: 0,
    // Legacy capability/obsolete-surface placeholders; no security policy engine.
    isSecureContext: true,
    styleMedia: undefined,
  })) {
    const descriptor = Object.getOwnPropertyDescriptor(
      {
        get [name]() {
          return value;
        },
      },
      name,
    );
    Object.defineProperty(Window.prototype, name, {
      get: descriptor.get,
      enumerable: true,
      configurable: true,
    });
  }
  for (const name of [
    'locationbar',
    'menubar',
    'personalbar',
    'scrollbars',
    'statusbar',
    'toolbar',
  ]) {
    const value = new BarProp(__windowEnvironmentToken);
    const descriptor = Object.getOwnPropertyDescriptor(
      {
        get [name]() {
          return value;
        },
      },
      name,
    );
    Object.defineProperty(Window.prototype, name, {
      get: descriptor.get,
      enumerable: true,
      configurable: true,
    });
  }
  Object.assign(globalThis, { BarProp, Screen, ScreenOrientation, VisualViewport });
};
