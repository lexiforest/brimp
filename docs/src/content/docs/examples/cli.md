---
title: CLI tutorial
description: Check your build, evaluate page JavaScript, save HTML and use browser profiles.
---

**You'll build:** a shell workflow that fetches a page and writes its useful
content to a file. Start the [local tutorial server](/quick-start/#1-start-the-example-site)
before the fetch commands.

## 1. Check native dependencies

```sh
brimp doctor
```

Successful output is JSON:

```json
{"crypto":"ok","javascriptCore":"ok","lexbor":"ok","libcurlImpersonate":"ok"}
```

This checks local initialization and a crypto digest. It does not test internet
access or whether a proxy can reach a site.

## 2. Extract structured data

```sh
brimp fetch http://127.0.0.1:8765/   --eval '({title: document.title, links: document.querySelectorAll("a").length})'
```

Expect `{"title":"Brimp tutorial","links":1}` (key order may differ).
The evaluated JSON value goes to stdout; diagnostics go to stderr.

Set a shorter deadline with a unit such as `s` or `ms`:

```sh
brimp fetch http://127.0.0.1:8765/ --timeout 10s   --eval 'document.querySelector("h1").textContent'
```

Expect `"Hello from JavaScript"`. The timeout covers the whole command, including
navigation, waits, scripts and extraction.

## 3. Save HTML or Markdown

```sh
brimp fetch http://127.0.0.1:8765/ --output example.html
brimp fetch http://127.0.0.1:8765/ --output example.md
```

The extensions select live HTML or extracted Markdown. Brimp refuses to replace
an existing file unless you add `--overwrite`. Use `--format raw` for original
response bytes; `--format json` means an extracted article object.

## 4. Use a profile

Run `brimp profile capture --output browser-profile.json`, open the printed localhost
URL in your browser, and click **Capture**. After it saves the profile:

```sh
brimp fetch http://127.0.0.1:8765/   --profile browser-profile.json   --eval '({ua: navigator.userAgent, platform: navigator.platform})'
```

The result reflects the supplied profile. For individual overrides, use
`--user-agent`, `--languages` or `--platform` without `--profile`.

## 5. Run browser JavaScript without navigating

```sh
printf '%s\n' 'document.body.innerHTML = "<h1>Local script</h1>"; document.body.textContent' | brimp run -
```

Expect `"Local script"`. Use `brimp run script.js` for a file. Scripts run in a
browser realm; bundle imports before passing a script to `run`.

## 6. Load a saved HTML document

```sh
cat > saved.html <<'HTML'
<!doctype html><title>Saved</title><a id="next" href="next?x=1">next</a>
<script>globalThis.pageScriptRan = true</script>
HTML

printf '%s\n' 'void 0' | brimp run - --html saved.html \
  --html-url https://example.com/dir/page.html \
  --eval '({url: location.href, href: document.getElementById("next").href,
            pageScript: typeof globalThis.pageScriptRan})'
```

Expect the link to resolve to `https://example.com/dir/next?x=1` and
`pageScript` to be `undefined`. `--html` parses the file at `--html-url` (default
`about:blank`) without running page scripts or fetching subresources. Init
scripts still run. Do not combine `--html` with a navigation URL.

Continue with [CLI workflows](/examples/cli-workflows/) for plugins, actions,
routes, streaming and tracing. The [generated CLI reference](/api/cli/) lists
all command flags and defaults.
