const native = require('./brimp_node.node')

class BrimpError extends Error {
  constructor(code, message, cause) {
    super(message)
    this.name = 'BrimpError'
    this.code = code
    this.cause = cause
  }
}

function translate(error) {
  const match = /^brimp ([a-z_]+): (.*)$/s.exec(String(error?.message ?? error))
  return match ? new BrimpError(match[1], match[2], error) : new BrimpError('internal', String(error), error)
}

async function call(operation) {
  try { return await operation() }
  catch (error) { throw translate(error) }
}

class Page {
  constructor(inner) { this._inner = inner }
  async goto(url, options = {}) {
    const token = new native.NativeCancellationToken()
    const signal = options.signal
    const cancel = () => token.cancel()
    if (signal?.aborted) cancel()
    signal?.addEventListener('abort', cancel, { once: true })
    try { return await call(() => this._inner.goto(String(url), options.timeoutMs ?? 30000, token)) }
    finally { signal?.removeEventListener('abort', cancel) }
  }
  async evaluate(expression) { return JSON.parse(await call(() => this._inner.evaluate(String(expression)))) }
  title() { return call(() => this._inner.title()) }
  textContent() { return call(() => this._inner.textContent()) }
  screenshot(options = {}) { return call(() => this._inner.screenshot(Boolean(options.fullPage))) }
  close() { return call(() => this._inner.close()) }
}

class Browser {
  constructor(inner) { this._inner = inner }
  async newPage(options = {}) { return new Page(await call(() => this._inner.newPage(options))) }
  close() { return call(() => this._inner.close()) }
}

async function launch(options = {}) {
  return new Browser(await call(() => native.NativeBrowser.launch(options.personaJson)))
}

module.exports = { BrimpError, Browser, Page, launch }
