import { importJWK, SignJWT, jwtVerify } from 'jose';
import { check } from './check.js';

globalThis.runWorkload = async () => {
  const key = await importJWK(
    { kty: 'oct', k: 'MDEyMzQ1Njc4OWFiY2RlZjAxMjM0NTY3ODlhYmNkZWY' },
    'HS256',
  );
  const token = await new SignJWT({ role: 'reader' })
    .setProtectedHeader({ alg: 'HS256' })
    .setSubject('corpus')
    .setIssuedAt(1700000000)
    .setExpirationTime(1700003600)
    .sign(key);
  const options = { algorithms: ['HS256'], currentDate: new Date(1700000100000) };
  const verified = await jwtVerify(token, key, options);
  const parts = token.split('.');
  parts[2] = (parts[2][0] === 'A' ? 'B' : 'A') + parts[2].slice(1);
  let invalidRejected = false;
  try {
    await jwtVerify(parts.join('.'), key, options);
  } catch (error) {
    invalidRejected = error.code === 'ERR_JWS_SIGNATURE_VERIFICATION_FAILED';
  }
  check(
    verified.payload.sub === 'corpus' && verified.payload.role === 'reader' && invalidRejected,
    'jose must verify claims and reject an invalid signature',
  );
  return { payload: verified.payload, algorithm: verified.protectedHeader.alg, invalidRejected };
};
