// Diff Brimp's Window surface against the Safari reference.
//
//     node scripts/idl/check.mjs [--verbose]
//
// Fails on any difference a probe can see: which names exist, the order they are
// enumerated in, the kind behind each name, and the descriptor flags of every
// member on every interface object and prototype. This is the gate the IDL
// surface has to keep green; `scripts/idl/capture.mjs` writes the reference.
import fs from 'node:fs';
import path from 'node:path';
import { PROBE } from './probe.mjs';

const ROOT = path.join(import.meta.dirname, '..', '..');
const REFERENCE = path.join(ROOT, 'tests', 'reference', 'safari-surface.json');
const verbose = process.argv.includes('--verbose');
const FLAGS = ['get', 'set', 'enumerable', 'configurable', 'writable'];

function describe(tuple) {
  if (!Array.isArray(tuple)) return String(tuple);
  const parts = FLAGS.filter((_, index) => tuple[index] === 1);
  // writable is already in FLAGS
  if (tuple[5] >= 0) parts.push(`arity ${tuple[5]}`);
  return parts.join(' ') || 'empty';
}

async function capture() {
  const { createSession } = await import(path.join(ROOT, 'bindings', 'node', 'index.js'));
  const session = await createSession();
  try {
    const page = await session.newPage();
    return JSON.parse(await page.evaluate(PROBE));
  } finally {
    await session.close();
  }
}

function compare(reference, observed) {
  const findings = [];
  const add = (kind, subject, expected, actual) =>
    findings.push({ kind, subject, expected, actual });

  const expectedNames = reference.names;
  const actualNames = observed.names;
  for (const name of expectedNames) if (!actualNames.includes(name)) add('missing name', name);
  for (const name of actualNames) if (!expectedNames.includes(name)) add('extra name', name);

  const shared = expectedNames.filter((name) => actualNames.includes(name));
  const orderDifference = shared.filter(
    (name, index) => actualNames.indexOf(name) !== actualNames.indexOf(shared[index]),
  );
  if (orderDifference.length)
    add(
      'order',
      `${orderDifference.length} names`,
      shared.slice(0, 8).join(','),
      actualNames.slice(0, 8).join(','),
    );

  for (const name of shared) {
    const want = reference.entries[name];
    const got = observed.entries[name];
    if (want === undefined && got === undefined) continue;
    if (want === undefined) {
      add('unexpected object', name);
      continue;
    }
    if (got === undefined) {
      add('missing object', name, want.kind);
      continue;
    }
    if (want.kind !== got.kind) add('kind', name, want.kind, got.kind);
    if ((want.name ?? '') !== (got.name ?? '')) add('function name', name, want.name, got.name);
    if ((want.parent_ctor ?? '') !== (got.parent_ctor ?? ''))
      add('constructor chain', name, want.parent_ctor, got.parent_ctor);
    if ((want.parent_proto ?? '') !== (got.parent_proto ?? ''))
      add('prototype chain', name, want.parent_proto, got.parent_proto);
    for (const surface of ['own', 'proto']) {
      const expectedMembers = want[surface] ?? {};
      const actualMembers = got[surface] ?? {};
      for (const member of Object.keys(expectedMembers))
        if (!(member in actualMembers)) add(`${surface} missing`, `${name}.${member}`);
      for (const member of Object.keys(actualMembers))
        if (!(member in expectedMembers)) add(`${surface} extra`, `${name}.${member}`);
      for (const member of Object.keys(expectedMembers).filter((key) => key in actualMembers))
        if (JSON.stringify(expectedMembers[member]) !== JSON.stringify(actualMembers[member]))
          add(
            `${surface} descriptor`,
            `${name}.${member}`,
            describe(expectedMembers[member]),
            describe(actualMembers[member]),
          );
    }
  }
  return findings;
}

const reference = JSON.parse(fs.readFileSync(REFERENCE, 'utf8'));
const observed = (await capture()).surface;
const findings = compare(reference, observed);
const byKind = new Map();
for (const finding of findings) byKind.set(finding.kind, (byKind.get(finding.kind) ?? 0) + 1);
console.log(
  `${reference.names.length} reference names from Safari ${reference.safari_version}, ` +
    `${observed.names.length} in Brimp`,
);
if (findings.length === 0) {
  console.log('no difference');
  process.exit(0);
}
for (const [kind, count] of [...byKind].sort((a, b) => b[1] - a[1]))
  console.log(`  ${count.toString().padStart(4)}  ${kind}`);
if (verbose)
  for (const finding of findings) {
    if (finding.expected === undefined && finding.actual === undefined)
      console.log(`  ${finding.kind}: ${finding.subject}`);
    else
      console.log(
        `  ${finding.kind}: ${finding.subject} — ${finding.expected} | ${finding.actual}`,
      );
  }
process.exit(1);
