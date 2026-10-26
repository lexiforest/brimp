import {test} from 'node:test';
import assert from 'node:assert/strict';
import {readFile} from 'node:fs/promises';
import {resolve} from 'node:path';
import {fileURLToPath, pathToFileURL} from 'node:url';

const binding = resolve(process.env.BRIMP_NODE_BINDING ?? fileURLToPath(new URL('../../bindings/node/index.js', import.meta.url)));
const {createSession} = await import(pathToFileURL(binding).href);

for (const mode of ['api', 'recursive']) test(`DOMPurify runs unchanged through a retained function with ${mode} diagnostics`, {timeout: 30000}, async () => {
  const source = await readFile(new URL('../node_modules/dompurify/dist/purify.js', import.meta.url), 'utf8');
  const session = await createSession({clock: {mode: 'logical'}, diagnostics: {mode, include: ['Document.prototype', 'Element.prototype', 'Node.prototype'], capacity: 4096}});
  try {
    const page = await session.newPage();
    await page.configureNetwork({offline: true});
    const sanitize = await page.evaluateHandle(`${source}\n;DOMPurify.sanitize`);
    try {
      await page.takeDiagnostics();
      for (const [dirty, expected] of [
        ['<p>Hello <strong>world</strong><script>alert(1)</script></p>', '<p>Hello <strong>world</strong></p>'],
        ['<img src="x" onerror="alert(2)">', '<img src="x">'],
      ]) {
        const result = await sanitize.call('(sanitize, input) => sanitize(input)', dirty);
        try { assert.equal(await result.jsonValue(), expected); }
        finally { await result.dispose(); }
      }
      const snapshot = await page.peekDiagnostics();
      const summary = await page.summarizeDiagnostics();
      assert.equal(summary.dropped, 0);
      assert.ok(summary.dependencies.some(entry => entry.path.startsWith('Document.prototype.createNodeIterator')));
      assert.ok(summary.dependencies.some(entry => entry.path.startsWith('Node.prototype.')));
      assert.deepEqual(await page.takeDiagnostics(), snapshot);
    } finally { await sanitize.dispose(); }
  } finally { await session.close(); }
});
