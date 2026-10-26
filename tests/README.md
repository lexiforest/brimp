# Browser regression tests

The script-debugging regression also loads the pinned DOMPurify bundle directly,
calls a retained function with multiple inputs, and checks non-consuming
dependency reports against the resulting DOM operations.

Run real browser libraries and applications against Brimp's Node binding. The
suite covers eleven workloads: React and Vue TodoMVC, jQuery, htmx, Lit,
DOMPurify, Readability, Axios, Defuddle, and two jose crypto cases.

## Run locally

Use Node.js 22.19 or newer and the native SDK environment described in
[the binding build instructions](../bindings/README.md).

From the repository root:

```sh
cargo build --locked -p brimp-python -p brimp-node
python3 bindings/prepare_local.py
npm ci --prefix tests --ignore-scripts --no-audit --no-fund
npm run prepare:fixtures --prefix tests
npm test --prefix tests
```

Preparation downloads a pinned, SHA-256-verified TodoMVC archive and copies its
upstream React/Vue builds unchanged. Library drivers use pinned npm dependencies
bundled for the browser. Source licenses, build inputs, and artifact hashes are
preserved under `dist/`. Re-run preparation after changing dependencies or fixtures.

Each workload runs in a separate process with a local HTTP fixture server and a
60-second deadline, including cleanup. Failed assertions, unexpected output,
crashes, and timeouts fail the test run. The JSON report at `results/report.json`
includes observations, checkpoints, console messages, and fixture requests.
CI uploads this report even when a test fails.

To test an installed Node package, set `BRIMP_NODE_BINDING` to its absolute
`index.js` path before running the tests.

## Maintain the expectations

`expected.json` contains the complete functional observations and checkpoints
from a successful Chromium reference run. It records the reference version,
date, report hash, package versions, and upstream source revision. Brimp must
match those results exactly, in addition to passing each driver's assertions.

When changing a workload or its dependencies, verify its behavior in Chromium
using Playwright and review the expected result before updating this file.
The test command never rewrites expectations from Brimp's output.

This suite is self-contained: it does not read `compatibility/`, install a
browser, or load jsdom/happy-dom. Full engine comparisons, coverage audits, and
detector observations remain optional development tools outside CI's regression
gate. Generated fixtures, caches, reports, and `node_modules/` are ignored by Git.
