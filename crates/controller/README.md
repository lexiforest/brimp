# brimp-controller

Native-independent Rust controller library, exposed through `brimp serve`.
It owns HTTP discovery, CDP WebSockets, browser contexts, worker pooling,
crash recovery, and resource limits. It does not link WebKit or the lite runtime.
The host adapter currently supports macOS only.

From the workspace root:

```sh
cargo build -p brimp-cli -p brimp-lite-worker
./target/debug/brimp serve --worker-path "$PWD/target/debug/lite-worker" --pool-size=2 --port=9222
```

`--worker-path` (or `BRIMP_WORKER_PATH`) accepts a lite-worker executable or a
matching `WebKitAutomationWorker.app`/executable. WebKit is built separately
from the adjacent WebKit checkout; supply any required framework environment
to the CLI so its children inherit it.

For workers that expose HTTP/CDP themselves:

```sh
brimp serve --worker-protocol=cdp --worker-path=/path/to/browser --pool-size=2 --port=9222
```

Repeat `--worker-arg=ARG` to pass child arguments. `{port}` placeholders are
replaced with a loopback port; without one, the controller appends
`--remote-debugging-port=PORT`. Closing a lease replaces its worker.

`get`, `crawl`, and `doctor` use this crate's owned worker connection directly,
without starting a server. The shared transport correlates responses, delivers
session events, bounds frames to 64 MiB, and enforces command deadlines.

## Validation

```sh
cargo build -p brimp-cli -p brimp-lite-worker
cargo test --workspace
python -m pytest crates/controller/tests
# Optional full WebKit acceptance:
BRIMP_TEST_WEBKIT_WORKER="$PWD/../WebKit/WebKitBuild/Release/WebKitAutomationWorker.app" python -m pytest crates/controller/tests -k webkit
```

The imported controller and worker contract is documented in
[docs/PROTOCOL.md](docs/PROTOCOL.md). Lite CDP support is documented separately
in [../worker/SUPPORT.md](../worker/SUPPORT.md).
