import { createRequire } from 'node:module'
import path from 'node:path'

const require = createRequire(import.meta.url)
const brimp = require(path.resolve(process.argv[3], 'index.js'))
const url = process.argv[2]
const browser = await brimp.launch()
const page = await browser.newPage()
await page.goto(url)
const result = {
  title: await page.title(),
  text: (await page.textContent()).includes('Hello bindings'),
  value: await page.evaluate('({answer: 42, values: [true, null]})'),
  png: (await page.screenshot()).subarray(0, 8).equals(Buffer.from('\x89PNG\r\n\x1a\n', 'binary')),
}
for (const [name, expression] of Object.entries({javascript: "throw new Error('boom')", unsupported: 'undefined'})) {
  try { await page.evaluate(expression) }
  catch (error) { result[name] = error.code }
}
const hanging = await browser.newPage()
try { await hanging.goto(url + '/hang', { timeoutMs: 50 }) }
catch (error) { result.timeout = error.code === 'timeout' }
await hanging.close(); await hanging.close()
await page.close(); await page.close()
try { await page.title() } catch (error) { result.closed = error.code }
await browser.close(); await browser.close()
console.log(JSON.stringify(result))
