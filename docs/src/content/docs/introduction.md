---
title: Introduction
description: Use browser JavaScript from a request-and-response workflow.
---

Brimp fetches web pages, runs their JavaScript, and gives you the updated HTML.
Use it from Python, Node.js, or the command line to extract content or automate
small DOM workflows.

## When an HTTP response is not enough

A site may return an HTML shell and use JavaScript to add its useful content.
An HTTP client retrieves the shell. Brimp also executes the page's scripts, so
you can query the document they create, follow links, fill forms, and extract text.

You keep a familiar request-and-response workflow: open a session, request a
page, inspect the result, then close the session. The [quick start](/quick-start/)
shows a heading changing from `Waiting for JavaScript` to `Hello from JavaScript`.

## The objects you work with

- A **session** owns pages and shares cookies, connections, and request defaults.
- A **page** holds the HTTP response and the current document. Its JavaScript can
  change that document after the response arrives.
- The **DOM** is the document tree you query and edit. `page.text` reads the
  original response; `page.html()` serializes the current DOM.

Python operations are synchronous. Node.js operations return Promises. The CLI
provides the same runtime for shell workflows. All three run Brimp in process;
they do not launch Chromium or a CDP server.

## Choose Brimp or a full browser

Brimp suits JavaScript-updated HTML, article extraction, shared-cookie requests,
and DOM interaction that does not depend on visual layout. It includes a
JavaScript engine, HTML parser, and network stack: JavaScriptCore, Lexbor, and
curl-impersonate. Browser profiles configure identity and other browser values.

Use a full browser through Playwright when you need screenshots, layout-based
interaction, media playback, browser workers, extensions, or CDP. Some browser
APIs in Brimp are placeholders so scripts can call them without stopping; check
the [browser support reference](/web-platform/) before relying on their results.

## Start with one workflow

1. [Install the Python or Node.js package](/install/), or build the CLI.
2. [Read your first live page](/quick-start/).
3. Continue with the [Python](/examples/python/), [Node.js](/examples/node/), or
   [CLI](/examples/cli/) tutorial.

For how the engine, DOM, networking, and plugins fit together, see
[Architecture](/architecture/javascript-runtime/).
