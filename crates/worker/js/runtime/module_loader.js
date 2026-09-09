
if (!globalThis.__brimpModuleLoader) {
    globalThis.__brimpModuleLoader = (() => {
        const definitions = new Map();
        const cache = new Map();
        const resolve = (specifier, parent) => new URL(specifier, parent).href;
        const register = (url, code) => {
            definitions.set(url, new Function(
                "require", "module", "exports", "__filename", "__dirname",
                `${code}\n//# sourceURL=${url}`
            ));
        };
        const load = url => {
            if (cache.has(url)) return cache.get(url).exports;
            const definition = definitions.get(url);
            if (!definition) throw new TypeError(`Module was not loaded: ${url}`);
            const module = { id: url, uri: url, exports: {} };
            cache.set(url, module);
            const require = specifier => {
                if (specifier === "url") {
                    return { pathToFileURL: value => new URL(value) };
                }
                return load(resolve(specifier, url));
            };
            require.resolve = specifier => resolve(specifier, url);
            require.toUrl = specifier => resolve(specifier, url);
            require.main = module;
            let directory = url;
            try { directory = new URL(".", url).href; } catch {}
            definition(require, module, module.exports, url, directory);
            return module.exports;
        };
        return Object.freeze({ register, evaluate: load });
    })();
}
