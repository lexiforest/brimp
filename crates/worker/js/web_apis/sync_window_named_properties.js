{
for (const name of __NAMES__) {
    if (!(name in window)) {
        Object.defineProperty(window, name, {
            configurable: true,
            enumerable: true,
            get() {
                const element = document.getElementById(name)
                    ?? document.getElementsByName(name)[0];
                return element instanceof HTMLIFrameElement ? element.contentWindow : element;
            },
        });
    }
}
const frameCount = document.getElementsByTagName("iframe").length;
const previousFrameCount = window.__brimpFrameIndexCount ?? 0;
for (let index = frameCount; index < previousFrameCount; index++) delete window[index];
Object.defineProperty(window, "__brimpFrameIndexCount", {
    value: frameCount,
    writable: true,
    configurable: true,
});
for (let index = 0; index < frameCount; index++) {
    if (!(index in window)) {
        Object.defineProperty(window, index, {
            configurable: true,
            enumerable: true,
            get() { return document.getElementsByTagName("iframe").item(index)?.contentWindow; },
        });
    }
}
}
