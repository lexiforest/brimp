import {resolve} from 'node:path';
import {fileURLToPath, pathToFileURL} from 'node:url';
import {workloads} from './catalog.mjs';
import {runScenario} from './helpers/scenarios.mjs';

const [id, origin] = process.argv.slice(2);
if (!workloads.includes(id) || new URL(origin).hostname !== '127.0.0.1') {
  throw new Error('worker requires a regression workload and local fixture origin');
}
const result = {id, status: 'setup_error', checkpoints: [], errors: [], console: [], blockedRequests: []};
const send = data => process.send?.(data);
const record = (key, value) => { if (result[key].length < 100) result[key].push(value); };
let session;
try {
  const binding = resolve(process.env.BRIMP_NODE_BINDING ?? fileURLToPath(new URL('../bindings/node/index.js', import.meta.url)));
  const {createSession} = await import(pathToFileURL(binding).href);
  session = await createSession({timeout: 12000});
  const inner = await session.newPage();
  inner.on('error', error => record('errors', error.message));
  inner.on('console', event => {
    if (['error', 'warn'].includes(event.level)) record('console', event.message.slice(0, 4000));
  });
  inner.route('*', route => {
    if (new URL(route.request.url).origin === origin) route.continueRequest();
    else {
      record('blockedRequests', route.request.url);
      route.fail('External HTTP blocked by regression tests');
    }
  });
  const page = {
    goto: url => inner.get(url), evaluate: source => inner.evaluate(source),
    fill: (selector, text) => inner.fill(selector, text),
    press: (selector, key) => inner.press(selector, key), click: selector => inner.click(selector),
  };
  result.status = 'running';
  send({type: 'progress', result});
  result.observation = await runScenario({id}, page, origin, name => {
    result.checkpoints.push(name);
    send({type: 'progress', result});
  });
  result.status = 'passed';
} catch (error) {
  if (result.status !== 'setup_error') result.status = 'failed';
  result.failure = {name: error.name, message: error.message, stack: error.stack};
} finally {
  // The parent retains this result even if native cleanup hangs or crashes.
  send({type: 'result', result});
  try { await session?.close(); }
  catch (error) { send({type: 'cleanup_error', message: String(error)}); }
  process.disconnect?.();
}
