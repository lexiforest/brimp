use brimp_runtime::runtime::native::{Session, SessionOptions};
use serde_json::json;
#[test]
fn crypto_vectors_derivation_and_cipher_parameters() {
    let session = Session::new(SessionOptions::default()).unwrap();
    let page = session.new_page().unwrap();
    assert_eq!(page.evaluate(r#"(async()=>{
 const s=crypto.subtle,bytes=h=>Uint8Array.from(h.match(/../g)||[],x=>parseInt(x,16)),hex=b=>Array.from(new Uint8Array(b),x=>x.toString(16).padStart(2,'0')).join('');
 const pb=await s.importKey('raw',new TextEncoder().encode('password'),'PBKDF2',false,['deriveBits','deriveKey']);
 if(hex(await s.deriveBits({name:'PBKDF2',hash:'SHA-1',salt:new TextEncoder().encode('salt'),iterations:1},pb,160))!=='0c60c80f961f0e71f3a9b524af6012062fe037a6')throw Error('pbkdf2 vector');
 const hk=await s.importKey('raw',new Uint8Array(22).fill(11),'HKDF',false,['deriveBits','deriveKey']);
 const params={name:'HKDF',hash:'SHA-256',salt:bytes('000102030405060708090a0b0c'),info:bytes('f0f1f2f3f4f5f6f7f8f9')};
 if(hex(await s.deriveBits(params,hk,336))!=='3cb25f25faacd57a90434f64d0362f2a2d2d0a90cf1a5a4c5db02d56ecc4c5bf34007208d5b887185865')throw Error('hkdf vector');
 const derived=await s.deriveKey(params,hk,{name:'AES-GCM',length:256},false,['encrypt']);
 if(derived.extractable||derived.algorithm.length!==256)throw Error('derived policy');
 for(const name of ['AES-CBC','AES-CTR']){
 const key=await s.importKey('raw',bytes('2b7e151628aed2a6abf7158809cf4f3c'),name,true,['encrypt','decrypt']);
 const data=bytes('6bc1bee22e409f96e93d7e117393172a'),p=name==='AES-CBC'?{name,iv:bytes('000102030405060708090a0b0c0d0e0f')}:{name,counter:bytes('f0f1f2f3f4f5f6f7f8f9fafbfcfdfeff'),length:128};
 const encrypted=await s.encrypt(p,key,data);if(hex(encrypted).slice(0,32)!==(name==='AES-CBC'?'7649abac8119b246cee98e9b12e9197d':'874d6191b620e3261bef6864990db6ce'))throw Error(name+' vector');
 if(hex(await s.decrypt(p,key,encrypted))!==hex(data))throw Error('decrypt');
 }
 const kw=await s.importKey('raw',bytes('000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f'),'AES-KW',false,['wrapKey','unwrapKey']);
 const target=await s.importKey('raw',bytes('00112233445566778899aabbccddeeff'),'AES-GCM',true,['encrypt']);
 const wrapped=await s.wrapKey('raw',target,kw,'AES-KW');if(hex(wrapped)!=='64e8c3f9ce0f5ba263e9777905818a2a93c8191e7d6e8ae7')throw Error('kw vector');
 const unwrapped=await s.unwrapKey('raw',wrapped,kw,'AES-KW','AES-GCM',false,['encrypt']);if(unwrapped.extractable)throw Error('unwrap policy');
 for(const length of [128,192,256])for(const tagLength of [32,64,96,104,112,120,128]){
 const k=await s.generateKey({name:'AES-GCM',length},true,['encrypt','decrypt']),p={name:'AES-GCM',iv:new Uint8Array(16),tagLength,additionalData:new Uint8Array([7])};
 const c=new Uint8Array(await s.encrypt(p,k,new Uint8Array([42])));if(new Uint8Array(await s.decrypt(p,k,c))[0]!==42)throw Error('gcm');c[0]^=1;
 if(await s.decrypt(p,k,c).then(()=>false,e=>e.name)!=='OperationError')throw Error('gcm authentication');
 }
 const ctr=await s.generateKey({name:'AES-CTR',length:128},false,['encrypt','decrypt']);const counter=new Uint8Array(16);counter[15]=255;
 const p={name:'AES-CTR',counter,length:1};const c=await s.encrypt(p,ctr,new Uint8Array(32));if(new Uint8Array(await s.decrypt(p,ctr,c)).some(x=>x!==0))throw Error('ctr wrap');
 if(await s.encrypt(p,ctr,new Uint8Array(33)).then(()=>false,e=>e.name)!=='OperationError')throw Error('ctr repeated counter');
 return true;
 })()"#).unwrap(),json!(true));
}
#[test]
fn asymmetric_keys_formats_signatures_encryption_and_policy() {
    let session = Session::new(SessionOptions::default()).unwrap();
    let page = session.new_page().unwrap();
    assert_eq!(page.evaluate(r#"(async()=>{
 const s=crypto.subtle,data=new TextEncoder().encode('signed message');
 for(const name of ['RSASSA-PKCS1-v1_5','RSA-PSS','ECDSA']){
 const algorithm=name==='ECDSA'?{name,namedCurve:'P-256'}:{name,modulusLength:2048,publicExponent:new Uint8Array([1,0,1]),hash:'SHA-256'};
 const pair=await s.generateKey(algorithm,true,['sign','verify']);const p=name==='RSA-PSS'?{name,saltLength:32}:name==='ECDSA'?{name,hash:'SHA-256'}:name;
 const signature=new Uint8Array(await s.sign(p,pair.privateKey,data));if(name==='ECDSA'&&signature.length!==64)throw Error('ECDSA format');
 for(const format of ['jwk','spki']){const key=await s.importKey(format,await s.exportKey(format,pair.publicKey),algorithm,true,['verify']);if(!await s.verify(p,key,signature,data))throw Error(name+' public format');}
 for(const format of ['jwk','pkcs8']){const key=await s.importKey(format,await s.exportKey(format,pair.privateKey),algorithm,false,['sign']);if(!await s.verify(p,pair.publicKey,await s.sign(p,key,data),data))throw Error(name+' private format');if(await s.exportKey('pkcs8',key).then(()=>false,e=>e.name)!=='InvalidAccessError')throw Error('extractable');}
 signature[0]^=1;if(await s.verify(p,pair.publicKey,signature,data))throw Error('invalid signature');
 const metadata=pair.privateKey.algorithm;metadata.name='HMAC';pair.privateKey.usages.push('verify');if(pair.privateKey.algorithm.name!==name||pair.privateKey.usages.includes('verify'))throw Error('metadata mutation');
 }
 const rsa=await s.generateKey({name:'RSA-OAEP',modulusLength:2048,publicExponent:new Uint8Array([1,0,1]),hash:'SHA-256'},false,['encrypt','decrypt']);
 const c=await s.encrypt({name:'RSA-OAEP',label:new Uint8Array([1])},rsa.publicKey,data);
 if(new TextDecoder().decode(await s.decrypt({name:'RSA-OAEP',label:new Uint8Array([1])},rsa.privateKey,c))!=='signed message')throw Error('OAEP');
 if(await s.decrypt('RSA-OAEP',rsa.privateKey,c).then(()=>false,e=>e.name)!=='OperationError')throw Error('OAEP label');
 for(const params of ['RSA-OAEP',{name:'RSA-OAEP',label:new Uint8Array()}]){
 const encrypted=await s.encrypt(params,rsa.publicKey,data);
 if(new TextDecoder().decode(await s.decrypt(params,rsa.privateKey,encrypted))!=='signed message')throw Error('OAEP empty label');
 }
 for(const namedCurve of ['P-256','P-384','P-521']){
 const a=await s.generateKey({name:'ECDH',namedCurve},false,['deriveBits','deriveKey']),b=await s.generateKey({name:'ECDH',namedCurve},false,['deriveBits']);
 const x=new Uint8Array(await s.deriveBits({name:'ECDH',public:b.publicKey},a.privateKey,256)),y=new Uint8Array(await s.deriveBits({name:'ECDH',public:a.publicKey},b.privateKey,256));
 if(x.some((v,i)=>v!==y[i]))throw Error('ECDH');
 const key=await s.deriveKey({name:'ECDH',public:b.publicKey},a.privateKey,{name:'AES-GCM',length:128},false,['encrypt']);if(key.type!=='secret')throw Error('deriveKey');
 }
 if(await s.generateKey({name:'ECDSA',namedCurve:'P-256'},true,['decrypt']).then(()=>false,e=>e.name)!=='SyntaxError')throw Error('usages');
 return true;
 })()"#).unwrap(),json!(true));
}
