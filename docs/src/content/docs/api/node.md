---
title: Node.js API
description: Asynchronous curl-shaped navigation with a live JavaScript Page.
---

```js
const brimp = require('@brimp/brimp')
```

Every request resolves to a live `Page`, not a detached response.

## Module helpers

`request`, `get`, `head`, `options`, `delete`, `post`, `put`, and `patch`
create a private Session and return its owning Page. Always close it:

```js
const page = await brimp.get('https://example.com')
try {
  console.log(page.statusCode, page.text, page.html)
  await page.click('#continue')
} finally {
  await page.close()
}
```

## Session

```js
const session = await brimp.createSession({
  headers: { 'X-Agent': 'brimp' },
  cookies: { manual: 'yes' },
  auth: ['agent', 'secret'],
  proxy: 'socks5h://127.0.0.1:1080',
  timeoutMs: 30_000,
  allowRedirects: true,
  maxRedirects: 30,
})
```

A Session owns the shared browser context, cookies, direct transport, coherent
persona, and request defaults. Its request/verb helpers create and return a new
live Page. `newPage({ proxy })` creates an un-navigated Page whose network scope
is immutable.

Browser subsystem options include `enableWorker`,
`enableStreamingNetworking`, `enableCanvas`, `enableWebGL`, `enableWebGPU`,
`enableWebAudio`, optional system audio output, and persistent storage.

## Page navigation

```js
await page.request(method, url, {
  params,
  data,
  content,
  json,
  multipart,
  headers,
  cookies,
  auth,
  timeoutMs,
  allowRedirects,
  maxRedirects,
  referer,
  signal,
})
```

All verb helpers resolve to the same Page. Form `data`, raw `content`, JSON,
and explicit `Multipart` bodies are mutually exclusive. GET and HEAD reject
bodies. AbortSignal cancellation applies to the complete rendered navigation.

The Page exposes latest `statusCode`, `reason`, `url`, `headers`, `content`,
`text`, `html`, `cookies`, `elapsed`, `ok`, `lastRequest`, `history`, and
`redirectCount`, `httpVersion`, `downloadedBytes`, `uploadedBytes`, and
`headerBytes`, plus `json()` and `raiseForStatus()`. Awaited evaluation and input
operations refresh the cached `html` serialization.

## Browser operations

```js
await page.evaluate(source)
await page.screenshot({ path, fullPage })
await page.extract({ contentSelector, removeImages, language, debug })
await page.hover(selector)
await page.click(selector)
await page.type(selector, text)
await page.tap(selector)
```

## Lifecycle and errors

`Page.close()` closes one Page; for a module-helper Page it also closes the
private Session. `Session.close()` closes all Pages and native resources.

Errors derive from `RequestError` and carry stable `code` values. `HTTPError`
exposes the failing live Page as `error.page`.

Transport fingerprints, persona headers, TLS trust, and proxy affinity are
scope-level browser decisions rather than arbitrary per-request curl options.
