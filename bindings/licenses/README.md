# Third-party notices

`scripts/prepare_native.py` copies these notices into the generated
`BRIMP_LICENSE_DIR`. The binding build scripts include that directory under
`bundled/licenses/native` in Python wheels and Node packages.

- `skia-LICENSE`: BSD-3-Clause notice for the Skia library linked statically by
  `skia-safe`/`skia-bindings`.
- `gcc-runtime/COPYING3` and `gcc-runtime/COPYING.RUNTIME`: GCC's GPLv3 and
  Runtime Library Exception 3.1 notices, copied by the Linux packaging helpers
  when bundling libatomic. Sourced from the GCC 14.2.0 release's license files
  in https://github.com/gcc-mirror/gcc/tree/releases/gcc-14.2.0.
