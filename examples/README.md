# Examples

The [script harness examples](harness/) run the same browser function repeatedly
from Node, Python or the CLI, using controlled network responses and logical time.
They inspect grouped API dependencies through the existing Session/Page API; see
the [debugging tutorial](../docs/src/content/docs/examples/debugging.md).

The [graphics/audio example](plugins/graphics-audio.js) demonstrates JavaScript
state and explicit asynchronous host extensions. The [worker host example](plugins/node-host-worker.mjs)
shows how to provide execution through the plugin bridge. The [SQLite storage
backend](plugins/sqlite_storage.py) demonstrates durable, origin-partitioned
transactions through the hostStorage plugin.

Native Python and Node usage is documented in [bindings](../bindings/README.md).

Larger standalone challenge reproductions live in the sibling
[brimp-examples](https://github.com/lexiforest/brimp-examples) repository.
