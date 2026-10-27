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

These are SDK-linked development builds. For distributable macOS archives, see
[package assembly](../docs/src/content/docs/development/packaging.md).

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
