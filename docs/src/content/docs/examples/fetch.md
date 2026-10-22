---
title: Fetch and extract
---

```python
from brimp import Session
from brimp.plugins import extraction
with Session(plugins=[extraction()], headers={"X-Client": "example"}) as session:
    page = session.get("https://example.com")
    page.query_selector("h1").set_text_content("Edited")
    print(page.extract()["contentMarkdown"])
    with page.stream("GET", "https://example.com/data") as response:
        for chunk in response:
            consume(chunk)
```

```javascript
import { createSession, Plugin } from 'brimp';
const plugin = new Plugin('example', '({global,host}) => { global.lookup = value => host.call("lookup", value); }', {
  lookup: async (context, value) => ({value})
});
const session = await createSession({plugins: [plugin]});
try {
  const page = await session.newPage();
  page.route('https://example.com/api*', route => route.fulfill({body: '{"ok":true}'}));
  console.log(await page.evaluate('lookup(42)'));
} finally { await session.close(); }
```

Route callbacks complete exactly once; exceptions and missing decisions abort.
Plugin callbacks receive copied values and cannot reenter their page. Storage,
Canvas, graphics/audio, and workers are optional or unavailable as described in
[Subsystem implementation](/architecture/subsystems/).
