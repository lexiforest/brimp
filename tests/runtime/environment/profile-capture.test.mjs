import test from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';

const load = async (name) =>
  (0, eval)(
    `(${await readFile(
      new URL(`../../../crates/runtime/js/internal/${name}.js`, import.meta.url),
      'utf8',
    )})`,
  );
const registry = await load('profile_registry');
const capture = await load('profile_capture');

test('capture distinguishes observations from omissions and validates measured values', async () => {
  const definitions = registry();
  definitions.forOwner('test').values('example', {
    observed: 1,
    absent: 2,
    blocked: 3,
    failed: 4,
    unsupported: 5,
    invalid: 6,
    nested: { measured: 7, omitted: 8 },
  });
  const progress = [];
  const profile = await capture(
    definitions.contract(),
    {
      example: async ({ read }) => ({
        observed: read(() => 42),
        absent: read(() => undefined),
        blocked: await read(async () => {
          throw new DOMException('Denied', 'NotAllowedError');
        }),
        failed: read(() => {
          throw new Error('Broken');
        }),
        invalid: read(() => 'wrong type'),
        nested: read(() => ({ measured: 9 })),
      }),
    },
    registry,
    { toolVersion: 'test', selectedPlugins: ['test'], onProgress: (r) => progress.push(r) },
  );
  assert.deepEqual(JSON.parse(JSON.stringify(profile.values)), {
    example: { observed: 42, nested: { measured: 9 } },
  });
  assert.deepEqual(
    Object.values(profile.capture.results).map((r) => r.status),
    ['captured', 'absent', 'permission-required', 'failed', 'unavailable', 'failed', 'captured'],
  );
  assert.equal(progress.length, 7);
  assert.ok(profile.capture.defaultsUsed.includes('example.nested.omitted'));
  assert.ok(profile.capture.defaultsUsed.includes('example.invalid'));
  assert.deepEqual(profile.capture.selectedPlugins, ['test']);
});

test('capture rejects non-JSON observations instead of silently serializing them', async () => {
  const definitions = registry();
  definitions.forOwner('test').values('example', { value: null });
  for (const value of [NaN, Infinity, 1n, { hidden: undefined }, [undefined]]) {
    const profile = await capture(
      definitions.contract(),
      {
        example: ({ read }) => ({ value: read(() => value) }),
      },
      registry,
    );
    assert.equal(profile.capture.results['example.value'].status, 'failed');
    assert.deepEqual(Object.keys(profile.values), []);
  }
});

test('a failed probe does not prevent later interfaces from being captured', async () => {
  const definitions = registry();
  definitions.forOwner('test').values('first', { value: 1 });
  definitions.forOwner('test').values('second', { value: 2 });
  const profile = await capture(
    definitions.contract(),
    {
      first: () => {
        throw new Error('Failed initialization');
      },
      second: ({ read }) => ({ value: read(() => 3) }),
    },
    registry,
  );
  assert.equal(profile.capture.results['first.value'].status, 'failed');
  assert.equal(profile.values.second.value, 3);
});

test('record captures retain per-entry failures and only apply observed entries', async () => {
  const definitions = registry();
  definitions
    .forOwner('test')
    .values('example', { states: { available: 'prompt', denied: 'prompt' } });
  const profile = await capture(
    definitions.contract(),
    {
      example: async ({ readRecord }) => ({
        states: await readRecord({
          available: () => 'granted',
          denied: () => {
            throw new DOMException('Denied', 'NotAllowedError');
          },
        }),
      }),
    },
    registry,
  );
  assert.deepEqual(profile.values.example.states, { available: 'granted' });
  assert.equal(profile.capture.results['example.states.denied'].status, 'permission-required');
  assert.ok(profile.capture.defaultsUsed.includes('example.states.denied'));
});

test('a stalled asynchronous probe times out and later interfaces still run', async () => {
  const definitions = registry();
  definitions.forOwner('test').values('first', { value: 1 });
  definitions.forOwner('test').values('second', { value: 2 });
  const profile = await capture(
    definitions.contract(),
    {
      first: () => new Promise(() => {}),
      second: ({ read }) => ({ value: read(() => 3) }),
    },
    registry,
  );
  assert.match(profile.capture.results['first.value'].reason, /timed out/);
  assert.equal(profile.values.second.value, 3);
});

test('capture records interface versions and rejects loading them against different definitions', async () => {
  const definitions = registry();
  definitions.forOwner('custom').values('example', { answer: 42 }, { version: 3 });
  const profile = await capture(
    definitions.contract(),
    {
      example: ({ read }) => ({ answer: read(() => 7) }),
    },
    registry,
  );
  const loaded = registry(profile);
  loaded.forOwner('custom').values('example', { answer: 42 }, { version: 4 });
  assert.throws(
    () => loaded.validate(),
    /capture requires custom version 3; loaded custom version 4/,
  );
});
