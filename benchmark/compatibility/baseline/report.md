# Browser workload baseline

Generated: 2026-10-22T00:26:40.206Z

Artifact manifest SHA-256: `232898e3b1d6513d1c75dd604c8f40315bf421dae1623a88f452bca0f5ec173e`

These are compatibility observations, not a browser-conformance or anti-detection score.
Detector findings do not expand the runtime contract. Timings include setup and are not benchmarks.

| Workload | Chromium execution | Brimp execution | Comparison |
| --- | --- | --- | --- |
| todomvc-react | passed | timeout | failed |
| todomvc-vue | passed | timeout | failed |
| jquery | passed | passed | matched |
| htmx | passed | timeout | failed |
| lit | passed | execution_error | failed |
| dompurify | passed | execution_error | failed |
| readability | passed | passed | matched |
| axios | passed | passed | matched |
| dexie | passed | execution_error | failed |
| jose | passed | execution_error | failed |
| defuddle | passed | passed | matched |
| modernizr | passed | passed | observed |
| botd | passed | passed | observed |
| creepjs | passed | timeout | failed |

## Findings

- **todomvc-react / brimp:** Timed out waiting for !!document.querySelector(".new-todo")
- **todomvc-react / brimp console findings:** TypeError: (Fn=t).attachEvent is not a function. (In '(Fn=t).attachEvent("onpropertychange",qn)', '(Fn=t).attachEvent' is undefined)
- **todomvc-vue / brimp:** Timed out waiting for !!document.querySelector(".new-todo")
- **todomvc-vue / brimp page errors:** TypeError: classList has no supported tokens
- **todomvc-vue / brimp console findings:** TypeError: classList has no supported tokens
- **htmx / brimp:** Timed out waiting for !!document.querySelector("form input[name=firstName]")
- **htmx / brimp page errors:** ReferenceError: Can't find variable: XPathEvaluator
- **htmx / brimp console findings:** ReferenceError: Can't find variable: XPathEvaluator
- **lit / brimp:** javascript: JavaScript exception: TypeError: this.attachShadow is not a function. (In 'this.attachShadow(this.constructor.shadowRootOptions)', 'this.attachShadow' is undefined)
- **dompurify / brimp:** javascript: JavaScript exception: TypeError: undefined is not an object (evaluating 'createNodeIterator.call')
- **dexie / brimp:** javascript: JavaScript exception: DatabaseClosedError: NotFoundError Object store not found  NotFoundError: Object store not found
- **jose / brimp:** javascript: JavaScript exception: ReferenceError: Can't find variable: structuredClone
- **modernizr / chromium observation:** `{"features":{"cookies":true,"localstorage":true,"sessionstorage":true,"indexeddb":true,"webworkers":true,"serviceworker":true,"websockets":true,"fetch":true,"promises":true,"es6collections":true,"mutationobserver":true,"canvas":true,"webgl":true}}`
- **modernizr / brimp observation:** `{"features":{"canvas":false,"cookies":true,"es6collections":true,"fetch":true,"indexeddb":false,"localstorage":false,"mutationobserver":true,"promises":true,"serviceworker":false,"sessionstorage":false,"webgl":false,"websockets":true,"webworkers":false}}`
- **botd / chromium observation:** `{"detection":{"bot":true,"botKind":"headless_chrome"}}`
- **botd / brimp observation:** `{"detection":{"bot":false}}`
- **creepjs / brimp:** Timed out waiting for !!window.Fingerprint && !!window.Creep && Object.keys(window.Fingerprint).length > 0 && Object.keys(window.Creep).length > 0
- **creepjs / brimp console findings:** ReferenceError: Can't find variable: getComputedStyle; TypeError: Cannot destructure property 'width' from null or undefined value; ReferenceError: Can't find variable: matchMedia; TypeError: canvas.getContext is not a function. (In 'canvas.getContext('2d')', 'canvas.getContext' is undefined); TypeError: undefined is not an object (evaluating 'document.fonts.check'); TypeError: svgBox.getBBox is not a function. (In 'svgBox.getBBox()', 'svgBox.getBBox' is undefined); TypeError: el.getClientRects is not a function. (In 'el.getClientRects()', 'el.getClientRects' is undefined); TypeError: undefined is not an object (evaluating 'workerScope.platform'); TypeError: undefined is not an object (evaluating 'workerScope.userAgent'); ReferenceError: Can't find variable: PluginArray; TypeError: undefined is not an object (evaluating 'crypto.subtle.digest')
- **creepjs / chromium observation:** `{"collectors":["workerScope","navigator","windowFeatures","headless","htmlElementVersion","cssMedia","css","screen","voices","media","canvas2d","canvasWebgl","maths","consoleErrors","timezone","clientRects","offlineAudioContext","fonts","lies","trash","capturedErrors","svg","resistance","intl","features"],"lies":{"data":{},"totalLies":0,"$hash":"f4c4e0371fde124bbb71790a9d668d18af1a5236965caed7959ee1aaa6bda301"},"capturedErrors":{"data":[],"$hash":"8fe32e407a1038ee38753b70e5374b3a46d6ae9d5f16cd5b73c53abaca8f5ed0"}}`

Full observations, checkpoints, page errors, local requests, and blocked HTTP requests are in the accompanying JSON.
