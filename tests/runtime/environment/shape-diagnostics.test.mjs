import test from 'node:test';
import assert from 'node:assert/strict';
import { createSession } from '../../../bindings/node/index.js';

test('diagnostics expose reflection shape metadata and leak warnings', async () => {
  const session = await createSession({ diagnostics: { capacity: 4096 } });
  try {
    const page = await session.newPage();
    await page.request('GET', 'data:text/html,');
    const manifest = await page.shapeManifest();
    assert.equal(manifest.format, 'brimp-shape');
    assert.ok(Object.hasOwn(manifest.claims, 'window.Navigator'));
    await page.evaluate(`(() => {
      Function.prototype.toString.call(Navigator);
      Object.getOwnPropertyDescriptor(Navigator.prototype, 'plugins');
      Object.keys(navigator);
      return 1;
    })()`);
    await page.evaluate(`(() => {
      const d = Object.getOwnPropertyDescriptor(Navigator.prototype, 'plugins');
      Object.defineProperty(Navigator.prototype, 'plugins', { ...d, enumerable: false });
      Object.getOwnPropertyDescriptor(Navigator.prototype, 'plugins');
      return 1;
    })()`);
    const report = await page.summarizeDiagnostics();
    assert.ok(Array.isArray(report.shapeDifferences));
    assert.ok(
      report.shapeDifferences.some(
        (difference) =>
          difference.kind === 'descriptorMismatch' &&
          difference.property === 'plugins',
      ),
    );
    assert.ok(report.coverage.limitations.some((text) => text.includes('reflection')));
    const batch = await page.takeDiagnostics();
    const toString = batch.records.find(
      (record) => record.path === 'Function.prototype.toString',
    );
    assert.equal(typeof toString.details.native, 'boolean');
    const descriptor = batch.records.find(
      (record) =>
        record.path === 'Object.getOwnPropertyDescriptor' &&
        record.details.property === 'plugins',
    );
    assert.equal(descriptor.details.target, 'Navigator.prototype');
    assert.equal(descriptor.details.descriptor.hasGet, true);
    assert.equal(descriptor.details.descriptor.enumerable, true);
    assert.ok(Array.isArray(batch.shapeDifferences));
  } finally {
    await session.close();
  }
});
