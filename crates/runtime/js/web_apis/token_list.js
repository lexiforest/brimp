// Live ordered token sets backed by canonical DOM attributes.
const __classLists = new WeakMap();

const __tokenListData = new WeakMap();
function __tokenListState(list) {
  const state = __tokenListData.get(list);
  if (!state) throw new TypeError('Illegal invocation');
  return state;
}
function __tokenListTokens(state) {
  return [
    ...new Set(
      (__getContentAttribute(state.element, state.attribute) ?? '')
        .split(/[\t\n\f\r ]+/)
        .filter(Boolean),
    ),
  ];
}
function __writeTokenSet(state, tokens) {
  if (tokens.length || __getContentAttribute(state.element, state.attribute) !== null)
    __setContentAttribute(state.element, state.attribute, tokens.join(' '));
}
function __validateDomToken(token) {
  if (token === '') throw new DOMException('The token must not be empty', 'SyntaxError');
  if (/[\t\n\f\r ]/.test(token))
    throw new DOMException('The token must not contain ASCII whitespace', 'InvalidCharacterError');
}
function __tokenListArgument(count, required) {
  if (count < required) throw new TypeError('Not enough arguments');
}
function __createTokenList(element, attribute = 'class') {
  const target = Object.create(DOMTokenList.prototype),
    state = { element, attribute };
  const indexed = (key) =>
    typeof key === 'string' && /^(0|[1-9][0-9]*)$/.test(key) && Number(key) < 4294967295;
  const list = new Proxy(target, {
    get(target, key, receiver) {
      if (indexed(key)) {
        const tokens = __tokenListTokens(state);
        if (Number(key) < tokens.length) return tokens[Number(key)];
      }
      return Reflect.get(target, key, receiver);
    },
    has(target, key) {
      return (
        (indexed(key) && Number(key) < __tokenListTokens(state).length) || Reflect.has(target, key)
      );
    },
    ownKeys(target) {
      return [
        ...__tokenListTokens(state).map((_, index) => String(index)),
        ...Reflect.ownKeys(target),
      ];
    },
    getOwnPropertyDescriptor(target, key) {
      if (indexed(key)) {
        const tokens = __tokenListTokens(state);
        if (Number(key) < tokens.length)
          return {
            value: tokens[Number(key)],
            writable: false,
            enumerable: true,
            configurable: true,
          };
      }
      return Reflect.getOwnPropertyDescriptor(target, key);
    },
    set(target, key, value, receiver) {
      return !indexed(key) && Reflect.set(target, key, value, receiver);
    },
    defineProperty(target, key, descriptor) {
      return !indexed(key) && Reflect.defineProperty(target, key, descriptor);
    },
    deleteProperty(target, key) {
      return (
        !(indexed(key) && Number(key) < __tokenListTokens(state).length) &&
        Reflect.deleteProperty(target, key)
      );
    },
    // A JS Proxy cannot expose new indexed keys on a non-extensible target.
    // Reject freezing instead of allowing later attribute changes to violate its invariants.
    preventExtensions() {
      return false;
    },
  });
  __tokenListData.set(list, state);
  return list;
}
class DOMTokenList {
  constructor() {
    throw new TypeError('Illegal constructor');
  }
  get length() {
    return __tokenListTokens(__tokenListState(this)).length;
  }
  get value() {
    const s = __tokenListState(this);
    return __getContentAttribute(s.element, s.attribute) ?? '';
  }
  set value(value) {
    const s = __tokenListState(this);
    __setContentAttribute(s.element, s.attribute, `${value}`);
  }
  item(index) {
    const s = __tokenListState(this);
    __tokenListArgument(arguments.length, 1);
    return __tokenListTokens(s)[+index >>> 0] ?? null;
  }
  contains(token) {
    const s = __tokenListState(this);
    __tokenListArgument(arguments.length, 1);
    token = `${token}`;
    return __tokenListTokens(s).includes(token);
  }
  add(...tokens) {
    const s = __tokenListState(this);
    tokens = tokens.map((token) => `${token}`);
    tokens.forEach(__validateDomToken);
    const values = __tokenListTokens(s);
    for (const token of tokens) if (!values.includes(token)) values.push(token);
    __writeTokenSet(s, values);
  }
  remove(...tokens) {
    const s = __tokenListState(this);
    tokens = tokens.map((token) => `${token}`);
    tokens.forEach(__validateDomToken);
    __writeTokenSet(
      s,
      __tokenListTokens(s).filter((token) => !tokens.includes(token)),
    );
  }
  toggle(token, force = undefined) {
    const s = __tokenListState(this);
    __tokenListArgument(arguments.length, 1);
    token = `${token}`;
    __validateDomToken(token);
    const values = __tokenListTokens(s),
      index = values.indexOf(token),
      present = index !== -1;
    const desired = force !== undefined ? Boolean(force) : !present;
    if (desired !== present) {
      if (desired) values.push(token);
      else values.splice(index, 1);
      __writeTokenSet(s, values);
    }
    return desired;
  }
  replace(token, newToken) {
    const s = __tokenListState(this);
    __tokenListArgument(arguments.length, 2);
    token = `${token}`;
    newToken = `${newToken}`;
    if (token === '' || newToken === '')
      throw new DOMException('The token must not be empty', 'SyntaxError');
    __validateDomToken(token);
    __validateDomToken(newToken);
    const values = __tokenListTokens(s),
      index = values.indexOf(token);
    if (index === -1) return false;
    values[index] = newToken;
    __writeTokenSet(s, [...new Set(values)]);
    return true;
  }
  supports(token) {
    const s = __tokenListState(this);
    __tokenListArgument(arguments.length, 1);
    token = __asciiLowercase(`${token}`);
    const tag = __callHost('localName', s.element);
    if (s.attribute === 'sandbox' && tag === 'iframe')
      return [
        'allow-downloads',
        'allow-forms',
        'allow-modals',
        'allow-pointer-lock',
        'allow-popups',
        'allow-popups-to-escape-sandbox',
        'allow-same-origin',
        'allow-scripts',
        'allow-storage-access-by-user-activation',
        'allow-top-navigation',
        'allow-top-navigation-by-user-activation',
        'allow-top-navigation-to-custom-protocols',
      ].includes(token);
    if (s.attribute !== 'rel') throw new TypeError('This attribute has no supported tokens');
    if (tag === 'a' || tag === 'area') return ['noopener', 'noreferrer', 'opener'].includes(token);
    if (tag === 'link')
      return [
        'alternate',
        'stylesheet',
        'preload',
        'modulepreload',
        'preconnect',
        'dns-prefetch',
        'icon',
        'manifest',
        'expect',
        'apple-touch-icon',
        'apple-touch-icon-precomposed',
      ].includes(token);
    throw new TypeError('This attribute has no supported tokens');
  }
  toString() {
    const s = __tokenListState(this);
    return __getContentAttribute(s.element, s.attribute) ?? '';
  }
}
for (const name of ['entries', 'keys', 'values', 'forEach'])
  Object.defineProperty(DOMTokenList.prototype, name, {
    value: Array.prototype[name],
    writable: true,
    enumerable: true,
    configurable: true,
  });
Object.defineProperty(DOMTokenList.prototype, Symbol.iterator, {
  value: Array.prototype.values,
  writable: true,
  configurable: true,
});
