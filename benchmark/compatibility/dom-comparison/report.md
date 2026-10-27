# Browser workload report

Generated: 2026-10-22T01:38:56.752Z

Artifact manifest SHA-256: `99b3add8a58037b9e5d63b744cc495a5d9ead5191d8ead02ac144ae5c7225b5f`

These are compatibility observations, not a browser-conformance or anti-detection score.
Detector findings do not expand the runtime contract. Timings include setup and are not benchmarks.

Candidate cells show execution / comparison to Chromium; — means a Brimp-only configuration.

| Workload | Chromium | Brimp | jsdom | happy-dom |
| --- | --- | --- | --- | --- |
| todomvc-react (default) | passed | passed / matched | passed / matched | timeout / failed |
| todomvc-vue (default) | passed | passed / matched | timeout / failed | passed / matched |
| jquery (default) | passed | passed / matched | passed / matched | passed / matched |
| htmx (default) | passed | passed / matched | timeout / failed | timeout / failed |
| lit (default) | passed | passed / matched | passed / matched | passed / matched |
| dompurify (default) | passed | passed / matched | passed / matched | assertion_failed / failed |
| readability (default) | passed | passed / matched | passed / matched | passed / matched |
| axios (default) | passed | passed / matched | passed / matched | passed / matched |
| dexie (default) | passed | execution_error / failed | execution_error / failed | execution_error / failed |
| dexie (storage) | passed | assertion_failed / failed | — | — |
| jose (default) | passed | passed / matched | execution_error / failed | execution_error / failed |
| defuddle (default) | passed | passed / matched | assertion_failed / failed | passed / matched |
| modernizr (default) | passed | passed / observed | passed / observed | passed / observed |
| botd (default) | passed | passed / observed | passed / observed | passed / observed |
| creepjs (default) | passed | timeout / failed | crash / failed | crash / failed |

## Functional summary (default configuration)

- brimp: 10/11 functional observations matched Chromium.
- jsdom: 6/11 functional observations matched Chromium.
- happy-dom: 6/11 functional observations matched Chromium.

Pinned Node DOM versions: jsdom 30.1.3; happy-dom 21.0.4. Node v26.8.2.

## Findings

- **todomvc-react / happy-dom:** Timed out waiting for !!document.querySelector(".new-todo")
- **todomvc-react / happy-dom page errors:** http://127.0.0.1:62027/app.bundle.js:3 //# sourceMappingURL=app.bundle.js.map} catch (error) { $happy_dom.dispatchError(error); }})                                                                                               SyntaxError: Unexpected end of input     at new Script (node:vm:118:7)     at [evaluateScript] (file:///Users/yifei/repos/imp/brimp/compatibility/node_modules/happy-dom/lib/window/BrowserWindow.js:2339:16)     at JavaScriptCompiler.compile (file:///Users/yifei/repos/imp/brimp/compatibility/node_modules/happy-dom/lib/javascript/JavaScriptCompiler.js:123:68)     at #loadScript (file:///Users/yifei/repos/imp/brimp/compatibility/node_modules/happy-dom/lib/nodes/html-script-element/HTMLScriptElement.js:591:35)     at process.processTicksAndRejections (node:internal/process
- **todomvc-react / happy-dom console findings:** DOMException [NotSupportedError]: Failed to load external stylesheet "http://127.0.0.1:62027/app.css". CSS file loading is disabled.; GET http://127.0.0.1:62027/learn.json 404 (Not Found); http://127.0.0.1:62027/app.bundle.js:3
- **todomvc-vue / jsdom:** Timed out waiting for !!document.querySelector(".new-todo")
- **htmx / jsdom:** Timed out waiting for !!document.querySelector("form input[name=firstName]")
- **htmx / jsdom page errors:** You must provide an XPath result type (0=any).: 9     at XPathExpression.evaluate (/Users/yifei/repos/imp/brimp/compatibility/node_modules/jsdom/lib/jsdom/level3/xpath.js:1738:15)     at processHXOnRoot (http://127.0.0.1:62078/workload.js:1963:32)     at findHxOnWildcardElements (http://127.0.0.1:62078/workload.js:1974:9)     at processNode (http://127.0.0.1:62078/workload.js:2130:15)     at Proxy.<anonymous> (http://127.0.0.1:62078/workload.js:3660:7)     at Proxy.callTheUserObjectsOperation (/Users/yifei/repos/imp/brimp/compatibility/node_modules/jsdom/lib/generated/idl/EventListener.js:26:30)     at innerInvokeEventListeners (/Users/yifei/repos/imp/brimp/compatibility/node_modules/jsdom/lib/jsdom/living/events/EventTarget-impl.js:378:16)     at invokeEventListeners (/Users/yifei/repos/i
- **htmx / happy-dom:** Timed out waiting for !!document.querySelector("form input[name=firstName]")
- **htmx / happy-dom page errors:** ReferenceError: XPathEvaluator is not defined     at http://127.0.0.1:62081/workload.js:1958:25     at http://127.0.0.1:62081/workload.js:3696:5     at Object.anonymous [as execute] (http://127.0.0.1:62081/workload.js:3698:3)     at #loadScript (file:///Users/yifei/repos/imp/brimp/compatibility/node_modules/happy-dom/lib/nodes/html-script-element/HTMLScriptElement.js:593:18)     at process.processTicksAndRejections (node:internal/process/task_queues:104:5)
- **htmx / happy-dom console findings:** ReferenceError: XPathEvaluator is not defined
- **dompurify / happy-dom:** active script and event handler must be removed
- **dompurify / happy-dom page errors:** ReferenceError: alert is not defined     at Object.anonymous [as execute] (http://127.0.0.1:62103/:1:39)     at #evaluateScript (file:///Users/yifei/repos/imp/brimp/compatibility/node_modules/happy-dom/lib/nodes/html-script-element/HTMLScriptElement.js:457:18)     at [connectedToDocument] (file:///Users/yifei/repos/imp/brimp/compatibility/node_modules/happy-dom/lib/nodes/html-script-element/HTMLScriptElement.js:332:41)     at [connectedToNode] (file:///Users/yifei/repos/imp/brimp/compatibility/node_modules/happy-dom/lib/nodes/node/Node.js:801:53)     at [connectedToNode] (file:///Users/yifei/repos/imp/brimp/compatibility/node_modules/happy-dom/lib/nodes/node/Node.js:793:58)     at [appendChild] (file:///Users/yifei/repos/imp/brimp/compatibility/node_modules/happy-dom/lib/nodes/node/Node.js
- **dompurify / happy-dom console findings:** ReferenceError: alert is not defined
- **dexie / brimp:** javascript: JavaScript exception: DatabaseClosedError: MissingAPIError IndexedDB API missing. Please visit https://tinyurl.com/y2uuvskb
- **dexie / jsdom:** MissingAPIError IndexedDB API missing. Please visit https://tinyurl.com/y2uuvskb
- **dexie / happy-dom:** MissingAPIError IndexedDB API missing. Please visit https://tinyurl.com/y2uuvskb
- **dexie / brimp:** javascript: JavaScript exception: AssertionError: IndexedDB records must survive page reload
- **jose / jsdom:** structuredClone is not defined
- **jose / happy-dom:** structuredClone is not defined
- **defuddle / jsdom:** Defuddle must remove clutter
- **defuddle / jsdom console findings:** Defuddle Error processing document: RangeError: Selector exceeds maximum allowed length of 2048.
- **modernizr / jsdom page errors:** Error: Not implemented: HTMLCanvasElement's getContext() method: without installing the canvas npm package     at exports.notImplemented (/Users/yifei/repos/imp/brimp/compatibility/node_modules/jsdom/lib/jsdom/browser/not-implemented.js:16:17)     at exports.notImplementedMethod (/Users/yifei/repos/imp/brimp/compatibility/node_modules/jsdom/lib/jsdom/browser/not-implemented.js:4:11)     at HTMLCanvasElementImpl.getContext (/Users/yifei/repos/imp/brimp/compatibility/node_modules/jsdom/lib/jsdom/living/nodes/HTMLCanvasElement-impl.js:43:5)     at HTMLCanvasElement.getContext (/Users/yifei/repos/imp/brimp/compatibility/node_modules/jsdom/lib/generated/idl/HTMLCanvasElement.js:133:44)     at Object.fn (http://127.0.0.1:62170/workload.js:1504:39)     at testRunner (http://127.0.0.1:62170/worklo
- **modernizr / chromium observation:** `{"features":{"cookies":true,"localstorage":true,"sessionstorage":true,"indexeddb":true,"webworkers":true,"serviceworker":true,"websockets":true,"fetch":true,"promises":true,"es6collections":true,"mutationobserver":true,"canvas":true,"webgl":true}}`
- **modernizr / brimp observation:** `{"features":{"canvas":false,"cookies":true,"es6collections":true,"fetch":true,"indexeddb":false,"localstorage":false,"mutationobserver":true,"promises":true,"serviceworker":false,"sessionstorage":false,"webgl":false,"websockets":true,"webworkers":false}}`
- **modernizr / jsdom observation:** `{"features":{"cookies":true,"localstorage":true,"sessionstorage":true,"indexeddb":false,"webworkers":false,"serviceworker":false,"websockets":true,"fetch":false,"promises":true,"es6collections":true,"mutationobserver":true,"canvas":false,"webgl":false}}`
- **modernizr / happy-dom observation:** `{"features":{"cookies":true,"localstorage":true,"sessionstorage":true,"indexeddb":false,"webworkers":false,"serviceworker":false,"websockets":true,"fetch":true,"promises":true,"es6collections":true,"mutationobserver":true,"canvas":false,"webgl":false}}`
- **botd / jsdom page errors:** Error: Not implemented: HTMLCanvasElement's getContext() method: without installing the canvas npm package     at exports.notImplemented (/Users/yifei/repos/imp/brimp/compatibility/node_modules/jsdom/lib/jsdom/browser/not-implemented.js:16:17)     at exports.notImplementedMethod (/Users/yifei/repos/imp/brimp/compatibility/node_modules/jsdom/lib/jsdom/browser/not-implemented.js:4:11)     at HTMLCanvasElementImpl.getContext (/Users/yifei/repos/imp/brimp/compatibility/node_modules/jsdom/lib/jsdom/living/nodes/HTMLCanvasElement-impl.js:43:5)     at HTMLCanvasElement.getContext (/Users/yifei/repos/imp/brimp/compatibility/node_modules/jsdom/lib/generated/idl/HTMLCanvasElement.js:133:44)     at getWebGL (http://127.0.0.1:62181/workload.js:514:40)     at http://127.0.0.1:62181/workload.js:67:24   
- **botd / chromium observation:** `{"detection":{"bot":true,"botKind":"headless_chrome"}}`
- **botd / brimp observation:** `{"detection":{"bot":false}}`
- **botd / jsdom observation:** `{"detection":{"bot":false}}`
- **botd / happy-dom observation:** `{"detection":{"bot":true,"botKind":"headless_chrome"}}`
- **creepjs / brimp:** Timed out waiting for !!window.Fingerprint && !!window.Creep && Object.keys(window.Fingerprint).length > 0 && Object.keys(window.Creep).length > 0
- **creepjs / brimp console findings:** ReferenceError: Can't find variable: getComputedStyle; TypeError: Cannot destructure property 'width' from null or undefined value; ReferenceError: Can't find variable: matchMedia; TypeError: canvas.getContext is not a function. (In 'canvas.getContext('2d')', 'canvas.getContext' is undefined); TypeError: undefined is not an object (evaluating 'document.fonts.check'); TypeError: svgBox.getBBox is not a function. (In 'svgBox.getBBox()', 'svgBox.getBBox' is undefined); TypeError: el.getClientRects is not a function. (In 'el.getClientRects()', 'el.getClientRects' is undefined); TypeError: undefined is not an object (evaluating 'workerScope.platform'); TypeError: undefined is not an object (evaluating 'workerScope.userAgent'); ReferenceError: Can't find variable: PluginArray
- **creepjs / jsdom:** Worker exited with code 1, signal null
- **creepjs / happy-dom:** Worker exited with code 1, signal null
- **creepjs / chromium observation:** `{"collectors":["workerScope","navigator","windowFeatures","headless","htmlElementVersion","cssMedia","css","screen","voices","media","canvas2d","canvasWebgl","maths","consoleErrors","timezone","clientRects","offlineAudioContext","fonts","lies","trash","capturedErrors","svg","resistance","intl","features"],"lies":{"data":{},"totalLies":0,"$hash":"f4c4e0371fde124bbb71790a9d668d18af1a5236965caed7959ee1aaa6bda301"},"capturedErrors":{"data":[],"$hash":"8fe32e407a1038ee38753b70e5374b3a46d6ae9d5f16cd5b73c53abaca8f5ed0"}}`

Full observations, checkpoints, page errors, local requests, and blocked HTTP requests are in the accompanying JSON.
