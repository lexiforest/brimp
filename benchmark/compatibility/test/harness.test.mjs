import {test} from 'node:test';
import assert from 'node:assert/strict';
import {mkdtemp, readFile, readdir, writeFile, rm} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {compare, markdown} from '../report.mjs';
import {executeWorker} from '../process.mjs';
import {startServer} from '../server.mjs';
import {workloads} from '../catalog.mjs';

test('diagnostic verdicts are observations, while functional results must match', () => {
  const pass = observation => ({status: 'passed', observation});
  assert.equal(compare({kind: 'diagnostic'}, pass({bot: false}), pass({bot: true})), 'observed');
  assert.equal(compare({kind: 'library'}, pass({value: 1}), pass({value: 2})), 'different');
  assert.equal(compare({kind: 'library'}, pass({value: 1}), pass({value: 1})), 'matched');
  assert.equal(compare({kind: 'diagnostic'}, pass({}), {status: 'timeout'}), 'failed');
  assert.equal(compare({kind: 'library'}, {status: 'assertion_failed'}, pass({})), 'invalid_reference');
  assert.equal(compare({kind: 'library'}, pass({}), {status: 'setup_error'}), 'unavailable');
  assert.equal(compare({kind: 'library'}, pass({}), {status: 'crash', initialized: false}), 'unavailable');
});

test('all 15 approved workloads are present and detectors have no plugins', () => {
  assert.equal(workloads.length, 15);
  assert.equal(new Set(workloads.map(item => item.id)).size, 15);
  assert.deepEqual(workloads.filter(item => item.kind === 'diagnostic').map(item => item.id), ['modernizr', 'botd', 'creepjs']);
  for (const item of workloads.filter(item => item.kind === 'diagnostic')) assert.deepEqual(item.plugins, []);
});

test('prepared upstream applications and CreepJS are byte-for-byte unchanged', async () => {
  const root = new URL('../', import.meta.url);
  const manifest = JSON.parse(await readFile(new URL('dist/manifest.json', root)));
  async function checkTree(source, destination) {
    for (const entry of await readdir(source, {withFileTypes: true})) {
      const from = new URL(entry.name + (entry.isDirectory() ? '/' : ''), source);
      const to = new URL(entry.name + (entry.isDirectory() ? '/' : ''), destination);
      if (entry.isDirectory()) await checkTree(from, to);
      else assert.deepEqual(await readFile(to), await readFile(from), `changed upstream file: ${to}`);
    }
  }
  for (const [id, source] of Object.entries(manifest.unchangedArtifacts)) {
    await checkTree(new URL(`.cache/${source}/`, root), new URL(`dist/${id}/`, root));
  }
});

test('isolated worker reports crashes, missing results, and enforced deadlines', async () => {
  const directory = await mkdtemp(join(tmpdir(), 'brimp-corpus-process-'));
  try {
    const script = join(directory, 'worker.mjs');
    await writeFile(script, 'process.exit(7)');
    assert.equal((await executeWorker(script, [], {timeout: 2000})).status, 'crash');
    await writeFile(script, 'process.disconnect()');
    assert.equal((await executeWorker(script, [], {timeout: 2000})).status, 'crash');
    await writeFile(script, 'process.on("SIGTERM",()=>{}); process.send({type:"progress",result:{checkpoints:["started"]}}); setInterval(()=>{},1000)');
    const result = await executeWorker(script, [], {timeout: 300, grace: 100});
    assert.equal(result.status, 'timeout');
    assert.deepEqual(result.checkpoints, ['started']);
    await writeFile(script, 'process.send({type:"result",result:{status:"passed",observation:{answer:42}}}); process.disconnect()');
    assert.equal((await executeWorker(script, [], {timeout: 2000})).observation.answer, 42);
  } finally { await rm(directory, {recursive: true, force: true}); }
});

test('fixture server restricts files and preserves submitted bytes', async () => {
  const directory = await mkdtemp(join(tmpdir(), 'brimp-corpus-server-'));
  const server = await startServer(directory, 'fixture', {files: {}});
  try {
    const echo = await fetch(server.origin + '/api/echo', {method: 'POST', body: 'name=Ada+Lovelace'});
    assert.deepEqual(await echo.json(), {method: 'POST', body: 'name=Ada+Lovelace'});
    assert.deepEqual([...new Uint8Array(await (await fetch(server.origin + '/api/bytes')).arrayBuffer())], [0, 127, 255]);
    assert.equal((await fetch(server.origin + '/%2e%2e%2fpackage.json')).status, 404);
    const contact = await fetch(server.origin + '/contact/1', {method: 'PUT', body: new URLSearchParams({firstName: '<Ada>', lastName: 'Lovelace', email: 'ada@example.test'})});
    assert.match(await contact.text(), /&lt;Ada&gt;/);
    assert.equal(server.requests.length, 4);
  } finally { await server.close(); await rm(directory, {recursive: true, force: true}); }
});

test('happy-dom loads synchronous scripts from the parent fixture server', async () => {
  const {fileURLToPath} = await import('node:url');
  const directory = await mkdtemp(join(tmpdir(), 'brimp-corpus-sync-'));
  const {mkdir} = await import('node:fs/promises');
  await mkdir(join(directory, 'jquery'));
  await writeFile(join(directory, 'jquery/index.html'), '<script src="/driver.js"></script>');
  await writeFile(join(directory, 'jquery/driver.js'), 'window.runWorkload = () => ({loaded: true});');
  const server = await startServer(directory, 'jquery', {files: {'jquery/index.html': '', 'jquery/driver.js': ''}});
  try {
    const result = await executeWorker(fileURLToPath(new URL('../worker.mjs', import.meta.url)),
      ['happy-dom', 'jquery', 'default', server.origin], {timeout: 10000});
    assert.equal(result.status, 'passed', JSON.stringify(result));
    assert.deepEqual(result.observation, {loaded: true});
    assert.ok(server.requests.some(request => request.path === '/driver.js'));
  } finally { await server.close(); await rm(directory, {recursive: true, force: true}); }
});

test('report includes detector verdict without turning it into a pass requirement', () => {
  const report = {createdAt: 'fixed', manifestSha256: 'hash', results: [{
    id: 'botd', kind: 'diagnostic', configuration: 'default', comparisons: {brimp: 'observed'}, runs: {
    chromium: {engine: 'chromium', status: 'passed', observation: {detection: {bot: false}}},
    brimp: {engine: 'brimp', status: 'passed', observation: {detection: {bot: true}}},
    },
  }]};
  assert.match(markdown(report), /"bot":true/);
  assert.match(markdown(report), /observed/);
});

test('named configurations and regression policy preserve detector scope', async () => {
  const {configurations, workloadConfigurations, isRegression} = await import('../catalog.mjs');
  assert.deepEqual(configurations.default, []);
  assert.deepEqual(workloadConfigurations({id:'dexie'}), ['default']);
  assert.equal(isRegression({id:'jquery',kind:'library',configuration:'default',comparisons:{brimp:'failed'}}),true);
  assert.equal(isRegression({id:'jose',kind:'library',configuration:'default',comparisons:{brimp:'matched',jsdom:'failed','happy-dom':'failed'}}),false);
  assert.equal(isRegression({id:'creepjs',kind:'diagnostic',configuration:'default',comparisons:{brimp:'failed'}}),false);
});
