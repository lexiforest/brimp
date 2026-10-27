# Browser workload corpus

Fifteen pinned workloads measure whether browser applications and libraries
execute correctly in Brimp, jsdom and happy-dom. Chromium, driven through
Playwright, runs the same files as a reference. This is a compatibility corpus, not a browser conformance
suite or a performance benchmark.

The [DOM coverage audit](coverage/README.md) complements application workloads
with targeted behavior probes guided by jsdom/happy-dom source and tests. Its
gap-focused counts are not comparable to the application-corpus match rate.

**Browser checks are observational.** CreepJS, BotD and Modernizr do not require
Brimp to impersonate a browser, hide automation, implement rendering, or expand
its supported API surface. A detector returning `bot: true` is a completed
execution with that finding. A detector crashing or never returning is an
execution failure. Both are recorded without patching the detector or runtime.

## Run

Use Node 24.15+ (24.x) or Node 26+, npm, and `tar`, matching the pinned jsdom
package's Node requirement. Prepare the native Node binding using
[the binding build instructions](../../bindings/README.md). Then:

```sh
cd benchmark/compatibility
npm ci --ignore-scripts
npx playwright install chromium
npm run prepare:fixtures
npm test
npm run check
```

On Linux, Playwright may require `npx playwright install --with-deps chromium`.
Native development builds require the matching JSC/curl loader environment;
on this repository's macOS setup:

```sh
export DYLD_FRAMEWORK_PATH="$PWD/../../../WebKit/WebKitBuild/Release"
npm run check
```

`BRIMP_NODE_BINDING=/absolute/path/to/brimp/index.js` selects an installed Node
package instead of `../../bindings/node/index.js`. `PLAYWRIGHT_BROWSERS_PATH` can
select a browser download directory. The report records the binding path, native
binary hash, loader overrides, Chromium/jsdom/happy-dom versions, Node version,
adapter settings and host platform.

```sh
node run.mjs --only jquery,axios,botd
node run.mjs --output results/my-run --timeout 60
node run.mjs --strict
```

The default output is `results/report.json`, `results/report.md`, and a copy of
the artifact `manifest.json`. Results are written after every workload. Each
engine/workload runs in its own child process, with a 60-second deadline that
includes setup and cleanup, followed by at most two seconds of termination grace.
The parent process serves fixtures so synchronous resource loads cannot deadlock
against a server in the same event loop.
DOM waits and native calls have shorter limits. A crashed or hung runtime cannot
stop the remaining scenarios. Run sequentially for comparable conditions.

The default exit status fails for invalid reference results, unavailable candidate
setup (including native crashes before initialization), or Brimp regressions in the eleven
functional workloads listed in `regressionWorkloads` in `catalog.mjs`.
Peer compatibility gaps remain observations, not Brimp regression gates.
`--strict` also fails for other
functional mismatches and incomplete detector executions; it never requires a
particular fingerprint, feature set, or bot classification.

All current workloads use the `default` configuration, which creates a session
with the default browser subsystem plugins. Dexie uses
core IndexedDB. Use `--only dexie` to select it. Detectors are not rerun with mocks
or identity overrides. Historical reports retain their original configuration labels.

## Node DOM environments

- [jsdom](https://github.com/jsdom/jsdom): `JSDOM.fromURL`,
  `runScripts: "dangerously"`, usable resources with a local-origin interceptor,
  and a virtual console. Visual emulation remains off. Explicit `goto` creates a
  new window with a shared cookie jar; jsdom does not supply browser navigation
  or persistent storage between these windows.
- [happy-dom](https://github.com/capricorn86/happy-dom/wiki/Browser): `Browser`
  and `BrowserPage.goto`, with `enableJavaScriptEvaluation: true`, local-origin
  fetch interceptors and virtual-console capture. Other browser settings retain
  their defaults, including disabled external CSS/image/iframe loading.

No Node globals, WebCrypto, fetch, encoders, IndexedDB or other polyfills are
injected into either window. These results measure unchanged browser artifacts
inside the libraries' window environments. They do not measure Node applications
that import jose directly, bundle module scripts differently, or explicitly
combine a DOM library with Node APIs. Those are useful but distinct configurations.
The adapters preserve the packages' normal user agents and library warnings.

## Workloads and assertions

| Workload | Scenario | Brimp plugins |
| --- | --- | --- |
| React TodoMVC | Add, edit, complete, filter, delete, reload | None |
| Vue TodoMVC | Same interactions through Vue | None |
| jQuery | Delegated click, DOM removal, successful-control form serialization | None |
| htmx | Click-to-edit GET, form PUT, response fragment swap | None |
| Lit | Custom element, shadow root, event-driven and property-driven updates | None |
| DOMPurify | Remove script, event handler and javascript URL; preserve safe markup | None |
| Readability | Extract title/byline/body; exclude navigation and newsletter | None |
| Axios | Browser XHR adapter: JSON/form POST, binary response, cancellation | None |
| Dexie | Indexed query, failed-transaction rollback, close/reopen, reload persistence | Storage |
| jose | Import an HS256 key, sign/verify claims, reject a damaged signature | None |
| jose-extended | RS256/PS256/ES256/ES384/ES512 JWTs and RSA-OAEP-256/ECDH-ES+A256KW JWE; reject damaged signatures/tags | None |
| Defuddle | Upstream full browser bundle: HTML, metadata and separate Markdown | None |
| Modernizr | Fixed feature detector selection, including asynchronous IndexedDB | None |
| BotD | `load({monitoring:false})`, then `detect()` | None |
| CreepJS | Full upstream page; collect `window.Fingerprint` and `window.Creep` | None |

The pinned TodoMVC applications keep state in React `useReducer` / Vue `ref`;
they intentionally start empty after reload. The scenario verifies that behavior.
Persistence is tested with Dexie, not added to the upstream TodoMVC applications.
The Dexie scenario checks core IndexedDB persistence across navigation.

Chromium and Brimp applications are driven through their input APIs. jsdom and
happy-dom use the same small DOM adapter: native value setters, input/change and
Enter keyboard events, and `element.click()`. These are simulated, untrusted
events and do not claim native input/default-action equivalence. Double-click editing
and the hover-only delete button use identical DOM event dispatch/click calls
in all engines. No screenshot, coordinate, or visibility parity is asserted.

Library drivers are small corpus-owned programs, rather than claims that an
entire upstream test suite passes. The Lit driver defines a test component.
The shared article fixture is authored here, with enough prose to exercise
extraction scoring. Defuddle uses `defuddle/full` directly, not Brimp's extraction
wrapper, `defuddle/node`, or a replacement DOM. Its normal defaults remain active;
`separateMarkdown:true` enables Markdown alongside HTML, and `useAsync:false`
keeps this local-document case independent of remote extraction services.

## Sources and reproducibility

- [TodoMVC](https://github.com/tastejs/todomvc) and
  [CreepJS](https://github.com/abrahamjuliot/creepjs) are pinned by commit and archive
  SHA-256 in `sources.json`. Preparation verifies downloaded archives before
  extraction. The committed upstream `dist` / `docs` files are copied unchanged,
  including their existing browser builds. TodoMVC dependency locks are retained.
- Libraries, Playwright, esbuild and Modernizr's builder are pinned in
  `package.json` and `package-lock.json`, including transitive package integrity.
  Browser drivers and unchanged library sources are bundled together with
  esbuild's browser target. This is a normal browser build, not a compatibility
  transformation tailored to Brimp. These bundled cases do not measure a full
  ESM loader; Vue's upstream application retains its original module script.
- The [htmx click-to-edit example](https://htmx.org/examples/click-to-edit/)
  supplies the initial markup. The local server implements its GET and PUT
  endpoints with deterministic responses. Axios uses local echo/bytes/slow routes.
- Preparation preserves source license/trademark notices and installed dependency
  license/package metadata under `dist/notices`. Source maps and esbuild input
  manifests accompany generated bundles.
- `dist/manifest.json` records source provenance, build inputs, configuration,
  package lock hash, and every generated/copied file hash. The runner verifies
  inputs and files before running, and records harness hashes in its report.
  Rebuild fixtures after changing a driver, dependency, catalog or preparation code.
- Downloaded archives are cached in `.cache`. Subsequent preparation runs use
  verified cached archives. Generated/cache/output directories are ignored by Git.

## Interpreting results

Execution status is one of `passed`, `assertion_failed`, `execution_error`,
`timeout`, `crash`, `cleanup_error`, or `setup_error`. A successful scenario must
produce a result and satisfy its assertions. Each report includes completed
checkpoints, exceptions, bounded console/process logs and local request records.

Comparisons are `matched` / `different` for successful functional workloads,
`observed` for successful diagnostic workloads, `failed` for candidate execution
failures, `unavailable` for candidate setup failures, or `invalid_reference` when the
Chromium scenario itself fails. Results from an invalid reference are never
reported as a compatibility success. Functional output comparisons preserve
content; no broad normalization hides differences. Readability text alone
normalizes whitespace explicitly in its driver.

Modernizr reports feature probes, not complete implementations. BotD reports
basic automation detection; the upstream project is currently stability-only.
CreepJS touches layout, graphics, fonts, audio, workers, and other intentionally
unsupported surfaces. Its collector findings are useful observations, not new
runtime requirements. The raw JSON preserves its available collector results.

The engines use routes/resource interceptors to restrict ordinary page requests
to the local fixture origin and record blocked external requests. jsdom documents
that synchronous XMLHttpRequest bypasses its resource interceptors; this harness
does not override that API. BotD's documented usage monitoring is off.
This routing is **not an OS network sandbox**: Node DOM script execution, browser service workers, WebRTC,
and browser background services are outside that guarantee. CreepJS's worker
behavior is left intact. This environment, headless Chromium, and platform/API
availability affect detector findings; they do not establish acceptance by a
particular website. Do not interpret a negative bot result as an evasion claim.

## Recorded functional results

The 2026-10-22 run in `results/dom-coverage-upgrade` selects eleven functional
regression workloads, including HS256 and asymmetric/JWE jose checks. All eleven
Chromium references pass. Brimp matches eleven, jsdom matches six, and happy-dom
matches six. The run uses jsdom 30.1.3, happy-dom 21.0.4 and Node 26.8.2.

These are unmodified browser-window workloads, not Node-native library imports or
a general ranking of DOM libraries. In particular, missing APIs in a peer window
do not establish the capabilities of Node's own crypto implementation.

Dexie and the three diagnostic workloads are not part of this selected run.
The historical Dexie result predates core session-owned IndexedDB persistence.
Run the full catalog to record all workloads and Dexie with core storage.
Detector outcomes are observations and do not expand core coverage. See
[core coverage](../../docs/src/content/docs/web-platform/index.md) and
[plugin lifetimes](../../docs/src/content/docs/guides/plugins.md) for supported behavior.
