# Brimp Node binding

Brimp exposes an asynchronous request/response API backed by the shared native
browser runtime. It runs in process and does not start a CDP server.

```js
const brimp = require('@brimp/brimp')

async function main() {
  const response = await brimp.get('https://example.com')
  console.log(response.statusCode)
  console.log(response.text) // original HTTP response
  console.log(response.html) // DOM after JavaScript
}
```

Persistent cookies, connections, JavaScript evaluation, screenshots, and
`AbortSignal` cancellation are available through a session:

```js
async function main() {
  const session = await brimp.createSession()
  try {
    const response = await session.get('https://example.com', { timeoutMs: 30_000 })
    response.raiseForStatus()
    console.log(await session.evaluate('document.title'))
    await session.hover('#menu')
    await session.type('#name', 'agent')
    await session.click('#submit')
    await session.screenshot({ path: 'page.png', fullPage: true })
  } finally {
    await session.close()
  }
}
```

Worker, streaming-networking, persistent-storage, Canvas 2D, WebGL, WebGPU,
and WebAudio APIs are disabled by default. Enable only the required surfaces
through `createSession(options)`. `enableWebAudio` uses a device-free sink;
`enableWebAudioOutput` also enables WebAudio and authorizes the system output
device.

This API is for asynchronous rendered-page extraction. Use `brimp cdp` when a
Puppeteer or Playwright browser/page interface is required. See `SUPPORT.md`
for the exact tested surface.
