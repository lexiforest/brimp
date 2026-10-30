import Dexie from 'dexie';
import { check } from './check.js';

function database() {
  const db = new Dexie('brimp-corpus');
  db.version(1).stores({ people: 'id,age' });
  return db;
}
globalThis.runWorkload = async () => {
  const db = database();
  await db.people.bulkPut([
    { id: 1, name: 'Ada', age: 36 },
    { id: 2, name: 'Grace', age: 85 },
  ]);
  let rolledBack = false;
  try {
    await db.transaction('rw', db.people, async () => {
      await db.people.put({ id: 3, name: 'rollback', age: 1 });
      throw new Error('deliberate rollback');
    });
  } catch (error) {
    rolledBack = error.message === 'deliberate rollback';
  }
  check(
    rolledBack && (await db.people.count()) === 2,
    'failed IndexedDB transaction must roll back',
  );
  const names = (await db.people.where('age').above(40).toArray()).map((person) => person.name);
  check(names.join() === 'Grace', 'indexed query must select the correct record');
  db.close();
  const reopened = database();
  check((await reopened.people.count()) === 2, 'closing and reopening must preserve records');
  reopened.close();
  return { names, rolledBack };
};
globalThis.readPersistedWorkload = async () => {
  const db = database();
  const names = (await db.people.toArray()).map((person) => person.name).sort();
  db.close();
  check(names.join() === 'Ada,Grace', 'IndexedDB records must survive page reload');
  return names;
};
