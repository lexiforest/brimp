# Node

See [native binding setup and current scope](../README.md).

```javascript
import { createSession } from 'brimp';

const session = await createSession({ headers: { 'X-Client': 'native-example' } });
try {
  const page = await session.get('https://example.com');
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
