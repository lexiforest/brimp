import { Readable } from 'node:stream';
import { callbackContext, configurePlugins } from './plugins.js';
export { Plugin, HostContext, pluginCatalog, hostStoragePlugin, storagePlugin, canvasPlugin, servicesPlugin, extractionPlugin, geometryPlugin, cssTypedOmPlugin, schedulingPlugin, graphicsPlugin, audioPlugin } from './plugins.js';
import { createRequire } from 'node:module';
const native = createRequire(import.meta.url)('./brimp.node');
const eventNames = {requestFinished:'request_finished',requestFailed:'request_failed',webSocket:'websocket',eventSource:'eventsource'};
const publicEventNames = Object.fromEntries(Object.entries(eventNames).map(([publicName, nativeName]) => [nativeName, publicName]));
function eventRecord(wire) {
  const raw = JSON.parse(wire);
  const value = Object.fromEntries(Object.entries(raw).map(([key, value]) => [key.replace(/_([a-z])/g, (_, char) => char.toUpperCase()), value]));
  value.event = publicEventNames[value.event] ?? value.event;
  if (value.event === 'request' && value.request.body != null) value.request.body = Buffer.from(value.request.body);
  if (value.event === 'webSocket') value.data = Buffer.from(value.data);
  return value;
}
const requestRecord = record => ({ ...record, body: record.body == null ? null : Buffer.from(record.body) });
const pairs = headers => Array.isArray(headers) ? headers : Object.entries(headers ?? {});

export class RuntimeError extends Error {
  constructor(message, code = 'internal') { super(message); this.name = this.constructor.name; this.code = code; }
}
export class JavaScriptError extends RuntimeError {}
export class StaleHandle extends RuntimeError {}
export class Closed extends RuntimeError {}
export class Timeout extends RuntimeError {}
export class Reentry extends RuntimeError {}
async function invoke(operation, pageId) {
  if (pageId !== undefined && callbackContext.getStore()?.pageId === pageId) throw new Reentry('callback cannot reenter the same page', 'reentry');
  try { return await operation(); }
  catch (error) {
    const code = error.message.split(':', 1)[0];
    const Type = { javascript: JavaScriptError, stale_handle: StaleHandle, closed: Closed, timeout: Timeout, reentry: Reentry }[code] ?? RuntimeError;
    throw new Type(error.message, code);
  }
}
function authHeaders(headers, auth) {
  let entries = pairs(headers);
  if (auth) entries = [...entries.filter(([k]) => k.toLowerCase() !== 'authorization'), ['Authorization', 'Basic ' + Buffer.from(`${auth[0]}:${auth[1]}`).toString('base64')]];
  return entries;
}
class Requests {
  get(url, options) { return this.request('GET', url, options); }
  post(url, options) { return this.request('POST', url, options); }
  put(url, options) { return this.request('PUT', url, options); }
  patch(url, options) { return this.request('PATCH', url, options); }
  delete(url, options) { return this.request('DELETE', url, options); }
  head(url, options) { return this.request('HEAD', url, options); }
  options(url, options) { return this.request('OPTIONS', url, options); }
}
export async function createSession({ headers, timeout = 30000, proxy, auth, plugins = [], identity = {}, clock = {}, diagnostics = null } = {}) {
  const token = {};
  const { descriptors, dispatch, dispose } = await configurePlugins(plugins, token);
  try {
    return new Session(await invoke(() => native.createNativeSession(authHeaders(headers, auth), timeout, proxy, descriptors, dispatch, identity, clock, diagnostics)), token, dispose);
  } catch (error) { await dispose(); throw error; }
}
export class Session extends Requests {
  constructor(inner, token, dispose) { super(); this._inner = inner; this._pluginToken = token; this._disposePlugins = dispose; }
  _checkCallback() { if (callbackContext.getStore()?.session === this._pluginToken) throw new Reentry('cannot create pages or close the session from its host callback', 'reentry'); }
  async newPage() { this._checkCallback(); return new Page(await invoke(() => this._inner.newPage()), this); }
  async request(method, url, options) {
    const page = await this.newPage();
    try { return await page.request(method, url, options); }
    catch (error) { await page.close(); throw error; }
  }
  setCookie(url, name, value) { this._inner.setCookie(url, name, value); }
  get cookies() { return this._inner.cookies().map(({ host_only, http_only, same_site, ...cookie }) => ({ ...cookie, hostOnly: host_only, httpOnly: http_only, sameSite: same_site })); }
  clearCookies() { this._inner.clearCookies(); }
  async close() { this._checkCallback(); try { await invoke(() => this._inner.close()); } finally { await this._disposePlugins(); } }
}
export class Page extends Requests {
  advanceTime(milliseconds, {maxTasks = 10000} = {}) { return invoke(() => this._inner.advanceTime(milliseconds, maxTasks), this._pageId); }
  runPendingTasks({maxTasks = 10000} = {}) { return invoke(() => this._inner.runPendingTasks(maxTasks), this._pageId); }
  takeDiagnostics() { return invoke(() => this._inner.takeDiagnostics(), this._pageId); }

  constructor(inner, session) { super(); this._inner = inner; this._session = session; this._pageId = inner.id; this.response = null; this._subscriptions = new Set(); this._routes = new Set(); }
  _checkCallback() { if (callbackContext.getStore()?.pageId === this._pageId) throw new Reentry('callback cannot reenter the same page', 'reentry'); }
  on(event, handler) {
    this._checkCallback();
    const subscription = this._inner.on(eventNames[event] ?? event, wire => {
      callbackContext.run({pageId: this._pageId, session: this._session._pluginToken}, async () => {
        try { await handler(eventRecord(wire)); } catch (error) { process.emitWarning(String(error)); }
      });
    });
    this._subscriptions.add(subscription);
    return () => { subscription.unsubscribe(); this._subscriptions.delete(subscription); };
  }
  route(pattern, handler, {stage = 'request'} = {}) {
    this._checkCallback();
    const registration = this._inner.route(pattern, stage, inner => {
      if (!inner.pending) return;
      callbackContext.run({pageId: this._pageId, session: this._session._pluginToken}, async () => {
        try { await handler(new Route(inner)); }
        catch (error) { if (inner.pending) inner.fail(String(error)); }
        finally { if (inner.pending) inner.fail('route handler returned without completing the route'); }
      });
    });
    this._routes.add(registration);
    return () => { registration.remove(); this._routes.delete(registration); };
  }
  setIdentity(identity) { return invoke(() => this._inner.setIdentity(identity), this._pageId); }
  configureNetwork({offline = false, latency = 0, bypassCache = false, maxRetainedBytes = 32*1024*1024, maxRetainedBodyBytes = 8*1024*1024, bufferedDownloadBytesPerSecond} = {}) {
    return invoke(() => this._inner.configureNetwork(offline, latency, bypassCache, maxRetainedBytes, maxRetainedBodyBytes, bufferedDownloadBytesPerSecond), this._pageId);
  }
  setExtraHeaders(headers) { this._checkCallback(); this._inner.setExtraHeaders(pairs(headers)); }
  clearCache() { this._checkCallback(); this._inner.clearCache(); }
  responseBody(requestId) { this._checkCallback(); return this._inner.responseBody(requestId); }
  async stream(method, url, {headers, body, timeout, allowRedirects = true, maxRedirects = 20, auth, signal} = {}) {
    const token = new native.NativeCancellation();
    let inner;
    const abort = () => { token.cancel(); inner?.cancel(); };
    signal?.addEventListener('abort', abort, {once: true});
    if (signal?.aborted) abort();
    try {
      inner = await invoke(() => this._inner.stream(method, url, authHeaders(headers, auth), body == null ? undefined : Buffer.from(body), timeout, allowRedirects, maxRedirects, token), this._pageId);
      const page = this;
      const readable = Readable.from((async function* () {
        try {
          for (;;) {
            const chunk = await invoke(() => inner.nextChunk(), page._pageId);
            if (chunk == null) break;
            yield chunk;
          }
        } finally { inner.cancel(); signal?.removeEventListener('abort', abort); }
      })(), {objectMode: false, highWaterMark: 64 * 1024});
      readable.once('close', () => {inner.cancel(); signal?.removeEventListener('abort', abort);});
      return {...inner.metadata, body: readable, cancel() {inner.cancel(); readable.destroy();}};
    } catch (error) { signal?.removeEventListener('abort', abort); throw error; }
  }
  async request(method, url, { headers, body, timeout, allowRedirects = true, maxRedirects = 20, auth, signal } = {}) {
    const token = new native.NativeCancellation();
    const abort = () => token.cancel();
    signal?.addEventListener('abort', abort, { once: true });
    if (signal?.aborted) token.cancel();
    try {
      const result = await invoke(() => this._inner.request(method, url, authHeaders(headers, auth), body == null ? undefined : Buffer.from(body), timeout, allowRedirects, maxRedirects, token), this._pageId);
      this.response = new Response(result);
      return this;
    } finally { signal?.removeEventListener('abort', abort); }
  }
  get statusCode() { return this.response.statusCode; }
  get content() { return this.response.content; }
  get text() { return this.response.text; }
  get url() { return this.response.url; }
  get headers() { return this.response.headers; }
  json() { return this.response.json(); }
  evaluate(source) { return invoke(() => this._inner.evaluate(source), this._pageId); }
  async evaluateHandle(source) { return new JSHandle(await invoke(() => this._inner.evaluateHandle(source), this._pageId), this._pageId); }
  async querySelector(selector) { const inner = await invoke(() => this._inner.querySelector(selector), this._pageId); return inner == null ? null : new ElementHandle(inner, this._pageId); }
  async querySelectorAll(selector) { return (await invoke(() => this._inner.querySelectorAll(selector), this._pageId)).map(inner => new ElementHandle(inner, this._pageId)); }
  addInitScript(source) { return invoke(() => this._inner.addInitScript(source), this._pageId); }
  click(selector) { return invoke(() => this._inner.click(selector), this._pageId); }
  hover(selector) { return invoke(() => this._inner.hover(selector), this._pageId); }
  tap(selector) { return invoke(() => this._inner.tap(selector), this._pageId); }
  focus(selector) { return invoke(() => this._inner.focus(selector), this._pageId); }
  typeText(selector, text) { return invoke(() => this._inner.typeText(selector, text), this._pageId); }
  fill(selector, text) { return invoke(() => this._inner.fill(selector, text), this._pageId); }
  press(selector, key, {modifiers = 0} = {}) { return invoke(() => this._inner.press(selector, key, modifiers), this._pageId); }
  dragTo(source, destination) { return invoke(() => this._inner.dragTo(source, destination), this._pageId); }
  setInputFiles(selector, files) { return invoke(() => this._inner.setInputFiles(selector, fileRecords(files)), this._pageId); }
  extract(options = {}) { return invoke(() => this._inner.extract(options), this._pageId); }
  html() { return invoke(() => this._inner.html(), this._pageId); }
  close() { return invoke(() => this._inner.close(), this._pageId); }
}
export class Response {
  constructor({ metadata, content }) {
    this.statusCode = metadata.status_code;
    this.reason = metadata.reason;
    this.url = metadata.url;
    this.headers = metadata.headers;
    this.history = metadata.history.map(({ status_code, request, ...hop }) => ({ ...hop, statusCode: status_code, request: requestRecord(request) }));
    this.request = requestRecord(metadata.request);
    this.cookies = metadata.cookies;
    this.elapsed = metadata.elapsed.secs * 1000 + metadata.elapsed.nanos / 1e6;
    this.httpVersion = metadata.http_version;
    this.downloadedBytes = metadata.downloaded_bytes;
    this.uploadedBytes = metadata.uploaded_bytes;
    this.headerBytes = metadata.header_bytes;
    this.content = content;
  }
  get text() { return this.content.toString('utf8'); }
  json() { return JSON.parse(this.text); }
}
export class JSHandle {
  constructor(inner, pageId) { this._inner = inner; this._pageId = pageId; }
  jsonValue() { return invoke(() => this._inner.jsonValue(), this._pageId); }
  async call(source, ...args) { return new JSHandle(await invoke(() => this._inner.call(source, args), this._pageId), this._pageId); }
  async property(name) { return new JSHandle(await invoke(() => this._inner.property(name), this._pageId), this._pageId); }
  propertyNames() { return invoke(() => this._inner.propertyNames(), this._pageId); }
  dispose() { return invoke(() => this._inner.dispose(), this._pageId); }
}
const fileRecords = files => files.map(({name, content, mimeType = '', lastModified = 0}) => ({name, bytes: [...content], mimeType, lastModified}));
export class ElementHandle extends JSHandle {
  focus() { return invoke(() => this._inner.focus(), this._pageId); }
  click() { return invoke(() => this._inner.click(), this._pageId); }
  setInputFiles(files) { return invoke(() => this._inner.setInputFiles(fileRecords(files)), this._pageId); }
  async _value(source, ...args) { const handle = await this.call(source, ...args); try { return await handle.jsonValue(); } finally { await handle.dispose(); } }
  getAttribute(name) { return this._value('(node, name) => node.getAttribute(name)', name); }
  setAttribute(name, value) { return this._value('(node, name, value) => { node.setAttribute(name, value); return null; }', name, value); }
  textContent() { return this._value('node => node.textContent'); }
  html() { return this._value('node => node.outerHTML'); }
  setTextContent(text) { return this._value('(node, text) => { node.textContent = text; return null; }', text); }
  remove() { return this._value('node => { node.remove(); return null; }'); }
  async querySelector(selector) { const inner = await invoke(() => this._inner.querySelector(selector), this._pageId); return inner == null ? null : new ElementHandle(inner, this._pageId); }
  async querySelectorAll(selector) { return (await invoke(() => this._inner.querySelectorAll(selector), this._pageId)).map(inner => new ElementHandle(inner, this._pageId)); }
}

export class Route {
  constructor(inner) { this._inner = inner; Object.assign(this, inner.info); this.request = {...this.request, body: this.request.body == null ? null : Buffer.from(this.request.body)}; }
  get pending() { return this._inner.pending; }
  body() { return this._inner.body(); }
  continueRequest({headers, body, ...overrides} = {}) {
    this._inner.continueRequest({...overrides, ...(headers === undefined ? {} : {headers: pairs(headers)}), ...(body === undefined ? {} : {body: [...Buffer.from(body)]})});
  }
  fulfill({status = 200, headers, body = Buffer.alloc(0)} = {}) { this._inner.fulfill(status, pairs(headers), Buffer.from(body)); }
  fail(message = 'request aborted by route') { this._inner.fail(message); }
  authenticate(username, password) { this._inner.authenticate(username, password); }
}
