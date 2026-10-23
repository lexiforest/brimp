---
title: Node.js tutorial
description: Navigate, query the DOM, save HTML, cancel requests and configure identity.
---

**You'll build:** an asynchronous page workflow using `@brimp/brimp`.
Save each example in an `.mjs` file and run it with Node.js 22.19 or newer.

Start the [local tutorial server](/quick-start/#1-start-the-example-site) first.

## 1. Create a session and read a page

```js
import { createSession } from '@brimp/brimp';

const session = await createSession();
try {
  const page = await session.get('http://127.0.0.1:8765/');
  console.log(page.statusCode);
  console.log(await page.evaluate('document.title'));
} finally {
  await session.close();
}
```

Expect `200` and `Brimp tutorial`. Closing the session closes all its pages.
Use `session.newPage()` when you need to attach routes or event listeners before
navigating with `page.get()`.

## 2. Work with DOM handles

```js
import { createSession } from '@brimp/brimp';

const session = await createSession();
try {
  const page = await session.get('http://127.0.0.1:8765/');
  const heading = await page.querySelector('h1');
  if (heading) {
    try {
      await heading.setTextContent('Edited by Node');
      console.log(await heading.textContent());
    } finally {
      await heading.dispose();
    }
  }
} finally {
  await session.close();
}
```

Expect `Edited by Node`. A detached node keeps its identity, but a handle from a
previous navigation raises `StaleHandle`.

## 3. Evaluate structured data and save HTML

```js
import { writeFile } from 'node:fs/promises';
import { createSession } from '@brimp/brimp';

const session = await createSession();
try {
  const page = await session.get('http://127.0.0.1:8765/');
  const result = await page.evaluate(`({
    title: document.title,
    links: document.querySelectorAll('a').length
  })`);
  console.log(result);
  await writeFile('example.html', await page.html(), 'utf8');
} finally {
  await session.close();
}
```

The result contains `title: 'Brimp tutorial'` and `links: 1`. Evaluation returns
JSON-compatible values and awaits Promises. File access runs in Node; page
JavaScript has no Node `process`, `require` or filesystem API.

## 4. Cancel a slow request

The fixture's `/slow` endpoint waits three seconds. Abort it after 100 ms:

```js
import { createSession } from '@brimp/brimp';

const session = await createSession();
const controller = new AbortController();
const timer = setTimeout(() => controller.abort(), 100);
try {
  await session.get('http://127.0.0.1:8765/slow', {
    timeout: 10_000,
    signal: controller.signal,
  });
} catch (error) {
  if (error.code !== 'cancelled') throw error;
  console.log('Request cancelled');
} finally {
  clearTimeout(timer);
  await session.close();
}
```

Timeouts use **milliseconds**. Requests and streams accept an `AbortSignal`.

## 5. Set identity and keep cookies

```js
import { createSession } from '@brimp/brimp';

const session = await createSession({identity: {languages: ['en-US', 'en']}});
try {
  session.setCookie('http://127.0.0.1:8765/', 'experiment', 'rendered');
  const page = await session.get('http://127.0.0.1:8765/echo');
  console.log(page.json().cookie);
  console.log(await page.evaluate('navigator.languages'));
} finally {
  await session.close();
}
```

Expect a cookie header containing `experiment=rendered` and `['en-US', 'en']`.
`identity` accepts `userAgent`, `languages` and `platform`; full persona-file
loading is available through the [CLI](/examples/cli/#4-use-a-persona).

Next: [extract and stream content](/examples/fetch/), or browse the
[generated Node API](/reference/node/).
