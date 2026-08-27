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

## Supported methods

The following list is exhaustive. An absent method returns CDP error `-32601`.

| Method | Behavior |
| --- | --- |
| `Browser.getVersion` | Reports Brimp and protocol identity. |
| `Target.getBrowserContexts` | Reports no non-default browser contexts. |
| `Target.setDiscoverTargets` | Controls target discovery events. |
| `Target.setAutoAttach` | Stores flattened auto-attach policy and applies it to new pages. |
| `Target.getTargets` | Lists live page targets. |
| `Target.getTargetInfo` | Describes a live page target. |
| `Target.createTarget` | Creates an `about:blank` Brimp page. |
| `Target.attachToTarget` | Creates a flattened session. |
| `Target.detachFromTarget` | Invalidates a session and emits its detach event. |
| `Target.closeTarget` | Closes a page and invalidates its sessions. |
| `Page.enable` | Enables page commands for a session. |
| `Page.setLifecycleEventsEnabled` | Controls navigation lifecycle events. |
| `Page.navigate` | Navigates and emits load events. |
| `Page.captureScreenshot` | Returns the viewport PNG as base64. |
| `Runtime.enable` | Enables runtime commands and advertises the default context. |
| `Runtime.evaluate` | Evaluates in the page owner thread and returns by value. |
| `Runtime.callFunctionOn` | Invokes a function with JSON by-value arguments. |
| `Runtime.runIfWaitingForDebugger` | Resumes a session waiting for a debugger. |

## Security and limits

- Loopback is the default and recommended bind.
- Non-loopback binds require `--allow-non-loopback` and emit a warning.
- Messages and event queues are bounded.
- The server has no authentication or authorization layer.
- Browser contexts and the complete Chrome DevTools Protocol are not implemented.
