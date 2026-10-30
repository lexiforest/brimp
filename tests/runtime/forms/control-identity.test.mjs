import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import { createSession } from '../../../scripts/coverage/session.mjs';
import {
  controlIdentityCases as cases,
  measureControlIdentity as measure,
} from '../../../scripts/coverage/control-identity-cases.mjs';
test('input/button/anchor prototype identity and registered/unregistered construction match Safari', async () => {
  const reference = JSON.parse(
    fs.readFileSync(new URL('../fixtures/control-identity.safari.json', import.meta.url)),
  );
  assert.deepEqual(
    reference.cases.map((c) => [c.tag, c.name]),
    cases,
  );
  const s = await createSession();
  try {
    const p = await s.newPage(),
      actual = JSON.parse(
        await p.evaluate(`JSON.stringify((${measure})(${JSON.stringify(cases)}))`),
      );
    assert.deepEqual(
      actual,
      reference.cases.map((x) => x.expected),
    );
  } finally {
    await s.close();
  }
});
