---
title: CDP API
description: HTTP discovery, WebSocket behavior, and supported Chrome DevTools methods.
---

`brimp cdp` exposes HTTP discovery and a bounded WebSocket server. Protocol
values are JSON and screenshots are base64 encoded. This is the only Brimp
interface that crosses a remote server/client boundary.

## Start the server

```sh
brimp cdp --bind 127.0.0.1:9222
```

The server prints its browser WebSocket URL after binding. Use the HTTP origin
with Puppeteer's `browserURL` option or connect directly to the WebSocket URL.

## Connect with Playwright

Install `playwright-core`; Brimp supplies the browser process, so Playwright
does not need to download Chromium:

```sh
npm install playwright-core
```

Start Brimp in one terminal:

```sh
brimp cdp --bind 127.0.0.1:9222
```

Then connect through Playwright's public CDP API. The endpoint may be the HTTP
origin shown below or the `ws://.../devtools/browser/...` URL printed by Brimp.

```js
import { chromium } from 'playwright-core'

const browser = await chromium.connectOverCDP('http://127.0.0.1:9222')

try {
  const context = browser.contexts()[0] ?? await browser.newContext()
  const page = await context.newPage()

  await page.setViewportSize({ width: 1280, height: 720 })
  await page.goto('https://example.com', { waitUntil: 'load' })

  console.log(await page.evaluate(() => document.title))
  await page.screenshot({ path: 'example.png' })
  await page.close()
} finally {
  await browser.close()
}
```

`connectOverCDP()` is the supported Playwright connection path. Do not call
`chromium.launch()`: that asks Playwright to start a Chromium executable rather
than attach to Brimp. Playwright documents CDP attachment as a lower-fidelity
connection than its native protocol; only the Brimp methods listed below are
available. See Playwright's official
[`BrowserType.connectOverCDP()` documentation](https://playwright.dev/docs/api/class-browsertype#browser-type-connect-over-cdp).

## Client support

| Client | Connection API | Status |
| --- | --- | --- |
| Playwright | `chromium.connectOverCDP(httpOrWsEndpoint)` | Tested against the repository's locked `playwright-core` workflow. |
| Puppeteer | `puppeteer.connect({ browserURL })` | Tested against the repository's locked `puppeteer-core` workflow. |
| Raw CDP | Discovery HTTP plus flattened WebSocket sessions | Tested by the Rust protocol workflow. |
| Chrome DevTools UI and arbitrary CDP tooling | Varies | Not a compatibility target; unsupported methods fail explicitly. |

## Request interception

Puppeteer's public interception API works over the CDP connection. A handler
must resolve every request by continuing, responding, or aborting it:

```js
await page.setRequestInterception(true)
page.on('request', request => {
  if (request.url().endsWith('/agent-data')) {
    void request.respond({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({ source: 'agent' }),
    })
  } else {
    void request.continue()
  }
})
```

Raw clients can use modern `Fetch.enable` plus `Fetch.requestPaused`, followed
by `Fetch.continueRequest`, `Fetch.fulfillRequest`, or `Fetch.failRequest`.
Legacy `Network.setRequestInterception` and
`Network.continueInterceptedRequest` are also supported. Interception applies at
the request stage to navigations, parser subresources, redirects, and JavaScript
Fetch requests. Brimp supports URL patterns but not resource-type filters,
response-stage interception, or authentication challenges. A target may have
at most 256 paused requests, and a fulfilled body may be at most 32 MiB.

## Supported methods

The following list is exhaustive. An absent method returns CDP error `-32601`.

| Method | Behavior |
| --- | --- |
| `Browser.getVersion` | Reports Brimp and protocol identity. |
| `Browser.getWindowForTarget` | Returns the single headless window and validates an optional target. |
| `Browser.getWindowBounds` | Returns the single headless window's bounds. |
| `Browser.setDownloadBehavior` | Accepted for Playwright initialization; downloads are not implemented. |
| `Target.getBrowserContexts` | Lists live non-default logical browser contexts. |
| `Target.createBrowserContext` | Creates a logical page group without cookie/storage isolation. |
| `Target.disposeBrowserContext` | Closes pages in and removes a logical context. |
| `Target.setDiscoverTargets` | Controls target discovery events. |
| `Target.setAutoAttach` | Stores flattened auto-attach policy and applies it to new pages. |
| `Target.getTargets` | Lists live page targets. |
| `Target.getTargetInfo` | Describes a live page target. |
| `Target.createTarget` | Creates an `about:blank` Brimp page. |
| `Target.attachToBrowserTarget` | Creates a flattened browser-control session. |
| `Target.attachToTarget` | Creates a flattened session. |
| `Target.detachFromTarget` | Invalidates a session and emits its detach event. |
| `Target.closeTarget` | Closes a page and invalidates its sessions. |
| `Target.activateTarget` | Validates a live target; Brimp has no foreground window state. |
| `Page.enable` | Enables page commands for a session. |
| `Page.disable` | Disables page and lifecycle events for a session. |
| `Page.getFrameTree` | Returns the current main frame. Child frames are not implemented. |
| `Page.setLifecycleEventsEnabled` | Controls navigation lifecycle events. |
| `Page.addScriptToEvaluateOnNewDocument` | Registers a script before document scripts. Named isolated worlds are not implemented. |
| `Page.removeScriptToEvaluateOnNewDocument` | Removes a registered preload script. |
| `Page.createIsolatedWorld` | Creates and recreates a named execution-context identity. Named contexts currently share the page's JavaScript realm. |
| `Page.navigate` | Navigates and emits load events. |
| `Page.reload` | Reloads the current URL with a fresh loader and execution context. |
| `Page.getNavigationHistory` | Returns the target's current index and committed main-frame entries. |
| `Page.navigateToHistoryEntry` | Navigates to a live entry without appending or discarding history. |
| `Page.getLayoutMetrics` | Returns main-frame viewport and content metrics. |
| `Page.captureScreenshot` | Returns a viewport or full-page PNG as base64. PNG clipping is limited to the viewport or full page from the origin at scale 1. |
| `DOM.describeNode` | Assigns a stable backend node ID to a page-owned remote DOM node. |
| `DOM.resolveNode` | Resolves a backend node ID back to a page-owned remote DOM node. |
| `DOM.getDocument` | Returns the current document root with a stable frontend/backend node ID. |
| `DOM.querySelector` | Queries beneath a registered DOM node and returns a stable node ID or zero. |
| `DOM.querySelectorAll` | Queries beneath a registered DOM node and returns stable node IDs. |
| `DOM.getAttributes` | Returns the element's current flattened name/value attribute list. |
| `DOM.getContentQuads` | Returns the node's current border-box quad. |
| `DOM.getBoxModel` | Returns the current layout box; content, padding, border, and margin share the available border-box geometry. |
| `DOM.scrollIntoViewIfNeeded` | Validates the node and invokes the runtime's current scroll-into-view behavior. |
| `DOM.focus` | Moves document focus to the resolved element and dispatches focus transitions. |
| `Runtime.enable` | Enables runtime commands and advertises the default context. |
| `Runtime.disable` | Disables runtime context events for a session. |
| `Runtime.evaluate` | Evaluates, optionally awaits promise settlement, and returns by value or as a page-owned remote object. |
| `Runtime.callFunctionOn` | Invokes a function with by-value or remote-object arguments and receiver, optionally awaiting promise settlement. |
| `Runtime.getProperties` | Returns own or inherited property descriptors without invoking getters, allocating child remote objects as needed. |
| `Runtime.releaseObject` | Releases a page-owned remote object. |
| `Runtime.releaseObjectGroup` | Releases every remote object allocated into the named group. |
| `Runtime.runIfWaitingForDebugger` | Resumes a session waiting for a debugger. |
| `Network.enable` | Enables main-document request, response, and completion events. |
| `Network.disable` | Disables network events for a session. |
| `Network.setCacheDisabled` | Accepted during client initialization; Brimp has no configurable page cache. |
| `Network.setExtraHTTPHeaders` | Applies string-valued headers to subsequent main-document navigation requests. |
| `Network.setRequestInterception` | Pauses matching navigation, subresource, and JavaScript Fetch requests at the request stage. Empty patterns disable interception. |
| `Network.continueInterceptedRequest` | Continues, modifies, fails, or fulfills a paused legacy request. Fulfillment accepts a base64 raw HTTP response. Authentication challenges are not implemented. |
| `Network.getResponseBody` | Returns the latest main-document response body by loader/request ID. |
| `Network.setUserAgentOverride` | Applies coherent request-header and `navigator` identity overrides; client-hint metadata is not implemented. |
| `Fetch.enable` | Pauses matching navigation, subresource, and JavaScript Fetch requests at the request stage. Response-stage and authentication interception are not implemented. |
| `Fetch.disable` | Disables Fetch interception and continues requests currently paused by it. |
| `Fetch.continueRequest` | Continues a paused request, optionally replacing its URL, method, body, or headers. |
| `Fetch.failRequest` | Fails a paused request with the requested network error reason. |
| `Fetch.fulfillRequest` | Fulfills a paused request with a status, headers, and optional base64 body. |
| `Emulation.setDeviceMetricsOverride` | Applies viewport width, height, and device-pixel ratio. |
| `Emulation.clearDeviceMetricsOverride` | Restores the default viewport. |
| `Emulation.setTouchEmulationEnabled` | Enables or disables touch-mode compatibility for CDP clients. |
| `Emulation.setFocusEmulationEnabled` | Accepted because the single headless page remains focused. |
| `Emulation.setEmulatedMedia` | Accepts existing default media values and rejects unsupported overrides. |
| `Emulation.setUserAgentOverride` | Alias of the coherent Network user-agent, language, and platform override. |
| `Input.dispatchMouseEvent` | Sends trusted pointer/mouse move, press, release, click, and double-click input through the browser input state machine. |
| `Input.dispatchKeyEvent` | Sends trusted keyboard input and applies supported editing and focus defaults. |
| `Input.dispatchTouchEvent` | Sends trusted touch and touch-pointer start, move, end, or cancel input, including tap compatibility clicks. |
| `Input.insertText` | Inserts text into the focused input or textarea and emits a trusted input event. |
| `Audits.enable`, `Audits.disable` | Accepted for client initialization; no audit events are emitted. |
| `CSS.enable`, `CSS.disable` | Accepted for client initialization; no CSS events are emitted. |
| `Log.enable`, `Log.disable` | Accepted for client initialization; no Log events are emitted. |
| `Performance.enable`, `Performance.disable` | Accepted for client initialization; metrics are not implemented. |
| `Security.enable`, `Security.disable` | Accepted for client initialization; no Security events are emitted. |

## Security and limits

- Loopback is the default and recommended bind.
- Non-loopback binds require `--allow-non-loopback` and emit a warning.
- Messages and event queues are bounded.
- The server has no authentication or authorization layer.
- Browser contexts and the complete Chrome DevTools Protocol are not implemented.
