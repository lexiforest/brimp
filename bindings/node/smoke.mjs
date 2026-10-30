import assert from 'node:assert/strict';
import { createSession, canvasPlugin, extractionPlugin } from '@brimp/brimp';

const session = await createSession({ plugins: [canvasPlugin(), extractionPlugin()] });
try {
  const page = await session.newPage();
  assert.equal(await page.evaluate('6 * 7'), 42);
  assert.equal(await page.evaluate("localStorage.x='installed'; localStorage.x"), 'installed');
  assert.deepEqual(
    await page.evaluate(
      "Array.from(document.createElement('canvas').getContext('2d').getImageData(0,0,1,1).data)",
    ),
    [0, 0, 0, 0],
  );
  await page.evaluate(
    "document.body.innerHTML='<article><h1>Package check</h1><p>Extracted from the installed package.</p></article>'; null",
  );
  const extracted = await page.extract({ contentSelector: 'article' });
  assert.match(extracted.content, /installed package/);
  console.log('ok');
} finally {
  await session.close();
}
