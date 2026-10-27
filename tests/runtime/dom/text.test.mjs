import test from 'node:test';
import assert from 'node:assert/strict';
import {createSession} from '../../../scripts/coverage/session.mjs';

async function evaluate(source) {
  const session = await createSession();
  try { return await (await session.newPage()).evaluate(source); }
  finally { await session.close(); }
}

test('Text construction and CharacterData interfaces expose ordinary DOM behavior', async () => {
  assert.deepEqual(await evaluate(`(() => {
    const errors = [];
    for (const operation of [() => new CharacterData(), () => CharacterData(), () => Text(), () => new Text(Symbol())]) {
      try { operation(); errors.push('none'); } catch (error) { errors.push(error.name); }
    }
    const node = new Text('hello');
    return [new Text().data, new Text(null).data, node.data, node.nodeType,
      node.ownerDocument === document, node.parentNode,
      Object.getPrototypeOf(node) === Text.prototype,
      Object.getPrototypeOf(Text.prototype) === CharacterData.prototype,
      Object.getPrototypeOf(CharacterData.prototype) === Node.prototype,
      Text.prototype.constructor === Text, CharacterData.prototype.constructor === CharacterData,
      Text.prototype[Symbol.toStringTag], CharacterData.prototype[Symbol.toStringTag], errors];
  })()`), ['', 'null', 'hello', 3, true, null, true, true, true, true, true, 'Text', 'CharacterData', Array(4).fill('TypeError')]);
});

test('wholeText combines adjacent text nodes and follows edits without crossing elements or comments', async () => {
  assert.deepEqual(await evaluate(`(() => {
    const parent = document.createElement('p');
    const a = new Text('one'), b = new Text(''), c = new Text('two'), d = new Text('three');
    parent.append(a, b, c, document.createComment('stop'), d, document.createElement('span'), new Text('four'));
    const before = [a.wholeText, b.wholeText, c.wholeText, d.wholeText];
    c.data = '!'; b.remove();
    const after = a.wholeText;
    a.remove();
    const descriptor = Object.getOwnPropertyDescriptor(Text.prototype, 'wholeText');
    return [before, after, a.wholeText, c.wholeText, descriptor.enumerable, descriptor.set === undefined];
  })()`), [['onetwo', 'onetwo', 'onetwo', 'three'], 'one!', 'one', '!', true, true]);
});

test('splitText preserves UTF-16 contents, sibling insertion, live ranges and mutation records', async () => {
  assert.deepEqual(await evaluate(`(() => {
    const parent = document.createElement('p'), node = new Text('A😀B');
    parent.append(node, document.createElement('br'));
    const range = document.createRange(); range.setStart(node, 3); range.setEnd(node, 4);
    const boundary = document.createRange(); boundary.setStart(parent, 1); boundary.collapse(true);
    const observer = new MutationObserver(() => {});
    observer.observe(parent, {subtree: true, childList: true, characterData: true, characterDataOldValue: true});
    const right = node.splitText(3);
    const records = observer.takeRecords().map(r => [r.type, r.oldValue, r.addedNodes.length]);
    observer.disconnect();
    const attached = [node.data, right.data, node.nextSibling === right, right.nextSibling.tagName,
      right.ownerDocument === document, range.startContainer === node, range.startOffset,
      range.endContainer === right, range.endOffset, boundary.startOffset, records];
    const detached = new Text('abcd');
    const detachedRange = document.createRange(); detachedRange.setStart(detached, 3); detachedRange.collapse(true);
    const tail = detached.splitText(2);
    const errors = [];
    for (const operation of [() => tail.splitText(), () => tail.splitText(3), () => tail.splitText(-1)]) {
      try { operation(); errors.push('none'); } catch (error) { errors.push(error.name); }
    }
    const empty = tail.splitText(tail.length);
    return [attached, [detached.data, tail.data, tail.parentNode, detachedRange.startContainer === detached,
      detachedRange.startOffset, empty.data], errors];
  })()`), [
    ['A😀', 'B', true, 'BR', true, true, 3, true, 1, 2,
      [['characterData', 'A😀B', 0], ['childList', null, 1]]],
    ['ab', 'cd', null, true, 2, ''], ['TypeError', 'IndexSizeError', 'IndexSizeError'],
  ]);
});
