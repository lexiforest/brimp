(() => {
  // WebAssembly Web API §2: consume a valid Response, then compile its bytes.
  // The C embedding exposes compilation, but does not provide WebCore's Fetch
  // streaming hooks. Fetch consumes the body with its existing error/cancellation path.
  const { compile, instantiate } = WebAssembly;
  const ResponseClass = Response;
  const arrayBuffer = Response.prototype.arrayBuffer;
  async function compileStreaming(source, options) {
    const response = await source;
    if (!(response instanceof ResponseClass))
      throw new TypeError('WebAssembly source must be a Response');
    const mime = response.headers.get('content-type');
    if (mime === null || mime.replace(/^[\t ]+|[\t ]+$/g, '').toLowerCase() !== 'application/wasm')
      throw new TypeError(
        'WebAssembly response must have application/wasm Content-Type without parameters',
      );
    if (
      !response.ok ||
      response.type === 'opaque' ||
      response.type === 'opaqueredirect' ||
      response.type === 'error'
    )
      throw new TypeError('WebAssembly response must be successful and accessible');
    return compile(await arrayBuffer.call(response), options);
  }
  async function instantiateStreaming(source, imports, options) {
    const module = await compileStreaming(source, options);
    const instance = await instantiate(module, imports);
    return { module, instance };
  }
  Object.defineProperties(WebAssembly, {
    compileStreaming: {
      value: compileStreaming,
      writable: true,
      configurable: true,
      enumerable: true,
    },
    instantiateStreaming: {
      value: instantiateStreaming,
      writable: true,
      configurable: true,
      enumerable: true,
    },
  });
})();
