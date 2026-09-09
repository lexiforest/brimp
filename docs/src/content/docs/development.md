---
title: Development
description: Build, test, and work on Brimp and its documentation.
---

## Architecture

Brimp separates command orchestration from browser execution. The `brimp`
process owns CLI commands, crawl scheduling, output, and worker supervision.
Browser workers execute navigation, JavaScript, extraction, and screenshots.
The controller communicates with them through CDP.

### Packages and source layout

| Package | Responsibility |
| --- | --- |
| `brimp-controller` | The `brimp` executable, CLI, browser client, CDP server, and worker supervision. |
| `brimp-protocol` | Shared CDP messages, framing limits, configuration, and result types. |
| `brimp-lite-worker` | The `lite-worker` executable, CDP dispatch, and native browser engine. |

Both executables depend on `brimp-protocol`; neither depends on the other.
Only the lite worker links JavaScriptCore and curl-impersonate.

```text
crates/controller/  CLI, browser operations, connections, transports, and server
crates/protocol/    Shared protocol types
crates/worker/src/  runtime, web_apis, dom, network, jsc, and persona modules
assets/defuddle/    Extraction script and licenses
scripts/           Native SDK preparation and checks
docs/              This documentation site
```

### Processes and communication

`fetch`, `crawl`, and `doctor` launch and own their worker without starting an
HTTP server. `serve` exposes HTTP discovery and WebSocket CDP and manages a
worker pool. In framed mode, each browser context leases one worker; its pages
share browser state, while separate contexts remain isolated.

Worker selection uses `--worker-path` or `BRIMP_WORKER_PATH`. Lite and external
WebKit workers use an inherited socket carrying length-prefixed CDP JSON
messages. The frame header is a four-byte big-endian length, with a 64 MiB
payload limit. Protocol traffic and diagnostics use separate channels.

With `--cdp`, the controller launches a worker with its own WebSocket endpoint
and a temporary profile. The server bridges its CDP traffic; in framed mode,
the server instead maps public context, target, and session identifiers to
worker identifiers. Pool limits, timeouts, and crash replacement belong to
the controller.

Startup counts toward command deadlines. Completion, failure, and cancellation
close the connection, terminate and reap the child process group, and remove
temporary profiles. Failed commands are not automatically replayed.

### Lite engine ownership

`runtime` is the lite worker's page implementation. `PageHandle` keeps each
page's JavaScriptCore runtime, canonical Blitz document, and task queues on one
owner thread. Other threads submit work through queues. Independent pages can
navigate concurrently; duplicate navigation on the same session is rejected.

CDP dispatch translates commands into runtime operations. Before creating pages,
the controller sends `Brimp.configure` to apply the persona, proxy, headers,
trust roots, storage, and navigation timeout. The worker validates the persona
and owns browser state.

`runtime` owns cookies, redirects, request identity, and lifecycle policy.
Network requests cross `ResourceLoader`; curl-impersonate owns transport and
connection reuse. JavaScript bindings operate on the same DOM used for layout
and rendering. Keep these boundaries when adding APIs.

Canvas, workers, and streaming networking are always enabled. Persistent storage
requires a backing path. Extraction runs the bundled Defuddle script against
the live document. Document and canvas text use the bundled fonts under
`crates/worker/assets/fonts`, with system-font discovery disabled.

See [Subsystem implementation](/architecture/subsystems/) for backend details,
[JavaScriptCore integration](/architecture/javascript-runtime/) for binding
ownership, and the [CDP reference](/api/cdp/) for supported commands.

### Platform boundaries

Worker launch and supervision currently support macOS. Some transport code
compiles on Linux and Windows, but those platforms do not yet provide complete
browser-worker hosting. The lite executable's inherited-socket transport is
Unix-only.

The external WebKit worker supports rendered output, evaluation, extraction,
and screenshots. It does not support the CLI's raw-response and robots
retrieval, custom headers/cookies, or lite-specific configuration. WebKit crawls
therefore require `--ignore-robots`.

## Native prerequisites

The CLI and controller build without native browser libraries. The lite worker
links JavaScriptCore and curl-impersonate. Browser commands currently require
macOS; native SDK preparation also supports Linux and Windows build targets.

### Prepare the native SDKs

From the repository root, download and verify the pinned SDK archives for your
target:

```sh
python3 scripts/prepare_native.py \
  --target aarch64-apple-darwin \
  --output .native/aarch64-apple-darwin
```

The script prints the environment values needed to build the worker. Apply those
values in your shell before running Cargo. In GitHub Actions, the script also
writes them to `GITHUB_ENV` for subsequent steps.

### Configure library paths

Set the JavaScriptCore and curl-impersonate library directories before building
`lite-worker`:

```sh
export BRIMP_JSC_LIB_DIR=/path/to/javascriptcore/lib
export BRIMP_CURL_LIB_DIR=/path/to/curl-impersonate/lib
```

The JavaScriptCore directory must contain the library for your platform:

| Platform | Library layout |
| --- | --- |
| macOS | `JavaScriptCore.framework/JavaScriptCore` |
| Linux | `libJavaScriptCore.so` |
| Windows | `JavaScriptCore.lib`, with runtime DLLs alongside the executable |

Without an explicit path, JavaScriptCore discovery uses the adjacent
`WebKit/WebKitBuild/Release` directory, and curl discovery uses `/usr/local/lib`.
Prepared SDKs use the explicit environment paths emitted by the preparation
script.

### Package native libraries

`crates/controller/package_release.py` bundles native libraries and their
licenses with `lite-worker` in release archives. The CLI/controller executable
does not link these browser libraries.

## Build and test

From the repository root:

```sh
cargo build -p brimp-controller -p brimp-lite-worker
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

The Rust CDP workflow exercises the same framed transport used by the worker executable.

For a quicker interface-specific cycle:

```sh
cargo test -p brimp-controller -p brimp-lite-worker
```

## Pre-commit formatting

Install and enable the repository hook once per checkout:

```sh
git config core.hooksPath .githooks
```

The shell hook runs `cargo fmt --all --check` whenever staged Rust files are
committed.

## Work on the docs

Install the locked documentation dependencies:

```sh
cd docs
npm ci
```

Start the live development server:

```sh
npm run dev
```

Build and preview the production output:

```sh
npm run build
npm run preview
```

The static production site is written to `docs/dist/` and is configured for
`https://docs.brimp.ai`.

## Deploy the docs

The `docs-pages.yml` workflow deploys production builds to GitHub Pages. In the
repository's Pages settings, choose **GitHub Actions** as the source, set the
custom domain to `docs.brimp.ai`, and enable HTTPS. Configure this DNS record:

```text
docs  CNAME  lexiforest.github.io
```

The custom domain belongs in GitHub's Pages settings. GitHub ignores a
repository `CNAME` file when deployment uses a custom Actions workflow.

## Documentation rules

- Treat the controller and worker READMEs and public source types as authoritative.
- Do not document planned APIs as if they exist.
- Keep examples runnable against the current release.
- Update the relevant API page whenever a public interface changes.
- Record implementation progress in the repository's `TODO.md`.
