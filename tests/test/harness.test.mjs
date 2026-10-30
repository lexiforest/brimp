import { test } from 'node:test';
import assert from 'node:assert/strict';
import { mkdtemp, writeFile, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { executeWorker } from '../helpers/process.mjs';
import { startServer } from '../helpers/server.mjs';

test('isolated worker reports crashes, missing results, and enforced deadlines', async () => {
  const directory = await mkdtemp(join(tmpdir(), 'brimp-corpus-process-'));
  try {
    const script = join(directory, 'worker.mjs');
    await writeFile(script, 'process.exit(7)');
    assert.equal((await executeWorker(script, [], { timeout: 2000 })).status, 'crash');
    await writeFile(script, 'process.disconnect()');
    assert.equal((await executeWorker(script, [], { timeout: 2000 })).status, 'crash');
    await writeFile(
      script,
      'process.on("SIGTERM",()=>{}); process.send({type:"progress",result:{checkpoints:["started"]}}); setInterval(()=>{},1000)',
    );
    const result = await executeWorker(script, [], { timeout: 300, grace: 100 });
    assert.equal(result.status, 'timeout');
    assert.deepEqual(result.checkpoints, ['started']);
    await writeFile(
      script,
      'process.send({type:"result",result:{status:"passed",observation:{answer:42}}}); process.disconnect()',
    );
    assert.equal((await executeWorker(script, [], { timeout: 2000 })).observation.answer, 42);
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
});

test('fixture server restricts files and preserves submitted bytes', async () => {
  const directory = await mkdtemp(join(tmpdir(), 'brimp-corpus-server-'));
  const server = await startServer(directory, 'fixture', { files: {} });
  try {
    const echo = await fetch(server.origin + '/api/echo', {
      method: 'POST',
      body: 'name=Ada+Lovelace',
    });
    assert.deepEqual(await echo.json(), { method: 'POST', body: 'name=Ada+Lovelace' });
    assert.deepEqual(
      [...new Uint8Array(await (await fetch(server.origin + '/api/bytes')).arrayBuffer())],
      [0, 127, 255],
    );
    assert.equal((await fetch(server.origin + '/%2e%2e%2fpackage.json')).status, 404);
    const contact = await fetch(server.origin + '/contact/1', {
      method: 'PUT',
      body: new URLSearchParams({
        firstName: '<Ada>',
        lastName: 'Lovelace',
        email: 'ada@example.test',
      }),
    });
    assert.match(await contact.text(), /&lt;Ada&gt;/);
    assert.equal(server.requests.length, 4);
  } finally {
    await server.close();
    await rm(directory, { recursive: true, force: true });
  }
});
