import { test } from 'node:test';
import assert from 'node:assert/strict';
import { createSession } from '../index.js';

async function pageWith(html) {
  const session = await createSession();
  const page = await session.newPage();
  await page.loadHtml(html, 'https://example.test/');
  await page.evaluate(`
    globalThis.log = [];
    for (const name of ['pointermove', 'pointerdown', 'pointerup', 'click', 'wheel'])
      document.addEventListener(name, (event) => log.push([
        event.type, event.target.id || event.target.localName,
        event.clientX, event.clientY, event.buttons, event.isTrusted,
      ]), true);
  `);
  return { session, page };
}

const centre = (page, selector) =>
  page.evaluate(
    `(() => { const box = document.querySelector(${JSON.stringify(selector)}).getBoundingClientRect();
       return [Math.round(box.x + box.width / 2), Math.round(box.y + box.height / 2)]; })()`,
  );

test('mouse input hits the element under the point with real coordinates', async () => {
  const { session, page } = await pageWith(
    '<div id="a" style="margin:20px;width:200px;height:100px">target</div><div id="b" style="height:80px"></div>',
  );
  try {
    const [x, y] = await centre(page, '#a');
    assert.deepEqual(await page.evaluate(`[document.elementFromPoint(${x}, ${y}).id]`), ['a']);

    await page.mouse.move(x - 60, y - 30, { steps: 3 });
    await page.mouse.click(x, y);
    assert.deepEqual(page.mouse.position, [x, y]);

    const log = await page.evaluate('log');
    assert.ok(log.some(([type]) => type === 'pointermove'));
    assert.deepEqual(log.at(-1), ['click', 'a', x, y, 0, true]);
    const down = log.find(([type]) => type === 'pointerdown');
    assert.deepEqual(down, ['pointerdown', 'a', x, y, 1, true]);
    assert.ok(log.every((entry) => entry[5] === true));
  } finally {
    await session.close();
  }
});

test('press, release and wheel are separate commands', async () => {
  const { session, page } = await pageWith('<button id="hold">hold</button>');
  try {
    const [x, y] = await centre(page, '#hold');
    await page.mouse.move(x, y);
    await page.evaluate('log.length = 0');
    await page.mouse.down();
    await page.evaluate('globalThis.held = true');
    await page.mouse.up();
    assert.deepEqual(await page.evaluate('log.map((entry) => entry[0])'), [
      'pointerdown',
      'pointerup',
      'click',
    ]);

    await page.evaluate('log.length = 0');
    await page.mouse.wheel(0, 120);
    assert.deepEqual(await page.evaluate('log.map((entry) => entry[0])'), ['wheel']);
  } finally {
    await session.close();
  }
});
