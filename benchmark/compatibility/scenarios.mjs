import assert from 'node:assert/strict';
import {setTimeout as delay} from 'node:timers/promises';

export async function until(page, source, timeout = 8000) {
  const deadline = Date.now() + timeout;
  do {
    if (await page.evaluate(source)) return;
    await delay(40);
  } while (Date.now() < deadline);
  const error = new Error(`Timed out waiting for ${source}`);
  error.name = 'TimeoutError';
  throw error;
}

export async function runScenario(workload, page, origin, checkpoint) {
  await page.goto(origin + '/');
  if (workload.id.startsWith('todomvc-')) {
    const rows = 'Array.from(document.querySelectorAll(".todo-list li label"), node => node.textContent)';
    await until(page, '!!document.querySelector(".new-todo")');
    for (const title of ['first observation', 'second observation']) {
      await page.fill('.new-todo', title);
      await page.press('.new-todo', 'Enter');
    }
    await until(page, 'document.querySelectorAll(".todo-list li").length === 2');
    assert.deepEqual(await page.evaluate(rows), ['first observation', 'second observation']);
    checkpoint('add');
    // Both engines dispatch the same DOM dblclick. Layout is not measured here.
    await page.evaluate('document.querySelector(".todo-list li label").dispatchEvent(new MouseEvent("dblclick", {bubbles:true})); null');
    await until(page, '!!document.querySelector(".todo-list li.editing .edit")');
    await page.fill('.todo-list li.editing .edit', 'edited observation');
    await page.press('.todo-list li.editing .edit', 'Enter');
    await until(page, 'document.querySelector(".todo-list li label").textContent === "edited observation"');
    checkpoint('edit');
    await page.click('.todo-list li .toggle');
    await until(page, 'document.querySelectorAll(".todo-list li.completed").length === 1');
    await page.click('.filters a[href="#/active"]');
    await until(page, 'document.querySelectorAll(".todo-list li").length === 1');
    assert.deepEqual(await page.evaluate(rows), ['second observation']);
    checkpoint('complete-and-filter');
    await page.click('.filters a[href="#/"]');
    await until(page, 'document.querySelectorAll(".todo-list li").length === 2');
    // The destroy button is hover-only visually; use the same DOM click in both.
    await page.evaluate('document.querySelector(".todo-list li.completed .destroy").click(); null');
    await until(page, 'document.querySelectorAll(".todo-list li").length === 1');
    assert.deepEqual(await page.evaluate(rows), ['second observation']);
    checkpoint('delete');
    await page.goto(origin + '/');
    await until(page, '!!document.querySelector(".new-todo")');
    // These pinned upstream apps use in-memory state (useReducer/ref([])).
    // Their actual reload contract is reset, not localStorage persistence.
    assert.deepEqual(await page.evaluate(rows), [], 'upstream app must reset on reload');
    checkpoint('reload-reset');
    return {remainingBeforeReload: ['second observation'], remainingAfterReload: []};
  }
  if (workload.id === 'htmx') {
    await page.click('[hx-get]');
    await until(page, '!!document.querySelector("form input[name=firstName]")');
    checkpoint('load-form');
    await page.fill('input[name=firstName]', 'Ada');
    await page.fill('input[name=lastName]', 'Lovelace');
    await page.fill('input[name=email]', 'ada@example.test');
    await page.click('form button');
    await until(page, '!!document.querySelector("#saved-contact")');
    const text = await page.evaluate('document.querySelector("#saved-contact").textContent');
    assert.equal(text, 'Ada Lovelace ada@example.test');
    checkpoint('submit-and-swap');
    return {text};
  }
  if (workload.id === 'modernizr') {
    await until(page, 'typeof Modernizr === "object" && (typeof Modernizr.indexeddb === "boolean" || Modernizr.indexeddb instanceof Boolean)');
    const results = await page.evaluate(`(() => {
      const keys = ['cookies','localstorage','sessionstorage','indexeddb','webworkers','serviceworker','websockets','fetch','promises','es6collections','mutationobserver','canvas','webgl'];
      return Object.fromEntries(keys.map(key => {
        const value = Modernizr[key];
        return [key, typeof value === 'boolean' || value instanceof Boolean ? value.valueOf() : null];
      }));
    })()`);
    assert.ok(Object.values(results).every(value => typeof value === 'boolean'), 'every selected detector must return a boolean');
    return {features: results};
  }
  if (workload.id === 'creepjs') {
    await until(page, '!!window.Fingerprint && !!window.Creep && Object.keys(window.Fingerprint).length > 0 && Object.keys(window.Creep).length > 0', 30000);
    return await page.evaluate('({fingerprint: window.Fingerprint, creep: window.Creep})');
  }
  await until(page, 'typeof runWorkload === "function"');
  const result = await page.evaluate('runWorkload()');
  checkpoint('library-assertions');
  if (workload.id === 'dexie') {
    await page.goto(origin + '/');
    await until(page, 'typeof readPersistedWorkload === "function"');
    result.persisted = await page.evaluate('readPersistedWorkload()');
    checkpoint('reload-persistence');
  }
  return result;
}
