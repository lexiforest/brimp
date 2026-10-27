import test from 'node:test';
import assert from 'node:assert/strict';
import {createSession} from '../../../bindings/node/index.js';

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
