import {test} from 'node:test';
import assert from 'node:assert/strict';
import {createSession, Plugin} from '../index.js';
import {writeFile, mkdtemp, rm} from 'node:fs/promises';
import {join} from 'node:path';
import {tmpdir} from 'node:os';

test('core WebCrypto vectors, key policy, binary views and authentication', async () => {
  const session = await createSession();
  try {
    const page = await session.newPage();
    const result = await page.evaluate(`(async () => {
      const hex = value => Array.from(new Uint8Array(value), b => b.toString(16).padStart(2,'0')).join('');
      const key = await crypto.subtle.importKey('raw', new Uint8Array(20).fill(11), {name:'HMAC',hash:'SHA-256'}, false, ['sign','verify']);
      const data = new TextEncoder().encode('Hi There');
      const signature = await crypto.subtle.sign('HMAC', key, data);
      const tampered = new Uint8Array(signature.slice(0)); tampered[0] ^= 1;
      const aes = await crypto.subtle.importKey('raw', new Uint8Array(16), 'AES-GCM', true, ['encrypt','decrypt']);
      const params = {name:'AES-GCM',iv:new Uint8Array(12)};
      const encrypted = await crypto.subtle.encrypt(params,aes,new Uint8Array(16));
      const decrypted = await crypto.subtle.decrypt(params,aes,encrypted);
      const wrong = new Uint8Array(encrypted.slice(0)); wrong[0] ^= 1;
      return {
        digest: hex(await crypto.subtle.digest('SHA-256', new Uint8Array([0,97,98,99,0]).subarray(1,4))),
        signature: hex(signature), valid: await crypto.subtle.verify('HMAC',key,signature,data),
        invalid: await crypto.subtle.verify('HMAC',key,tampered,data), encrypted:hex(encrypted), decrypted:hex(decrypted),
        auth:await crypto.subtle.decrypt(params,aes,wrong).then(()=>false,e=>e.name),
        extract:await crypto.subtle.exportKey('raw',key).then(()=>false,e=>e.name),
        usage:await crypto.subtle.sign('HMAC',aes,data).then(()=>false,e=>e.name),
        unsupported:await crypto.subtle.digest('unknown',data).then(()=>false,e=>e.name),
        jwk:await crypto.subtle.exportKey('jwk',aes),
      };
    })()`);
    assert.deepEqual(result, {
      digest:'ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad',
      signature:'b0344c61d8db38535ca8afceaf0bf12b881dc200c9833da726e9376c2e32cff7',
      valid:true, invalid:false,
      encrypted:'0388dace60b6a392f328c2b971b2fe78ab6e47d42cec13bdf53a67b21257bddf',
      decrypted:'00000000000000000000000000000000', auth:'OperationError', extract:'InvalidAccessError', usage:'InvalidAccessError', unsupported:'NotSupportedError',
      jwk:{kty:'oct',k:'AAAAAAAAAAAAAAAAAAAAAA',alg:'A128GCM',key_ops:['encrypt','decrypt'],ext:true},
    });
  } finally { await session.close(); }
});

test('file plugins receive options and trusted session identity', async () => {
  const directory = await mkdtemp(join(tmpdir(), 'brimp-plugin-'));
  const path = join(directory, 'plugin.js');
  await writeFile(path, '({global, host, options}) => { global.config = options; global.context = () => host.call("context"); }');
  const plugin = Plugin.fromFile(path, {name:'file',options:{answer:42},handlers:{context: c => ({session:c.sessionId,page:c.pageId,realm:c.realmId,url:c.url,origin:c.origin})}});
  const first = await createSession({plugins:[plugin]}), second = await createSession({plugins:[plugin]});
  try {
    const a = await first.newPage(), b = await first.newPage(), c = await second.newPage();
    assert.deepEqual(await a.evaluate('config'), {answer:42});
    const [x,y,z] = await Promise.all([a,b,c].map(page=>page.evaluate('context()')));
    assert.equal(x.session,y.session); assert.notEqual(x.session,z.session); assert.notEqual(x.page,y.page);
    assert.equal(x.origin,null);
    assert.deepEqual(await a.evaluate('Object.defineProperty(globalThis,"location",{value:{href:"https://spoof.test"},configurable:true}); context()'),x);
  } finally { await first.close(); await second.close(); await rm(directory,{recursive:true,force:true}); }
});

test('file plugin paths and host exception stacks stay outside the page', async () => {
  const directory = await mkdtemp(join(tmpdir(), 'brimp-private-'));
  const path = join(directory, 'private-plugin.js');
  await writeFile(path, '({global,host}) => { global.fail = () => host.call("fail"); global.localFailure = () => {throw new Error("public detail")}; }');
  const plugin = Plugin.fromFile(path, {name:'private',handlers:{fail() {throw new Error(`private host detail: ${path}`);}}});
  const session = await createSession({plugins:[plugin]});
  try {
    const page = await session.newPage();
    const observed = new Promise(resolve => page.on('error', resolve));
    const result = await page.evaluate('fail().catch(e => ({name:e.name,message:e.message,stack:e.stack}))');
    assert.equal(result.name, 'OperationError');
    assert.equal(result.message, 'The operation failed.');
    assert.ok(!JSON.stringify(result).includes(path));
    assert.match((await observed).message, /private host detail/);
    const stack = await page.evaluate('try {localFailure()} catch(e) {e.stack}');
    assert.ok(!stack.includes(path) && !stack.includes('file:///'));
    assert.ok(!(await page.evaluate('localFailure.toString()')).includes('public detail'));
  } finally {await session.close();}
  await writeFile(path, '() => {throw new Error("installation detail")}');
  const broken = await createSession({plugins:[Plugin.fromFile(path,{name:'broken'})]});
  try {
    await assert.rejects(broken.newPage(), error => error.message.includes('installation detail') && error.message.includes('private-plugin.js'));
  } finally {await broken.close(); await rm(directory,{recursive:true,force:true});}
});

test('session plugin state survives navigation, isolates origins/sessions, and disposes once', async () => {
  const {createServer} = await import('node:http');
  const server = createServer((_request, response) => response.end('<!doctype html><p>fixture</p>'));
  await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
  const url = `http://127.0.0.1:${server.address().port}/`;
  let started = 0, disposed = 0;
  const plugin = Plugin.fromFile(new URL('../../../examples/plugins/preferences.js', import.meta.url), {
    name:'preferences',
    setup() {
      started++;
      const state = new Map();
      const partition = c => { if (!c.origin) throw new Error('opaque'); if (!state.has(c.origin)) state.set(c.origin,new Map()); return state.get(c.origin); };
      return {handlers:{get:(c,k)=>partition(c).get(k)??null,set:(c,k,v)=>{partition(c).set(k,v);return null;}},dispose:()=>{disposed++;state.clear();}};
    },
  });
  const a = await createSession({plugins:[plugin]}), b = await createSession({plugins:[plugin]});
  try {
    const p = await a.get(url); await p.evaluate('preferences.set("theme","dark")');
    await p.get(url+'next'); assert.equal(await p.evaluate('preferences.get("theme")'),'dark');
    const other = await a.get(url); assert.equal(await other.evaluate('preferences.get("theme")'),'dark');
    const isolated = await b.get(url); assert.equal(await isolated.evaluate('preferences.get("theme")'),null);
    assert.equal(started,2);
  } finally { await a.close(); await a.close(); await b.close(); await new Promise(resolve=>server.close(resolve)); }
  assert.equal(disposed,2);
});

test('session plugin setup is rolled back if later setup fails', async () => {
  let disposed = 0;
  await assert.rejects(createSession({plugins:[
    new Plugin('one','() => {}',{}, {},()=>({dispose(){disposed++;}})),
    new Plugin('two','() => {}',{}, {},()=>{throw new Error('setup failed');}),
  ]}),/setup failed/);
  assert.equal(disposed,1);
});

test('generated nonextractable keys remain native and usage restrictions survive prototype tampering', async () => {
  const session = await createSession();
  try {
    const page = await session.newPage();
    assert.deepEqual(await page.evaluate(`(async () => {
      const key = await crypto.subtle.generateKey({name:'HMAC',hash:'SHA-256'},false,['verify']);
      let intercepted = false;
      const get = WeakMap.prototype.get, includes = Array.prototype.includes;
      WeakMap.prototype.get = function(...args) {intercepted=true; return get.apply(this,args);};
      Array.prototype.includes = () => true;
      try {
        const denied = await crypto.subtle.sign('HMAC',key,new Uint8Array([1])).then(()=>false,()=>true);
        const exported = await crypto.subtle.exportKey('raw',key).then(()=>false,e=>e.name);
        return {denied,exported,intercepted};
      } finally {WeakMap.prototype.get=get;Array.prototype.includes=includes;}
    })()`), {denied:true,exported:'InvalidAccessError',intercepted:false});
  } finally {await session.close();}
});
