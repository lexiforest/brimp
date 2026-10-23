# Brimp CLI

`brimp` embeds `brimp-runtime` and uses its sessions, page owners, network
transport and plugins. It provides `doctor`, `fetch`, `run`, and plugin discovery.

Build with the native SDK paths described in [runtime setup](../runtime/README.md):

```sh
cargo build --release -p brimp-cli
export DYLD_FRAMEWORK_PATH="$PWD/../WebKit/WebKitBuild/Release" # local macOS SDK
./target/release/brimp doctor
./target/release/brimp fetch https://example.com --eval document.title
./target/release/brimp run script.js
./target/release/brimp plugins list
```

Use `cargo install --path crates/cli --locked` to install into Cargo's bin directory.
The executable requires the configured JavaScriptCore and curl-impersonate SDKs
at runtime. Build-script rpaths select their directories on Unix. Distributing a
standalone executable also requires those native libraries and their notices;
this crate does not create a relocatable release archive. Bundled plugin and
native dependency licenses are copied to the crate's Cargo build output directory.

The [CLI reference](../../docs/src/content/docs/api/cli.md) documents options,
execution order, JSON actions/routes, output contracts and limitations.
`brimp help fetch` and `brimp help run` list every option.

```sh
cargo build -p brimp-cli
python -m pytest crates/cli/tests/test_cli.py
cargo clippy -p brimp-cli --all-targets -- -D warnings
```

Tests use local HTTP fixtures. `BRIMP_CLI` selects an alternate built executable.
