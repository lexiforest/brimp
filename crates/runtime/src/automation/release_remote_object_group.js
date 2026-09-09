(() => {
    const state = globalThis.__brimpCdpRemoteObjects;
    if (!state) return 0;
    const ids = state.groups.get(__OBJECT_GROUP__);
    if (!ids) return 0;
    for (const id of ids) {
        state.objects.delete(id);
        state.objectGroups.delete(id);
    }
    state.groups.delete(__OBJECT_GROUP__);
    return ids.size;
})()
