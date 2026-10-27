import {readFile} from 'node:fs/promises';
import {createHash} from 'node:crypto';
import {dirname, join, resolve} from 'node:path';
import {fileURLToPath, pathToFileURL} from 'node:url';
import {workloads, configurations, workloadConfigurations} from './catalog.mjs';
import {runScenario} from './scenarios.mjs';
import {createNodeDOM} from './node-dom.mjs';

const root = dirname(fileURLToPath(import.meta.url));
const [engine, id, configuration = 'default', origin] = process.argv.slice(2);
const entry = workloads.find(item => item.id === id);
const workload = entry && workloadConfigurations(entry).includes(configuration) ? {...entry, plugins: configurations[configuration]} : null;
if (!workload || !['chromium', 'brimp', 'jsdom', 'happy-dom'].includes(engine)) throw new Error('worker requires an engine and workload ID');
if (['jsdom', 'happy-dom'].includes(engine) && configuration !== 'default') throw new Error('Brimp plugins do not apply to Node DOM implementations');
const result = {engine, id, configuration, kind: workload.kind, initialized: false, plugins: engine === 'brimp' ? workload.plugins : [], status: 'setup_error', checkpoints: [], errors: [], console: [], blockedRequests: []};
const started = performance.now();
let close;
const send = data => process.send?.(data);
const record = (key, value) => { if (result[key].length < 100) result[key].push(value); };

try {
  if (!origin || new URL(origin).hostname !== '127.0.0.1') throw new Error('worker requires a local fixture origin');
  let page;
  if (engine === 'chromium') {
    const {chromium} = await import('playwright');
    const browser = await chromium.launch({headless: true});
    close = () => browser.close();
    result.engineVersion = browser.version();
    const context = await browser.newContext();
    await context.route('**/*', route => {
      const url = route.request().url();
      if (new URL(url).origin === origin) return route.continue();
      record('blockedRequests', url);
      return route.abort('blockedbyclient');
    });
    const inner = await context.newPage();
    inner.setDefaultTimeout(8000);
    inner.on('pageerror', error => record('errors', String(error.stack ?? error)));
    inner.on('console', message => { if (['error', 'warning'].includes(message.type())) record('console', message.text().slice(0, 4000)); });
    page = {
      goto: url => inner.goto(url, {waitUntil: 'load', timeout: 15000}),
      evaluate: source => inner.evaluate(source),
      fill: (selector, text) => inner.locator(selector).first().fill(text),
      press: (selector, key) => inner.locator(selector).first().press(key),
      click: selector => inner.locator(selector).first().click(),
    };
  } else if (engine === 'brimp') {
    const binding = resolve(process.env.BRIMP_NODE_BINDING ?? join(root, '../../bindings/node/index.js'));
    const api = await import(pathToFileURL(binding).href);
    const session = await api.createSession({timeout: 12000});
    close = () => session.close();
    result.binding = binding;
    result.nativeSha256 = createHash('sha256').update(await readFile(join(dirname(binding), 'brimp.node'))).digest('hex');
    result.loader = {frameworkPath: process.env.DYLD_FRAMEWORK_PATH ?? null, libraryPath: process.env.DYLD_LIBRARY_PATH ?? null};
    const inner = await session.newPage();
    inner.on('error', error => record('errors', error.message));
    inner.on('console', event => { if (['error', 'warn'].includes(event.level)) record('console', event.message.slice(0, 4000)); });
    inner.route('*', route => {
      if (new URL(route.request.url).origin === origin) route.continueRequest();
      else { record('blockedRequests', route.request.url); route.fail('External HTTP blocked by corpus runner'); }
    });
    page = {
      goto: url => inner.get(url), evaluate: source => inner.evaluate(source),
      fill: (selector, text) => inner.fill(selector, text),
      press: (selector, key) => inner.press(selector, key), click: selector => inner.click(selector),
    };
  } else {
    const adapter = await createNodeDOM(engine, origin, record);
    page = adapter.page;
    close = adapter.close;
    result.engineVersion = adapter.version;
    result.settings = adapter.settings;
  }
  result.status = 'running';
  result.initialized = true;
  send({type: 'progress', result});
  result.observation = await runScenario(workload, page, origin, name => {
    result.checkpoints.push(name);
    send({type: 'progress', result});
  });
  result.status = 'passed';
} catch (error) {
  if (result.status !== 'setup_error') {
    result.status = error.name === 'AssertionError' || /AssertionError/.test(error.message) ? 'assertion_failed'
      : /timeout|timed out/i.test(`${error.name} ${error.message}`) ? 'timeout' : 'execution_error';
  }
  result.failure = {name: error.name, message: error.message, stack: error.stack};
} finally {
  result.elapsedMs = Math.round(performance.now() - started);
  // Emit before cleanup so a stuck native destructor cannot hide the finding.
  send({type: 'result', result});
  try { await close?.(); }
  catch (error) { send({type: 'cleanup_error', message: String(error)}); }
  process.disconnect?.();
}
