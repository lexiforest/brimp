import { readFile, writeFile } from 'node:fs/promises';
import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { fileURLToPath } from 'node:url';
import { isDeepStrictEqual } from 'node:util';
import { executeWorker } from '../process.mjs';
import { probes } from './probes.mjs';
const root = fileURLToPath(new URL('.', import.meta.url));
const sources = {
  jsdom: {
    path: process.env.JSDOM_SOURCE ?? '/tmp/brimp-coverage-jsdom',
    url: 'https://github.com/jsdom/jsdom',
  },
  'happy-dom': {
    path: process.env.HAPPY_DOM_SOURCE ?? '/tmp/brimp-coverage-happy-dom',
    url: 'https://github.com/capricorn86/happy-dom',
  },
};
for (const source of Object.values(sources))
  source.commit = execFileSync('git', ['-C', source.path, 'rev-parse', 'HEAD'], {
    encoding: 'utf8',
  }).trim();
const report = {
  createdAt: new Date().toISOString(),
  host: { node: process.version, platform: process.platform, arch: process.arch },
  sources,
  policy: {
    sourceReview:
      'Cloned HEAD source/tests guide independently authored probes; execution uses corpus-pinned npm releases, not builds of these checkouts.',
    reference: 'Chromium; failed references are invalid, not compatibility evidence.',
    scope: 'Targeted initial audit; not full upstream suites or total feature coverage.',
    plugins: 'Storage, geometry and Typed OM run in the default core.',
    isolation:
      'Fresh realm per probe, worker per engine; 90s engine deadline. Records the currently built runtime binding.',
  },
  hashes: {},
  runs: {},
  comparisons: [],
};
for (const name of ['probes.mjs', 'worker.mjs', 'run.mjs'])
  report.hashes[name] = createHash('sha256')
    .update(await readFile(root + name))
    .digest('hex');
report.packageLockSha256 = createHash('sha256')
  .update(await readFile(new URL('../package-lock.json', import.meta.url)))
  .digest('hex');
for (const engine of ['chromium', 'brimp', 'jsdom', 'happy-dom']) {
  report.runs[engine] = await executeWorker(root + 'worker.mjs', [engine], {
    timeout: 90000,
    initial: { engine, probes: [] },
  });
  console.log(
    engine,
    report.runs[engine].status,
    report.runs[engine].probes?.length,
    report.runs[engine].failure ?? '',
  );
  await writeFile(root + 'report.json', JSON.stringify(report, null, 2) + '\n');
}
for (const probe of probes) {
  const ref = report.runs.chromium.probes.find((r) => r.id === probe.id);
  const row = { id: probe.id, area: probe.area, comparisons: {} };
  for (const engine of ['brimp', 'jsdom', 'happy-dom']) {
    const value = report.runs[engine].probes.find((r) => r.id === probe.id);
    row.comparisons[engine] =
      ref?.status !== 'completed' || !ref.observation?.ok
        ? 'invalid_reference'
        : value?.status !== 'completed'
          ? 'unavailable'
          : isDeepStrictEqual(ref.observation, value.observation)
            ? 'matched'
            : value.observation.ok
              ? 'different'
              : 'throws';
  }
  row.peerConfirmedGap =
    ['different', 'throws'].includes(row.comparisons.brimp) &&
    ['jsdom', 'happy-dom'].some((e) => row.comparisons[e] === 'matched');
  report.comparisons.push(row);
}
report.surfaceCandidates = {};
for (const peer of ['jsdom', 'happy-dom']) {
  report.surfaceCandidates[peer] = {};
  for (const [name, members] of Object.entries(report.runs[peer].inventory ?? {})) {
    if (!members) continue;
    const own = report.runs.brimp.inventory?.[name];
    const missing = members.filter((n) => !own?.includes(n));
    if (missing.length)
      report.surfaceCandidates[peer][name] = { constructorAbsent: !own, members: missing };
  }
}
await writeFile(root + 'report.json', JSON.stringify(report, null, 2) + '\n');
const lines = [
  '# DOM coverage probe results',
  '',
  `Generated ${report.createdAt}. Targeted checks; not an overall conformance score.`,
  '',
  'Runtime releases and source-review checkouts are recorded separately in report.json.',
  '',
  '| Probe | Area | Brimp core | jsdom | happy-dom | Brimp + bundled plugins | Peer-confirmed gap |',
  '| --- | --- | --- | --- | --- | --- | --- |',
];
for (const row of report.comparisons)
  lines.push(
    `| ${row.id} | ${row.area} | ${Object.values(row.comparisons).join(' | ')} | ${row.peerConfirmedGap ? 'yes' : ''} |`,
  );
lines.push(
  '',
  'An exception is recorded with its name/message in report.json. Surface candidates include inherited prototype members; instance properties, descriptors and semantics need investigation. Plugin rows are intentional default-core omissions, not missing plugin implementations.',
  '',
);
await writeFile(root + 'results.md', lines.join('\n'));
console.log(
  'Peer-confirmed gaps:',
  report.comparisons
    .filter((r) => r.peerConfirmedGap)
    .map((r) => r.id)
    .join(', '),
);
if (
  report.comparisons.some(
    (r) => r.comparisons.brimp === 'invalid_reference' || r.comparisons.brimp === 'unavailable',
  )
)
  process.exitCode = 1;
