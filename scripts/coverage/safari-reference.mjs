// Reuse measured reference data explicitly; never silently substitute it for a failed run.
export function reuseSafariReference(previous, rows, probeHash, html) {
  if (previous?.runtimes?.safari?.status !== 'measured') throw Error('A measured Safari reference is required');
  if (previous.generator_sources?.['scripts/coverage/probe.mjs'] !== probeHash
      || previous.fixture?.html !== html) throw Error('Safari probe or fixture changed; collect a fresh reference');
  if (Object.keys(previous.fields).length !== rows.length || rows.some(row => {
    const field = previous.fields[row.id];
    return !field?.safari || field.target !== row.target || field.name !== row.name
      || !previous.targets?.[row.target]?.safari;
  })) throw Error('Safari inventory changed; collect a fresh reference');
  return {
    runtime: {...previous.runtimes.safari, reused: true,
      measured_at: previous.runtimes.safari.measured_at ?? previous.generated_at,
      fixture: previous.runtimes.safari.fixture ?? previous.fixture},
    observations: {
      fields: Object.fromEntries(rows.map(row => [row.id, previous.fields[row.id].safari])),
      targets: Object.fromEntries(Object.entries(previous.targets).map(([name, target]) => [name, target.safari])),
      environment: previous.runtimes.safari.environment,
      cleanup_errors: previous.runtimes.safari.cleanup_errors,
    },
  };
}
