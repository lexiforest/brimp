---
title: Node.js API
description: Reference for the asynchronous @brimp/brimp package.
---

```js
const { launch, BrimpError } = require('@brimp/brimp')
```

The Node binding is an asynchronous, in-process native addon.

## `launch()`

```ts
launch(options?: { personaJson?: string }): Promise<Browser>
```

Creates a browser backed by the shared runtime. `personaJson` is JSON text using
Brimp's versioned persona schema.

## `Browser`

### `browser.newPage()`

```ts
newPage(options?: {
  enableWorker?: boolean
  enableStreamingNetworking?: boolean
  storagePath?: string
  storageQuotaBytes?: number
}): Promise<Page>
```

Creates an `about:blank` page with its own owner thread. Worker, streaming
networking, and persistent-storage APIs are absent unless their page options
enable them. `storagePath` enables persistence with a 1 GiB default quota.

### `browser.close()`

```ts
close(): Promise<void>
```

Closes child pages and then the browser. Closing more than once is safe.

## `Page`

### `page.goto()`

```ts
goto(
  url: string,
  options?: { timeoutMs?: number; signal?: AbortSignal },
): Promise<void>
```

Navigates the page. The default timeout is 30,000 milliseconds. Aborting the
signal cancels the core operation and its network request.

### `page.evaluate()`

```ts
evaluate(expression: string): Promise<unknown>
```

Evaluates JavaScript and returns a JSON-compatible value.

### Document output

```ts
title(): Promise<string>
textContent(): Promise<string>
```

`title()` returns the current document title. `textContent()` returns the
canonical document text output.

### `page.screenshot()`

```ts
screenshot(options?: { fullPage?: boolean }): Promise<Buffer>
```

Returns a PNG `Buffer` for the viewport or full page.

### `page.close()`

```ts
close(): Promise<void>
```

Closes page resources. Closing more than once is safe.

## Errors

Operations reject with `BrimpError`. Its `code` is one of:

```text
invalid_input  transport  http_status  navigation  javascript
timeout        cancelled  unsupported  closed      screenshot  internal
```

## Current boundary

The native Node API does not expose locators, browser modes, raw CDP dispatch,
or a request/response object. Use [`brimp cdp`](/api/cdp/) when the supported
Puppeteer interface is the better fit.
