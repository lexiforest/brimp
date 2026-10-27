---
title: Package native bindings
description: Assemble and verify relocatable Python and Node packages.
---

Development builds link against the installed SDKs. For macOS distribution,
assemble relocatable packages from an already built wheel and Node extension:

```sh
cargo build -p brimp-python -p brimp-node
python3 bindings/prepare_local.py
maturin build --locked --manifest-path bindings/python/Cargo.toml --out /tmp/brimp-wheels
python3 bindings/package_native.py \
  --wheel /tmp/brimp-wheels/brimp-0.1.1-cp311-abi3-macosx_11_0_arm64.whl \
  --jsc ../WebKit/WebKitBuild/Release/JavaScriptCore.framework/JavaScriptCore \
  --curl .native/curl-sdk/lib/libcurl-impersonate.dylib \
  --notices ../WebKit/Source/JavaScriptCore \
  --notices ../WebKit/Source/WTF \
  --notices ../curl-impersonate/LICENSE \
  --notices ../curl-impersonate/build/deps/src \
  --out /tmp/brimp-packages
```

Supply the SDK libraries and notices matching the actual build. The assembler
copies inputs into temporary staging directories, rewrites their load commands,
ad-hoc signs the copies, rebuilds wheel RECORD hashes, and emits SHA-256 files.
It never edits the SDK or the original extension. Input and bundled library
hashes, dependencies, architecture and minimum OS are recorded in
`native/manifest.json`. Archive ordering, ownership and timestamps are fixed.
Unknown non-system dependencies fail assembly instead of silently producing an
incomplete package. SDK notices are copied from the supplied files/directories.
All native inputs must contain the single current-host architecture; universal
binaries and cross-compilation are outside this assembler's contract.

The wheel platform tag is derived from the maximum minimum-macOS version across
the extension and supplied libraries. The local development SDK currently
requires macOS 27, so those artifacts are tagged `macosx_27_0_arm64`; they must
not be advertised as macOS 11-compatible. Build SDKs targeting an older supported
OS to produce an older-OS package. The npm package declares its platform/CPU;
its manifest records the minimum OS.

Test the assembled wheel and npm archive outside the source checkout, with
`DYLD_FRAMEWORK_PATH`, `DYLD_LIBRARY_PATH`, `PYTHONPATH` and SDK-path overrides
removed. The bundled libraries should be sufficient. This is package relocation,
not Apple notarization or a claim of release verification on other OS versions.
Linux and Windows retain the existing SDK-linked workflow. Their native bundling
has not been implemented or verified by this macOS assembly step.
