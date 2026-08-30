# CDP support matrix

This matrix is intentionally exhaustive. Any method absent from it returns CDP
error `-32601` rather than a success stub.

| Method | Behavior |
| --- | --- |
| `Browser.getVersion` | Reports Brimp and protocol identity. |
| `Browser.getWindowForTarget` | Returns the single headless window and its bounds, validating an optional target. |
| `Browser.getWindowBounds` | Returns the single headless window's bounds. |
| `Browser.setDownloadBehavior` | Accepted for Playwright initialization; downloads are not implemented. |
| `Target.getBrowserContexts` | Lists live non-default logical browser contexts. |
| `Target.createBrowserContext` | Creates a logical page-group context. Cookie and storage isolation are not implemented. |
| `Target.disposeBrowserContext` | Closes pages in and removes a logical context. |
| `Target.setDiscoverTargets` | Enables/disables target discovery events. |
| `Target.setAutoAttach` | Stores flattened auto-attach policy and applies it to new page targets. |
| `Target.getTargets` | Lists live page targets. |
| `Target.getTargetInfo` | Describes a live page target. |
| `Target.createTarget` | Creates an `about:blank` Brimp page. |
| `Target.attachToBrowserTarget` | Creates a flattened session for the browser control target. |
| `Target.attachToTarget` | Creates a flattened session with a monotonic ID. |
| `Target.detachFromTarget` | Invalidates a session and emits its detach event. |
| `Target.closeTarget` | Closes the page and invalidates its sessions. |
| `Target.activateTarget` | Validates a live target; headless Brimp has no foreground window state to change. |
| `Page.enable` | Enables page commands for a session. |
| `Page.disable` | Disables page and lifecycle events for a session. |
| `Page.getFrameTree` | Returns the current main-frame metadata. Child frames are not implemented. |
| `Page.setLifecycleEventsEnabled` | Controls navigation lifecycle events. |
| `Page.addScriptToEvaluateOnNewDocument` | Registers a script that runs after the new realm is installed and before document scripts. Named isolated worlds are not implemented. |
| `Page.removeScriptToEvaluateOnNewDocument` | Removes a registered preload script. |
| `Page.createIsolatedWorld` | Creates and recreates a named execution-context identity. Named contexts currently share the page's JavaScript realm. |
| `Page.navigate` | Navigates through `web-runtime` and emits load events. |
| `Page.reload` | Navigates the target to its current URL with a fresh loader and execution context. |
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
| `Runtime.evaluate` | Evaluates in the page owner thread, optionally awaiting promise settlement, and returns by value or as a page-owned remote object. |
| `Runtime.callFunctionOn` | Invokes a function with by-value or remote-object arguments and an optional remote receiver, optionally awaiting promise settlement. |
| `Runtime.getProperties` | Returns own or inherited property descriptors without invoking getters, allocating child remote objects as needed. |
| `Runtime.releaseObject` | Releases a page-owned remote object. |
| `Runtime.releaseObjectGroup` | Releases every remote object allocated into the named group. |
| `Runtime.runIfWaitingForDebugger` | Resumes a session created with debugger waiting enabled. |
| `Network.enable` | Enables main-document request, response, and completion events. Subresource events are not implemented. |
| `Network.disable` | Disables network events for a session. |
| `Network.setCacheDisabled` | Accepted for client initialization; Brimp does not expose a configurable page cache. |
| `Network.setExtraHTTPHeaders` | Applies string-valued headers to subsequent main-document navigation requests. |
| `Network.getResponseBody` | Returns the latest main-document response body by loader/request ID. |
| `Network.setUserAgentOverride` | Applies coherent request-header and `navigator` identity overrides; client-hint metadata is not implemented. |
| `Emulation.setDeviceMetricsOverride` | Applies width, height, and device-pixel ratio to the page viewport. Mobile emulation is not implemented. |
| `Emulation.clearDeviceMetricsOverride` | Restores Brimp's default page viewport. |
| `Emulation.setTouchEmulationEnabled` | Enables or disables touch-mode compatibility for CDP clients. |
| `Emulation.setFocusEmulationEnabled` | Accepted because Brimp's single headless page remains focused. |
| `Emulation.setEmulatedMedia` | Accepts Brimp's existing default media values and rejects unsupported overrides. |
| `Emulation.setUserAgentOverride` | Alias of the coherent Network user-agent, language, and platform override. |
| `Input.dispatchMouseEvent` | Sends trusted pointer/mouse move, press, release, click, and double-click input through the browser input state machine. |
| `Input.dispatchKeyEvent` | Sends trusted keyboard input and applies supported editing and focus defaults. |
| `Input.dispatchTouchEvent` | Sends trusted touch and touch-pointer start, move, end, or cancel input, including tap compatibility clicks. |
| `Input.insertText` | Inserts text into the focused input or textarea and emits a trusted input event. |
| `Audits.enable`, `Audits.disable` | Accepted subscription state for client initialization; no audit events are emitted. |
| `CSS.enable`, `CSS.disable` | Accepted subscription state for client initialization; no CSS domain events are emitted. |
| `Log.enable`, `Log.disable` | Accepted subscription state for client initialization; no Log domain events are emitted. |
| `Performance.enable`, `Performance.disable` | Accepted subscription state for client initialization; performance metrics are not implemented. |
| `Security.enable`, `Security.disable` | Accepted subscription state for client initialization; no Security domain events are emitted. |
