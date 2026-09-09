if (!state.backendNodes || !state.backendNodes.has(__BACKEND_NODE_ID__)) {
    throw new Error("Unknown backend node: __BACKEND_NODE_ID__");
}
const value = state.backendNodes.get(__BACKEND_NODE_ID__);
