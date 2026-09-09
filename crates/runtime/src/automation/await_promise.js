(() => {
    const state = globalThis.__brimpCdpRemoteObjects
        || (globalThis.__brimpCdpRemoteObjects = {
            next: 1,
            objects: new Map(),
            groups: new Map(),
            objectGroups: new Map(),
        });
    __BODY__
    const pending = { settled: false, rejected: false, value: undefined };
    state.pendingPromise = pending;
    Promise.resolve(value).then(
        result => {
            pending.value = result;
            pending.settled = true;
        },
        error => {
            pending.value = error;
            pending.rejected = true;
            pending.settled = true;
        },
    );
    return true;
})()
