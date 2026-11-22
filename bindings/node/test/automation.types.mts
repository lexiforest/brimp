import {type Session, type Page, type Frame, type Locator, type Mouse} from '../index.js';

async function automationContract(session: Session, page: Page, signal: AbortSignal) {
  const pages: Page[] = session.pages;
  const frames: Frame[] = await page.frames();
  const field: Locator = page.frameLocator('#checkout').locator('input').nth(0);
  await field.fill('value', {timeout: 1000, signal});
  await field.waitFor({state: 'visible', signal});
  const mouse: Mouse = page.mouse;
  await mouse.click(10, 20, {delay: 10});
  await page.evaluate('42', {timeout: 1000, signal});
  await page.waitForFunction('value => window.ready === value', {arg: true, signal});
  await page.waitForNetworkIdle({quiet: 100, timeout: 1000, signal});
  await page.waitForEvent('response', {predicate: record => record.response.status === 200, signal});
  await Promise.all([page.waitForNavigation({waitUntil: 'load'}), page.get('http://example.test/', {waitUntil: 'domcontentloaded'})]);
  const popup: Page = await page.waitForPopup({signal});
  return {pages, frames, popup};
}
void automationContract;
