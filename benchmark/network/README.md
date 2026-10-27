# HTTP transport comparison

This benchmark measures the runtime resource adapter against direct brimp-net
requests. Both paths use the same curl multi transport. Timings include
adapter/body collection costs and run-to-run variation.

Recorded on 2026-10-21, macOS 27.0 / arm64, Rust 1.94.1, debug profile.
Both clients use the installed curl-impersonate library and its native chrome136
profile, with default headers disabled and no redirects or proxies. HTTP/3
preference is disabled for this HTTP/1.1 comparison.

Each workload uses a local HTTP/1.1 keep-alive server. After a warmup batch for
each client, seven measured batches alternate execution order. Timings include
receiving and collecting all response bytes and checking their contents. They
exclude compilation, client initialization and warmup. The server runs in the
same process. Clients and connections are reused across batches.

| Workload | Concurrent requests | Bytes per response | Runtime adapter median batch (ms) | Direct brimp-net median batch (ms) |
| --- | ---: | ---: | ---: | ---: |
| Small serial | 1 | 1,024 | 0.153 | 0.136 |
| Small concurrent | 16 | 1,024 | 2.078 | 2.068 |
| Concurrent, 30 ms server delay | 16 | 1,024 | 36.300 | 36.851 |
| Bulk concurrent | 8 | 2,097,152 | 35.937 | 36.238 |

Raw measurements: [integrated.json](integrated.json). These local samples do not
establish general performance parity or production throughput. They do not
measure TLS, HTTP/2 or HTTP/3, remote latency, CPU utilization, peak memory,
cold startup, streaming uploads or browser page loading. Debug-build validation
and allocation costs are included, especially in the bulk case.

Fingerprint catalog resolution runs once per client on its first request.
Clones share the result, including errors. Warmup resolves it before measurement.

## Reproduce

```sh
cargo build -p brimp-runtime --example network_benchmark
DYLD_FRAMEWORK_PATH="$PWD/../WebKit/WebKitBuild/Release" \
  target/debug/examples/network_benchmark > benchmark/network/integrated.json
```

Set native SDK paths as described in the runtime documentation. The example
links the runtime and needs JSC, although it does not create JS realms.
brimp-net itself has no JSC dependency. Do not run competing builds or
benchmarks while measuring.

The benchmark verifies every response's size and contents. Independent
brimp-net tests require concurrent same-host requests to reach the server before
releasing responses and verify that an unread large response does not block a
second request. These checks establish concurrency behavior without timing ratios
as correctness assertions.
