import * as idb from 'fake-indexeddb';
import installStorage from './storage-base.js';
import { configureStorageBackend } from './storage-idb.js';
export function install(context) {
  configureStorageBackend(context.host);
  // The entire bundle is evaluated for each realm. No cross-origin/global
  // database singleton is shared implicitly by the embedding process.
  for (const [name, value] of Object.entries(idb)) {
    if (name !== 'default' && name !== 'forceCloseDatabase') context.global[name] = value;
  }
  // fake-indexeddb dispatches these hooks but leaves their initial values absent.
  for (const name of ['onabort', 'onclose', 'onerror', 'onversionchange']) {
    Object.defineProperty(idb.IDBDatabase.prototype, name, {
      value: null,
      writable: true,
      enumerable: true,
      configurable: true,
    });
  }
  return installStorage(context);
}
