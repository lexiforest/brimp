import { chromium } from 'playwright-core';
const endpoint = process.argv[2];
if (!endpoint) throw new Error('usage: node tests/playwright_smoke_test.mjs http://127.0.0.1:9222');
const browser = await chromium.connectOverCDP(endpoint);
try {
  const context = await browser.newContext();
  const page = await context.newPage();
  await page.goto('data:text/html,<title>Worker</title><h1>Hello</h1>');
  if (await page.title() !== 'Worker') throw new Error('wrong page title');
  if ((await page.screenshot()).length < 100) throw new Error('empty screenshot');
  await context.close();
  console.log('PASS: Playwright discovery, navigation, evaluation, and screenshot');
} finally {
  await browser.close();
}
