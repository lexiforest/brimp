import test from 'node:test';
import assert from 'node:assert/strict';
import { createSession } from '../index.js';

test('identity overrides accept browser identities and reject unsafe header values', async () => {
  const session = await createSession();
  try {
    const page = await session.newPage();
    await page.setIdentity({ languages: ['fr-FR', 'fr'], platform: 'Win32' });
    assert.deepEqual(
      await page.evaluate('[navigator.language, navigator.languages, navigator.platform]'),
      ['fr-FR', ['fr-FR', 'fr'], 'Win32'],
    );
    await page.setIdentity({ userAgent: 'Mozilla/5.0 (X11) Firefox/1' });
    assert.equal(await page.evaluate('navigator.userAgent'), 'Mozilla/5.0 (X11) Firefox/1');
    await assert.rejects(page.setIdentity({ userAgent: 'bad\r\nheader' }));
  } finally {
    await session.close();
  }
});
