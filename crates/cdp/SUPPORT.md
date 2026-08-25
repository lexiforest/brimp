# CDP support matrix

This matrix is intentionally exhaustive. Any method absent from it returns CDP
error `-32601` rather than a success stub.

| Method | Behavior |
| --- | --- |
| `Browser.getVersion` | Reports Brimp and protocol identity. |
| `Target.getBrowserContexts` | Reports no non-default browser contexts. |
| `Target.setDiscoverTargets` | Enables/disables target discovery events. |
| `Target.setAutoAttach` | Stores flattened auto-attach policy and applies it to new page targets. |
| `Target.getTargets` | Lists live page targets. |
| `Target.getTargetInfo` | Describes a live page target. |
| `Target.createTarget` | Creates an `about:blank` Brimp page. |
| `Target.attachToTarget` | Creates a flattened session with a monotonic ID. |
| `Target.detachFromTarget` | Invalidates a session and emits its detach event. |
| `Target.closeTarget` | Closes the page and invalidates its sessions. |
| `Page.enable` | Enables page commands for a session. |
| `Page.setLifecycleEventsEnabled` | Controls navigation lifecycle events. |
| `Page.navigate` | Navigates through `web-runtime` and emits load events. |
| `Page.captureScreenshot` | Returns the Brimp viewport PNG as base64. |
| `Runtime.enable` | Enables runtime commands and advertises the default context. |
| `Runtime.evaluate` | Evaluates in the page owner thread and returns by value. |
| `Runtime.callFunctionOn` | Invokes a function with JSON by-value arguments. |
| `Runtime.runIfWaitingForDebugger` | Resumes a session created with debugger waiting enabled. |
