---
title: Extract, intercept and stream
description: Turn live DOM into Markdown, provide a fixture response and download a body in chunks.
---

Start the [local tutorial server](/quick-start/#1-start-the-example-site).
Each example below opens and closes its own session.

## 1. Extract an edited article

Install Defuddle explicitly in the session, change the heading, then extract the
current document:

```python
import brimp

with brimp.Session(plugins=[brimp.plugins.extraction()]) as session:
    page = session.get("http://127.0.0.1:8765/")
    heading = page.query_selector("h1")
    if heading is not None:
        with heading:
            heading.set_text_content("Edited article")
    article = page.extract()
    print(article["contentMarkdown"])
```

The Markdown contains `Edited article`. Extraction uses DOM, attributes and inline
styles; it does not need a screenshot or layout engine.

The same workflow in Node:

```js
import { createSession, extractionPlugin } from '@brimp/brimp';

const session = await createSession({plugins: [extractionPlugin()]});
try {
  const page = await session.get('http://127.0.0.1:8765/');
  const heading = await page.querySelector('h1');
  if (heading) {
    try {
      await heading.setTextContent('Edited article');
    } finally {
      await heading.dispose();
    }
  }
  console.log((await page.extract()).contentMarkdown);
} finally {
  await session.close();
}
```

## 2. Supply an API response with a route

Register the route before making the request. This one supplies JSON without
sending the matching request to the server:

```js
import { createSession } from '@brimp/brimp';

const session = await createSession();
try {
  const page = await session.newPage();
  page.route('http://127.0.0.1:8765/api/*', route => route.fulfill({
    headers: {'Content-Type': 'application/json'},
    body: JSON.stringify({source: 'route'}),
  }));
  await page.get('http://127.0.0.1:8765/');
  console.log(await page.evaluate('fetch("/api/status").then(r => r.json())'));
} finally {
  await session.close();
}
```

Expect `{ source: 'route' }`. The last matching rule wins. Each callback must
fulfill, continue, fail or authenticate exactly once; returning without a decision
aborts the request. Callbacks cannot reenter their own page.

## 3. Download without replacing the DOM

```python
from pathlib import Path
import brimp

with brimp.Session() as session:
    page = session.new_page()
    with page.stream("GET", "http://127.0.0.1:8765/download") as response:
        with Path("download.bin").open("wb") as output:
            for chunk in response:
                output.write(chunk)
    print(Path("download.bin").stat().st_size)
```

Expect `53248` bytes. The context manager cancels an abandoned stream. Streaming
does not navigate or replace the page's DOM.

In Node, pipe the readable body to a file:

```js
import { createWriteStream } from 'node:fs';
import { pipeline } from 'node:stream/promises';
import { createSession } from '@brimp/brimp';

const session = await createSession();
try {
  const page = await session.newPage();
  const response = await page.stream('GET', 'http://127.0.0.1:8765/download');
  await pipeline(response.body, createWriteStream('download.bin'));
} finally {
  await session.close();
}
```

Native streaming uses a bounded queue and backpressure. Response interception
has a separate 32 MiB buffered limit; it is not a streaming route handler.

Next: [add a custom plugin](/examples/plugins/#3-connect-page-code-to-a-host-handler)
or explore the [generated API references](/api/).
