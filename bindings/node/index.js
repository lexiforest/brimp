const fs = require('node:fs/promises')
const native = require('./brimp_node.node')

class BrimpError extends Error {
  constructor(message, { code = 'internal', cause } = {}) {
    super(message, { cause })
    this.name = this.constructor.name
    this.code = code
  }
}

class ConnectionError extends BrimpError {}
class Timeout extends BrimpError {}
class TooManyRedirects extends ConnectionError {}
class InvalidRequest extends BrimpError {}
class InvalidURL extends InvalidRequest {}
class JavaScriptError extends BrimpError {}

class HTTPError extends BrimpError {
  constructor(message, response) {
    super(message, { code: 'http_status' })
    this.response = response
  }
}

function translate(error) {
  const match = /^brimp ([a-z_]+): (.*)$/s.exec(String(error?.message ?? error))
  if (!match) return new BrimpError(String(error), { cause: error })
  const [, code, detail] = match
  if (code === 'transport') {
    return detail.includes('redirect limit')
      ? new TooManyRedirects(detail, { code, cause: error })
      : new ConnectionError(detail, { code, cause: error })
  }
  if (code === 'timeout') return new Timeout(detail, { code, cause: error })
  if (code === 'invalid_input') {
    return /url/i.test(detail)
      ? new InvalidURL(detail, { code, cause: error })
      : new InvalidRequest(detail, { code, cause: error })
  }
  if (code === 'javascript') return new JavaScriptError(detail, { code, cause: error })
  return new BrimpError(detail, { code, cause: error })
}

async function call(operation) {
  try { return await operation() }
  catch (error) { throw translate(error) }
}

class Headers {
  constructor(entries = []) {
    this._entries = entries.map(([name, value]) => [String(name), String(value)])
    this._values = new Map()
    this._names = new Map()
    for (const [name, value] of this._entries) {
      const key = name.toLowerCase()
      if (!this._names.has(key)) this._names.set(key, name)
      const values = this._values.get(key) ?? []
      values.push(value)
      this._values.set(key, values)
    }
  }

  get(name) {
    const values = this._values.get(String(name).toLowerCase())
    return values ? values.join(', ') : undefined
  }

  getAll(name) {
    return [...(this._values.get(String(name).toLowerCase()) ?? [])]
  }

  has(name) { return this._values.has(String(name).toLowerCase()) }
  entries() { return this[Symbol.iterator]() }
  keys() { return this._names.values() }
  get raw() { return this._entries.map(entry => [...entry]) }

  *values() {
    for (const name of this._names.values()) yield this.get(name)
  }

  *[Symbol.iterator]() {
    for (const name of this._names.values()) yield [name, this.get(name)]
  }
}

class Response {
  constructor(inner) {
    this.statusCode = inner.statusCode
    this.reason = inner.reason
    this.url = inner.url
    this.headers = new Headers(inner.headers)
    this.content = inner.content
    this.html = inner.html ?? null
    this.cookies = Object.fromEntries(inner.cookies)
    this.elapsed = inner.elapsed
  }

  get ok() { return this.statusCode < 400 }

  get encoding() {
    const match = /(?:^|;)\s*charset=([^;\s]+)/i.exec(this.headers.get('content-type') ?? '')
    return match ? match[1].replace(/^["']|["']$/g, '') : 'utf-8'
  }

  get text() {
    try { return new TextDecoder(this.encoding).decode(this.content) }
    catch { return new TextDecoder('utf-8').decode(this.content) }
  }

  json() { return JSON.parse(this.text) }

  raiseForStatus() {
    if (!this.ok) throw new HTTPError(`${this.statusCode} ${this.reason} for url: ${this.url}`, this)
  }
}

function addParams(url, params) {
  if (params == null) return String(url)
  let parsed
  try { parsed = new URL(String(url)) }
  catch (error) {
    throw new InvalidURL(String(error.message ?? error), { code: 'invalid_input', cause: error })
  }
  const entries = params instanceof URLSearchParams ? params : Object.entries(params)
  for (const [name, rawValue] of entries) {
    const values = Array.isArray(rawValue) ? rawValue : [rawValue]
    for (const value of values) parsed.searchParams.append(name, String(value))
  }
  return parsed.toString()
}

class Session {
  constructor(inner) {
    this._inner = inner
    this.headers = {}
    this.cookies = {}
    this._closed = false
  }

  async get(url, options = {}) {
    this._ensureOpen()
    const timeoutMs = options.timeoutMs ?? 30_000
    if (!Number.isInteger(timeoutMs) || timeoutMs <= 0 || timeoutMs > 0xffff_ffff) {
      throw new InvalidRequest('timeoutMs must be a positive 32-bit integer', { code: 'invalid_input' })
    }
    const headers = { ...this.headers, ...(options.headers ?? {}) }
    const protectedHeaders = Object.keys(headers)
      .map(name => name.toLowerCase())
      .filter(name => name === 'user-agent' || name === 'accept-language')
    if (protectedHeaders.length) {
      throw new InvalidRequest(
        `persona-owned headers cannot be overridden: ${[...new Set(protectedHeaders)].sort().join(', ')}; configure a persona instead`,
        { code: 'invalid_input' },
      )
    }
    const cookies = { ...this.cookies, ...(options.cookies ?? {}) }
    if (Object.keys(cookies).length) {
      headers.Cookie = Object.entries(cookies).map(([name, value]) => `${name}=${value}`).join('; ')
    }
    const token = new native.NativeCancellationToken()
    const signal = options.signal
    const cancel = () => token.cancel()
    if (signal?.aborted) cancel()
    signal?.addEventListener('abort', cancel, { once: true })
    try {
      const inner = await call(() => this._inner.get(
        addParams(url, options.params),
        timeoutMs,
        token,
        Object.entries(headers).map(([name, value]) => [String(name), String(value)]),
      ))
      const response = new Response(inner)
      Object.assign(this.cookies, response.cookies)
      return response
    } finally {
      signal?.removeEventListener('abort', cancel)
    }
  }

  async evaluate(expression) {
    this._ensureOpen()
    return JSON.parse(await call(() => this._inner.evaluate(String(expression))))
  }

  async screenshot(options = {}) {
    this._ensureOpen()
    const content = await call(() => this._inner.screenshot(Boolean(options.fullPage)))
    if (options.path != null) await fs.writeFile(options.path, content)
    return content
  }

  async close() {
    if (!this._closed) {
      await call(() => this._inner.close())
      this._closed = true
    }
  }

  _ensureOpen() {
    if (this._closed) throw new BrimpError('session is closed', { code: 'closed' })
  }
}

async function createSession(options = {}) {
  return new Session(await call(() => native.NativeSession.create(options)))
}

async function get(url, options = {}) {
  const sessionKeys = [
    'personaJson', 'caBundle', 'enableWorker', 'enableStreamingNetworking',
    'enableCanvas', 'enableWebGL', 'enableWebGPU', 'enableWebAudio',
    'enableWebAudioOutput', 'storagePath', 'storageQuotaBytes',
  ]
  const sessionOptions = {}
  const requestOptions = { ...options }
  for (const key of sessionKeys) {
    if (key in requestOptions) {
      sessionOptions[key] = requestOptions[key]
      delete requestOptions[key]
    }
  }
  const session = await createSession(sessionOptions)
  try { return await session.get(url, requestOptions) }
  finally { await session.close() }
}

module.exports = {
  BrimpError,
  ConnectionError,
  Headers,
  HTTPError,
  InvalidRequest,
  InvalidURL,
  JavaScriptError,
  Response,
  Session,
  Timeout,
  TooManyRedirects,
  createSession,
  get,
}
