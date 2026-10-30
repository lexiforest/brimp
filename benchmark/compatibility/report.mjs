import { isDeepStrictEqual } from 'node:util';

export function compare(workload, reference, candidate) {
  if (reference.status !== 'passed') return 'invalid_reference';
  if (candidate.status === 'setup_error' || candidate.initialized === false) return 'unavailable';
  if (candidate.status !== 'passed') return 'failed';
  if (workload.kind === 'diagnostic') return 'observed';
  return isDeepStrictEqual(reference.observation, candidate.observation) ? 'matched' : 'different';
}

export function markdown(report) {
  const lines = [
    '# Browser workload report',
    '',
    `Generated: ${report.createdAt}`,
    '',
    `Artifact manifest SHA-256: \`${report.manifestSha256}\``,
    '',
    'These are compatibility observations, not a browser-conformance or anti-detection score.',
    'Detector findings do not expand the runtime contract. Timings include setup and are not benchmarks.',
    '',
    'Candidate cells show execution / comparison to Chromium; — means a Brimp-only configuration.',
    '',
    '| Workload | Chromium | Brimp | jsdom | happy-dom |',
    '| --- | --- | --- | --- | --- |',
  ];
  for (const row of report.results) {
    const cells = ['brimp', 'jsdom', 'happy-dom'].map((engine) =>
      row.runs[engine] ? `${row.runs[engine].status} / ${row.comparisons[engine]}` : '—',
    );
    lines.push(
      `| ${row.id} (${row.configuration}) | ${row.runs.chromium.status} | ${cells.join(' | ')} |`,
    );
  }
  lines.push('', '## Functional summary (default configuration)', '');
  const functional = report.results.filter(
    (row) => row.configuration === 'default' && row.kind !== 'diagnostic',
  );
  for (const engine of ['brimp', 'jsdom', 'happy-dom']) {
    const matched = functional.filter((row) => row.comparisons[engine] === 'matched').length;
    lines.push(
      `- ${engine}: ${matched}/${functional.length} functional observations matched Chromium.`,
    );
  }
  if (report.versions)
    lines.push(
      '',
      `Pinned Node DOM versions: jsdom ${report.versions.jsdom}; happy-dom ${report.versions['happy-dom']}. Node ${report.host.node}.`,
    );
  lines.push('', '## Findings', '');
  for (const row of report.results) {
    for (const run of Object.values(row.runs)) {
      if (run.failure)
        lines.push(
          `- **${row.id} / ${run.engine}:** ${run.failure.message.replace(/[\r\n|]/g, ' ').slice(0, 800)}`,
        );
      if (run.errors?.length)
        lines.push(
          `- **${row.id} / ${run.engine} page errors:** ${run.errors
            .join('; ')
            .replace(/[\r\n|]/g, ' ')
            .slice(0, 800)}`,
        );
      if (run.status !== 'passed' && run.console?.length) {
        const messages = [...new Set(run.console.map((message) => message.split('\n')[0]))];
        lines.push(
          `- **${row.id} / ${run.engine} console findings:** ${messages.join('; ').replace(/\|/g, ' ').slice(0, 1600)}`,
        );
      }
    }
    if (row.kind === 'diagnostic') {
      for (const run of Object.values(row.runs))
        if (run.observation) {
          const summary =
            row.id === 'creepjs'
              ? {
                  collectors: Object.keys(run.observation.fingerprint),
                  lies: run.observation.fingerprint.lies,
                  capturedErrors: run.observation.fingerprint.capturedErrors,
                }
              : run.observation;
          lines.push(
            `- **${row.id} / ${run.engine} observation:** \`${JSON.stringify(summary).replace(/`/g, "'").slice(0, 2500)}\``,
          );
        }
    }
  }
  lines.push(
    '',
    'Full observations, checkpoints, page errors, local requests, and blocked HTTP requests are in the accompanying JSON.',
    '',
  );
  return lines.join('\n');
}
