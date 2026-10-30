import fs from 'node:fs';
import { createRequire } from 'node:module';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { parseArgs } from 'node:util';

export const tracking = {
  correctness:
    'Browser-script compatibility is the goal. Safari is the browser reference, not a reason to remove useful placeholders or tighten harmless calls. The legacy project defines the inventory and supplies existing call contracts. Document deliberate synthetic behavior and assess changes by script continuation, return shapes, callbacks and promises; full standards conformance is not required.',
  verification:
    'Verify important behavior in installed Safari before marking a behavioral contract done: representative ordinary use, meaningful errors and DOM effects. Avoid exhaustive conversion matrices and obscure WebKit implementation quirks unless a real workload needs them. For legacy placeholders, verify the Brimp contract and compare Safari surface/default behavior, recording intentional limits. Run focused regressions for each change; reserve broad suites for shared infrastructure changes or batch milestones. Historical Chromium results are supplementary only. Surface measurements alone do not verify function behavior.',
  scope:
    'Brimp is a JavaScript runtime, not a full browser. Implement useful lightweight behavior; for heavy browser subsystems copy the legacy placeholder instead of deferring the field. Label placeholder contracts explicitly and record differences from Safari. Known correct values and behavior take precedence over legacy mistakes when implementing actual semantics.',
  placement:
    'Choose behavior and placement separately. Implement core DOM and easy API semantics correctly; adapt legacy placeholders for heavy features without replacing working behavior. Common DOM/environment surfaces belong in core, including their explicit placeholders. Optional execution backends belong in focused plugins; independent value objects, shared scheduling and browser storage belong in core. Implementation cost alone does not make a field a plugin. Record core or with_plugins in progress.target and name the owning plugin in the contract or notes.',
  ownership:
    'Edit fields.<field_id>.progress directly. Regeneration preserves these records by field ID; observations remain generated. New fields start unreviewed, regardless of measured presence.',
  summary_command: 'node scripts/coverage/progress.mjs',
  filter_example: 'node scripts/coverage/progress.mjs --status in_progress',
  statuses: {
    unreviewed:
      'No implementation contract has been selected; this does not mean the field is missing.',
    planned: 'A target configuration and acceptance contract have been selected.',
    in_progress: 'Implementation or verification of the selected contract is underway.',
    blocked: 'Work cannot proceed; notes must explain the blocker.',
    done: 'The selected contract was verified; tests must identify the supporting tests or reproducible checks.',
    deferred: 'Work is postponed; notes must explain why.',
    not_planned: 'An explicit decision not to implement this field; notes must explain why.',
  },
  targets: ['core', 'with_plugins'],
  contract:
    'A concrete acceptance criterion, including useful callable no-ops when appropriate. Done means this contract is met, not full browser conformance.',
  tests:
    'List of repository test paths/IDs or reproducible verification commands. These are human/agent-recorded evidence references, not automatically executed by the progress reader.',
  rules:
    'planned, in_progress and done require target and contract; done also requires tests. blocked, deferred and not_planned require notes. No status is inferred from surface presence or Safari agreement.',
  summary:
    'Progress totals are computed from current records by the summary command, so manual edits cannot leave cached totals stale.',
};

export function initialProgress() {
  return { status: 'unreviewed', target: null, contract: null, tests: [], notes: null };
}

export function validateProgress(progress, id) {
  const fail = (message) => {
    throw Error(`${id}.progress: ${message}`);
  };
  if (!progress || typeof progress !== 'object' || Array.isArray(progress))
    fail('expected a mapping');
  const keys = Object.keys(initialProgress());
  if (Object.keys(progress).length !== keys.length || keys.some((k) => !Object.hasOwn(progress, k)))
    fail(`expected exactly: ${keys.join(', ')}`);
  if (typeof progress.status !== 'string' || !Object.hasOwn(tracking.statuses, progress.status))
    fail('unknown status');
  if (progress.target !== null && !tracking.targets.includes(progress.target))
    fail('target must be core, with_plugins or null');
  const nonempty = (s) => typeof s === 'string' && s.trim().length > 0;
  for (const k of ['contract', 'notes'])
    if (progress[k] !== null && !nonempty(progress[k])) fail(`${k} must be nonempty text or null`);
  if (!Array.isArray(progress.tests) || !progress.tests.every(nonempty))
    fail('tests must be a list of nonempty evidence references');
  if (
    ['planned', 'in_progress', 'done'].includes(progress.status) &&
    (!progress.target || !progress.contract)
  )
    fail('selected work requires target and contract');
  if (progress.status === 'done' && !progress.tests.length)
    fail('done requires verification evidence in tests');
  if (['blocked', 'deferred', 'not_planned'].includes(progress.status) && !progress.notes)
    fail('this status requires an explanation in notes');
}

export function preserveProgress(fields, previousFields = {}) {
  // Reject losing decisions for removed/renamed fields. Resolve the old record
  // explicitly; do not create aliases or retain obsolete inventory entries.
  for (const [id, field] of Object.entries(previousFields)) {
    if (field.progress === undefined) continue;
    validateProgress(field.progress, id);
    const p = field.progress;
    if (
      !Object.hasOwn(fields, id) &&
      (p.status !== 'unreviewed' || p.target || p.contract || p.tests.length || p.notes)
    ) {
      throw Error(
        `Tracked field removed from inventory: ${id}. Resolve its progress record before regenerating.`,
      );
    }
  }
  for (const [id, field] of Object.entries(fields)) {
    const progress = previousFields[id]?.progress ?? initialProgress();
    validateProgress(progress, id);
    field.progress = structuredClone(progress);
  }
}

export function summarizeProgress(fields, status) {
  if (status !== undefined && !Object.hasOwn(tracking.statuses, status))
    throw Error(`Unknown status: ${status}`);
  const statuses = Object.fromEntries(Object.keys(tracking.statuses).map((s) => [s, 0]));
  const targets = { unspecified: 0, core: 0, with_plugins: 0 };
  const selected = [];
  for (const [id, field] of Object.entries(fields)) {
    const p = field.progress;
    validateProgress(p, id);
    statuses[p.status]++;
    targets[p.target ?? 'unspecified']++;
    if (status !== undefined && p.status === status) selected.push({ id, ...p });
  }
  return {
    total_fields: Object.keys(fields).length,
    statuses,
    targets,
    ...(status === undefined ? {} : { filter: status, fields: selected }),
  };
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  try {
    const { values } = parseArgs({
      options: { status: { type: 'string' } },
      allowPositionals: false,
    });
    const require = createRequire(new URL('../../docs/package.json', import.meta.url));
    const file = fileURLToPath(new URL('../../COVERAGE.yaml', import.meta.url));
    const report = require('yaml').parse(fs.readFileSync(file, 'utf8'));
    console.log(JSON.stringify(summarizeProgress(report.fields, values.status), null, 2));
  } catch (error) {
    console.error(error.message);
    process.exitCode = 1;
  }
}
