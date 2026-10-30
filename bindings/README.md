# Python and Node bindings

These adapters expose the shared JSC/Lexbor/curl runtime through Python’s
synchronous Session/Page API and Node’s Promise-based API. Python releases the
GIL during native waits; Node runs those waits away from its event loop.

## Build and test

Prepare the native SDKs as described in the
[development guide](../docs/src/content/docs/development.md). Rebuild and copy
both extensions before testing:

```sh
cargo build -p brimp-python -p brimp-node
python3 bindings/prepare_local.py
PYTHONPATH=bindings/python/python python3 -m pytest bindings/python/tests
node --test bindings/node/test/*.test.mjs
```

These are SDK-linked development builds. Release workflows build self-contained
wheels and Node platform packages for macOS arm64, Linux x86_64/arm64 and
Windows x64; see [package assembly](../docs/src/content/docs/development/packaging.md).

`npm install @brimp/brimp` installs the `@brimp/brimp-<platform>-<arch>` optional
dependency selected for the host. A source checkout keeps using the local
`bindings/node/brimp.node` fallback after `prepare_local.py`.

## Documentation

- [Sessions, pages and requests](https://docs.brimp.ai/guides/automation/)
- [Values and handles](https://docs.brimp.ai/guides/values/)
- [Plugins](https://docs.brimp.ai/guides/plugins/)
- [Diagnostics and time](https://docs.brimp.ai/guides/diagnostics/)
- [Browser support](https://docs.brimp.ai/web-platform/)
- [Python package](python/README.md) and [Node package](node/README.md)

## Test ownership

`node/test/` and `python/tests/` verify host contracts: options, value and error
conversion, callbacks, streams, handle/session lifecycle and plugin backends.
Shared binding fixtures live in `fixtures/`.

Browser behavior belongs in [`tests/runtime/`](../tests/runtime/); real-library
workloads live in `tests/test/`. Both use Node as a driver. See the
[test guide](../tests/README.md) for their commands.
