# jsc-sys

Raw Rust FFI declarations for JavaScriptCore's public C API.

This crate owns the unsafe ABI boundary used by Brimp. It defines opaque JSC
handle types and functions plus platform-specific dynamic linkage; higher-level
code should normally use the safe-ish `jsc` crate instead.

## Linking

The build script expects `BRIMP_JSC_LIB_DIR` to contain:

- `JavaScriptCore.framework/JavaScriptCore` on macOS;
- `JavaScriptCore.lib` on Windows; or
- `libJavaScriptCore.so` on Linux.

It defaults to the adjacent WebKit release build used for local macOS
development. Override it for a packaged SDK:

```sh
BRIMP_JSC_LIB_DIR=/path/to/jsc-sdk/lib cargo check -p jsc-sys
```
