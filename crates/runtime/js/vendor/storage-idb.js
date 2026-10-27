// Adapter for the pinned fake-indexeddb engine. Its transaction semantics remain
// in JS; the session backend owns committed records and cross-realm arbitration.
import Database from './node_modules/fake-indexeddb/build/esm/lib/Database.js';
import ObjectStore from './node_modules/fake-indexeddb/build/esm/lib/ObjectStore.js';
import Index from './node_modules/fake-indexeddb/build/esm/lib/Index.js';
import { encode, decode } from './storage-codec.js';
let host;
export function configureStorageBackend(value) {
  host = value;
}
export function validateOrigin() {
  host({ op: 'persistent' });
}
const transactions = new WeakMap();
function acquire(name) {
  return new Promise((resolve) => {
    const id = host({ op: 'lockRequest', name: '-indexeddb:' + name, mode: 'exclusive' }, () =>
      resolve(id),
    );
  });
}
function release(id) {
  if (id !== undefined) host({ op: 'lockRelease', id });
}
function save(db) {
  const stores = [...db.rawObjectStores.values()].map((store) => ({
    name: store.name,
    keyPath: store.keyPath,
    autoIncrement: store.autoIncrement,
    generator: store.keyGenerator?.num,
    records: [...store.records.values()],
    indexes: [...store.rawIndexes.values()].map((index) => ({
      name: index.name,
      keyPath: index.keyPath,
      multiEntry: index.multiEntry,
      unique: index.unique,
      records: [...index.records.values()],
    })),
  }));
  host({
    op: 'set',
    namespace: 'indexeddb',
    key: db.name,
    value: encode({ name: db.name, version: db.version, stores }),
  });
}
function load(name, existing) {
  const text = host({ op: 'get', namespace: 'indexeddb', key: name });
  if (text === null) return undefined;
  const snapshot = decode(text);
  const db = existing ?? new Database(name, snapshot.version);
  db.version = snapshot.version;
  const names = new Set();
  for (const item of snapshot.stores) {
    names.add(item.name);
    let store = db.rawObjectStores.get(item.name);
    if (!store) {
      store = new ObjectStore(db, item.name, item.keyPath, item.autoIncrement);
      db.rawObjectStores.set(item.name, store);
    }
    store.records.clear();
    for (const record of item.records) store.records.put(record);
    if (store.keyGenerator) store.keyGenerator.num = item.generator;
    const indexNames = new Set();
    for (const data of item.indexes) {
      indexNames.add(data.name);
      let index = store.rawIndexes.get(data.name);
      if (!index) {
        index = new Index(store, data.name, data.keyPath, data.multiEntry, data.unique);
        store.rawIndexes.set(data.name, index);
      }
      index.initialized = true;
      index.records.clear();
      for (const record of data.records) index.records.put(record);
    }
    for (const name of store.rawIndexes.keys())
      if (!indexNames.has(name)) store.rawIndexes.delete(name);
  }
  for (const name of db.rawObjectStores.keys())
    if (!names.has(name)) db.rawObjectStores.delete(name);
  return db;
}
export async function beginOpen(factory, name) {
  name = String(name);
  const id = await acquire(name);
  try {
    const existing = factory._databases.get(name);
    const db = load(name, existing);
    if (db) {
      factory._databases.set(name, db);
    } else factory._databases.delete(name);
    return id;
  } catch (error) {
    release(id);
    throw error;
  }
}
export function endOpen(factory, name, id, deleted = false) {
  const db = factory._databases.get(String(name));
  try {
    if (deleted && !db) host({ op: 'delete', namespace: 'indexeddb', key: String(name) });
    else if (db) save(db);
  } finally {
    release(id);
  }
}
export function beginTransaction(tx) {
  if (tx.mode === 'versionchange' || transactions.has(tx)) return true;
  // Mark it started while the native queue owns the wait; the engine must not
  // schedule it again or start a conflicting transaction in this realm.
  transactions.set(tx, undefined);
  tx._started = true;
  acquire(tx.db.name).then((id) => {
    if (tx._state === 'finished') {
      release(id);
      return;
    }
    transactions.set(tx, id);
    try {
      const version = tx.db.version;
      const db = load(tx.db.name, tx.db._rawDatabase);
      if (!db || version !== db.version) {
        tx._abort('VersionError');
        return;
      }
      tx._start();
    } catch (error) {
      tx._abort(error.name ?? 'UnknownError');
    }
  });
  return false;
}
export function endTransaction(tx, committed) {
  if (tx.mode === 'versionchange') return true; // The factory owns this lease.
  const id = transactions.get(tx);
  if (id === undefined) return true;
  try {
    if (committed && tx.mode !== 'readonly') save(tx.db._rawDatabase);
  } catch (error) {
    transactions.delete(tx);
    release(id);
    tx._abort('UnknownError');
    return false;
  }
  transactions.delete(tx);
  release(id);
  return true;
}
export function databases() {
  return host({ op: 'list', namespace: 'indexeddb' }).map((name) => {
    const { version } = decode(host({ op: 'get', namespace: 'indexeddb', key: name }));
    return { name, version };
  });
}
