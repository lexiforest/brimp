import {generateKeyPair, exportJWK, importJWK, SignJWT, jwtVerify, CompactEncrypt, compactDecrypt} from 'jose';
import {check} from './check.js';

globalThis.runWorkload = async () => {
  const signatures=[];
  for (const alg of ['RS256','PS256','ES256','ES384','ES512']) {
    const {publicKey,privateKey}=await generateKeyPair(alg,{extractable:true});
    const imported=await importJWK(await exportJWK(publicKey),alg);
    const token=await new SignJWT({scope:'read'}).setProtectedHeader({alg}).setSubject('extended').sign(privateKey);
    const verified=await jwtVerify(token,imported,{algorithms:[alg]});
    const parts=token.split('.');parts[2]=(parts[2][0]==='A'?'B':'A')+parts[2].slice(1);
    let rejected=false;try{await jwtVerify(parts.join('.'),imported,{algorithms:[alg]});}catch(e){rejected=e.code==='ERR_JWS_SIGNATURE_VERIFICATION_FAILED';}
    check(verified.payload.sub==='extended'&&rejected,`${alg} verification and rejection`);
    signatures.push(alg);
  }
  const encryption=[];
  for (const alg of ['RSA-OAEP-256','ECDH-ES+A256KW']) {
    const {publicKey,privateKey}=await generateKeyPair(alg);
    const plaintext=new TextEncoder().encode('encrypted browser workload');
    const token=await new CompactEncrypt(plaintext).setProtectedHeader({alg,enc:'A256GCM'}).encrypt(publicKey);
    const decoded=await compactDecrypt(token,privateKey);
    const parts=token.split('.');parts[4]=(parts[4][0]==='A'?'B':'A')+parts[4].slice(1);
    let rejected=false;try{await compactDecrypt(parts.join('.'),privateKey);}catch(e){rejected=e.code==='ERR_JWE_DECRYPTION_FAILED';}
    check(new TextDecoder().decode(decoded.plaintext)==='encrypted browser workload'&&rejected,`${alg} encryption and authentication`);
    encryption.push(alg);
  }
  return {signatures,encryption,tamperingRejected:true};
};
