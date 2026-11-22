import { test } from 'node:test';
import assert from 'node:assert/strict';
import { createServer } from 'node:http';
import { once } from 'node:events';
import { createSession, Timeout, Closed } from '../index.js';

test('locators resolve replacements, enforce strictness, and wait for actionability', async () => {
  const session = await createSession({ plugins: [] });
  try {
    const page = await session.newPage();
    await page.loadHtml('<button id="go" disabled style="width:100px;height:40px">old</button>');
    const button = page.locator('#go');
    await page.evaluate(`setTimeout(() => {
      const replacement = document.createElement('button'); replacement.id = 'go'; replacement.style.cssText = 'width:100px;height:40px'; replacement.textContent = 'new'; document.querySelector('#go').replaceWith(replacement);
      document.querySelector('#go').onclick = () => window.clicked = true;
    }, 30)`);
    await button.click({ timeout: 2000 });
    assert.equal(await page.evaluate('clicked'), true);
    assert.equal(await button.textContent(), 'new');
    await page.evaluate(`document.body.insertAdjacentHTML('beforeend', '<button>second</button>')`);
    await assert.rejects(page.locator('button').click(), /Strict locator/);
    assert.equal(await page.locator('button').nth(1).textContent(), 'second');
    assert.equal(await page.locator('button').count(), 2);
  } finally {
    await session.close();
  }
});

test('frame input, function waits, timeouts and cancellation leave page usable', async () => {
  const session = await createSession({ plugins: [], timeout: 0 });
  try {
    const page = await session.newPage();
    await page.loadHtml(
      '<iframe id="frame" srcdoc="&lt;input id=field style=width:100px;height:30px&gt;"></iframe>',
    );
    const frame = page.frameLocator('#frame');
    assert.equal((await page.frames()).length, 1);
    await frame.locator('#field').fill('hello', { timeout: 2000 });
    assert.equal(await frame.evaluate(`document.querySelector('#field').value`), 'hello');
    assert.deepEqual(await frame.evaluate('({value: 42, nested: [true]})'), {
      value: 42,
      nested: [true],
    });
    await page.evaluate('setTimeout(() => window.answer = 42, 20)');
    assert.equal(await page.waitForFunction('() => window.answer', { timeout: 1000 }), 42);
    await page.waitForSelector('#missing', { state: 'detached' });
    await assert.rejects(page.waitForSelector('#missing', { timeout: 50 }), Timeout);
    const abort = new AbortController();
    const pending = assert.rejects(
      page.waitForFunction('false', { signal: abort.signal }),
      /cancelled/,
    );
    setTimeout(() => abort.abort(), 30);
    await pending;
    assert.equal(await page.evaluate('2 + 2'), 4);
    await assert.rejects(page.waitForFunction("Promise.reject('')"), /Error/);
    const event = assert.rejects(page.waitForResponse('http://unused.test/'), Closed);
    await page.close();
    await event;
  } finally {
    await session.close();
  }
});

test('navigation, response events, script navigation and independent popups', async () => {
  const server = createServer((request, response) => {
    response.setHeader('Content-Type', 'text/html');
    response.end('<title>destination</title><button>Go</button>');
  }).listen(0, '127.0.0.1');
  await once(server, 'listening');
  const url = `http://127.0.0.1:${server.address().port}`;
  const session = await createSession({ plugins: [] });
  try {
    const page = await session.newPage();
    const response = page.waitForResponse(url + '/');
    const navigation = page.waitForNavigation();
    await page.get(url + '/', { waitUntil: 'domcontentloaded' });
    assert.equal((await response).response.status, 200);
    await navigation;
    await page.waitForLoadState();
    await page.evaluate("void fetch('/background')");
    await page.waitForNetworkIdle({ quiet: 20, timeout: 2000 });
    const opened = page.waitForPopup({ timeout: 5000 });
    await page.evaluate(`void (window.child = window.open(${JSON.stringify(url + '/popup')}))`);
    const popup = await opened;
    assert.ok(session.pages.includes(popup));
    assert.equal(await popup.currentUrl(), url + '/popup');
    assert.equal(await popup.evaluate('document.title'), 'destination');
    await popup.close();
    assert.equal(await page.evaluate('child.closed'), true);
    const next = page.waitForNavigation({ timeout: 5000 });
    await page.evaluate(`location.href = ${JSON.stringify(url + '/next')}`);
    await next;
    assert.equal(await page.currentUrl(), url + '/next');
    const secondOpened = page.waitForPopup({ timeout: 5000 });
    await page.evaluate(`void window.open(${JSON.stringify(url + '/second')})`);
    const second = await secondOpened;
    await page.close();
    assert.equal(await second.evaluate('document.title'), 'destination');
    assert.ok(!session.pages.includes(page));
    const thirdOpened = second.waitForPopup({ timeout: 5000 });
    await second.evaluate("void (window.child = window.open('about:blank'))");
    const third = await thirdOpened;
    const pending = assert.rejects(third.waitForResponse(url + '/unused'), Closed);
    await second.evaluate('child.close()');
    await pending;
  } finally {
    await session.close();
    server.closeAllConnections();
    await new Promise((resolve) => server.close(resolve));
  }
});
