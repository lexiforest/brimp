import test from 'node:test';
import assert from 'node:assert/strict';
import { createSession } from '../../../scripts/coverage/session.mjs';

async function evaluate(source) {
  const session = await createSession();
  try {
    return await (await session.newPage()).evaluate(source);
  } finally {
    await session.close();
  }
}

test('NodeList has a non-Array interface with readonly length and indexed entries', async () => {
  assert.deepEqual(
    await evaluate(`(() => {
    const parent = document.createElement('div'); parent.innerHTML = '<i></i><b></b>';
    const list = parent.childNodes, first = list[0];
    let error; try { new NodeList(); } catch (e) { error = e.name; }
    const writes = [Reflect.set(list, '0', null), Reflect.set(list, 'length', 0), Reflect.deleteProperty(list, '0')];
    list.extra = 7;
    return [Array.isArray(list), Object.getPrototypeOf(NodeList.prototype) === Object.prototype,
      Object.getPrototypeOf(list) === NodeList.prototype, NodeList.prototype.constructor === NodeList,
      Object.prototype.toString.call(list), error, typeof list.push, writes,
      list[0] === first, list.length, list.extra, Object.keys(list),
      Object.getOwnPropertyDescriptor(NodeList.prototype, 'length').enumerable];
  })()`),
    [
      false,
      true,
      true,
      true,
      '[object NodeList]',
      'TypeError',
      'undefined',
      [false, false, false],
      true,
      2,
      7,
      ['0', '1', 'extra'],
      true,
    ],
  );
});

test('NodeList childNodes remains live while selector and mutation-record lists are snapshots', async () => {
  assert.deepEqual(
    await evaluate(`(() => {
    const parent = document.createElement('div'); parent.innerHTML = '<i></i><b></b>';
    const live = parent.childNodes, snapshot = parent.querySelectorAll('*'), first = live[0];
    const observer = new MutationObserver(() => {}); observer.observe(parent, {childList:true});
    const added = document.createElement('u'); parent.append(added);
    const recorded = observer.takeRecords()[0].addedNodes; observer.disconnect();
    first.remove(); added.remove();
    return [live === parent.childNodes, live.length, live[0].tagName, Object.keys(live),
      snapshot.length, snapshot[0] === first, snapshot[1].tagName,
      recorded.length, recorded[0] === added, recorded.item(1)];
  })()`),
    [true, 1, 'B', ['0'], 2, true, 'B', 1, true, null],
  );
});

test('NodeList item converts indexes and reports missing arguments and invalid receivers', async () => {
  assert.deepEqual(
    await evaluate(`(() => {
    const parent = document.createElement('div'); parent.innerHTML = '<i></i><b></b>';
    const list = parent.childNodes, errors = [];
    for (const operation of [() => list.item(), () => list.item(Symbol()), () => list.item(1n), () => NodeList.prototype.item.call({}, 0)]) {
      try { operation(); errors.push('none'); } catch (e) { errors.push(e.name); }
    }
    return [[0, '1', 1.9, -1, 99, 4294967296].map(index => list.item(index)?.tagName ?? null),
      list[99] === undefined, errors];
  })()`),
    [['I', 'B', 'B', null, null, 'I'], true, Array(4).fill('TypeError')],
  );
});

test('NodeList iteration uses generic Array iterators and observes live mutations', async () => {
  assert.deepEqual(
    await evaluate(`(() => {
    const parent = document.createElement('div'); parent.innerHTML = '<i></i><b></b>';
    const list = parent.childNodes, iterator = list.values();
    const first = iterator.next().value.tagName; parent.append(document.createElement('u'));
    const entries = Array.from(list.entries(), ([i, n]) => [i, n.tagName]);
    const callbacks = [], context = {};
    list.forEach(function(node, index, collection) { callbacks.push([node.tagName,index,collection===list,this===context]); }, context);
    const identity = ['entries', 'keys', 'values', 'forEach'].map(name => NodeList.prototype[name] === Array.prototype[name]);
    return [identity, NodeList.prototype[Symbol.iterator] === NodeList.prototype.values,
      Array.from(NodeList.prototype.values.call({0:'generic',length:1})),
      first, Array.from(iterator, n => n.tagName), Array.from(list.keys()), entries, callbacks];
  })()`),
    [
      Array(4).fill(true),
      true,
      ['generic'],
      'I',
      ['B', 'U'],
      [0, 1, 2],
      [
        [0, 'I'],
        [1, 'B'],
        [2, 'U'],
      ],
      [
        ['I', 0, true, true],
        ['B', 1, true, true],
        ['U', 2, true, true],
      ],
    ],
  );
});
