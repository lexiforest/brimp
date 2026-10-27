# brimp-net

Rust HTTP and WebSocket transport built on `libcurl-impersonate`, using standard
`http` and `http-body` types. Requires the curl-impersonate SDK; JavaScriptCore
is not needed to build or test this crate.

See the [network transport guide](https://docs.brimp.ai/development/network/)
for setup, examples, configuration, streaming behavior and runtime integration.

From the repository root:

```sh
cargo test -p brimp-net
cargo clippy -p brimp-net --all-targets -- -D warnings
```
