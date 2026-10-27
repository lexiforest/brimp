const __navigatorServiceConstructionToken = Symbol('navigator service construction');
const __transientActivationDuration = 5000;
const __userActivationNow = Date.now.bind(Date);
let __lastUserActivation = -Infinity;
let __hasBeenUserActive = false;

class UserActivation {
  constructor(...args) {
    const token = args[0];
    if (token !== __navigatorServiceConstructionToken) throw new TypeError('Illegal constructor');
  }
  get hasBeenActive() {
    return __hasBeenUserActive;
  }
  get isActive() {
    return __userActivationNow() - __lastUserActivation < __transientActivationDuration;
  }
}

const __userActivation = new UserActivation(__navigatorServiceConstructionToken);

function __notifyUserActivation() {
  __hasBeenUserActive = true;
  __lastUserActivation = __userActivationNow();
}

Object.defineProperty(Navigator.prototype, 'userActivation', {
  get() {
    return __userActivation;
  },
  configurable: true,
});
class Crypto {
  constructor() {
    throw new TypeError('Illegal constructor');
  }
  getRandomValues(array) {
    if (
      !ArrayBuffer.isView(array) ||
      array instanceof DataView ||
      ![
        Int8Array,
        Uint8Array,
        Uint8ClampedArray,
        Int16Array,
        Uint16Array,
        Int32Array,
        Uint32Array,
        BigInt64Array,
        BigUint64Array,
      ].some((type) => array instanceof type)
    ) {
      throw new DOMException(
        'The provided ArrayBufferView is not an integer typed array',
        'TypeMismatchError',
      );
    }
    if (array.byteLength > 65536) {
      throw new DOMException('The requested length exceeds 65,536 bytes', 'QuotaExceededError');
    }
    const bytes = JSON.parse(__callHost('cryptoRandomBytes', this, array.byteLength));
    new Uint8Array(array.buffer, array.byteOffset, array.byteLength).set(bytes);
    return array;
  }
  randomUUID() {
    const bytes = this.getRandomValues(new Uint8Array(16));
    bytes[6] = (bytes[6] & 0x0f) | 0x40;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    const hex = Array.from(bytes, (byte) => byte.toString(16).padStart(2, '0'));
    return `${hex.slice(0, 4).join('')}-${hex.slice(4, 6).join('')}-${hex.slice(6, 8).join('')}-${hex.slice(8, 10).join('')}-${hex.slice(10).join('')}`;
  }
}

const __crypto = Object.create(Crypto.prototype);
Object.defineProperty(globalThis, 'crypto', {
  value: __crypto,
  writable: false,
  configurable: true,
});
