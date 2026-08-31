---
title: Node.js API
description: Reference for the asynchronous @brimp/brimp package.
---

```js
const brimp = require('@brimp/brimp')
```

The Node binding is asynchronous and in process. Pages in one session run
concurrently while sharing its cookie jar.

## `brimp.get()`

```ts
get(url: string | URL, options?: SessionOptions & PageOptions & GetOptions): Promise<Response>
```

Creates a temporary session, performs one GET navigation, closes the native
session, and returns a detached response.

## `createSession()`

```ts
createSession(options?: {
  personaJson?: string
  caBundle?: string
  enableWorker?: boolean
  enableStreamingNetworking?: boolean
  enableCanvas?: boolean
  enableWebGL?: boolean
  enableWebGPU?: boolean
  enableWebAudio?: boolean
  enableWebAudioOutput?: boolean
  storagePath?: string
  storageQuotaBytes?: number
}): Promise<Session>
```

Creates one persistent browsing session. `caBundle` trusts private certificate
authorities without disabling verification. Browser subsystems are disabled by
default. Supplying `storagePath` enables persistent storage with a 1 GiB default
quota.

## `Session.newPage()`

```ts
session.newPage({ proxy?: string }): Promise<Page>
```

Creates an independently concurrent page. Its direct, HTTP, SOCKS5, or SOCKS5H
proxy is immutable and covers redirects, subresources, Fetch, workers,
streaming requests, and WebSockets for the page's lifetime.

## `Page.get()`

```ts
page.get(url, {
  params,
  headers,
  cookies,
  timeoutMs: 30_000,
  signal,
}): Promise<Response>
```

Performs a GET navigation with a fresh JavaScript realm. Query values may be
scalars or arrays. Session headers and cookies are merged with request values;
request values take precedence. Cookies are inserted into the browser-managed
jar before navigation, so normal scoping and redirect rules apply. `User-Agent` and `Accept-Language` are
persona-owned. An `AbortSignal` cancels the core operation and network request.

## Page utilities

```ts
page.evaluate(source): Promise<unknown>
page.screenshot({ path?, fullPage? }): Promise<Buffer>
page.extract({ contentSelector?, removeImages?, language?, debug? }): Promise<ExtractedDocument>
page.click(selector): Promise<void>
page.hover(selector): Promise<void>
page.type(selector, text): Promise<void>
page.tap(selector): Promise<void>
page.close(): Promise<void>
session.close(): Promise<void>
```

Evaluation returns JSON-compatible values. Screenshots return PNG bytes and
write the same bytes when `path` is provided. Extraction runs the pinned
Defuddle browser bundle against the current live DOM and returns content,
Markdown, and metadata. Input methods hit-test and send
trusted browser input events; `hover` moves without pressing and `type` focuses the matched control first. A
missing selector rejects with code `invalid_input`. Closing either level is idempotent;
operations after close fail with code `closed`.

## `Response`

| Member | Meaning |
| --- | --- |
| `statusCode`, `reason`, `url` | Final response status and URL. |
| `headers` | Case-insensitive `Headers` collection with `get()`, `getAll()`, and `raw`. |
| `content`, `text` | Original response `Buffer` and decoded text. |
| `html` | Post-JavaScript serialized DOM for HTML, otherwise `null`. |
| `cookies`, `elapsed`, `ok` | Response cookies, elapsed seconds, and status below 400. |

`response.json()` decodes the response text. `response.raiseForStatus()` throws
`HTTPError` for 4xx and 5xx responses.

## Errors

All errors derive from `BrimpError` and carry a stable `code`. Specialized
classes include `ConnectionError`, `Timeout`, `TooManyRedirects`,
`InvalidRequest`, `InvalidURL`, `HTTPError`, and `JavaScriptError`.

The initial API supports GET only. Use [`brimp cdp`](/api/cdp/) when a
Puppeteer or Playwright browser/page interface is required.
