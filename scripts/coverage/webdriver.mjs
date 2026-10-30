import { randomUUID } from 'node:crypto';

// Use the installed Safari, not Playwright WebKit or another browser engine.
const evaluatePageScript = `
const [source, key] = arguments;
const script = document.createElement('script');
script.textContent = 'try { globalThis[' + JSON.stringify(key) + '] = {ok:true,value:(' + source + ')}; }' +
  'catch(error) { globalThis[' + JSON.stringify(key) + '] = {ok:false,name:error.name,message:error.message,stack:error.stack}; }';
try {
  document.documentElement.appendChild(script);
  return globalThis[key];
} finally {
  script.remove();
  delete globalThis[key];
}`;
export async function webdriver(method, endpoint, body) {
  const driver = process.env.SAFARI_WEBDRIVER_URL || 'http://127.0.0.1:4447';
  const response = await fetch(driver + endpoint, {
    method,
    ...(body === undefined
      ? {}
      : { headers: { 'content-type': 'application/json' }, body: JSON.stringify(body) }),
    signal: AbortSignal.timeout(60000),
  });
  const payload = await response.json();
  if (!response.ok || payload.value?.error)
    throw Error(payload.value?.message || JSON.stringify(payload));
  return payload.value;
}

export async function createSafariSession() {
  const session = await webdriver('POST', '/session', {
    capabilities: { alwaysMatch: { browserName: 'safari' } },
  });
  const endpoint = `/session/${session.sessionId}`;
  return {
    async newPage() {
      await webdriver('POST', `${endpoint}/url`, {
        url:
          'data:text/html;charset=utf-8,' +
          encodeURIComponent(
            '<!doctype html><html><head><title>Brimp behavior reference</title></head><body></body></html>',
          ),
      });
      return {
        async get(url) {
          await webdriver('POST', `${endpoint}/url`, { url });
          return this;
        },
        async press(selector, key) {
          const keys = {
            Backspace: '\ue003',
            Delete: '\ue017',
            ArrowLeft: '\ue012',
            ArrowRight: '\ue014',
          };
          if (!(key in keys)) throw Error(`Unsupported coverage key: ${key}`);
          await this.typeText(selector, keys[key]);
        },
        async typeText(selector, text) {
          const element = await webdriver('POST', `${endpoint}/element`, {
            using: 'css selector',
            value: selector,
          });
          const id = element['element-6066-11e4-a52e-4f735466cecf'];
          await webdriver('POST', `${endpoint}/element/${id}/value`, { text });
        },
        async evaluate(source) {
          // Safari WebDriver's direct function execution introduces a microtask
          // checkpoint after custom-element callbacks that normal page scripts
          // do not. Execute the workload as a real script in the page instead.
          const result = await webdriver('POST', `${endpoint}/execute/sync`, {
            script: evaluatePageScript,
            args: [source, `__brimpCoverage_${randomUUID()}`],
          });
          if (!result) throw Error('Safari did not execute the coverage page script');
          if (!result.ok) {
            const error = new Error(result.message);
            error.name = result.name;
            if (result.stack) error.stack = `${error.name}: ${error.message}\n${result.stack}`;
            throw error;
          }
          return result.value;
        },
      };
    },
    close() {
      return webdriver('DELETE', endpoint);
    },
  };
}
