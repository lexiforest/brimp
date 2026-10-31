---
title: API reference
description: Look up Python and Node.js methods or CLI commands and options.
---

Choose your interface:

| Reference | What you'll find |
| --- | --- |
| [Python package](/reference/python/) | Session, Page, handles, response data and exceptions |
| [Python plugins](/reference/python/plugins/) | Plugin installers, factories and host context |
| [Node package](/reference/node/) | Session and page methods, options, event records and TypeScript types |
| [CLI](/api/cli/) | Commands, flags, defaults and accepted values |

For a complete program and its expected output, start with the [tutorials](/examples/).

## Find the object you need

| You want to work with | Use |
| --- | --- |
| Cookies and request defaults | Session |
| Navigation, evaluation, input and routes | Page |
| Original HTTP bytes and metadata | Page's response snapshot |
| Current serialized DOM | `page.html()`; await it in Node |
| A DOM node or JavaScript object kept for later use | ElementHandle / JSHandle; dispose when finished |
| Browser subsystems and their limits | [Browser support](/web-platform/) |
| Extraction and custom host callbacks | [Plugins](/guides/plugins/) |

Python timeouts are seconds; Node timeouts are milliseconds. A session owns its
pages. Navigation invalidates the previous realm's handles.

See [browser support](/web-platform/) for the JavaScript APIs available inside a
page. The references above describe the Python, Node.js and CLI interfaces.
