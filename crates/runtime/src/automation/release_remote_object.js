(() => {
    const state = globalThis.__brimpCdpRemoteObjects;
    if (!state) return false;
    const group = state.objectGroups.get(__OBJECT_ID__);
    if (group !== undefined) {
        state.groups.get(group)?.delete(__OBJECT_ID__);
        state.objectGroups.delete(__OBJECT_ID__);
    }
    return state.objects.delete(__OBJECT_ID__);
})()
