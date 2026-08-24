# jsc-sys

Raw Rust FFI declarations for JavaScriptCore's public C API.

This crate owns the unsafe ABI boundary used by Brimp. It defines opaque JSC
handle types, public C functions, and macOS framework linkage; higher-level code
should normally use the safe-ish `jsc` crate instead.

## Linking

The build script currently targets macOS and looks for:

```text
../WebKit/WebKitBuild/Release/JavaScriptCore.framework/JavaScriptCore
```

Override the framework directory when necessary:

```sh
BRIMP_JSC_FRAMEWORK_DIR=/path/to/WebKitBuild/Release cargo check -p jsc-sys
```

Only JavaScriptCore's public C API is bound. Private WebKit C++ interfaces and
static JSC linkage are intentionally outside this crate's scope.
