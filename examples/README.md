# Examples

The [graphics/audio example](plugins/graphics-audio.js) demonstrates JavaScript
state and explicit asynchronous host extensions. The [worker host example](plugins/node-host-worker.mjs)
shows how to provide execution through the plugin bridge.

Native Python and Node usage is documented in [bindings](../bindings/README.md).
The [SQLite storage backend](plugins/sqlite_storage.py) demonstrates durable,
origin-partitioned transactions through the hostStorage plugin.
