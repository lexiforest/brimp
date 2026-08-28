(() => {
    const node = globalThis.__brimpCdpRemoteObjects?.backendNodes?.get(__NODE_ID__);
    if (!node || typeof node.getAttributeNames !== "function") {
        throw new Error("DOM node has no attributes");
    }
    return node.getAttributeNames().flatMap(name => [name, node.getAttribute(name)]);
})()
