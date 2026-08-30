---
title: Node.js API
description: Reference for the asynchronous @brimp/brimp package.
---

```js
const brimp = require('@brimp/brimp')
```

The Node binding is asynchronous and in process. Sessions are sequential; use
separate sessions for concurrent browsing.

## `brimp.get()`

```ts
get(url: string | URL, options?: SessionOptions & GetOptions): Promise<Response>
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

## `Session.get()`

```ts
session.get(url, {
  params,
  headers,
  cookies,
  timeoutMs: 30_000,
  signal,
}): Promise<Response>
```

Performs a GET navigation with a fresh JavaScript realm. Query values may be
scalars or arrays. Session headers/cookies are merged with request values;
request values take precedence. `User-Agent` and `Accept-Language` are
persona-owned. An `AbortSignal` cancels the core operation and network request.

## Session utilities

```ts
session.evaluate(source): Promise<unknown>
session.screenshot({ path?, fullPage? }): Promise<Buffer>
session.click(selector): Promise<void>
session.hover(selector): Promise<void>
session.type(selector, text): Promise<void>
session.tap(selector): Promise<void>
session.close(): Promise<void>
```

Evaluation returns JSON-compatible values. Screenshots return PNG bytes and
write the same bytes when `path` is provided. Input methods hit-test and send
trusted browser input events; `hover` moves without pressing and `type` focuses the matched control first. A
missing selector rejects with code `invalid_input`. Closing is idempotent;
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
