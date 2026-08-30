import { createRequire } from 'node:module'
import fs from 'node:fs/promises'
import os from 'node:os'
import path from 'node:path'

const require = createRequire(import.meta.url)
const brimp = require(path.resolve(process.argv[3], 'index.js'))
const url = process.argv[2]
const session = await brimp.createSession()
session.headers['X-Binding'] = 'node'
session.cookies.manual = 'yes'
const response = await session.get(url, { params: { q: ['one', 'two'] } })
const title = await session.evaluate('document.title')
const value = await session.evaluate('({answer: 42, values: [true, null]})')
await session.hover('#submit')
await session.click('#submit')
await session.type('#name', 'agent')
await session.tap('#tap')
const inputResult = await session.evaluate("({value: document.querySelector('#name').value, events: inputEvents})")
const evaluationErrors = {}
for (const [name, expression] of Object.entries({javascript: "throw new Error('boom')", unsupported: 'undefined'})) {
  try { await session.evaluate(expression) }
  catch (error) { evaluationErrors[name] = error.code }
}
const directory = await fs.mkdtemp(path.join(os.tmpdir(), 'brimp-node-conformance-'))
const screenshotPath = path.join(directory, 'page.png')
const screenshot = await session.screenshot({ path: screenshotPath })
const inspected = await session.get(url + '/inspect')
const inspection = inspected.json()
const result = {
  title,
  text: response.html.includes('Hello bindings'),
  response: response.statusCode === 200 && response.ok && response.text.includes('Hello bindings'),
  query: response.url.endsWith('?q=one&q=two'),
  headers: response.headers.get('CONTENT-TYPE') === 'text/html; charset=utf-8',
  state: inspection.header === 'node' && inspection.cookie.includes('manual=yes') && inspection.cookie.includes('server=ready'),
  value,
  input: inputResult.value === 'agent' && inputResult.events.every(event => event.trusted),
  hover: inputResult.events.some(event => event.id === 'submit' && event.type === 'pointermove'),
  touch: inputResult.events.some(event => event.id === 'tap' && event.type === 'click' && event.pointerType === 'touch'),
  png: screenshot.subarray(0, 8).equals(Buffer.from('\x89PNG\r\n\x1a\n', 'binary')) && (await fs.readFile(screenshotPath)).equals(screenshot),
  oneShot: (await brimp.get(url)).statusCode === 200,
}
Object.assign(result, evaluationErrors)
await fs.rm(directory, { recursive: true })
const missing = await session.get(url + '/missing')
try { missing.raiseForStatus() }
catch (error) { result.http = error instanceof brimp.HTTPError && error.response === missing }
try { await session.get(url + '/hang', { timeoutMs: 50 }) }
catch (error) { result.timeout = error.code === 'timeout' }
const controller = new AbortController()
controller.abort()
try { await session.get(url + '/hang', { signal: controller.signal }) }
catch (error) { result.cancelled = error.code === 'cancelled' }
await session.close(); await session.close()
try { await session.evaluate('document.title') } catch (error) { result.closed = error.code }
console.log(JSON.stringify(result))
