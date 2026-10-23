import {test} from 'node:test';
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {mkdir, readFile, readdir, writeFile} from 'node:fs/promises';
import {dirname, join} from 'node:path';
import {fileURLToPath} from 'node:url';
import {workloads} from '../catalog.mjs';
import {executeWorker} from '../helpers/process.mjs';
import {startServer} from '../helpers/server.mjs';

const root = fileURLToPath(new URL('../', import.meta.url));
const json = async path => JSON.parse(await readFile(join(root, path), 'utf8'));
const expected = await json('expected.json');
const manifest = await json('dist/manifest.json');
const hash = bytes => createHash('sha256').update(bytes).digest('hex');
assert.deepEqual(Object.keys(expected.cases), workloads);
assert.deepEqual(manifest.packages[''].dependencies, expected.dependencies, 'review expectations when changing library versions');
assert.deepEqual(manifest.sources, expected.sources, 'review expectations when changing upstream applications');
assert.equal(hash(await readFile(join(root, 'package-lock.json'))), manifest.packageLockSha256, 'rerun npm run prepare:fixtures');
for (const [path, digest] of Object.entries(manifest.inputs)) {
  assert.equal(hash(await readFile(join(root, path))), digest, `build input changed: ${path}; rerun npm run prepare:fixtures`);
}
for (const [path, digest] of Object.entries(manifest.files)) {
  assert.equal(hash(await readFile(join(root, 'dist', path))), digest, `artifact changed: ${path}`);
}

test('upstream TodoMVC builds are unchanged', async () => {
  async function checkTree(source, destination) {
    for (const entry of await readdir(source, {withFileTypes: true})) {
      const from = join(source, entry.name), to = join(destination, entry.name);
      if (entry.isDirectory()) await checkTree(from, to);
      else assert.deepEqual(await readFile(to), await readFile(from), `changed upstream file: ${to}`);
    }
  }
  for (const [id, source] of Object.entries(manifest.unchangedArtifacts)) {
    await checkTree(join(root, '.cache', source), join(root, 'dist', id));
  }
});

const report = {createdAt: new Date().toISOString(), reference: expected.reference, results: []};
const destination = join(root, 'results/report.json');
await mkdir(dirname(destination), {recursive: true});
await writeFile(destination, JSON.stringify(report, null, 2) + '\n');
for (const id of workloads) {
  test(`${id}: browser behavior`, {timeout: 65000}, async () => {
    const server = await startServer(join(root, 'dist'), id, manifest);
    let result;
    try {
      result = await executeWorker(join(root, 'worker.mjs'), [id, server.origin], {initial: {id}});
      assert.equal(result.status, 'passed', JSON.stringify(result, null, 2));
      assert.deepEqual(result.checkpoints, expected.cases[id].checkpoints);
      assert.deepEqual(result.observation, expected.cases[id].observation);
    } catch (error) {
      if (result) result.regression = error.message;
      throw error;
    } finally {
      await server.close();
      report.results.push({...result, id, requests: server.requests});
      await writeFile(destination, JSON.stringify(report, null, 2) + '\n');
    }
  });
}
