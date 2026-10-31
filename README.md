# Brimp

> [!WARNING]
> Brimp is in alpha stage, API is not stable and subject to change.

Getting the html with `curl_cffi` is easy, but oftentimes, you need to render the page
with JavaScript, and some websites rejects you because you are not using a browser.

Brimp is the browser runtime for agents and bots, it tries to add the missing JS and
DOM layer for curl_cffi. So you can get not only static html pages, but also the
dynamically rendered ones, and you can even interact with the page, a more browser-ish
experience.

Under the hood, brimp combines JavaScriptCore from WebKit and curl-impersonate
as the network stack. But, if you are familiar with `requests` or `curl_cffi`, it feels
like the same simple request/response workflow with a JavaScript-rendered HTML result.
Brimp is written in Rust, and offers both Python and Node.js bindings.

Documentation: [docs.brimp.ai](https://docs.brimp.ai)

## Features

- Page rendered with DOM and JS, so you don't get the empty html placeholder.
- `curl-impersonate` as the network stack, providing perfect Ja3/TLS, http 2&3 fingerprints.
- The familiar `curl_cffi` API and experience from the same maintainer.
- Much faster than playwright with Chromium, on par with popular alternatives.
- Plugin system for you to fill the gaps between any website that's blocking you.
- Python and Nodejs bindings.
- Pre-compiled, so you don't have to compile on your machine.
- MIT licensed.

||chromium|camoufox|cloakbrowser|lightpanda|obscura|brimp|
|---|---|---|---|---|---|---|
|JS & DOM|✅|✅|✅|✅|✅|✅|
|http/3|✅|✅|✅|❌|❌|✅|
|CDP|✅|✅|✅|☑️|☑️|❌<sup>1</sup>|
|requests-like|❌|❌|❌|❌|❌|✅|
|screenshot|✅|✅|✅|❌|✅|❌|
|JS engine|V8|SpiderMonkey|V8|V8|V8|JSC|
|open source|✅|✅|❌|☑️<sup>2</sup>|✅|✅|
|ja3 fingerprints|☑️<sup>3</sup>|☑️<sup>3</sup>|☑️<sup>3</sup>|❌|✅|✅|
|fast?|🐢|🐢|🐢|🐇|🐇|🐇|

<small>
Notes:
<ol>
<li>This is a deliberate decision.</li>
<li>Lightpanda is under the AGPL license.</li>
<li>Fingerprints are fixed to the browser version</li>
</ol>
</small>

For performance comparisons, see the [benchmark](benchmark/native/README.md).

## Install

### For humans

CLI:

```
curl -fsSL https://brimp.ai/install.sh | sh
```

You can also download the prebuilt binary tarballs from github release page.

Python:

```sh
pip install brimp
```

Node.js:

```
npm install @brimp/brimp
```

### For agents

Paste this prompt:

```plain
Install this tool: https://github.com/lexiforest/brimp
```

## Usage

### CLI

The CLI runs the browser runtime in-process.

Check the runtime, extract a live page, evaluate JavaScript, or save the HTML:

```sh
# Run a JavaScript expression on a page
brimp fetch https://example.com --eval 'document.title'

# Extract markdown
brimp fetch https://example.com --output example.md

# Capture a profile from a real browser
brimp profile capture --output browser-profile.json

# Use that profile to visit a website
brimp fetch https://example.com --profile browser-profile.json
```

### Python

The Python binding is synchronous and in-process. `page.text` is the
original HTTP response text; `Page.html()` is the live DOM serialized after
page scripts execute.

```python
import brimp

with brimp.Session() as session:
    page = session.get("https://example.com")
    print(page.status_code)
    print(page.html())
```

Use a Session to retain cookies and connections and to access the current
rendered document:

```python
import brimp

with brimp.Session() as session:
    page = session.get("https://example.com/?q=browser")
    page.raise_for_status()
    heading = page.query_selector("h1")
    if heading is not None:
        print(heading.text_content())
        heading.dispose()
```

### Node.js

```js
import { createSession } from '@brimp/brimp';

const session = await createSession();
try {
  const page = await session.get('https://example.com');
  console.log(await page.evaluate('document.title'));
} finally {
  await session.close();
}
```

## Development

### Architecture

The implemented runtime supports:

- static HTML, JavaScript DOM mutation, selectors, inline styles, and non-rendering CSSOM;
- HTTP(S) navigation through a transport-neutral `ResourceLoader`;
- linked CSS, classic inline/external scripts, `defer`, and `async`;
- parser pausing around blocking scripts;
- events, timers, microtasks, Promise-based `fetch`, cookies, `Location`, and `Navigator`.

The canonical owner-thread automation API is exposed through:

- the `brimp` CLI for evaluation and page extraction;
- the Python and Node.js Session/Page bindings.

The CLI and bindings delegate browser operations to `brimp-runtime` in-process.
The runtime uses `brimp-lexbor` for DOM, `brimp-jsc` for JavaScriptCore, and
`brimp-net` for networking.

### CI

The project is tested and built automatically on GitHub Actions. All PR must pass
the automatic tests.

### Testing

```sh
cargo build -p brimp-cli -p brimp-python -p brimp-node
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## License

MIT.
