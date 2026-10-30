import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import { createSession } from '../../../scripts/coverage/session.mjs';
import {
  anchorURLFields,
  anchorURLReadCases,
  anchorURLWriteCases,
  measureAnchorURLs,
} from '../../../scripts/coverage/anchor-url-cases.mjs';

for (const [name, cases] of [
  ['reads', anchorURLReadCases],
  ['writes', anchorURLWriteCases],
]) {
  test(`anchor URL ${name} match ${cases.length} Safari cases`, async (t) => {
    const ref = JSON.parse(
      fs.readFileSync(new URL('../fixtures/anchor-urls.safari.json', import.meta.url)),
    )[name];
    assert.deepEqual(
      ref.map(({ expected, ...c }) => c),
      cases,
    );
    const session = await createSession();
    try {
      const differences = [];
      for (let i = 0; i < cases.length; i += 400) {
        const batch = cases.slice(i, i + 400);
        await t.test(`cases ${i + 1}-${i + batch.length}`, async () => {
          // Detached documents and their wrappers live until page teardown.
          // Independent cases must not accumulate all 4,568 documents in one realm.
          const page = await session.newPage();
          try {
            const actual = JSON.parse(
              await page.evaluate(
                `JSON.stringify((${measureAnchorURLs})(${JSON.stringify(batch)},${name === 'writes'}))`,
              ),
            );
            for (let j = 0; j < batch.length; ++j)
              if (JSON.stringify(actual[j]) !== JSON.stringify(ref[i + j].expected))
                differences.push({ ...ref[i + j], actual: actual[j] });
          } finally {
            await page.close();
          }
        });
      }
      assert.equal(
        differences.length,
        0,
        JSON.stringify({ total: differences.length, first: differences.slice(0, 12) }, null, 2),
      );
    } finally {
      await session.close();
    }
  });
}

async function evaluate(source) {
  const session = await createSession();
  try {
    return await (await session.newPage()).evaluate(source);
  } finally {
    await session.close();
  }
}

test('anchor URL descriptors, conversion and receiver checks match Safari', async () => {
  assert.deepEqual(
    await evaluate(`(() => {
    const failures = [], fields = ${JSON.stringify(anchorURLFields)};
    for (const name of fields) {
      const d = Object.getOwnPropertyDescriptor(HTMLAnchorElement.prototype, name);
      if (!d.enumerable || !d.configurable || d.get.name !== 'get ' + name || d.get.length !== 0) failures.push([name, 'getter descriptor']);
      if (name === 'origin') { if (d.set !== undefined) failures.push('readonly origin'); }
      else if (d.set.name !== 'set ' + name || d.set.length !== 1) failures.push([name, 'setter descriptor']);
      for (const receiver of [{}, document.createElement('area'), Object.create(HTMLAnchorElement.prototype)]) {
        try { d.get.call(receiver); failures.push([name, 'getter brand']); } catch (e) { if (e.name !== 'TypeError') failures.push(e.name); }
        if (d.set) {
          let converted = false;
          try { d.set.call(receiver, {toString() { converted = true; return 'x'; }}); failures.push([name, 'setter brand']); }
          catch (e) { if (e.name !== 'TypeError' || converted) failures.push([name, 'brand before conversion']); }
        }
      }
    }
    const d = Object.getOwnPropertyDescriptor(HTMLAnchorElement.prototype, 'toString');
    if (!d.enumerable || !d.configurable || !d.writable || d.value.name !== 'toString' || d.value.length !== 0) failures.push('stringifier descriptor');
    try { new d.value(); failures.push('constructible stringifier'); } catch (e) { if (e.name !== 'TypeError') failures.push(e.name); }
    try { d.value.call({href:'fake'}); failures.push('stringifier receiver'); } catch (e) { if (e.name !== 'TypeError') failures.push(e.name); }
    const a = document.createElement('a');
    a.href = 'https://initial.test/';
    a.hash = {toString() { a.href = 'https://changed.test/path'; return 'after'; }};
    if (a.href !== 'https://changed.test/path#after') failures.push('reentrant conversion');
    a.port = {toString() { a.removeAttribute('href'); return '80'; }};
    if (a.hasAttribute('href')) failures.push('removed during conversion');
    return failures;
  })()`),
    [],
  );
});

test('anchor URLs use canonical attributes and live owning-document bases through cloning and adoption', async () => {
  assert.deepEqual(
    await evaluate(`(() => {
    const failures = [], owner = document.implementation.createHTMLDocument();
    const base = owner.createElement('base'); base.href = 'https://first.test/dir/'; owner.head.append(base);
    const a = owner.createElement('a'); a.setAttributeNS('urn:other', 'href', 'https://foreign.test/');
    if (a.href !== '') failures.push('namespace');
    a.href = '../path?query#part';
    const href = Object.getOwnPropertyDescriptor(HTMLAnchorElement.prototype, 'href');
    const stringify = HTMLAnchorElement.prototype.toString;
    Object.defineProperty(a, 'href', {get(){throw Error('public href')},set(){throw Error('public href')}});
    a.getAttribute = a.setAttribute = a.hasAttribute = () => { throw Error('public attributes'); };
    a.__setUrlComponent = () => { throw Error('public implementation hook'); };
    if (a.host !== 'first.test' || stringify.call(a) !== 'https://first.test/path?query#part') failures.push('canonical access');
    a.hash = 'next';
    if (href.get.call(a) !== 'https://first.test/path?query#next') failures.push('canonical write');
    href.set.call(a, '../relative');
    base.href = 'https://second.test/deeper/';
    if (href.get.call(a) !== 'https://second.test/relative') failures.push('live base');
    const secondBase = owner.createElement('base'); secondBase.href = 'https://ignored.test/'; owner.head.append(secondBase);
    if (a.host !== 'second.test') failures.push('first base');
    base.remove(); if (a.host !== 'ignored.test') failures.push('base removal');
    const foreign = document.implementation.createHTMLDocument(), otherBase = foreign.createElement('base');
    otherBase.href = 'https://adopted.test/folder/'; foreign.head.append(otherBase);
    const imported = foreign.importNode(a, true), cloned = a.cloneNode(true);
    if (imported.href !== 'https://adopted.test/relative' || cloned.href !== 'https://ignored.test/relative') failures.push('clone/import');
    foreign.adoptNode(a);
    if (href.get.call(a) !== imported.href || a.getAttributeNS('urn:other','href') !== 'https://foreign.test/') failures.push('adoption');
    return failures;
  })()`),
    [],
  );
});

test('anchor query encoding follows its owning document without rewriting raw href attributes', async () => {
  const { measureAnchorURLEncoding } =
    await import('../../../scripts/coverage/anchor-url-encoding.mjs');
  const expected = JSON.parse(
    fs.readFileSync(new URL('../fixtures/anchor-url-encoding.safari.json', import.meta.url)),
  );
  const session = await createSession();
  try {
    assert.deepEqual(await measureAnchorURLEncoding(await session.newPage()), expected);
  } finally {
    await session.close();
  }
});
