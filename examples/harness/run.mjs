import assert from 'node:assert/strict';
import {readFile} from 'node:fs/promises';
import {createSession} from '../../bindings/node/index.js';

const source = await readFile(new URL('./page.js', import.meta.url), 'utf8');
const session = await createSession({
  clock: {mode: 'logical', epochMs: 1000},
  diagnostics: {capacity: 4096},
});
try {
  const page = await session.newPage();
  await page.route('*', route => {
    if (route.request.url === 'https://fixture.invalid/config') {
      route.fulfill({headers: {'content-type': 'application/json', 'access-control-allow-origin': '*'}, body: '{"prefix":"Hello "}'});
    } else route.fail('No fixture for this request');
  });
  const fn = await page.evaluateHandle(`${source}\n;compute`);
  try {
    await page.click('#run');
    // Begin a new diagnostic window for calls, excluding bundle initialization.
    await page.takeDiagnostics();
    const results = [];
    for (const input of ['Ada', 'Grace']) {
      const result = await fn.call('(fn, input) => fn(input)', input);
      try { results.push(await result.jsonValue()); }
      finally { await result.dispose(); }
    }
    assert.deepEqual(results, [
      {text: 'Hello Ada', calls: 1, clicks: 1, now: 1005},
      {text: 'Hello Grace', calls: 2, clicks: 1, now: 1010},
    ]);
    const snapshot = await page.peekDiagnostics();
    const report = await page.summarizeDiagnostics();
    assert.equal(report.retained, snapshot.records.length);
    assert.deepEqual(await page.takeDiagnostics(), snapshot);
    console.log(JSON.stringify({results, report}, null, 2));
  } finally { await fn.dispose(); }
} finally { await session.close(); }
