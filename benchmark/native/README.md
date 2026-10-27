# Native runtime measurements

Recorded on macOS 27 / arm64 with a debug build and 20 iterations per workload.
Each workload ran sequentially. Time excludes compilation; process peak RSS
includes startup. These are single recorded samples, not a statistical throughput
claim or measurements of every runtime revision.

| Workload | Elapsed time (ms) | Peak RSS (MiB) |
| --- | ---: | ---: |
| DOM parse/select/mutate/serialize | 223.73 | 17.05 |
| Python request/script/query/edit/result | 1,858.45 | 76.81 |
| Node request/script/query/edit/result | 1,844.97 | 114.78 |

The DOM workload creates 3,000 articles with nested markup per document,
selects them, mutates 100 inline declarations, and serializes the document.
The bindings each reuse a page across 20 local HTTP requests containing 3,000
elements, execute a script, query/edit/read a node, and check the collection size.
The binding timings include the local fixture server. The standalone DOM process
does not initialize JSC, unlike the binding workloads.

The recorded samples used the default macOS dynamic-loader environment.
The local JSC SDK retains the system framework install name, so that environment
loads the system framework. Set DYLD_FRAMEWORK_PATH to select a specific JSC SDK
and record that choice alongside any new measurements.

Raw results: [final.json](final.json).
Python's ru_maxrss is bytes on macOS; Node reports KiB.

## Reproduce

Build and prepare the native extensions, then run:

```sh
CARGO_INCREMENTAL=0 cargo build -p brimp-runtime --example dom_benchmark
/usr/bin/time -l target/debug/examples/dom_benchmark
PYTHONPATH=bindings/python/python python3 benchmark/native/python_benchmark.py
node benchmark/native/node_benchmark.mjs
```

Run sequentially without concurrent builds. On other systems, use the local
equivalent of `/usr/bin/time -l` and normalize peak RSS units. Record the host,
build profile, native library selection and source revision with results.
