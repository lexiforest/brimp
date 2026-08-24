# network

Transport-neutral resource loading for Brimp.

The `ResourceLoader` trait is the single boundary used for navigation, CSS,
scripts, images, fonts, and JavaScript `fetch()`. Tests and embedders can provide
their own loader without coupling DOM or binding code to an HTTP client.

`CurlResourceLoader` is the default implementation. It uses the adjacent
`bimp-net` libcurl-impersonate wrapper with a Chrome profile, redirect following,
browser-like headers, and HTTP/2 negotiation. Transfers run on worker threads;
they do not enter JavaScriptCore directly.

## Core API

```rust
use network::{ResourceLoader, ResourceRequest};

async fn load(loader: &dyn ResourceLoader) -> Result<Vec<u8>, network::NetworkError> {
    let response = loader
        .fetch(ResourceRequest::get("https://example.com/"))
        .await?;
    Ok(response.body)
}
```

The integration test binds a loopback listener, so its environment must permit
local networking:

```sh
cargo test -p network
```
