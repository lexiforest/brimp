---
title: Network transport
description: Use and develop the brimp-net HTTP and WebSocket transport.
---

`brimp-net` is a small Rust wrapper around `libcurl-impersonate`. It exposes an
HTTP client that uses `curl_easy_impersonate` for browser-like TLS and HTTP
transport while returning standard `http` responses and Hyper-compatible `http_body::Body` streams.

In addition to native impersonation targets, `Config::impersonation_target`
accepts named profiles from curl-cffi's `fingerprints.json` catalog. Brimp-net
looks in `$IMPERSONATE_CONFIG_DIR`, `$XDG_CONFIG_HOME/impersonate`, and then
`~/.config/impersonate`. Resolution happens once on the first request and is
shared by client clones, including resolution failures. Create a new client to
use changed catalog contents.

## Requirements

This crate uses the same dynamically linked curl SDK as the runtime.
CI builds curl-impersonate from source, including its BoringSSL WebCrypto
interface. See [native prerequisites](/development/#native-prerequisites). Set
`BRIMP_CURL_LIB_DIR` to the library directory (default `/usr/local/lib`).
The target platform determines the required library:

- `libcurl-impersonate.dylib`
- `libcurl-impersonate.so`
- `libcurl-impersonate_imp.lib` and the runtime DLL on Windows

The command-line `curl-impersonate` wrapper alone is not enough; the linked
library must export `curl_easy_impersonate`.

## Collected Responses

Use blocking `send_collect` for small responses that can be buffered in memory.
It calls `curl_easy_perform` on the calling thread. In async code, use
`send(...).await?.into_body().collect().await?` instead.

```rust
use brimp_net::{Client, Config, RedirectPolicy};
use http::{Method, Request};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new(Config {
        impersonation_target: "chrome150".to_string(),
        redirect_policy: RedirectPolicy::Follow,
        ..Config::default()
    });

    let request = Request::builder()
        .method(Method::GET)
        .uri("https://example.com/")
        .body(None)?;
    let response = client.send_collect(request)?;

    println!("{} {}", response.status, response.effective_url);
    Ok(())
}
```

## Streaming Responses

Use `send` when the caller should consume the response body incrementally.

```rust
use brimp_net::{Client, Config};
use http::Request;
use http_body_util::{BodyExt, Empty, Full};
use bytes::Bytes;

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new(Config::default());
    let request = Request::builder()
        .uri("https://example.com/")
        .body(Empty::<Bytes>::new())?;

    let mut response = client.send(request).await?;
    while let Some(frame) = response.body_mut().frame().await {
        if let Some(bytes) = frame?.data_ref() {
            println!("received {} bytes", bytes.len());
        }
    }

    let request = Request::builder()
        .method("POST")
        .uri("https://example.com/")
        .body(Full::new(Bytes::from_static(b"hello")))?;
    let _response = client.send(request).await?;

    Ok(())
}
```

## Configuration

`Config::default()` uses the `chrome150` impersonation target, 10
second connect timeout, 30 second request timeout, no redirect following, and
libcurl-impersonate default headers disabled. Callers own page-context headers
such as User-Agent, client hints, and cookies.

Set `default_headers` to `true` only when you want libcurl-impersonate to add
its profile default headers. When decoded response bodies are returned, `content-encoding`,
`content-length`, and `transfer-encoding` headers are removed because they no
longer describe the body bytes delivered to Rust.

### Proxies

Set `Config::proxy` with a parsed `Proxy` to route every transfer through an
HTTP or SOCKS5 proxy. HTTP proxies tunnel HTTPS requests with CONNECT.
`socks5://` resolves destination names locally; `socks5h://` delegates that
resolution to the proxy.

```rust
use brimp_net::{Client, Config, Proxy};

let client = Client::new(Config {
    proxy: Some(Proxy::parse("socks5h://proxy.example:1080")?),
    ..Config::default()
});
# let _ = client;
# Ok::<(), Box<dyn std::error::Error>>(())
```

## Execution and limits

Async `send` uses one curl multi coordinator per client. Clones share that
coordinator and its connection cache. HTTP/2 multiplexing is enabled; the crate
no longer forces all requests to a host onto one connection. Standard `http`
and `http-body` types provide interoperability without depending on Hyper's
HTTP implementation.

Response bytes use a 16-chunk channel. A full channel pauses that curl transfer;
reading a chunk wakes the coordinator to resume it. This uses
[curl pause/resume](https://curl.se/libcurl/c/curl_easy_pause.html), which replays
the unconsumed chunk on resume. No blocking channel send
runs on the shared curl thread. Small chunks are delivered immediately.
The normal streaming path does not retain a second copy of the entire body.
Curl can buffer additional data internally, particularly for decoded or
multiplexed responses; the channel capacity is not a total process memory limit.

Dropping a pending send future or response body cancels that transfer. Dropping
the last client closes its coordinator and cancels outstanding response bodies;
keep a client alive while consuming responses. Completion errors are delivered
after queued body bytes, including when the data channel was full.

The submission queue holds at most 256 commands; overload returns an error.
At most 64 idle easy handles are retained. Request bodies are currently collected
before submission: streaming uploads and request trailers are not implemented.
Response trailers are not exposed. `send_collect` and explicit response
collection buffer the whole response and do not impose a total body-size limit.
Callers must enforce their own collection limits.

`BRIMP_NET_TRACE` enables transport diagnostics. `BRIMP_NET_CAPTURE_DIR` enables
full-body diagnostic capture; `BRIMP_NET_CAPTURE_HOST` filters captured URLs.
Capture is opt-in and intentionally retains bodies, so it changes memory and I/O
costs. `BRIMP_CURL_LIB_DIR` selects the library directory for linking.

## Browser integration boundary

The runtime uses this crate for production HTTP and WebSocket transport.
Standard HTTP request/response/body types remain the transport boundary. The
runtime owns resource classification, session cookies, redirects, interception,
authentication callbacks, body limits, navigation cancellation, and `data:` URLs.

Use `Credentials` and `OrderedHeaders` request extensions for negotiated
origin/proxy authentication and explicit header ordering. `Config` supports a
custom CA bundle, HTTP/3 preference and a bounded submission queue. Response
extensions contain `TransferInfo`; read `snapshot()` after body EOF for finalized
wire byte counters. WebSockets use `open_websocket` and a separate handle/event
interface; they are not HTTP bodies or part of the runtime's ResourceLoader trait.

The runtime shares one asynchronous dispatch runtime per session for resource
policy. Standalone CurlResourceLoader users get an owned dispatcher. The
brimp-net client owns its curl multi coordinator and connection cache.
See [comparison measurements](https://github.com/lexiforest/brimp/blob/main/benchmark/network/README.md) for the
recorded runtime-adapter comparison and its limits.

## Validation

```sh
cargo test -p brimp-net
cargo clippy -p brimp-net --all-targets -- -D warnings
```

Tests use loopback HTTP servers and need permission to bind local ports. They
cover concurrent requests, slow readers and pause/resume byte integrity, early
chunk delivery, cancellation, client closure, redirects, and transfer errors.
No JavaScriptCore SDK is required to build or test this crate alone.
