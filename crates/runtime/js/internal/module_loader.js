(() => {
  const parse = JSON.parse;
  const CompileFunction = Function;
  const mapGet = Function.prototype.call.bind(Map.prototype.get);
  const mapSet = Function.prototype.call.bind(Map.prototype.set);
  const mapHas = Function.prototype.call.bind(Map.prototype.has);
  const ResolveURL = URL;
  const definitions = new Map();
  const cache = new Map();
  const resolve = (specifier, parent) => new ResolveURL(specifier, parent).href;
  const register = (url, code) => {
    mapSet(
      definitions,
      url,
      new CompileFunction(
        'require',
        'module',
        'exports',
        '__filename',
        '__dirname',
        `${code}\n//# sourceURL=${url}`,
      ),
    );
  };
  const load = (url) => {
    if (mapHas(cache, url)) return mapGet(cache, url).exports;
    const definition = mapGet(definitions, url);
    if (!definition) throw new TypeError(`Module was not loaded: ${url}`);
    const module = { id: url, uri: url, exports: {} };
    mapSet(cache, url, module);
    const require = (specifier) => {
      if (specifier === 'url') {
        return { pathToFileURL: (value) => new ResolveURL(value) };
      }
      return load(resolve(specifier, url));
    };
    require.resolve = (specifier) => resolve(specifier, url);
    require.toUrl = (specifier) => resolve(specifier, url);
    require.main = module;
    let directory = url;
    try {
      directory = new ResolveURL('.', url).href;
    } catch {}
    definition(require, module, module.exports, url, directory);
    return module.exports;
  };
  return (serialized) => {
    const { op, url, code } = parse(serialized);
    if (op === 'register') register(url, code);
    else if (op === 'evaluate') load(url);
  };
})();
