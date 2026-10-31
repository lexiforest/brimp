# Node

See the [Node.js tutorial](https://docs.brimp.ai/examples/node/)
and [development setup](https://docs.brimp.ai/development/).

Browser subsystem plugins are enabled by default. An explicit `plugins` list
replaces them; `createSession({plugins: []})` selects core only. Use
`browserPlugins()` to extend or customize the default set.

```javascript
import { createSession } from '@brimp/brimp';

const session = await createSession({ headers: { 'X-Client': 'native-example' } });
try {
  const page = await session.get('https://example.com');
  page.raiseForStatus();
  console.log(page.statusCode, page.url);
  const heading = await page.querySelector('h1');
  await heading.setTextContent('Edited in the DOM');
  console.log(await heading.textContent());
  await heading.dispose();
  console.log(await page.evaluate('Promise.resolve(document.title)'));
} finally {
  await session.close();
}
```
