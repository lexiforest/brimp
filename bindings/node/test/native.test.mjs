import { test, before, after } from 'node:test';
import assert from 'node:assert/strict';
import { createServer } from 'node:http';
import { once } from 'node:events';
import { createSession, StaleHandle, JavaScriptError, Closed } from '../index.js';
const html = '<!doctype html><main><b>old</b></main><script>globalThis.beforeLater = !document.querySelector("#later"); document.querySelector("b").textContent = "script";</script><p id="later">later</p>';
let server, url;
before(async () => {
  server = createServer(async (request, response) => {
    if (request.url === '/large') { response.end(Buffer.from(Array.from({length: 2*1024*1024}, (_, i) => i % 251))); return; }
    if (request.url === '/redirect') { response.writeHead(302, { location: '/' }); response.end(); return; }
    if (request.method === 'POST') { const chunks = []; for await (const chunk of request) chunks.push(chunk); response.setHeader('content-type', 'application/octet-stream'); response.end(Buffer.concat(chunks)); return; }
    if (request.url === '/headers') { response.setHeader('content-type', 'application/json'); response.end(JSON.stringify(request.headers)); return; }
    if (request.url === '/slow') { request.on('close', () => response.destroy()); return; }
    response.setHeader('content-type', 'text/html'); response.end(html);
  }).listen(0, '127.0.0.1');
  await once(server, 'listening'); url = `http://127.0.0.1:${server.address().port}`;
});
after(async () => { server.closeAllConnections(); await new Promise(resolve => server.close(resolve)); });

test('request, script execution, DOM edits, detached and stale handles', async () => {
  const session = await createSession();
  try {
    const page = await session.get(url + '/redirect');
    assert.equal(page.statusCode, 200); assert.equal(page.text, html); assert.equal(page.response.history.length, 1);
    assert.equal(await page.evaluate('beforeLater'), true);
    const root = await page.querySelector('main'), node = await root.querySelector('b');
    assert.equal(await node.textContent(), 'script');
    await node.remove(); await node.setTextContent('detached');
    assert.equal(await node.textContent(), 'detached'); assert.equal(await page.querySelector('b'), null);
    await page.get(url); await assert.rejects(node.textContent(), StaleHandle);
    const handle = await page.evaluateHandle('({a: 1})');
    assert.deepEqual(await handle.propertyNames(), ['a']);
    const result = await handle.call('(value, n) => value.a + n', 41);
    assert.equal(await result.jsonValue(), 42); await result.dispose(); await handle.dispose();
    await assert.rejects(handle.jsonValue(), StaleHandle);
    await page.close(); await assert.rejects(page.evaluate('42'), Closed);
  } finally { await session.close(); }
});
test('Promises keep the event loop running, binary bodies, errors, and abort', async () => {
  const session = await createSession();
  try {
    const data = Buffer.from([0, 255, 42]); const page = await session.post(url, { body: data });
    assert.deepEqual(page.content, data);
    let ticked = false; const timer = setTimeout(() => { ticked = true; }, 2);
    assert.equal(await page.evaluate('new Promise(resolve => setTimeout(() => resolve(42), 20))'), 42);
    assert.equal(ticked, true); clearTimeout(timer);
    await assert.rejects(page.evaluate('Promise.reject(new Error("boom"))'), JavaScriptError);
    assert.equal(await page.evaluate('42'), 42);
    const abort = new AbortController(); setTimeout(() => abort.abort(), 20);
    await assert.rejects(page.get(url + '/slow', { signal: abort.signal }), /cancelled/);
  } finally { await session.close(); }
});
test('concurrent pages, isolated sessions, and reuse', async () => {
  const first = await createSession(), second = await createSession();
  try {
    first.setCookie(url, 'secret', 'one');
    const [a, b, c] = await Promise.all([first.get(url), first.get(url), second.get(url)]);
    assert.equal(await a.evaluate('document.cookie'), 'secret=one'); assert.equal(await b.evaluate('document.cookie'), 'secret=one'); assert.equal(await c.evaluate('document.cookie'), '');
    await a.evaluate('globalThis.onlyA = 1'); assert.equal(await b.evaluate('typeof onlyA'), 'undefined');
    await a.get(url); assert.equal(await a.evaluate('typeof onlyA'), 'undefined');
    await first.close(); assert.equal(await c.evaluate('42'), 42);
  } finally { await first.close(); await second.close(); }
});

test('close interrupts pending evaluation', async () => {
  const session = await createSession({ timeout: 2000 });
  try {
    const page = await session.newPage();
    const pending = assert.rejects(page.evaluate('new Promise(() => {})'), Closed);
    await new Promise(resolve => setTimeout(resolve, 20));
    await page.close(); await pending;
  } finally { await session.close(); }
});


test('request defaults, overrides, and basic auth', async () => {
  const session = await createSession({ headers: { 'X-Default': 'default', 'X-Keep': 'keep' }, auth: ['user', 'pass'] });
  try {
    const page = await session.get(url + '/headers', { headers: { 'X-Default': 'override' } });
    assert.equal(page.json()['x-default'], 'override');
    assert.equal(page.json()['x-keep'], 'keep');
    assert.equal(page.json().authorization, 'Basic dXNlcjpwYXNz');
  } finally { await session.close(); }
});

test('network routes, ordered events, callback reentry, response overrides and controls', async () => {
  const {Reentry} = await import('../index.js');
  const session = await createSession();
  try {
    const page = await session.newPage(); const events = [];
    let loaded; let load = new Promise(resolve => { loaded = resolve; });
    for (const name of ['request','response','requestFinished','console','mutation']) page.on(name, event => {events.push(event);});
    page.on('lifecycle', event => {if(event.name === 'load') loaded();});
    let off = page.route('*', async route => {
      await new Promise(resolve => setTimeout(resolve, 5));
      await assert.rejects(page.evaluate('1'), Reentry);
      route.continueRequest({headers: {'x-intercepted':'yes'}});
    });
    await page.get(url + '/headers'); await load;
    assert.equal(page.json()['x-intercepted'], 'yes');
    assert.deepEqual(events.slice(0,3).map(event => event.event), ['request','response','requestFinished']);
    assert.deepEqual(page.responseBody(events[0].request.id), page.content);
    off();
    off = page.route('*', async route => {
      assert.ok(route.body().length > 0);
      route.continueRequest({status:201,body:Buffer.from('changed'),headers:{'content-type':'text/plain'}});
    }, {stage:'response'});
    await page.get(url); assert.equal(page.statusCode, 201); assert.equal(page.text, 'changed'); off();
    off = page.route('*', () => {});
    await assert.rejects(page.get(url), /without completing/); off();
    await page.configureNetwork({offline:true});
    await assert.rejects(page.get(url), /offline/);
    assert.equal(await page.evaluate('navigator.onLine'), false);
  } finally {await session.close();}
});


test('stream backpressure, identity, redirects, and AbortSignal', async () => {
  const session = await createSession({identity: {userAgent: 'Mozilla/5.0 (Test) Test/1 Safari/605.1.15', languages: ['fr', 'en']}});
  try {
    const page = await session.newPage();
    assert.deepEqual(await page.evaluate('[navigator.userAgent,navigator.language]'), ['Mozilla/5.0 (Test) Test/1 Safari/605.1.15','fr']);
    await page.setIdentity({userAgent:'Mozilla/5.0 (Test) Test/2 Safari/605.1.15'});
    const headers = await page.stream('GET', url + '/headers');
    let chunks = []; for await (const chunk of headers.body) chunks.push(chunk);
    assert.equal(JSON.parse(Buffer.concat(chunks))['user-agent'], 'Mozilla/5.0 (Test) Test/2 Safari/605.1.15');
    const response = await page.stream('GET', url + '/large');
    await new Promise(resolve => setTimeout(resolve, 50));
    assert.equal(await page.evaluate('6*7'),42);
    chunks = []; for await (const chunk of response.body) chunks.push(chunk);
    assert.deepEqual(Buffer.concat(chunks),Buffer.from(Array.from({length:2*1024*1024},(_, i)=>i%251)));
    const redirected = await page.stream('GET', url + '/redirect');
    chunks=[]; for await (const chunk of redirected.body) chunks.push(chunk);
    assert.equal(Buffer.concat(chunks).toString(),html);
    const controller = new AbortController();
    const cancelled = await page.stream('GET', url + '/large', {signal:controller.signal});
    controller.abort();
    await assert.rejects(async()=>{for await (const chunk of cancelled.body) {}}, /cancelled/);
  } finally {await session.close();}
});
