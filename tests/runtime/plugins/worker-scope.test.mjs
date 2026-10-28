import test from 'node:test';
import assert from 'node:assert/strict';
import { createSession, workersPlugin } from '../../../bindings/node/index.js';

async function evaluate(source, plugins = [workersPlugin()]) {
  const session = await createSession({ plugins });
  try {
    return await (await session.newPage()).evaluate(source);
  } finally {
    await session.close();
  }
}

test('Workers execute a blob script and exchange cloned messages both ways', async () => {
  assert.deepEqual(
    await evaluate(`(async () => {
    const source = [
      'self.onmessage = event => { self.postMessage({ doubled: event.data.value * 2, name: self.name, href: self.location.href }); };',
      'self.postMessage({ ready: true, offlineCanvas: typeof OffscreenCanvas });',
    ].join('\\n');
    const url = URL.createObjectURL(new Blob([source], { type: 'text/javascript' }));
    const worker = new Worker(url, { name: 'fixture' });
    const seen = [];
    const finished = new Promise(resolve => {
      worker.onmessage = event => { seen.push(event.data); if (seen.length === 2) resolve(); };
      worker.onerror = event => { seen.push({ error: String(event.message) }); resolve(); };
    });
    setTimeout(() => worker.postMessage({ value: 21 }), 5);
    await Promise.race([finished, new Promise(resolve => setTimeout(resolve, 3000))]);
    URL.revokeObjectURL(url);
    worker.terminate();
    worker.postMessage({ value: 1 });
    return [seen[0], seen[1].doubled, seen[1].name, seen[1].href === url, seen.length, seen[0].offlineCanvas];
  })()`),
    [{ ready: true, offlineCanvas: 'function' }, 42, 'fixture', true, 2, 'function'],
  );
});

test('Worker failures surface as error events and terminate stops delivery', async () => {
  assert.deepEqual(
    await evaluate(`(async () => {
    const missing = new Worker('https://example.invalid/worker.js');
    const failure = await new Promise(resolve => {
      missing.onerror = event => resolve(String(event.message));
      setTimeout(() => resolve('timeout'), 2000);
    });
    const source = 'self.postMessage("first"); self.onmessage = () => self.postMessage("late");';
    const url = URL.createObjectURL(new Blob([source], { type: 'text/javascript' }));
    const worker = new Worker(url);
    const seen = [];
    worker.onmessage = event => seen.push(event.data);
    await new Promise(resolve => setTimeout(resolve, 20));
    worker.postMessage('ignored');
    worker.terminate();
    await new Promise(resolve => setTimeout(resolve, 20));
    let importScriptsError = null;
    const second = new Worker(URL.createObjectURL(new Blob(['try { self.importScripts("x.js"); } catch (error) { self.postMessage(error.name); }'], { type: 'text/javascript' })));
    const name = await new Promise(resolve => { second.onmessage = event => resolve(event.data); setTimeout(() => resolve('timeout'), 2000); });
    second.terminate();
    return [failure.includes('Cannot load worker script'), seen, name];
  })()`),
    [true, ['first'], 'NetworkError'],
  );
});
