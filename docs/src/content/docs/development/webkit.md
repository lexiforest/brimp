---
title: WebKit backend
description: Build the macOS or Linux WebKit host and exercise the shared Python and Node APIs.
---

The WebKit backend runs a full browser in a separate process. Python and Node
share a Rust adapter, which starts a small embedding host and exchanges
length-prefixed JSON over private pipes. WebKit owns navigation, JavaScript,
layout, cookies and input. The existing `runtime` engine remains the default.

This backend is under development on macOS and Linux. The helper uses the Brimp
WebKit fork and its local embedding interface. The release pipeline packages the helper and engine libraries in the separate
`brimp-webkit` / `@brimp/webkit` distributions. See the
[packaging guide](/development/packaging/#webkit-companion-packages) for installation,
platform requirements and the engine release prerequisite.

## Build on macOS

The pinned Brimp fork revision is recorded in `tools/webkit/REVISION`. It is
based on Safari 27.2 (`WebKit-7625.2.8`, upstream commit
`7dc13736dc3c83fcb8bb837c0c681eaa3da8b16c`) and already includes the local
Automation connection and byte-preserving Inspector response access. The
Automation connection explicitly opts into Inspector access; it opens no port.
`tools/webkit/local-automation.patch` records that final commit for review or
replay on its parent, `2cb482246e0f7294d1489e026e6f5c6ed2135ec3`.

From the Brimp repository root, using the adjacent checkout:

```sh
git -C ../WebKit checkout "$(cat tools/webkit/REVISION)"
cd ../WebKit
Tools/Scripts/build-webkit --release '--only=Everything up to WebKit' -jobs 8 DEBUG_INFORMATION_FORMAT=dwarf COMPILATION_CACHE_ENABLE_CACHING=NO
cd ../brimp
python3 tools/webkit/build.py --webkit ../WebKit
cargo build -p brimp-python -p brimp-node
python3 bindings/prepare_local.py
```

Do not apply the patch again to the pinned revision. Rebuild after changing
revisions; binaries from the earlier development baseline do not validate this
Safari release baseline. Build the complete dependency
graph: a framework-only build of a fresh checkout lacks generated WebCore
headers. The selected scheme includes dependencies and browser subprocesses,
without the unrelated WebKit test tools. The cache flag avoids invalid cached Swift/C++ modules encountered with
the local Xcode toolchain.

The helper builder creates `target/webkit/brimp-webkit`. This development launcher
selects the checkout's frameworks, then verifies the helper's handshake and the
framework it actually loaded. Keep that checkout and build directory available
while using the launcher.

## Build on Linux

For a fresh Ubuntu 26.04 development node, use `tools/linux/README.md` and
`tools/linux/setup.sh`. The default runner builds the engine, Brimp bindings and
embedding host natively, then runs the Python/Node backend contracts. Docker is
not required. Optional Ubuntu 24.04 packaging checks are documented separately
in `tools/linux/PACKAGING.md`. The commands below describe individual host steps.

The Linux host uses WebKitGTK 6.0 from the Brimp fork, with its curl-impersonate
network backend. Build and install the SDK using the adjacent WebKit checkout's
`Tools/Scripts/linux-webkit/build.sh`. The SDK must include the local Automation
connection, binary request setters, immediate page closure and nullable cookie
replacement APIs. A distribution WebKitGTK package cannot supply these APIs.

On the validated Ubuntu 26.04 x64 environment:

```sh
sudo apt-get install clang pkg-config nlohmann-json3-dev libgtk-4-dev libsoup-3.0-dev xvfb xauth dbus-x11
python3 tools/webkit/build.py --sdk /opt/brimp/sdk/webkit-curl
```

The builder compiles `main-gtk.cpp`, creates `target/webkit/brimp-webkit`, and
checks the loaded library through a real protocol handshake under Xvfb. Keep
the SDK at its installation prefix, including its browser subprocesses and
resources. The launcher selects that SDK's libraries and directly executes the
host so Brimp owns its process lifetime. GTK, font and other SDK system
dependencies must also be installed; this development launcher is not a
self-contained distribution.

Run applications as a normal user with a display and a D-Bus session. For a
headless machine, wrap the application in `dbus-run-session -- xvfb-run -a` and
provide a private, user-owned `XDG_RUNTIME_DIR`. The host retains WebKit's
process sandbox. Its Automation and Inspector connection uses inherited pipes
and does not open a debugging port.

## Select WebKit

```python
import brimp

with brimp.Session(
    engine="webkit",
    executable_path="target/webkit/brimp-webkit",
) as session:
    page = session.new_page()
    page.get("https://example.com")
    print(page.status_code, page.evaluate("document.title"))
```

```js
import { createSession } from '@brimp/brimp';

const session = await createSession({
  engine: 'webkit',
  executablePath: 'target/webkit/brimp-webkit',
});
try {
  const page = await session.newPage();
  await page.get('https://example.com');
  console.log(page.statusCode, await page.evaluate('document.title'));
} finally {
  await session.close();
}
```

`brimp.AsyncSession` accepts the same engine options. Without an explicit
executable path, Python and Node resolve the installed WebKit companion package.
Each session
owns a helper and an ephemeral WebKit data store. Pages within a session share
cookies and browser storage.

WebKit sessions default to an empty plugin list. Runtime subsystem plugins,
profiles, identity overrides, logical clocks, diagnostics and persistent
runtime storage are rejected. WebKit uses its own network stack; curl proxy and
fingerprint settings do not configure it. The Linux SDK currently selects
curl-impersonate's `safari270` profile at engine build time.

The initial adapter covers navigation, response snapshots, evaluation and
handles, selectors and locators, waits, native mouse/keyboard input, page events,
popups, initialization scripts, cookies and cache clearing. Routing, streaming,
file uploads, drag-and-drop, extraction and cross-origin frame locators still
need backend implementations. Unsupported operations raise an error.
Mutation, WebSocket, EventSource and pre-creation popup event subscriptions are
also unsupported. Request snapshots require HTTP(S); custom redirect policies,
keyboard modifiers and interpolated mouse movement are not implemented yet.
Set page headers before navigation when changing an existing session header;
conflicting per-request overrides are rejected.
Inspector can coalesce repeated response headers. Byte access preserves the
resource bytes after transport decompression, before character decoding.

Timeouts and cancellation stop polling and pending promise operations. If a
synchronous script blocks its content process, cancellation terminates that
process; affected pages close. This prevents an infinite loop from continuing
after its caller has returned. Private controller references stay in Inspector
and are never installed as `__brimp*` properties on the page.

## Run shared contracts

```sh
export BRIMP_WEBKIT_EXECUTABLE="$PWD/target/webkit/brimp-webkit"
PYTHONPATH=bindings/python/python python3 -m pytest bindings/python/tests/test_browser_backend.py
node --test bindings/node/test/browser-backend.test.mjs
```

These fixtures run against both engines and cover actual local HTTP responses,
non-UTF-8 response bytes, binary request bodies, redirects, script execution,
trusted input, cookies, handles, popups, process exit and cancellation.
Without the environment variable, WebKit cases are skipped. A configured but
broken helper fails the tests.

On Linux, after building the bindings and configuring their native SDK
environment, run both suites with the display/session setup included:

```sh
bash tools/webkit/test-linux.sh target/webkit/brimp-webkit
```

Set `PYTHON` if the pytest environment uses a different Python executable.
The engine's separate `Tools/Scripts/linux-webkit/test.sh` verifies the HTTP,
TLS, HTTP/2, proxy and WebSocket transport against the installed SDK. Both
levels of tests are needed: an engine build alone does not validate Brimp's
`engine="webkit"` integration.
