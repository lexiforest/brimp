---
title: API reference
description: References generated from Python source, TypeScript declarations and Clap commands.
---

Choose your interface:

| Reference | Generated from |
| --- | --- |
| [Python package](/reference/python/) | Python source signatures, properties and docstrings, via mkdocstrings |
| [Python plugins](/reference/python/plugins/) | Plugin installers, factories and host context |
| [Node package](/reference/node/) | `bindings/node/index.d.ts`, via TypeDoc |
| [CLI](/api/cli/) | The executable's Clap command definitions |

These references regenerate when the documentation builds. They are not copied
signature lists. For step-by-step examples, start with the [tutorials](/examples/).

## Choose the right boundary

| You want to work with | Use |
| --- | --- |
| Cookies and request defaults | Session |
| Navigation, evaluation, input and routes | Page |
| Original HTTP bytes and metadata | Page's response snapshot |
| Current serialized DOM | `page.html()`; await it in Node |
| Rooted DOM/JavaScript objects | ElementHandle / JSHandle; dispose when finished |
| Specialized APIs and host callbacks | Explicit plugins |

Python timeouts are seconds; Node timeouts are milliseconds. A session owns its
pages. Navigation invalidates the previous realm's handles.

See [core APIs and plugins](/architecture/subsystems/) for browser API coverage
and limits. These host references describe the automation interfaces, not the
entire web platform.
