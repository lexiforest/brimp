const __cryptoKeys = new WeakMap();
const __cryptoGet = Function.prototype.call.bind(WeakMap.prototype.get, __cryptoKeys);
const __cryptoSet = Function.prototype.call.bind(WeakMap.prototype.set, __cryptoKeys);
const __cryptoStringify = JSON.stringify, __cryptoParse = JSON.parse;
const __cryptoKeyToken = {};
function __cryptoError(message, name = 'OperationError') { return new DOMException(message, name); }
function __cryptoBytes(value) {
    if (value instanceof ArrayBuffer) return Array.from(new Uint8Array(value));
    if (ArrayBuffer.isView(value)) return Array.from(new Uint8Array(value.buffer, value.byteOffset, value.byteLength));
    throw new TypeError('Expected a BufferSource');
}
function __cryptoName(value) {
    const name = typeof value === 'string' ? value : value?.name;
    if (name === undefined) throw new TypeError('Algorithm name is required');
    const upper=String(name).toUpperCase();
    return upper === 'RSASSA-PKCS1-V1_5' ? 'RSASSA-PKCS1-v1_5' : upper;
}
function __cryptoHash(value) {
    const name=__cryptoName(value);
    if(!['SHA-1','SHA-256','SHA-384','SHA-512'].includes(name))throw __cryptoError('Unsupported hash','NotSupportedError');
    return name;
}
function __cryptoInteger(value, name, maximum = 0xffffffff) {
    const number=Number(value);
    if(!Number.isInteger(number)||number<0||number>maximum)throw new TypeError(`Invalid ${name}`);
    return number;
}
function __cryptoAlgorithm(value) {
    const algorithm=__cryptoName(value), result={algorithm};
    if(['HMAC','RSA-PSS','RSA-OAEP','RSASSA-PKCS1-v1_5'].includes(algorithm))result.hash=__cryptoHash(value.hash);
    else if(['ECDSA','ECDH'].includes(algorithm)) {
        result.curve=String(value.namedCurve);
        if(!['P-256','P-384','P-521'].includes(result.curve))throw __cryptoError('Unsupported named curve','NotSupportedError');
    }else if(!['AES-GCM','AES-CBC','AES-CTR','AES-KW','HKDF','PBKDF2'].includes(algorithm))throw __cryptoError('Unsupported key algorithm','NotSupportedError');
    if(typeof value === 'object' && value.length!==undefined)result.length=__cryptoInteger(value.length,'length');
    return result;
}
function __cryptoLength(algorithm) {
    if(algorithm.algorithm==='HMAC')return algorithm.length??(['SHA-384','SHA-512'].includes(algorithm.hash)?1024:512);
    if(algorithm.algorithm.startsWith('AES-')) {
        if(![128,192,256].includes(algorithm.length))throw __cryptoError('Invalid AES key length');
        return algorithm.length;
    }
    throw __cryptoError('Unsupported derived key algorithm','NotSupportedError');
}
function __cryptoInvoke(request) {
    let result;
    try {result=__cryptoParse(__callHost('cryptoOperation',globalThis,__cryptoStringify(request)));}
    catch {throw __cryptoError('Cryptographic operation failed');}
    if(result.error)throw __cryptoError(result.error.message,result.error.name);
    return result.value;
}
function __cryptoDescriptor(value) {
    if(value.publicKey)return {publicKey:__cryptoDescriptor(value.publicKey),privateKey:__cryptoDescriptor(value.privateKey)};
    return new CryptoKey(__cryptoKeyToken,value);
}
class CryptoKey {
    constructor(token,record) {
        if(token!==__cryptoKeyToken)throw new TypeError('Illegal constructor');
        __cryptoSet(this,record);
    }
    get type(){return __cryptoRecord(this).type;}
    get extractable(){return __cryptoRecord(this).extractable;}
    get usages(){return __cryptoRecord(this).usages.slice();}
    get algorithm(){
        const a=__cryptoRecord(this).algorithm;
        return {...a,...(a.hash?{hash:{...a.hash}}:{}),...(a.publicExponent?{publicExponent:new Uint8Array(a.publicExponent)}:{})};
    }
    get [Symbol.toStringTag](){return 'CryptoKey';}
}
function __cryptoRecord(key) {
    const record=__cryptoGet(key);if(!record)throw new TypeError('Expected a CryptoKey');return record;
}
function __cryptoOperation(algorithm,key,operation,usage=operation) {
    const record=__cryptoRecord(key), name=__cryptoName(algorithm);
    if(record.algorithm.name!==name||!record.usages.includes(usage))throw __cryptoError('Key does not permit this operation','InvalidAccessError');
    return {algorithm:name,handle:record.handle,operation,usage};
}
function __cryptoCipherRequest(algorithm,key,operation,usage=operation) {
    const r=__cryptoOperation(algorithm,key,operation,usage);
    if(r.algorithm==='AES-GCM') {
        r.iv=__cryptoBytes(algorithm.iv);r.aad=algorithm.additionalData===undefined?[]:__cryptoBytes(algorithm.additionalData);
        r.tagLength=algorithm.tagLength===undefined?128:__cryptoInteger(algorithm.tagLength,'tagLength');
    }else if(r.algorithm==='AES-CBC')r.iv=__cryptoBytes(algorithm.iv);
    else if(r.algorithm==='AES-CTR') {r.iv=__cryptoBytes(algorithm.counter);r.length=__cryptoInteger(algorithm.length,'counter length');}
    else if(r.algorithm==='RSA-OAEP')r.label=algorithm.label===undefined?[]:__cryptoBytes(algorithm.label);
    else if(r.algorithm!=='AES-KW')throw __cryptoError('Unsupported encryption algorithm','NotSupportedError');
    return r;
}
function __cryptoDeriveRequest(algorithm,key,length,operation) {
    const r=__cryptoOperation(algorithm,key,operation);
    if(r.algorithm==='ECDH') {
        r.peer=__cryptoRecord(algorithm.public).handle;
        if(length===null)length={'P-256':256,'P-384':384,'P-521':528}[__cryptoRecord(key).algorithm.namedCurve];
    }else if(r.algorithm==='HKDF'||r.algorithm==='PBKDF2') {
        r.hash=__cryptoHash(algorithm.hash);r.salt=__cryptoBytes(algorithm.salt);
        if(r.algorithm==='HKDF')r.info=__cryptoBytes(algorithm.info);
        else r.iterations=__cryptoInteger(algorithm.iterations,'iterations');
        if(length===null)throw __cryptoError('Derivation length is required');
    }else throw __cryptoError('Unsupported derivation algorithm','NotSupportedError');
    r.length=__cryptoInteger(length,'length');return r;
}
class SubtleCrypto {
    constructor(){throw new TypeError('Illegal constructor');}
    async digest(algorithm,data){return Uint8Array.from(__cryptoInvoke({operation:'digest',algorithm:__cryptoHash(algorithm),data:__cryptoBytes(data)})).buffer;}
    async generateKey(algorithm,extractable,keyUsages){
        const r={...__cryptoAlgorithm(algorithm),operation:'generate',extractable:Boolean(extractable),usages:Array.from(keyUsages,String)};
        if(r.algorithm.startsWith('RSA')) {
            r.modulusLength=__cryptoInteger(algorithm.modulusLength,'modulusLength');
            if(!(algorithm.publicExponent instanceof Uint8Array))throw new TypeError('Expected Uint8Array publicExponent');
            r.exponent=__cryptoBytes(algorithm.publicExponent);
        }else if(r.algorithm.startsWith('AES-')||r.algorithm==='HMAC')r.length=__cryptoLength(r);
        return __cryptoDescriptor(__cryptoInvoke(r));
    }
    async importKey(format,keyData,algorithm,extractable,keyUsages){
        const r={...__cryptoAlgorithm(algorithm),operation:'import',format:String(format),extractable:Boolean(extractable),usages:Array.from(keyUsages,String)};
        if(r.format==='jwk')r.jwk=__cryptoParse(__cryptoStringify(keyData));else r.key=__cryptoBytes(keyData);
        return __cryptoDescriptor(__cryptoInvoke(r));
    }
    async exportKey(format,key){
        const value=__cryptoInvoke({operation:'export',handle:__cryptoRecord(key).handle,format:String(format)});
        return format==='jwk'?value:Uint8Array.from(value).buffer;
    }
    async sign(algorithm,key,data){
        const r=__cryptoOperation(algorithm,key,'sign');r.data=__cryptoBytes(data);
        if(r.algorithm==='ECDSA')r.hash=__cryptoHash(algorithm.hash);
        if(r.algorithm==='RSA-PSS')r.saltLength=__cryptoInteger(algorithm.saltLength,'saltLength',0x7fffffff);
        return Uint8Array.from(__cryptoInvoke(r)).buffer;
    }
    async verify(algorithm,key,signature,data){
        const r=__cryptoOperation(algorithm,key,'verify');r.data=__cryptoBytes(data);r.signature=__cryptoBytes(signature);
        if(r.algorithm==='ECDSA')r.hash=__cryptoHash(algorithm.hash);
        if(r.algorithm==='RSA-PSS')r.saltLength=__cryptoInteger(algorithm.saltLength,'saltLength',0x7fffffff);
        return __cryptoInvoke(r);
    }
    async encrypt(algorithm,key,data){return Uint8Array.from(__cryptoInvoke({...__cryptoCipherRequest(algorithm,key,'encrypt'),data:__cryptoBytes(data)})).buffer;}
    async decrypt(algorithm,key,data){return Uint8Array.from(__cryptoInvoke({...__cryptoCipherRequest(algorithm,key,'decrypt'),data:__cryptoBytes(data)})).buffer;}
    async wrapKey(format,key,wrappingKey,algorithm){
        const r=__cryptoCipherRequest(algorithm,wrappingKey,'encrypt','wrapKey');
        const exported=await this.exportKey(format,key);
        let bytes=format==='jwk'?new TextEncoder().encode(__cryptoStringify(exported)):new Uint8Array(exported);
        // JWK wrapping with AES-KW pads JSON with spaces to an eight-byte boundary.
        if(format==='jwk' && r.algorithm==='AES-KW' && bytes.length%8){const padded=new Uint8Array(Math.ceil(bytes.length/8)*8);padded.fill(32);padded.set(bytes);bytes=padded;}
        r.data=__cryptoBytes(bytes);return Uint8Array.from(__cryptoInvoke(r)).buffer;
    }
    async unwrapKey(format,wrapped,key,algorithm,unwrappedAlgorithm,extractable,usages){
        const r={...__cryptoCipherRequest(algorithm,key,'decrypt','unwrapKey'),data:__cryptoBytes(wrapped)};
        const decoded=Uint8Array.from(__cryptoInvoke(r));let data=decoded;
        if(format==='jwk'){try{data=__cryptoParse(new TextDecoder().decode(decoded));}catch{throw __cryptoError('Invalid wrapped JWK','DataError');}}
        return this.importKey(format,data,unwrappedAlgorithm,extractable,usages);
    }
    async deriveBits(algorithm,key,length=null){return Uint8Array.from(__cryptoInvoke(__cryptoDeriveRequest(algorithm,key,length,'deriveBits'))).buffer;}
    async deriveKey(algorithm,key,derivedAlgorithm,extractable,usages){
        const derived={...__cryptoAlgorithm(derivedAlgorithm),extractable:Boolean(extractable),usages:Array.from(usages,String)};
        derived.length=__cryptoLength(derived);
        return __cryptoDescriptor(__cryptoInvoke({...__cryptoDeriveRequest(algorithm,key,derived.length,'deriveKey'),derived}));
    }
}
const __subtleCrypto=Object.create(SubtleCrypto.prototype);
Object.defineProperty(Crypto.prototype,'subtle',{get(){return __subtleCrypto;},enumerable:true,configurable:true});
Object.assign(globalThis,{CryptoKey,SubtleCrypto});
