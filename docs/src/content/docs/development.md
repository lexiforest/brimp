---
title: Development
description: Build, test, and work on Brimp and its documentation.
---

## Repository layout

```text
crates/web-runtime/   Canonical browser automation runtime
crates/web-bindings/  JavaScript and Web API bindings
crates/browser-dom/   DOM, style, and layout integration
crates/network/       curl-impersonate transport
crates/screenshot/    CPU PNG rendering
crates/cli/           brimp executable
crates/cdp/           CDP protocol library and conformance workflow
bindings/python/      CPython ABI3 package
bindings/node/        Node native addon and JavaScript adapter
persona/              Versioned persona schema
docs/                 This Starlight site
```

`web-runtime` is the only browser implementation. Every external interface
delegates navigation, JavaScript, lifecycle, and screenshots to its owner-thread
automation API.

## Native prerequisites

Source builds dynamically link JavaScriptCore and curl-impersonate. Set:

```sh
export BRIMP_JSC_LIB_DIR=/path/to/javascriptcore/lib
export BRIMP_CURL_LIB_DIR=/path/to/curl-impersonate/lib
```

`BRIMP_JSC_LIB_DIR` must contain `JavaScriptCore.framework` on macOS,
`JavaScriptCore.lib` on Windows, or `libJavaScriptCore.so` on Linux.

## Build and test

From the repository root:

```sh
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
./bindings/package-test.sh
./crates/cdp/puppeteer-test.sh
```

The package test is the combined macOS ARM64 Python and Node check. The
The client workflow installs its exact locked Puppeteer and Playwright
dependencies into a temporary directory and connects both to `brimp cdp`.

For a quicker interface-specific cycle:

```sh
cargo test -p brimp-cli -p brimp-cdp
python3 bindings/python/test_api.py
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

- Treat each binding's `SUPPORT.md` and public source types as authoritative.
- Do not document planned APIs as if they exist.
- Keep examples runnable against the current release.
- Update the relevant API page whenever a public interface changes.
- Record implementation progress in the repository's `TODO.md`.

## Architecture constraints

Keep transport mechanics behind `network::ResourceLoader`; do not let DOM or
JavaScript bindings depend directly on a concrete HTTP client. Keep native
bindings and CDP as adapters over `web-runtime`, not alternate browser
implementations. Remove obsolete paths when interfaces change rather than
maintaining compatibility layers.
