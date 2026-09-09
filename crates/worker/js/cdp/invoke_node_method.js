(() => {
    const state = globalThis.__brimpCdpRemoteObjects;
    const node = __NODE__;
    const method = __METHOD__;
    if (!node || typeof node[method] !== "function") {
        throw new Error("DOM node does not support " + method);
    }
    node[method]();
    return true;
})()
