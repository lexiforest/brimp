Object.defineProperty(globalThis, "crossOriginIsolated", {
    value: false,
    configurable: true,
});
delete globalThis.SharedArrayBuffer;
