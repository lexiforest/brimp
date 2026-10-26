import test from 'node:test';
import assert from 'node:assert/strict';
import {createSession} from '../../../scripts/coverage/session.mjs';

test('DOM text preserves UTF-16 through creation, replacement, cloning and mutation records', async () => {
  const session = await createSession();
  try {
    const page = await session.newPage();
    assert.deepEqual(await page.evaluate(`(() => {
      const failures = [], parent = document.createElement('a');
      let value = '';
      for (let unit = 0; unit <= 0xffff; ++unit) value += String.fromCharCode(unit) + 'x';
      const text = document.createTextNode(value), comment = document.createComment(value);
      parent.append(text, comment);
      const foreign = document.implementation.createHTMLDocument();
      for (const node of [text, comment, parent, parent.cloneNode(true), foreign.importNode(parent, true)]) {
        if (node.textContent !== value) failures.push('creation or clone');
      }
      foreign.adoptNode(parent);
      if (parent.text !== value || parent.ownerDocument !== foreign) failures.push('adoption');
      const observer = new MutationObserver(() => {});
      observer.observe(parent, {subtree: true, characterData: true, characterDataOldValue: true, childList: true});
      const next = String.fromCharCode(0xdc00, 0, 0xd800);
      text.textContent = next;
      comment.data = next;
      const records = observer.takeRecords();
      if (records.length !== 2 || records.some(r => r.type !== 'characterData' || r.oldValue !== value)) failures.push('oldValue');
      if (text.data !== next || comment.nodeValue !== next) failures.push('replacement');
      parent.text = next;
      const replacement = observer.takeRecords();
      if (replacement.length !== 1 || replacement[0].removedNodes[0] !== text ||
          replacement[0].addedNodes[0] !== parent.firstChild) failures.push('child replacement');
      if (text.parentNode !== null || text.data !== next || parent.text !== next) failures.push('detached text');
      if (parent.innerHTML !== next || parent.cloneNode(true).text !== next) failures.push('serialization');
      parent.textContent = value;
      if (parent.text !== value) failures.push('element textContent');
      observer.disconnect();
      return failures;
    })()`), []);
  } finally { await session.close(); }
});
