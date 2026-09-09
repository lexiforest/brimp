(() => {
    const state = globalThis.__brimpCdpRemoteObjects;
    const objectId = __OBJECT_ID__;
    if (!state || !state.objects.has(objectId)) {
        throw new Error("Unknown remote object: " + objectId);
    }
    const node = state.objects.get(objectId);
    if (!node || typeof node.nodeType !== "number") {
        throw new Error("Remote object is not a DOM node: " + objectId);
    }
    if (!state.nodeIds) {
        state.nextNode = 1;
        state.nodeIds = new WeakMap();
        state.backendNodes = new Map();
    }
    let backendNodeId = state.nodeIds.get(node);
    if (backendNodeId === undefined) {
        backendNodeId = state.nextNode++;
        state.nodeIds.set(node, backendNodeId);
        state.backendNodes.set(backendNodeId, node);
    }
    return JSON.stringify({
        nodeId: backendNodeId,
        backendNodeId,
        nodeType: node.nodeType,
        nodeName: node.nodeName || "",
        localName: node.localName || "",
        nodeValue: node.nodeValue || "",
        childNodeCount: node.childNodes ? node.childNodes.length : 0,
    });
})()
