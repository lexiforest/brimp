import { createRequire } from 'node:module'
import fs from 'node:fs/promises'
import os from 'node:os'
import path from 'node:path'

const require = createRequire(import.meta.url)
const brimp = require(path.resolve(process.argv[3], 'index.js'))
const url = process.argv[2]
const session = await brimp.createSession({ params: { base: 'yes', q: 'old' } })
session.headers['X-Binding'] = 'node'
session.cookies.manual = 'yes'
const response = await session.get(url, { params: { q: ['one', 'two'] } })
const page = response
const initialText = response.html.includes('Hello bindings')
const initialResponse = response.statusCode === 200 && response.ok && response.text.includes('Hello bindings')
const initialQuery = response.url.endsWith('?base=yes&q=one&q=two')
const initialHeaders = response.headers.get('CONTENT-TYPE') === 'text/html; charset=utf-8'
const initialTransfer = response.httpVersion.startsWith('HTTP/') && response.downloadedBytes === response.content.length && response.headerBytes > 0
const title = await page.evaluate('document.title')
const value = await page.evaluate('({answer: 42, values: [true, null]})')
await page.hover('#submit')
await page.click('#submit')
await page.type('#name', 'agent')
await page.tap('#tap')
const inputResult = await page.evaluate("({value: document.querySelector('#name').value, events: inputEvents})")
const evaluationErrors = {}
for (const [name, expression] of Object.entries({javascript: "throw new Error('boom')", unsupported: 'undefined'})) {
  try { await page.evaluate(expression) }
  catch (error) { evaluationErrors[name] = error.code }
}
const directory = await fs.mkdtemp(path.join(os.tmpdir(), 'brimp-node-conformance-'))
const screenshotPath = path.join(directory, 'page.png')
const screenshot = await page.screenshot({ path: screenshotPath })
const inspected = await page.get(url + '/inspect')
const inspection = inspected.json()
const result = {
  title,
  text: initialText,
  response: initialResponse,
  query: initialQuery,
  headers: initialHeaders,
  transfer: initialTransfer,
  state: inspection.header === 'node' && inspection.cookie.includes('manual=yes') && inspection.cookie.includes('server=ready'),
  value,
  input: inputResult.value === 'agent' && inputResult.events.every(event => event.trusted),
  hover: inputResult.events.some(event => event.id === 'submit' && event.type === 'pointermove'),
  touch: inputResult.events.some(event => event.id === 'tap' && event.type === 'click' && event.pointerType === 'touch'),
  png: screenshot.subarray(0, 8).equals(Buffer.from('\x89PNG\r\n\x1a\n', 'binary')) && (await fs.readFile(screenshotPath)).equals(screenshot),
}
const oneShot = await brimp.get(url)
result.oneShot = oneShot.statusCode === 200
await oneShot.close()
await page.post(url + '/echo', { json: { binding: 'node' } })
const posted = page.json()
result.post = posted.method === 'POST' && posted.body === '{"binding":"node"}'
await page.get(url + '/redirect')
result.redirect = page.url === url + '/final' && page.redirectCount === 1 && page.history[0].statusCode === 302
Object.assign(result, evaluationErrors)
await fs.rm(directory, { recursive: true })
const missing = await page.get(url + '/missing')
try { missing.raiseForStatus() }
catch (error) { result.http = error instanceof brimp.HTTPError && error.page === missing }
try { await page.get(url + '/hang', { timeoutMs: 50 }) }
catch (error) { result.timeout = error.code === 'timeout' }
const controller = new AbortController()
controller.abort()
try { await page.get(url + '/hang', { signal: controller.signal }) }
catch (error) { result.cancelled = error.code === 'cancelled' }
await session.close(); await session.close()
try { await page.evaluate('document.title') } catch (error) { result.closed = error.code }
console.log(JSON.stringify(result))
