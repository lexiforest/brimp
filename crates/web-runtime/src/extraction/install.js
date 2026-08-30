(() => {
    const Constructor = globalThis.Defuddle;
    delete globalThis.Defuddle;
    if (typeof Constructor !== "function") {
        throw new Error("Defuddle bundle did not install its constructor");
    }
    return optionsJson => {
        const options = JSON.parse(optionsJson);
        const methods = ["debug", "error", "info", "log", "warn"];
        const saved = methods.map(name => console[name]);
        for (const name of methods) console[name] = () => {};
        try {
            return JSON.stringify(new Constructor(document, options).parse());
        } finally {
            methods.forEach((name, index) => { console[name] = saved[index]; });
        }
    };
})()
