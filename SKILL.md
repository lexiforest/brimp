---
name: brimp
description: Install and use Brimp, a browser JavaScript runtime for agents. Use when asked to install lexiforest/brimp, extract JavaScript-rendered web content, or integrate Brimp into a Python or Node.js project.
---

# Brimp

Scalable browser runtime for agents.

Brimp executes browser JavaScript and DOM operations with JavaScriptCore and
curl-impersonate. It provides a standalone CLI and in-process Python and Node.js
bindings. It does not paint pages or provide screenshots or a CDP endpoint.

## Install

For a request such as `Install this tool: https://github.com/lexiforest/brimp`,
install the **CLI**, then verify it below. Use the Python or Node.js package when
the user asks for that interface or is integrating Brimp into that language's
application. Install only the interface needed for the task.

Published distributions include native dependencies. A normal installation needs
no repository checkout, Rust toolchain, SDK preparation or Chromium download.

Supported systems:

| System | Requirement |
| --- | --- |
| macOS | Apple silicon, macOS 11 or newer |
| Linux | x86_64 or ARM64, glibc 2.34 or newer |
| Windows | x64 |

If the host is unsupported or no matching release asset is available, report the
specific blocker. Use the [source build guide](https://docs.brimp.ai/development/build-from-source/)
when the user requests a source build.

### CLI on macOS or Linux

```sh
curl -fsSL https://brimp.ai/install.sh | sh
```

The installer verifies SHA-256, checks runtime initialization and places `brimp`
in `$HOME/.local/bin`, with its native libraries under `$HOME/.local/lib/brimp`.
If the binary directory is outside `PATH`, use the full executable path for
verification and report the PATH instruction printed by the installer.

To honor a requested version or installation directory, pass installer options:

```sh
curl -fsSL https://brimp.ai/install.sh | sh -s -- --version vX.Y.Z --prefix /chosen/prefix
```

Replace the example version and directory with the requested values.

### CLI on Windows

Find the requested version, or the latest release, at
[GitHub Releases](https://github.com/lexiforest/brimp/releases).
Download `brimp-v<VERSION>-x86_64-pc-windows-msvc.zip` and its `.sha256` sidecar.
Compare the archive's `Get-FileHash -Algorithm SHA256` result with the sidecar
before extracting it. Extract the complete directory to a user-writable install
location and keep `brimp.exe`, its DLLs and licenses together. Run the executable
by its full path; report the directory to add to `PATH`.

### Python

Use Python 3.11 or newer and the project's existing environment. If no environment
exists, create a virtual environment with `python3 -m venv .venv` (use `python`
on Windows). Install with that environment's interpreter:

```sh
python -m pip install brimp
```

Verify using the same interpreter:

```python
import brimp

with brimp.Session() as session:
    page = session.new_page()
    page.evaluate('document.body.innerHTML = "<h1>Brimp ready</h1>"')
    assert page.evaluate("document.querySelector('h1').textContent") == "Brimp ready"
```

### Node.js

Use Node.js 22.19 or newer. Install `@brimp/brimp` with the project's package
manager (`npm install @brimp/brimp` in an npm project). Keep platform optional
dependencies enabled: they supply the native runtime.

Run this verification as an ES module (`.mjs`, or `node --input-type=module`):

```js
import { createSession } from '@brimp/brimp';

const session = await createSession();
try {
  const page = await session.newPage();
  await page.evaluate('document.body.innerHTML = "<h1>Brimp ready</h1>"');
  const text = await page.evaluate("document.querySelector('h1').textContent");
  if (text !== 'Brimp ready') throw new Error(`Unexpected DOM result: ${text}`);
} finally {
  await session.close();
}
```

## Verify the CLI

Run the installed executable, using its full path if needed:

```sh
brimp --version
brimp doctor
printf '%s\n' 'document.body.innerHTML = "<h1>Brimp ready</h1>"; document.querySelector("h1").textContent' | brimp run -
```

Expect all four doctor fields (`javascriptCore`, `lexbor`, `libcurlImpersonate`,
`crypto`) to be `ok`, and the final command to print the JSON string
`"Brimp ready"`. On PowerShell, pipe the JavaScript string to `brimp.exe run -`
without `printf`. These checks need no network access.

Report the installed version, executable path or project environment, and the
verification results. Include any remaining PATH step. An installation request
does not require fetching a public site or changing application code.

## Use Brimp

```sh
brimp fetch https://example.com --format markdown
brimp fetch https://example.com --eval '({title: document.title, links: document.querySelectorAll("a").length})'
brimp fetch https://example.com --output page.html
```

`fetch` navigates and runs page scripts. `--eval` writes a JSON value; `--format
markdown` extracts article content. Output extensions also select formats, and
overwriting a file requires `--overwrite`. `run` executes classic or bundled
JavaScript in a browser realm; bundle module imports first.

For application code, use `session.get(url)` and retain the session for shared
cookies and connections. Its `Page` result owns the HTTP metadata and live DOM;
call `page.raise_for_status()` in Python or `page.raiseForStatus()` in Node.js to
raise `HTTPError` for 4xx/5xx statuses. `page.text` is the original response text; `page.html()`
serializes the live DOM. Python calls are synchronous; Node navigation and
evaluation return Promises. Close sessions when finished. Python request timeouts use
seconds; Node request timeouts use milliseconds. Return a Promise from
`evaluate()` to wait for asynchronous work before reading its result.

Read the relevant reference when the task needs more than the examples above:

- [Installation](https://docs.brimp.ai/install/) for platform requirements and package setup.
- [CLI tutorial](https://docs.brimp.ai/examples/cli/) and `brimp <command> --help` for command options.
- [Python reference](https://docs.brimp.ai/reference/python/) or [Node.js reference](https://docs.brimp.ai/reference/node/) for binding signatures.
- [Sessions, pages and requests](https://docs.brimp.ai/guides/automation/) for cookies, routes and asynchronous results.
- [Browser support](https://docs.brimp.ai/web-platform/) for implemented web APIs and rendering limits.
