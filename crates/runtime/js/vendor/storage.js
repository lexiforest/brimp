import * as idb from 'fake-indexeddb';
import installStorage from './storage-base.js';
export function install(context) {
    // The entire bundle is evaluated for each realm. No cross-origin/global
    // database singleton is shared implicitly by the embedding process.
    for (const [name, value] of Object.entries(idb)) {
        if (name !== 'default' && name !== 'forceCloseDatabase') context.global[name] = value;
    }
    return installStorage(context);
}
