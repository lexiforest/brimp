(() => {
    const state = globalThis.__brimpCdpRemoteObjects;
    const node = __NODE__;
    if (!node || typeof node.getBoundingClientRect !== "function") {
        throw new Error("DOM node has no layout box");
    }
    const rect = node.getBoundingClientRect();
    if (rect.width <= 0 || rect.height <= 0) return { quads: [] };
    return {
        quads: [[
            rect.left,
            rect.top,
            rect.right,
            rect.top,
            rect.right,
            rect.bottom,
            rect.left,
            rect.bottom,
        ]],
    };
})()
