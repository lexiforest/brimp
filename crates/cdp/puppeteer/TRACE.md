# Pinned Puppeteer protocol trace

The checked workflow uses exactly `puppeteer-core` 24.43.1. The successful
request sequence, recorded with `DEBUG=puppeteer:protocol:*`, is:

1. `Target.getBrowserContexts`
2. `Target.setDiscoverTargets`
3. `Target.setAutoAttach`
4. `Target.createTarget`
5. `Target.attachToTarget`
6. `Page.enable`
7. `Page.setLifecycleEventsEnabled`
8. `Runtime.enable`
9. `Page.navigate`
10. `Runtime.evaluate`
11. `Page.captureScreenshot`
12. `Target.closeTarget`

The server returned no errors. Navigation loaded the loopback fixture through
Brimp, evaluation returned its title plus the number 42, and the decoded
screenshot began with the PNG signature. `workflow.mjs` asserts all three.
