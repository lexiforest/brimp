import assert from 'node:assert/strict';
import http from 'node:http';
import puppeteer from 'puppeteer-core';

const browserURL = process.env.BRIMP_CDP_URL;
if (!browserURL) throw new Error('BRIMP_CDP_URL is required');

const fixture = http.createServer((_request, response) => {
  const body = '<!doctype html><title>Puppeteer CDP</title><main>Hello Puppeteer</main><input id="name"><div style="height:1200px"></div>';
  response.writeHead(200, {'content-type': 'text/html', 'content-length': Buffer.byteLength(body)});
  response.end(body);
});
await new Promise(resolve => fixture.listen(0, '127.0.0.1', resolve));

const browser = await puppeteer.connect({
  browserURL,
  issuesEnabled: false,
});
try {
  const page = await browser.newPage();
  await page.setViewport({width: 640, height: 480, deviceScaleFactor: 1});
  await page.goto(`http://127.0.0.1:${fixture.address().port}/`, {waitUntil: 'load'});
  const evaluated = await page.evaluate(() => ({title: document.title, answer: 6 * 7}));
  assert.deepEqual(evaluated, {title: 'Puppeteer CDP', answer: 42});
  const main = await page.$('main');
  assert.ok(main);
  assert.equal(await page.evaluate(element => element.textContent, main), 'Hello Puppeteer');
  await page.evaluate(() => document.querySelector('main').addEventListener('click', () => globalThis.mainClicked = true));
  await main.click();
  assert.equal(await page.evaluate(() => globalThis.mainClicked), true);
  await main.dispose();
  const input = await page.$('#name');
  await input.type('Brimp');
  assert.equal(await page.evaluate(element => element.value, input), 'Brimp');
  await input.dispose();
  const screenshot = await page.screenshot({encoding: 'binary', fullPage: true});
  assert.ok(Buffer.from(screenshot).subarray(0, 8).equals(Buffer.from('\x89PNG\r\n\x1a\n', 'binary')));
  assert.ok(Buffer.from(screenshot).readUInt32BE(20) > 480);
  await page.close();
  process.stdout.write(JSON.stringify({title: evaluated.title, answer: evaluated.answer, png: true}) + '\n');
} finally {
  browser.disconnect();
  await new Promise(resolve => fixture.close(resolve));
}
