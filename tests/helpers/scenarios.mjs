import assert from 'node:assert/strict';
import { setTimeout as delay } from 'node:timers/promises';

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
    const rows =
      'Array.from(document.querySelectorAll(".todo-list li label"), node => node.textContent)';
    await until(page, '!!document.querySelector(".new-todo")');
    for (const title of ['first observation', 'second observation']) {
      await page.fill('.new-todo', title);
      await page.press('.new-todo', 'Enter');
    }
    await until(page, 'document.querySelectorAll(".todo-list li").length === 2');
    assert.deepEqual(await page.evaluate(rows), ['first observation', 'second observation']);
    checkpoint('add');
    // Dispatch the DOM dblclick used by the reference run; layout is not measured.
    await page.evaluate(
      'document.querySelector(".todo-list li label").dispatchEvent(new MouseEvent("dblclick", {bubbles:true})); null',
    );
    await until(page, '!!document.querySelector(".todo-list li.editing .edit")');
    await page.fill('.todo-list li.editing .edit', 'edited observation');
    await page.press('.todo-list li.editing .edit', 'Enter');
    await until(
      page,
      'document.querySelector(".todo-list li label").textContent === "edited observation"',
    );
    checkpoint('edit');
    await page.click('.todo-list li .toggle');
    await until(page, 'document.querySelectorAll(".todo-list li.completed").length === 1');
    await page.click('.filters a[href="#/active"]');
    await until(page, 'document.querySelectorAll(".todo-list li").length === 1');
    assert.deepEqual(await page.evaluate(rows), ['second observation']);
    checkpoint('complete-and-filter');
    await page.click('.filters a[href="#/"]');
    await until(page, 'document.querySelectorAll(".todo-list li").length === 2');
    // The destroy button is hover-only visually; use the reference's DOM click.
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
    return { remainingBeforeReload: ['second observation'], remainingAfterReload: [] };
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
    return { text };
  }
  await until(page, 'typeof runWorkload === "function"');
  const result = await page.evaluate('runWorkload()');
  checkpoint('library-assertions');
  return result;
}
