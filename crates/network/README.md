# network

Transport-neutral resource loading for Brimp.

The `ResourceLoader` trait is the single boundary used for navigation, CSS,
scripts, images, fonts, and JavaScript `fetch()`. Tests and embedders can provide
their own loader without coupling DOM or binding code to an HTTP client.

`CurlResourceLoader` is the default implementation. It owns a bounded,
long-lived libcurl-impersonate multi executor and pools reset easy handles.
Loader clones share that executor; transfers do not create request threads or
enter JavaScriptCore.

The default `chrome136` transport profile does not install curl's HTTP header
set. The resolved browsing-context persona supplies User-Agent and language
headers in insertion order, keeping transport and JavaScript identity aligned.
Curl's cookie engine and redirect
following are disabled: `web-runtime` owns cookies and applies every redirect
hop. Configuration is immutable for a loader's lifetime, and another loader is
required for a different profile, proxy, timeout, queue bound, or body limit.

Native discovery uses `/usr/local/lib` by default. Set `BRIMP_CURL_LIB_DIR` to
another directory or `BRIMP_CURL_STATIC=1` to prefer the static library.

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
