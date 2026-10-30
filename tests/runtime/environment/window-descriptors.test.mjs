import test from 'node:test';
import assert from 'node:assert/strict';
import { createSession } from '../../../bindings/node/index.js';

const safariNative = (name) => `function ${name}() {\n    [native code]\n}`;
const chromeNative = (name) => `function ${name}() { [native code] }`;

test('Safari identity exposes Window members as own accessors with WebKit native sources', async () => {
  const session = await createSession();
  try {
    const page = await session.newPage();
    assert.deepEqual(
      await page.evaluate(`(() => {
      const d = name => Object.getOwnPropertyDescriptor(window, name);
      const getter = name => d(name).get;
      const shape = name => {
        const descriptor = d(name);
        return [descriptor !== undefined, 'value' in descriptor, descriptor.enumerable,
          descriptor.configurable, typeof descriptor.set, getter(name).name, getter(name).length];
      };
      const texts = [
        Function.prototype.toString.call(getter('document')),
        Function.prototype.toString.call(getter('innerWidth')),
        Function.prototype.toString.call(document.querySelector),
        Function.prototype.toString.call(Array.prototype.map),
        Function.prototype.toString.call(Function.prototype.toString),
        Function.prototype.toString.call(window.fetch),
        window.fetch.name,
        navigator.webdriver,
        Object.getOwnPropertyDescriptor(Navigator.prototype, 'webdriver').get.name,
      ];
      const chain = [];
      let object = window;
      for (let index = 0; index < 5; index++) {
        chain.push(object.constructor?.name);
        object = Object.getPrototypeOf(object);
      }
      return {
        shape: {
          document: shape('document'),
          window: shape('window'),
          top: shape('top'),
          location: shape('location'),
          navigator: shape('navigator'),
          performance: shape('performance'),
          innerWidth: shape('innerWidth'),
        },
        texts,
        chain,
        navigatorOwn: Object.getOwnPropertyNames(navigator),
        standalone: [navigator.standalone, Object.getOwnPropertyDescriptor(Navigator.prototype, 'standalone').get.name],
        legacy: [window.event, window.offscreenBuffering, window.webkitIndexedDB === indexedDB],
        integrity: (() => {
          const before = document;
          const deleted = delete window.document;
          window.document = {fake: true};
          window.navigator = {fake: true};
          return [deleted, window.document === before, navigator.fake === undefined];
        })(),
        frame: (() => {
          const element = document.createElement('iframe');
          element.srcdoc = '<html><body>frame</body></html>';
          document.body.appendChild(element);
          const child = element.contentWindow;
          const documentGetter = Object.getOwnPropertyDescriptor(child, 'document').get;
          const values = [
            child.navigator !== navigator,
            Object.getPrototypeOf(child.navigator) !== Object.getPrototypeOf(navigator),
            Object.getOwnPropertyNames(child.navigator).length,
            documentGetter.name,
            Function.prototype.toString.call(documentGetter),
            Object.getOwnPropertyDescriptor(HTMLIFrameElement.prototype, 'contentWindow').enumerable,
          ];
          element.remove();
          return values;
        })(),
      };
    })()`),
      {
        shape: {
          document: [true, false, true, false, 'undefined', 'get document', 0],
          window: [true, false, true, false, 'undefined', 'get window', 0],
          top: [true, false, true, false, 'undefined', 'get top', 0],
          location: [true, false, true, false, 'function', 'get location', 0],
          navigator: [true, false, true, true, 'undefined', 'get navigator', 0],
          performance: [true, false, true, true, 'function', 'get performance', 0],
          innerWidth: [true, false, true, true, 'function', 'get innerWidth', 0],
        },
        texts: [
          safariNative('document'),
          safariNative('innerWidth'),
          safariNative('querySelector'),
          safariNative('map'),
          safariNative('toString'),
          safariNative('fetch'),
          'fetch',
          false,
          'get webdriver',
        ],
        chain: ['Window', 'Window', 'EventTarget', 'EventTarget', 'Object'],
        navigatorOwn: [],
        standalone: [false, 'get standalone'],
        legacy: [undefined, true, true],
        integrity: [false, true, true],
        frame: [true, true, 0, 'get document', safariNative('document'), true],
      },
    );
  } finally {
    await session.close();
  }
});

test('Chromium identity switches native sources to the Blink shape', async () => {
  const session = await createSession({
    identity: {
      userAgent:
        'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/150.0.0.0 Safari/537.36',
    },
  });
  try {
    const page = await session.newPage();
    assert.deepEqual(
      await page.evaluate(`(() => [
      navigator.userAgent.includes('Chrome/150'),
      Function.prototype.toString.call(Object.getOwnPropertyDescriptor(window, 'document').get),
      Object.getOwnPropertyDescriptor(window, 'document').get.name,
      Function.prototype.toString.call(Object.getOwnPropertyDescriptor(window, 'innerWidth').get),
      Function.prototype.toString.call(Object.getOwnPropertyDescriptor(window, 'innerWidth').set),
      Function.prototype.toString.call(Array.prototype.map),
      Function.prototype.toString.call(document.querySelector),
      Function.prototype.toString.call(window.fetch),
      Function.prototype.toString.call(JSON.stringify),
      Function.prototype.toString.call(Math.max),
      Function.prototype.toString.call(Reflect.get),
    ])()`),
      [
        true,
        `function get document() { [native code] }`,
        'get document',
        `function get innerWidth() { [native code] }`,
        `function set innerWidth() { [native code] }`,
        chromeNative('map'),
        chromeNative('querySelector'),
        chromeNative('fetch'),
        chromeNative('stringify'),
        chromeNative('max'),
        chromeNative('get'),
      ],
    );
  } finally {
    await session.close();
  }
});
