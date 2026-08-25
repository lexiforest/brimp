import assert from 'node:assert/strict';
import http from 'node:http';
import puppeteer from 'puppeteer-core';

const browserURL = process.env.BRIMP_CDP_URL;
if (!browserURL) throw new Error('BRIMP_CDP_URL is required');

const fixture = http.createServer((_request, response) => {
  const body = '<!doctype html><title>Puppeteer CDP</title><main>Hello Puppeteer</main>';
  response.writeHead(200, {'content-type': 'text/html', 'content-length': Buffer.byteLength(body)});
  response.end(body);
});
await new Promise(resolve => fixture.listen(0, '127.0.0.1', resolve));

const browser = await puppeteer.connect({
  browserURL,
  defaultViewport: null,
  networkEnabled: false,
  issuesEnabled: false,
});
try {
  const connection = browser._connection;
  const {targetId} = await connection.send('Target.createTarget', {url: 'about:blank'});
  const session = await connection.createSession({
    targetId,
    type: 'page',
    title: '',
    url: 'about:blank',
    attached: false,
    canAccessOpener: false,
  });
  await session.send('Page.enable');
  await session.send('Page.setLifecycleEventsEnabled', {enabled: true});
  await session.send('Runtime.enable');
  await session.send('Page.navigate', {url: `http://127.0.0.1:${fixture.address().port}/`});
  const evaluated = await session.send('Runtime.evaluate', {
    expression: '({title: document.title, answer: 6 * 7})',
    returnByValue: true,
  });
  assert.deepEqual(evaluated.result.value, {title: 'Puppeteer CDP', answer: 42});
  const screenshot = await session.send('Page.captureScreenshot', {format: 'png'});
  assert.ok(Buffer.from(screenshot.data, 'base64').subarray(0, 8).equals(Buffer.from('\x89PNG\r\n\x1a\n', 'binary')));
  await connection.send('Target.closeTarget', {targetId});
  process.stdout.write(JSON.stringify({title: evaluated.result.value.title, answer: evaluated.result.value.answer, png: true}) + '\n');
} finally {
  browser.disconnect();
  await new Promise(resolve => fixture.close(resolve));
}
