# Pinned Puppeteer and Playwright protocol traces

The checked workflow uses exactly `puppeteer-core` 24.43.1 and only its public
API. The successful request sequence, recorded with
`DEBUG=puppeteer:protocol:*`, includes:

1. `Target.getBrowserContexts`
2. `Target.setDiscoverTargets`
3. `Target.setAutoAttach`
4. `Target.createTarget`, followed by target-created and auto-attach events
5. session-scoped `Target.setAutoAttach` and `Runtime.runIfWaitingForDebugger`
6. `Network.enable`
7. `Page.enable`, `Page.getFrameTree`, and `Page.setLifecycleEventsEnabled`
8. `Runtime.enable`, `Performance.enable`, and `Log.enable`
9. `Page.addScriptToEvaluateOnNewDocument`
10. `Emulation.setDeviceMetricsOverride` and disabled touch emulation
11. `Page.navigate`, including main-document network, frame, context, and load events
12. `Runtime.callFunctionOn`
13. `Page.captureScreenshot`
14. `Target.closeTarget`

Puppeteer's optional `WebMCP.enable` request returns method-not-found and is
tolerated. Named `Page.createIsolatedWorld` contexts have stable lifecycle
identities but currently share Brimp's page JavaScript realm. Navigation loaded
the loopback fixture through Brimp, public `page.evaluate()` returned its title
plus the number 42, public element-handle query/evaluation/click operated on the
main element, and public `page.screenshot()` returned a full-page PNG.

The checked `playwright-core` 1.62.1 workflow connects through public
`chromium.connectOverCDP()`, creates a context and page, sets the viewport,
navigates, evaluates by way of Playwright's page-owned utility object, captures
a PNG, and closes the page/context/browser. This additionally exercises default
browser-context identity, Playwright's focus/media bootstrap, and remote-object
receiver, argument, and release behavior.
