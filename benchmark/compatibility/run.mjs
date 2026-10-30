import { createHash } from 'node:crypto';
import { mkdir, readFile, readdir, writeFile } from 'node:fs/promises';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { parseArgs } from 'node:util';
import { workloads, configurations, workloadConfigurations, isRegression } from './catalog.mjs';
import { compare, markdown } from './report.mjs';
import { executeWorker } from './process.mjs';
import { startServer } from './server.mjs';

const root = dirname(fileURLToPath(import.meta.url));
const { values } = parseArgs({
  options: {
    only: { type: 'string' },
    configuration: { type: 'string' },
    output: { type: 'string', default: 'results' },
    timeout: { type: 'string', default: '60' },
    strict: { type: 'boolean', default: false },
  },
});
const timeout = Number(values.timeout) * 1000;
if (!Number.isFinite(timeout) || timeout < 1000 || timeout > 300000)
  throw new Error('--timeout must be between 1 and 300 seconds');
const selection = values.only ? values.only.split(',') : workloads.map((item) => item.id);
for (const id of selection)
  if (!workloads.some((item) => item.id === id)) throw new Error(`Unknown workload: ${id}`);
const requestedConfigurations = values.configuration?.split(',');
for (const name of requestedConfigurations ?? [])
  if (!Object.hasOwn(configurations, name)) throw new Error(`Unknown configuration: ${name}`);
const selected = workloads
  .filter((item) => selection.includes(item.id))
  .flatMap((workload) =>
    workloadConfigurations(workload)
      .filter((name) => !requestedConfigurations || requestedConfigurations.includes(name))
      .map((configuration) => ({
        ...workload,
        configuration,
        plugins: configurations[configuration],
      })),
  );
if (!selected.length) throw new Error('No workloads selected for these configurations');
const hash = (bytes) => createHash('sha256').update(bytes).digest('hex');
const manifestBytes = await readFile(join(root, 'dist/manifest.json'));
const manifest = JSON.parse(manifestBytes);
if (manifest.packageLockSha256 !== hash(await readFile(join(root, 'package-lock.json'))))
  throw new Error('Dependencies changed; rerun npm run prepare:fixtures');
for (const [path, expected] of Object.entries(manifest.inputs)) {
  if (hash(await readFile(join(root, path))) !== expected)
    throw new Error(`Build input changed: ${path}; rerun npm run prepare:fixtures`);
}
// Refuse to run changed/missing artifacts under a stale provenance manifest.
for (const [path, expected] of Object.entries(manifest.files)) {
  if (hash(await readFile(join(root, 'dist', path))) !== expected)
    throw new Error(`Artifact integrity mismatch: ${path}`);
}

const execute = async (engine, workload) => {
  // A separate event loop is necessary for implementations using synchronous
  // resource requests. The worker must never serve its own HTTP fixtures.
  const server = await startServer(join(root, 'dist'), workload.id, manifest);
  try {
    const result = await executeWorker(
      join(root, 'worker.mjs'),
      [engine, workload.id, workload.configuration, server.origin],
      {
        timeout,
        initial: { engine, id: workload.id, kind: workload.kind, initialized: false },
      },
    );
    return { ...result, requests: server.requests };
  } finally {
    await server.close();
  }
};

const report = {
  schema: 2,
  createdAt: new Date().toISOString(),
  manifestSha256: hash(manifestBytes),
  sources: manifest.sources,
  versions: Object.fromEntries(Object.entries(manifest.packages[''].dependencies)),
  host: { platform: process.platform, arch: process.arch, node: process.version },
  policy: {
    httpRouting:
      'local-origin routes/interceptors; jsdom synchronous XHR bypasses interceptors; not an OS network sandbox',
    detectorVerdicts: 'observational',
    detectorsExpandRuntimeScope: false,
    nodeDOM:
      'Unmodified browser-window APIs; script execution enabled; no host polyfills. Simulated DOM input.',
  },
  results: [],
};
report.harness = {};
for (const name of (await readdir(root)).filter((name) => name.endsWith('.mjs')).sort())
  report.harness[name] = hash(await readFile(join(root, name)));
const destination = resolve(root, values.output);
await mkdir(destination, { recursive: true });
await writeFile(join(destination, 'manifest.json'), manifestBytes);
for (const workload of selected) {
  console.log(
    `[${report.results.length + 1}/${selected.length}] ${workload.id} (${workload.configuration})`,
  );
  const runs = { chromium: await execute('chromium', workload) };
  const comparisons = {};
  const candidates =
    workload.configuration === 'default' ? ['brimp', 'jsdom', 'happy-dom'] : ['brimp'];
  for (const engine of candidates) {
    runs[engine] = await execute(engine, workload);
    comparisons[engine] = compare(workload, runs.chromium, runs[engine]);
    console.log(`  ${engine}: ${runs[engine].status}; ${comparisons[engine]}`);
  }
  report.results.push({
    id: workload.id,
    configuration: workload.configuration,
    kind: workload.kind,
    runs,
    comparisons,
  });
  await writeFile(join(destination, 'report.json'), JSON.stringify(report, null, 2) + '\n');
  await writeFile(join(destination, 'report.md'), markdown(report));
}
console.log(`Report: ${join(destination, 'report.md')}`);
// Previously matched functional workloads are regression gates. Known gaps and
// detector findings remain observations; --strict gates all remaining gaps.
if (
  report.results.some(
    (row) =>
      isRegression(row) ||
      Object.values(row.comparisons).some(
        (comparison) =>
          comparison === 'invalid_reference' ||
          comparison === 'unavailable' ||
          (values.strict && !['matched', 'observed'].includes(comparison)),
      ),
  )
)
  process.exitCode = 1;
