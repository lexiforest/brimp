# Browser workload report

Generated: 2026-10-22T01:27:22.412Z

Artifact manifest SHA-256: `5c005da06037166f9b1e741b755bfd43d33120bc39853e9968c3cd75e30e2606`

These are compatibility observations, not a browser-conformance or anti-detection score.
Detector findings do not expand the runtime contract. Timings include setup and are not benchmarks.

| Workload | Chromium execution | Brimp execution | Comparison |
| --- | --- | --- | --- |
| todomvc-react (default) | passed | passed | matched |
| todomvc-vue (default) | passed | passed | matched |
| jquery (default) | passed | passed | matched |
| htmx (default) | passed | passed | matched |
| lit (default) | passed | passed | matched |
| dompurify (default) | passed | passed | matched |
| readability (default) | passed | passed | matched |
| axios (default) | passed | passed | matched |
| dexie (default) | passed | execution_error | failed |
| dexie (storage) | passed | assertion_failed | failed |
| jose (default) | passed | passed | matched |
| defuddle (default) | passed | passed | matched |
| modernizr (default) | passed | passed | observed |
| botd (default) | passed | passed | observed |
| creepjs (default) | passed | timeout | failed |

## Findings

- **dexie / brimp:** javascript: JavaScript exception: DatabaseClosedError: MissingAPIError IndexedDB API missing. Please visit https://tinyurl.com/y2uuvskb
- **dexie / brimp:** javascript: JavaScript exception: AssertionError: IndexedDB records must survive page reload
- **modernizr / chromium observation:** `{"features":{"cookies":true,"localstorage":true,"sessionstorage":true,"indexeddb":true,"webworkers":true,"serviceworker":true,"websockets":true,"fetch":true,"promises":true,"es6collections":true,"mutationobserver":true,"canvas":true,"webgl":true}}`
- **modernizr / brimp observation:** `{"features":{"canvas":false,"cookies":true,"es6collections":true,"fetch":true,"indexeddb":false,"localstorage":false,"mutationobserver":true,"promises":true,"serviceworker":false,"sessionstorage":false,"webgl":false,"websockets":true,"webworkers":false}}`
- **botd / chromium observation:** `{"detection":{"bot":true,"botKind":"headless_chrome"}}`
- **botd / brimp observation:** `{"detection":{"bot":false}}`
- **creepjs / brimp:** Timed out waiting for !!window.Fingerprint && !!window.Creep && Object.keys(window.Fingerprint).length > 0 && Object.keys(window.Creep).length > 0
- **creepjs / brimp console findings:** ReferenceError: Can't find variable: getComputedStyle; TypeError: Cannot destructure property 'width' from null or undefined value; ReferenceError: Can't find variable: matchMedia; TypeError: canvas.getContext is not a function. (In 'canvas.getContext('2d')', 'canvas.getContext' is undefined); TypeError: undefined is not an object (evaluating 'document.fonts.check'); TypeError: svgBox.getBBox is not a function. (In 'svgBox.getBBox()', 'svgBox.getBBox' is undefined); TypeError: el.getClientRects is not a function. (In 'el.getClientRects()', 'el.getClientRects' is undefined); TypeError: undefined is not an object (evaluating 'workerScope.platform'); TypeError: undefined is not an object (evaluating 'workerScope.userAgent'); ReferenceError: Can't find variable: PluginArray
- **creepjs / chromium observation:** `{"collectors":["workerScope","navigator","windowFeatures","headless","htmlElementVersion","cssMedia","css","screen","voices","media","canvas2d","canvasWebgl","maths","consoleErrors","timezone","clientRects","offlineAudioContext","fonts","lies","trash","capturedErrors","svg","resistance","intl","features"],"lies":{"data":{},"totalLies":0,"$hash":"f4c4e0371fde124bbb71790a9d668d18af1a5236965caed7959ee1aaa6bda301"},"capturedErrors":{"data":[],"$hash":"8fe32e407a1038ee38753b70e5374b3a46d6ae9d5f16cd5b73c53abaca8f5ed0"}}`

Full observations, checkpoints, page errors, local requests, and blocked HTTP requests are in the accompanying JSON.
