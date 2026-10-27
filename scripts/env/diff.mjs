#!/usr/bin/env node
// Compare supplied captured values with observations from the same interface recipes in Brimp.
// Usage: node scripts/env/diff.mjs --profile browser-profile.json [--json]
import {readFile} from 'node:fs/promises';
import {createSession} from '../../bindings/node/index.js';

const args = process.argv.slice(2);
const index = args.indexOf('--profile');
if (index < 0 || !args[index + 1]) throw new Error('Supply --profile browser-profile.json (created by brimp profile capture)');
const profile = JSON.parse(await readFile(args[index + 1], 'utf8'));
const [registry, capture] = await Promise.all(['profile_registry', 'profile_capture'].map(name =>
  readFile(new URL(`../../crates/runtime/js/internal/${name}.js`, import.meta.url), 'utf8')));
const session = await createSession({profile});
try {
  const page = await session.newPage();
  const contract = await page.profileContract();
  const probes = contract.interfaces.filter(i => i.probe).map(i =>
    `probes[${JSON.stringify(i.scope)}] = (${i.probe});`).join('\n');
  const observed = await page.evaluate(`(async () => {
    ${registry}\n${capture}
    const probes = Object.create(null);
    ${probes}
    return captureBrowserProfile(${JSON.stringify(contract)}, probes, createProfileRegistry);
  })()`);
  const mismatches = [];
  const unmeasured = [];
  function compare(path, expected, actual) {
    if (typeof expected === 'number' && typeof actual === 'number' &&
        Math.abs(expected - actual) <= 1e-9 * Math.max(1, Math.abs(expected))) return;
    if (expected && typeof expected === 'object' && !Array.isArray(expected)) {
      for (const [key, value] of Object.entries(expected)) compare(`${path}.${key}`, value, actual?.[key]);
    } else if (JSON.stringify(expected) !== JSON.stringify(actual)) {
      mismatches.push({path, expected, actual: actual ?? null});
    }
  }
  for (const [scope, fields] of Object.entries(profile.values ?? {})) {
    for (const [key, expected] of Object.entries(fields)) {
      const path = `${scope}.${key}`;
      const result = observed.capture.results[path];
      if (result?.status !== 'captured') unmeasured.push({path, ...result});
      else compare(path, expected, observed.values[scope]?.[key]);
    }
  }
  const report = {mismatches, unmeasured, defaultsUsed: (await page.profileReport()).defaultsUsed};
  if (args.includes('--json')) console.log(JSON.stringify(report, null, 2));
  else {
    console.log(`${mismatches.length} mismatches; ${unmeasured.length} values cannot be measured by their recipes.`);
    for (const item of mismatches) console.log(JSON.stringify(item));
    for (const item of unmeasured) console.log(`${item.path}: ${item.reason ?? item.status}`);
  }
  process.exitCode = mismatches.length ? 1 : 0;
} finally { await session.close(); }
