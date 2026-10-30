import { createRequire } from 'node:module';
import { format } from 'node:util';
import { until } from './scenarios.mjs';

const require = createRequire(import.meta.url);

// DOM implementations have no trusted keyboard/pointer input. Keep this small
// adapter shared, and leave all application code and browser APIs untouched.
export function domInput(evaluate) {
  const act = (selector, body) =>
    evaluate(`(() => {
    const element = document.querySelector(${JSON.stringify(selector)});
    if (!element) throw new Error('Missing input target: ' + ${JSON.stringify(selector)});
    ${body}
    return null;
  })()`);
  return {
    fill: (selector, text) =>
      act(
        selector,
        `
      element.focus();
      const prototype = element instanceof HTMLTextAreaElement ? HTMLTextAreaElement.prototype : HTMLInputElement.prototype;
      Object.getOwnPropertyDescriptor(prototype, 'value').set.call(element, ${JSON.stringify(text)});
      element.dispatchEvent(new Event('input', {bubbles: true, composed: true}));
      element.dispatchEvent(new Event('change', {bubbles: true}));
    `,
      ),
    press: (selector, key) => {
      if (key !== 'Enter') throw new Error(`Unsupported corpus key: ${key}`);
      return act(
        selector,
        `
        element.focus();
        let keydownAllowed = true;
        for (const type of ['keydown', 'keypress', 'keyup']) {
          if (type === 'keypress' && !keydownAllowed) continue;
          const allowed = element.dispatchEvent(new KeyboardEvent(type, {
            key: 'Enter', code: 'Enter', keyCode: 13, which: 13,
            charCode: type === 'keypress' ? 13 : 0, bubbles: true, cancelable: true
          }));
          if (type === 'keydown') keydownAllowed = allowed;
        }
      `,
      );
    },
    click: (selector) => act(selector, 'element.click();'),
  };
}

export async function createNodeDOM(engine, origin, record) {
  const guard = (url) => {
    if (new URL(url).origin !== origin) {
      record('blockedRequests', url);
      throw new Error(`External HTTP blocked by corpus runner: ${url}`);
    }
  };
  const observe = (window) => {
    window.addEventListener('error', (event) =>
      record('errors', String(event.error?.stack ?? event.message)),
    );
    window.addEventListener('unhandledrejection', (event) =>
      record('errors', String(event.reason?.stack ?? event.reason)),
    );
  };
  let rawEvaluate, navigate, close, settings;
  if (engine === 'jsdom') {
    const { JSDOM, VirtualConsole, CookieJar, requestInterceptor } = await import('jsdom');
    const virtualConsole = new VirtualConsole();
    virtualConsole.on('jsdomError', (error) => record('errors', String(error.stack ?? error)));
    for (const level of ['warn', 'error'])
      virtualConsole.on(level, (...args) => record('console', format(...args).slice(0, 4000)));
    const cookieJar = new CookieJar();
    let dom;
    settings = {
      runScripts: 'dangerously',
      resources: 'usable with same-origin interceptor',
      pretendToBeVisual: false,
      navigation: 'new JSDOM.fromURL per goto; shared cookie jar',
    };
    navigate = async (url) => {
      guard(url);
      dom?.window.close();
      dom = await JSDOM.fromURL(url, {
        runScripts: 'dangerously',
        virtualConsole,
        cookieJar,
        beforeParse: observe,
        resources: {
          interceptors: [
            requestInterceptor((request) => {
              guard(request.url);
            }),
          ],
        },
      });
    };
    rawEvaluate = (source) => dom.window.eval(source);
    close = () => dom?.window.close();
  } else if (engine === 'happy-dom') {
    const { Browser } = await import('happy-dom');
    settings = {
      enableJavaScriptEvaluation: true,
      navigation: 'BrowserPage.goto; other defaults retained',
    };
    const browser = new Browser({
      settings: {
        enableJavaScriptEvaluation: true,
        navigation: { beforeContentCallback: observe },
        fetch: {
          interceptor: {
            beforeAsyncRequest: async ({ request }) => {
              guard(request.url);
            },
            beforeSyncRequest: ({ request }) => {
              guard(request.url);
            },
          },
        },
      },
    });
    const inner = browser.newPage();
    inner.virtualConsolePrinter.addEventListener('print', () => {
      for (const entry of inner.virtualConsolePrinter.read()) {
        if (entry.level >= 2) record('console', format(...entry.message).slice(0, 4000));
      }
    });
    navigate = async (url) => {
      guard(url);
      await inner.goto(url);
    };
    rawEvaluate = (source) => inner.evaluate(source);
    close = () => browser.close();
  } else throw new Error(`Unknown Node DOM implementation: ${engine}`);

  // The scenarios exchange JSON data, as the other engines do over their IPC.
  // Copy across realms so host assertions do not compare foreign prototypes.
  const evaluate = async (source) => {
    const value = await rawEvaluate(source);
    return value === undefined ? undefined : JSON.parse(JSON.stringify(value));
  };
  const page = {
    evaluate,
    ...domInput(evaluate),
    goto: async (url) => {
      await navigate(url);
      await until(page, 'document.readyState === "complete"', 15000);
    },
  };
  return { page, close, settings, version: require(`${engine}/package.json`).version };
}
