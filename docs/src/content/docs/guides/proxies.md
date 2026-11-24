---
title: Proxies and request filtering
description: Choose proxies and filter browser requests from Python.
---

Set a session proxy to cover navigation and requests made by page JavaScript:

```python
import brimp

with brimp.Session(
    proxy="http://proxy.example:8080",
    proxy_auth=("username", "password"),
    trust_env=False,
) as session:
    with session.get("https://example.com/") as page:
        page.raise_for_status()
        print(page.status_code)
```

Supported proxy URLs are `http://`, `https://`, `socks5://` and `socks5h://`.
HTTP proxies tunnel HTTPS destinations with CONNECT. HTTPS proxies also encrypt
the connection to the proxy. `socks5://` resolves destination hostnames locally;
`socks5h://` resolves them through the proxy. Proxy credentials are separate from
origin `auth` credentials.

## Override a request

Pass `proxy`, `proxies`, `proxy_auth` or `trust_env` to `session.get()`, other HTTP
methods, or `page.stream()`. These options also work with `brimp.get()` and
`brimp.AsyncSession`.

- Omitted options and `proxy=None` inherit session settings.
- `proxy=""` forces a direct connection.
- A navigation override applies to its main document and redirects. Scripts,
  stylesheets and later JavaScript requests use session settings and routes.
- Changing the proxy does not carry the previous proxy's separate credentials
  along. Supply `proxy_auth` with the new proxy when needed.

Use `proxies` instead of `proxy` to select by scheme or hostname:

```python
import brimp

with brimp.Session(
    proxies={
        "all": "socks5h://proxy.example:1080",
        "https://api.example.com": "http://api-proxy.example:8080",
        "all://localhost": "",
    },
    trust_env=False,
) as session:
    with session.get("https://api.example.com/items") as page:
        print(page.status_code)
```

Keys are checked in this order: `scheme://hostname`, `all://hostname`, `scheme`,
`all`. Host keys omit ports and paths. A matching request setting takes precedence
over a session setting. WebSockets use the `http` mapping for `ws` and `https`
for `wss`.

`trust_env=True` is the default. When no configured proxy matches, Brimp reads
`http_proxy`, `https_proxy` and `all_proxy`, with `no_proxy` exclusions. Uppercase
variants work except `HTTP_PROXY` on Unix; lowercase values take precedence.
Windows environment names are case-insensitive, so `HTTP_PROXY` and `http_proxy`
select the same proxy. Explicit
proxy settings, including route overrides, take precedence over environment
settings and their exclusions. `trust_env=False` disables environment proxy use.

## Route by resource type

Register `session.route()` before navigation to cover all existing and future
pages. The callback can block a request, supply a response, or select a proxy:

```python
import brimp
from urllib.parse import urlsplit


def policy(route):
    request = route.request
    host = urlsplit(request["url"]).hostname
    if host == "analytics.example.com":
        route.fail("analytics blocked")
    elif request["resource_type"] in {"document", "fetch", "xhr"}:
        route.continue_request(proxy="socks5h://proxy.example:1080")
    else:
        route.continue_request(proxy="")


with brimp.Session(trust_env=False) as session:
    remove_route = session.route("*", policy)
    with session.get("https://example.com/") as page:
        print(page.status_code)
    remove_route()
```

Resource types identify the initiator: `document`, `script`, `stylesheet`,
`fetch`, `xhr`, `eventsource` and `websocket`. Frame documents and scripts keep
the corresponding document/script type. Brimp does not fetch images or fonts
solely to make them available to filters. A JavaScript `fetch()` remains `fetch`
regardless of the URL's extension.

Patterns use glob matching. A matching page route takes precedence over a
session route at the same stage; within either scope, the last registration
wins. The selected handler completes the request exactly once. There is no
implicit chain of handlers. Redirects are routed again at each destination.

Use `route.continue_request(proxy=..., proxy_auth=(...))` to choose authenticated
proxies. Use the callback's own state to select from a proxy pool or keep a
hostname on one proxy. Brimp does not rotate proxies or retry directly when a
proxy fails. Cookies remain shared within the session when the proxy changes.

Request routes can continue or fail WebSocket connections. They cannot fulfill a
WebSocket handshake, and response/authentication routes do not intercept it.
Proxy overrides are only valid at the request stage.

For asyncio, register with `await session.route(pattern, handler)`. The handler
and returned removal function remain synchronous. As with page routes, handlers
run on the session callback dispatcher and cannot call back into that session.

Matching routes bypass cache reads and writes, so cached content cannot skip a
filter or retain a supplied response after its route is removed. Other cached
responses and transport connections are separated by proxy and credentials.
