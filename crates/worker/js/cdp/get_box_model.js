(() => {
    const state = globalThis.__brimpCdpRemoteObjects;
    const node = __NODE__;
    if (!node || typeof node.getBoundingClientRect !== "function") {
        throw new Error("DOM node has no layout box");
    }
    const rect = node.getBoundingClientRect();
    const quad = [
        rect.left,
        rect.top,
        rect.right,
        rect.top,
        rect.right,
        rect.bottom,
        rect.left,
        rect.bottom,
    ];
    return {
        model: {
            content: quad,
            padding: quad,
            border: quad,
            margin: quad,
            width: rect.width,
            height: rect.height,
        },
    };
})()
