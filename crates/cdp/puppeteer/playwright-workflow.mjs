import assert from 'node:assert/strict';
import http from 'node:http';
import {chromium} from 'playwright-core';

const browserURL = process.env.BRIMP_CDP_URL;
if (!browserURL) throw new Error('BRIMP_CDP_URL is required');

const fixture = http.createServer((_request, response) => {
  const body = '<!doctype html><title>Playwright CDP</title><main>Hello Playwright</main>';
  response.writeHead(200, {'content-type': 'text/html', 'content-length': Buffer.byteLength(body)});
  response.end(body);
});
await new Promise(resolve => fixture.listen(0, '127.0.0.1', resolve));

const browser = await chromium.connectOverCDP(browserURL);
let context;
try {
  context = browser.contexts()[0] ?? await browser.newContext();
  const page = await context.newPage();
  await page.setViewportSize({width: 640, height: 480});
  await page.goto(`http://127.0.0.1:${fixture.address().port}/`, {waitUntil: 'load'});
  const evaluated = await page.evaluate(() => ({title: document.title, answer: 6 * 7}));
  assert.deepEqual(evaluated, {title: 'Playwright CDP', answer: 42});
  const screenshot = await page.screenshot();
  assert.ok(screenshot.subarray(0, 8).equals(Buffer.from('\x89PNG\r\n\x1a\n', 'binary')));
  await page.close();
  process.stdout.write(JSON.stringify({client: 'playwright', title: evaluated.title, answer: evaluated.answer, png: true}) + '\n');
} finally {
  await context?.close();
  await browser.close();
  await new Promise(resolve => fixture.close(resolve));
}
