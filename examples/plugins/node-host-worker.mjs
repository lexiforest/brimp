import { Worker } from 'node:worker_threads';
import { createSession, Plugin } from '../../bindings/node/index.js';

// Executes caller-supplied Node worker source. This does not implement browser Worker semantics.
const workers = new Map();
let next = 0;
const plugin = new Plugin('host-worker', `({global, host}) => {
  global.runHostWorker = async source => {
    const id = await host.call('start', source);
    return {
      subscribe(callback) { return host.subscribe('worker:' + id, callback); },
      post(message) { return host.call('post', id, message); },
      close() { return host.call('stop', id); },
    };
  };
  return () => host.call('dispose');
}`, {
  start(context, source) {
    const id = String(++next);
    const worker = new Worker(source, {eval: true});
    workers.set(id, {worker, page: context.pageId, realm: context.realmId});
    worker.on('message', value => { if (!context.cancelled) context.emit('worker:' + id, value); });
    worker.on('error', error => { if (!context.cancelled) context.emit('worker:' + id, {error: error.message}); });
    return id;
  },
  post(context, id, value) {
    const entry = workers.get(id);
    if (!entry || entry.realm !== context.realmId) throw new Error('unknown worker');
    entry.worker.postMessage(value); return null;
  },
  async stop(context, id) {
    const entry = workers.get(id);
    if (!entry || entry.realm !== context.realmId) throw new Error('unknown worker');
    workers.delete(id); await entry.worker.terminate(); return null;
  },
  async dispose(context) {
    for (const [id, entry] of workers) if (entry.realm === context.realmId) {
      workers.delete(id); await entry.worker.terminate();
    }
    return null;
  },
});
const session = await createSession({plugins: [plugin]});
try {
  const page = await session.newPage();
  console.log(await page.evaluate(`(async () => {
    const worker = await runHostWorker("const {parentPort} = require('node:worker_threads'); parentPort.on('message', value => parentPort.postMessage(value * 2));");
    const result = new Promise(resolve => worker.subscribe(resolve));
    await worker.post(21);
    const value = await result;
    await worker.close();
    return value;
  })()`));
} finally { await session.close(); }
