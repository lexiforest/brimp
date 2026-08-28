for (const name of __NAMES__) {
    if (!(name in window)) {
        Object.defineProperty(window, name, {
            configurable: true,
            enumerable: true,
            get() {
                return document.getElementById(name)
                    ?? document.getElementsByName(name)[0]
                    ?? undefined;
            },
        });
    }
}
