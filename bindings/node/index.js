const fs = require('node:fs/promises')
const fsSync = require('node:fs')
const crypto = require('node:crypto')
const path = require('node:path')
const native = require('./brimp_node.node')

class RequestError extends Error {
  constructor(message, { code = 'internal', cause } = {}) {
    super(message, { cause })
    this.name = this.constructor.name
    this.code = code
  }
}
class ConnectionError extends RequestError {}
class Timeout extends RequestError {}
class TooManyRedirects extends ConnectionError {}
class InvalidRequest extends RequestError {}
class InvalidURL extends InvalidRequest {}
class JavaScriptError extends RequestError {}
class SessionClosed extends RequestError {}
class HTTPError extends RequestError {
  constructor(message, page) {
    super(message, { code: 'http_status' })
    this.page = page
  }
}

function translate(error) {
  const match = /^brimp ([a-z_]+): (.*)$/s.exec(String(error?.message ?? error))
  if (!match) return new RequestError(String(error), { cause: error })
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
  if (code === 'closed') return new SessionClosed(detail, { code, cause: error })
  return new RequestError(detail, { code, cause: error })
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
  getAll(name) { return [...(this._values.get(String(name).toLowerCase()) ?? [])] }
  has(name) { return this._values.has(String(name).toLowerCase()) }
  entries() { return this[Symbol.iterator]() }
  keys() { return this._names.values() }
  get raw() { return this._entries.map(entry => [...entry]) }
  *values() { for (const name of this._names.values()) yield this.get(name) }
  *[Symbol.iterator]() { for (const name of this._names.values()) yield [name, this.get(name)] }
}

class Multipart {
  constructor() { this._parts = [] }
  addPart({ name, data, localPath, filename, contentType } = {}) {
    if ((data == null) === (localPath == null)) {
      throw new InvalidRequest('multipart part requires exactly one of data or localPath', { code: 'invalid_input' })
    }
    this._parts.push({ name: String(name), data, localPath, filename, contentType })
    return this
  }
  encode() {
    const boundary = `----BrimpFormBoundary${crypto.randomBytes(12).toString('hex')}`
    const chunks = []
    let size = 0
    const append = value => {
      const buffer = Buffer.isBuffer(value) ? value : Buffer.from(value)
      size += buffer.length
      if (size > 64 * 1024 * 1024) throw new InvalidRequest('multipart body exceeds 67108864 bytes', { code: 'invalid_input' })
      chunks.push(buffer)
    }
    for (const part of this._parts) {
      const payload = part.localPath == null ? part.data : fsSync.readFileSync(part.localPath)
      const name = part.name.replaceAll('"', '%22').replaceAll('\r', '%0D').replaceAll('\n', '%0A')
      let disposition = `Content-Disposition: form-data; name="${name}"`
      const filename = part.filename ?? (part.localPath == null ? undefined : path.basename(part.localPath))
      if (filename != null) disposition += `; filename="${String(filename).replaceAll('"', '%22')}"`
      append(`--${boundary}\r\n${disposition}\r\n`)
      if (part.contentType != null) append(`Content-Type: ${part.contentType}\r\n`)
      append('\r\n'); append(payload); append('\r\n')
    }
    append(`--${boundary}--\r\n`)
    return { body: Buffer.concat(chunks), contentType: `multipart/form-data; boundary=${boundary}` }
  }
}

function paramEntries(params) {
  if (params == null) return []
  const entries = params instanceof URLSearchParams ? params.entries() : Object.entries(params)
  return [...entries].flatMap(([name, value]) => (Array.isArray(value) ? value : [value]).map(item => [String(name), String(item)]))
}

function mergeParams(defaults, overrides) {
  const base = paramEntries(defaults)
  const extra = paramEntries(overrides)
  if (overrides == null) return base
  const replaced = new Set(extra.map(([name]) => name))
  return base.filter(([name]) => !replaced.has(name)).concat(extra)
}

function addParams(url, params) {
  if (params.length === 0) return String(url)
  let parsed
  try { parsed = new URL(String(url)) }
  catch (error) { throw new InvalidURL(String(error.message ?? error), { code: 'invalid_input', cause: error }) }
  for (const [name, value] of params) parsed.searchParams.append(name, value)
  return parsed.toString()
}

function prepareBody(options, headers) {
  const supplied = ['data', 'content', 'json', 'multipart'].filter(key => options[key] != null)
  if (supplied.length > 1) throw new InvalidRequest('use only one of data, content, json, or multipart', { code: 'invalid_input' })
  let body
  let contentType
  if (options.multipart != null) {
    if (!(options.multipart instanceof Multipart)) throw new InvalidRequest('multipart must be a Multipart instance', { code: 'invalid_input' })
    ;({ body, contentType } = options.multipart.encode())
  } else if (options.json != null) {
    body = Buffer.from(JSON.stringify(options.json)); contentType = 'application/json'
  } else if (options.content != null) {
    body = Buffer.isBuffer(options.content) ? options.content : Buffer.from(options.content)
  } else if (options.data != null) {
    if (Buffer.isBuffer(options.data) || typeof options.data === 'string') body = Buffer.from(options.data)
    else { body = Buffer.from(new URLSearchParams(options.data).toString()); contentType = 'application/x-www-form-urlencoded' }
  }
  const names = new Set(Object.keys(headers).map(name => name.toLowerCase()))
  if (body != null && contentType != null && !names.has('content-type')) headers['Content-Type'] = contentType
  if (body != null && !names.has('content-length')) headers['Content-Length'] = String(body.length)
  return body
}

class Page {
  constructor(inner, session) {
    this._inner = inner
    this._session = session
    this._closed = false
    this._ownsSession = false
    this._navigated = false
  }
  async request(method, url, options = {}) {
    this._ensureOpen()
    method = String(method).toUpperCase()
    const timeoutMs = options.timeoutMs ?? this._session.timeoutMs
    if (!Number.isInteger(timeoutMs) || timeoutMs <= 0 || timeoutMs > 0xffff_ffff) {
      throw new InvalidRequest('timeoutMs must be a positive 32-bit integer', { code: 'invalid_input' })
    }
    const headers = { ...this._session.headers, ...(options.headers ?? {}) }
    const protectedHeaders = Object.keys(headers).map(name => name.toLowerCase()).filter(name => name === 'user-agent' || name === 'accept-language')
    if (protectedHeaders.length) {
      throw new InvalidRequest(`persona-owned headers cannot be overridden: ${[...new Set(protectedHeaders)].sort().join(', ')}; configure a persona instead`, { code: 'invalid_input' })
    }
    const referer = options.referer ?? this._session.referer
    if (referer != null && !Object.keys(headers).some(name => name.toLowerCase() === 'referer')) headers.Referer = String(referer)
    const auth = options.auth ?? this._session.auth
    if (auth != null) {
      if (!Array.isArray(auth) || auth.length !== 2) throw new InvalidRequest('auth must be a [username, password] pair', { code: 'invalid_input' })
      headers.Authorization = `Basic ${Buffer.from(`${auth[0]}:${auth[1]}`).toString('base64')}`
    }
    const body = prepareBody(options, headers)
    if ((method === 'GET' || method === 'HEAD') && body != null) throw new InvalidRequest(`${method} navigation cannot have a body`, { code: 'invalid_input' })
    const cookies = { ...this._session.cookies, ...(options.cookies ?? {}) }
    const token = new native.NativeCancellationToken()
    const signal = options.signal
    const cancel = () => token.cancel()
    if (signal?.aborted) cancel()
    signal?.addEventListener('abort', cancel, { once: true })
    try {
      const result = await call(() => this._inner.request(
        method, addParams(url, mergeParams(this._session.params, options.params)), timeoutMs, token,
        Object.entries(headers).map(([name, value]) => [String(name), String(value)]),
        Object.entries(cookies).map(([name, value]) => [String(name), String(value)]),
        body,
        options.allowRedirects ?? this._session.allowRedirects,
        options.maxRedirects ?? this._session.maxRedirects,
      ))
      this._applyNavigation(result)
      return this
    } finally { signal?.removeEventListener('abort', cancel) }
  }
  _applyNavigation(result) {
    this.statusCode = result.statusCode
    this.reason = result.reason
    this.url = result.url
    this.headers = new Headers(result.headers)
    this.content = result.content
    this.html = result.html ?? null
    this.cookies = Object.fromEntries(result.cookies)
    this.elapsed = result.elapsed
    this.httpVersion = result.httpVersion ?? null
    this.downloadedBytes = result.downloadedBytes
    this.uploadedBytes = result.uploadedBytes
    this.headerBytes = result.headerBytes
    this.lastRequest = Object.freeze({ ...result.request, headers: new Headers(result.request.headers) })
    this.history = Object.freeze(result.history.map(entry => Object.freeze({
      ...entry, headers: new Headers(entry.headers), request: Object.freeze({ ...entry.request, headers: new Headers(entry.request.headers) }),
    })))
    this.redirectCount = this.history.length
    this._navigated = true
  }
  get ok() { this._ensureNavigated(); return this.statusCode < 400 }
  get encoding() {
    this._ensureNavigated()
    const match = /(?:^|;)\s*charset=([^;\s]+)/i.exec(this.headers.get('content-type') ?? '')
    return this._encoding ?? (match ? match[1].replace(/^['"]|['"]$/g, '') : this._session.defaultEncoding)
  }
  set encoding(value) { this._encoding = value == null ? undefined : String(value) }
  get text() {
    try { return new TextDecoder(this.encoding).decode(this.content) }
    catch { return new TextDecoder('utf-8').decode(this.content) }
  }
  json() { return JSON.parse(this.text) }
  raiseForStatus() { if (!this.ok) throw new HTTPError(`${this.statusCode} ${this.reason} for url: ${this.url}`, this) }
  async _refreshHtml() { if (this.html != null) this.html = await call(() => this._inner.html()) }
  async get(url, options) { return this.request('GET', url, options) }
  async head(url, options = {}) { return this.request('HEAD', url, { allowRedirects: false, ...options }) }
  async options(url, options) { return this.request('OPTIONS', url, options) }
  async delete(url, options) { return this.request('DELETE', url, options) }
  async post(url, options) { return this.request('POST', url, options) }
  async put(url, options) { return this.request('PUT', url, options) }
  async patch(url, options) { return this.request('PATCH', url, options) }
  async evaluate(expression) { const value = JSON.parse(await call(() => this._inner.evaluate(String(expression)))); await this._refreshHtml(); return value }
  async screenshot(options = {}) { const bytes = await call(() => this._inner.screenshot(Boolean(options.fullPage))); if (options.path != null) await fs.writeFile(options.path, bytes); return bytes }
  async extract(options = {}) { return JSON.parse(await call(() => this._inner.extract(JSON.stringify(options)))) }
  async click(selector) { await call(() => this._inner.click(String(selector))); await this._refreshHtml() }
  async hover(selector) { await call(() => this._inner.hover(String(selector))); await this._refreshHtml() }
  async type(selector, text) { await call(() => this._inner.typeText(String(selector), String(text))); await this._refreshHtml() }
  async tap(selector) { await call(() => this._inner.tap(String(selector))); await this._refreshHtml() }
  async close() { if (!this._closed) { if (this._ownsSession) await this._session.close(); else await call(() => this._inner.close()); this._closed = true } }
  _ensureOpen() { this._session._ensureOpen(); if (this._closed) throw new SessionClosed('page is closed', { code: 'closed' }) }
  _ensureNavigated() { this._ensureOpen(); if (!this._navigated) throw new InvalidRequest('page has not navigated', { code: 'invalid_input' }) }
}

class Session {
  constructor(inner, options = {}) {
    this._inner = inner
    this.headers = { ...(options.headers ?? {}) }
    this.params = options.params
    this.cookies = { ...(options.cookies ?? {}) }
    this.auth = options.auth
    this.proxy = options.proxy
    this.referer = options.referer
    this.timeoutMs = options.timeoutMs ?? 30_000
    this.allowRedirects = options.allowRedirects ?? true
    this.maxRedirects = options.maxRedirects ?? 30
    this.defaultEncoding = options.defaultEncoding ?? 'utf-8'
    this._closed = false
  }
  async newPage(options = {}) { this._ensureOpen(); return new Page(await call(() => this._inner.newPage(options.proxy ?? this.proxy)), this) }
  async request(method, url, options = {}) { const page = await this.newPage({ proxy: options.proxy }); try { return await page.request(method, url, options) } catch (error) { await page.close(); throw error } }
  async get(url, options) { return this.request('GET', url, options) }
  async head(url, options = {}) { return this.request('HEAD', url, { allowRedirects: false, ...options }) }
  async options(url, options) { return this.request('OPTIONS', url, options) }
  async delete(url, options) { return this.request('DELETE', url, options) }
  async post(url, options) { return this.request('POST', url, options) }
  async put(url, options) { return this.request('PUT', url, options) }
  async patch(url, options) { return this.request('PATCH', url, options) }
  async close() { if (!this._closed) { await call(() => this._inner.close()); this._closed = true } }
  _ensureOpen() { if (this._closed) throw new SessionClosed('session is closed', { code: 'closed' }) }
}

const nativeOptionNames = [
  'personaJson', 'caBundle', 'enableWorker', 'enableStreamingNetworking', 'enableCanvas',
  'enableWebGL', 'enableWebGPU', 'enableWebAudio', 'enableWebAudioOutput', 'storagePath',
  'storageQuotaBytes',
]
async function createSession(options = {}) {
  const nativeOptions = Object.fromEntries(nativeOptionNames.filter(key => key in options).map(key => [key, options[key]]))
  return new Session(await call(() => native.NativeSession.create(nativeOptions)), options)
}
async function request(method, url, options = {}) {
  const session = await createSession(options)
  try { const page = await session.request(method, url, options); page._ownsSession = true; return page }
  catch (error) { await session.close(); throw error }
}
const get = (url, options) => request('GET', url, options)
const head = (url, options = {}) => request('HEAD', url, { allowRedirects: false, ...options })
const options = (url, requestOptions) => request('OPTIONS', url, requestOptions)
const del = (url, requestOptions) => request('DELETE', url, requestOptions)
const post = (url, requestOptions) => request('POST', url, requestOptions)
const put = (url, requestOptions) => request('PUT', url, requestOptions)
const patch = (url, requestOptions) => request('PATCH', url, requestOptions)

module.exports = {
  ConnectionError, Headers, HTTPError, InvalidRequest, InvalidURL, JavaScriptError, Multipart,
  Page, RequestError, Session, SessionClosed, Timeout, TooManyRedirects, createSession, delete: del,
  get, head, options, patch, post, put, request,
}
