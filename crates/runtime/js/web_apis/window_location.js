function __replaceCharacterData(node, offset, count, data) {
  const current = __callHost('textContent', node);
  if (offset > current.length)
    throw new DOMException('offset is outside the data', 'IndexSizeError');
  count = Math.min(count, current.length - offset);
  const previous = __characterDataEdit;
  __characterDataEdit = { target: node, offset, count, length: data.length };
  try {
    __callHost(
      'setTextContent',
      node,
      current.slice(0, offset) + data + current.slice(offset + count),
    );
  } finally {
    __characterDataEdit = previous;
  }
}

class CharacterData extends Node {
  constructor() {
    throw new TypeError('Illegal constructor');
  }
  get previousElementSibling() {
    return __callHost('previousElementSibling', this);
  }
  get nextElementSibling() {
    return __callHost('nextElementSibling', this);
  }
  get data() {
    return __callHost('textContent', this);
  }
  set data(value) {
    value = value === null ? '' : `${value}`;
    __replaceCharacterData(this, 0, Infinity, value);
  }
  get length() {
    return __callHost('textContent', this).length;
  }
  substringData(offset, count) {
    if (arguments.length < 2) throw new TypeError('Not enough arguments');
    offset = +offset >>> 0;
    count = +count >>> 0;
    const data = __callHost('textContent', this);
    if (offset > data.length)
      throw new DOMException('offset is outside the data', 'IndexSizeError');
    return data.slice(offset, offset + count);
  }
  appendData(data) {
    if (arguments.length < 1) throw new TypeError('Not enough arguments');
    data = `${data}`;
    __replaceCharacterData(this, __callHost('textContent', this).length, 0, data);
  }
  insertData(offset, data) {
    if (arguments.length < 2) throw new TypeError('Not enough arguments');
    offset = +offset >>> 0;
    data = `${data}`;
    __replaceCharacterData(this, offset, 0, data);
  }
  deleteData(offset, count) {
    if (arguments.length < 2) throw new TypeError('Not enough arguments');
    offset = +offset >>> 0;
    count = +count >>> 0;
    __replaceCharacterData(this, offset, count, '');
  }
  replaceData(offset, count, data) {
    if (arguments.length < 3) throw new TypeError('Not enough arguments');
    offset = +offset >>> 0;
    count = +count >>> 0;
    data = `${data}`;
    __replaceCharacterData(this, offset, count, data);
  }
  before(...nodes) {
    __beforeNode(this, nodes);
  }
  after(...nodes) {
    __afterNode(this, nodes);
  }
  replaceWith(...nodes) {
    __replaceNode(this, nodes);
  }
  remove() {
    __removeNode(this);
  }
}

class Text extends CharacterData {
  get wholeText() {
    let first = this,
      previous;
    while (
      (previous = __callHost('previousSibling', first)) &&
      [3, 4].includes(__callHost('nodeType', previous))
    )
      first = previous;
    let result = '';
    for (
      let node = first;
      node && [3, 4].includes(__callHost('nodeType', node));
      node = __callHost('nextSibling', node)
    ) {
      result += __callHost('textContent', node);
    }
    return result;
  }
  splitText(offset) {
    if (arguments.length < 1) throw new TypeError('Not enough arguments');
    offset = +offset >>> 0;
    const data = __callHost('textContent', this);
    if (offset > data.length) throw new DOMException('Invalid offset', 'IndexSizeError');
    const node = __callHost(
      'createTextNode',
      __callHost('ownerDocument', this),
      data.slice(offset),
    );
    const parent = __callHost('parentNode', this);
    const index = parent ? __callHost('childNodes', parent).indexOf(this) : -1;
    const moved = [];
    if (parent)
      __rangeBoundaries((r, n, o) => {
        if (r[n] === this && r[o] > offset) {
          const nextOffset = r[o] - offset;
          moved.push(() => {
            r[n] = node;
            r[o] = nextOffset;
          });
        } else if (r[n] === parent && r[o] === index + 1)
          moved.push(() => {
            r[o]++;
          });
      });
    __replaceCharacterData(this, offset, data.length - offset, '');
    if (parent) __callHost('insertBefore', parent, node, __callHost('nextSibling', this));
    for (const update of moved) update();
    return node;
  }
  constructor(data = '') {
    return __callHost('createTextNode', document, `${data}`);
  }
}
class Comment extends CharacterData {
  constructor(data = '') {
    return document.createComment(String(data));
  }
}
class CDATASection extends Text {
  constructor() {
    throw new TypeError('Illegal constructor');
  }
}
class ProcessingInstruction extends CharacterData {
  constructor() {
    throw new TypeError('Illegal constructor');
  }
  get target() {
    return '';
  }
}
class DocumentFragment extends Node {
  constructor() {
    return document.createDocumentFragment();
  }
  get children() {
    return __elementChildren(this);
  }
  get childElementCount() {
    return __callHost('childElementCount', this);
  }
  get firstElementChild() {
    return __callHost('firstElementChild', this);
  }
  get lastElementChild() {
    return __callHost('lastElementChild', this);
  }
  getElementById(id) {
    id = String(id);
    if (id === '') return null;
    return [...this.querySelectorAll('[id]')].find((element) => element.id === id) ?? null;
  }
  querySelector(selector) {
    return __callHost('querySelector', this, String(selector));
  }
  querySelectorAll(selector) {
    return __createNodeList(__callHost('querySelectorAll', this, String(selector)));
  }
  append(...nodes) {
    __appendNodes(this, nodes);
  }
  prepend(...nodes) {
    __prependNodes(this, nodes);
  }
  replaceChildren(...nodes) {
    __replaceChildren(this, nodes);
  }
}
class WindowProperties extends EventTarget {
  constructor() {
    super();
    throw new TypeError('Illegal constructor');
  }
}
function Window() {
  throw new TypeError('Illegal constructor');
}
Window.prototype = Object.getPrototypeOf(globalThis);
// WebKit keeps an intermediate WindowProperties prototype but its constructor
// is inherited from EventTarget, so constructor.name walks as EventTarget.
delete WindowProperties.prototype.constructor;
Object.setPrototypeOf(Window.prototype, WindowProperties.prototype);
Object.defineProperty(Window.prototype, 'constructor', {
  value: Window,
  writable: true,
  enumerable: false,
  configurable: true,
});
Object.defineProperties(Window.prototype, {
  event: {
    get() {
      return undefined;
    },
    enumerable: true,
    configurable: true,
  },
  offscreenBuffering: {
    get() {
      return true;
    },
    enumerable: true,
    configurable: true,
  },
  webkitIndexedDB: {
    get() {
      return globalThis.indexedDB;
    },
    enumerable: true,
    configurable: true,
  },
  clientInformation: {
    get() {
      return this.navigator;
    },
    enumerable: true,
    configurable: true,
  },
  origin: {
    get() {
      return this.location.origin;
    },
    enumerable: true,
    configurable: true,
  },
  frameElement: {
    get() {
      return null;
    },
    enumerable: true,
    configurable: true,
  },
  closed: {
    get() {
      return false;
    },
    enumerable: true,
    configurable: true,
  },
  frames: {
    get() {
      return this;
    },
    enumerable: true,
    configurable: true,
  },
  length: {
    get() {
      return document.getElementsByTagName('iframe').length;
    },
    enumerable: true,
    configurable: true,
  },
});
for (const name of ['name', 'status']) {
  const values = new WeakMap();
  Object.defineProperty(Window.prototype, name, {
    get() {
      return values.get(this) ?? '';
    },
    set(value) {
      values.set(this, String(value));
    },
    enumerable: true,
    configurable: true,
  });
}

class Location {
  constructor() {
    throw new TypeError('Illegal constructor');
  }
  get href() {
    return __callHost('location', this, 'href');
  }
  set href(value) {
    this.assign(value);
  }
  get protocol() {
    return __callHost('location', this, 'protocol');
  }
  set protocol(value) {
    __setLocationComponent(this, 'protocol', value);
  }
  get host() {
    return __callHost('location', this, 'host');
  }
  set host(value) {
    __setLocationComponent(this, 'host', value);
  }
  get hostname() {
    return __callHost('location', this, 'hostname');
  }
  set hostname(value) {
    __setLocationComponent(this, 'hostname', value);
  }
  get port() {
    return __callHost('location', this, 'port');
  }
  set port(value) {
    __setLocationComponent(this, 'port', value);
  }
  get pathname() {
    return __callHost('location', this, 'pathname');
  }
  set pathname(value) {
    __setLocationComponent(this, 'pathname', value);
  }
  get search() {
    return __callHost('location', this, 'search');
  }
  set search(value) {
    __setLocationComponent(this, 'search', value);
  }
  get hash() {
    return __callHost('location', this, 'hash');
  }
  set hash(value) {
    const url = new URL(this.href);
    url.hash = String(value);
    this.assign(url.href);
  }
  get origin() {
    return __callHost('location', this, 'origin');
  }
  get ancestorOrigins() {
    return __emptyAncestorOrigins;
  }
  assign(url) {
    __navigateLocation(this, url, false);
  }
  replace(url) {
    __navigateLocation(this, url, true);
  }
  reload() {
    __callHost('locationNavigate', this, this.href);
  }
  toString() {
    return this.href;
  }
}

// Top-level facade only; the lightweight frame model does not track origin lists.
const __emptyAncestorOrigins = Object.freeze({
  length: 0,
  item() {
    return null;
  },
  contains() {
    return false;
  },
  *[Symbol.iterator]() {},
  [Symbol.toStringTag]: 'DOMStringList',
});
function __setLocationComponent(receiver, component, value) {
  const url = new URL(receiver.href);
  url[component] = __toUSVString(value);
  receiver.assign(url.href);
}

function __navigateLocation(receiver, value, replace) {
  const oldURL = receiver.href,
    next = new URL(__toUSVString(value), oldURL);
  const current = new URL(oldURL),
    oldHash = current.hash;
  current.hash = '';
  const destination = new URL(next.href);
  destination.hash = '';
  if (current.href === destination.href && (next.hash || oldHash)) {
    if (next.href === oldURL) return;
    if (replace) history.replaceState(null, '', next.href);
    else history.pushState(null, '', next.href);
    setTimeout(() => {
      window.dispatchEvent(__markTrustedEvent(new PopStateEvent('popstate', { state: null })));
      window.dispatchEvent(
        __markTrustedEvent(new HashChangeEvent('hashchange', { oldURL, newURL: next.href })),
      );
    }, 0);
  } else __callHost('locationNavigate', receiver, next.href);
}

class Navigator {
  // Identity getters are installed from the browser profile after bindings.
  constructor() {
    throw new TypeError('Illegal constructor');
  }
  javaEnabled() {
    return false;
  }
  // Legacy ignored-result placeholder: no beacon queue or network request.
  sendBeacon() {}
}

const __historyConstructorToken = {};
const __historyEntries = new WeakMap();
const __historyIndexes = new WeakMap();
const __historyScrollRestoration = new WeakMap();

function __historyData(history) {
  const entries = __historyEntries.get(history);
  if (entries === undefined)
    throw new TypeError('History method called on an incompatible receiver');
  return { entries, index: __historyIndexes.get(history) };
}

function __historyUrl(url) {
  if (url === undefined || url === null || url === '') return location.href;
  const resolved = new URL(__toUSVString(url), location.href);
  if (resolved.origin !== location.origin) {
    throw new DOMException('History state URL must be same-origin', 'SecurityError');
  }
  return resolved.href;
}

class History {
  constructor(token) {
    if (token !== __historyConstructorToken) throw new TypeError('Illegal constructor');
    __historyEntries.set(this, [{ url: location.href, state: null }]);
    __historyIndexes.set(this, 0);
    __historyScrollRestoration.set(this, 'auto');
  }
  get length() {
    return __historyData(this).entries.length;
  }
  get state() {
    const { entries, index } = __historyData(this);
    return entries[index].state;
  }
  get scrollRestoration() {
    __historyData(this);
    return __historyScrollRestoration.get(this);
  }
  set scrollRestoration(value) {
    __historyData(this);
    value = String(value);
    if (value === 'auto' || value === 'manual') __historyScrollRestoration.set(this, value);
  }
  pushState(data, unused, url = null) {
    if (arguments.length < 2) throw new TypeError('History.pushState requires data and title');
    String(unused);
    const state = structuredClone(data);
    const href = __callHost('historyUpdateUrl', this, __historyUrl(url));
    const { entries, index } = __historyData(this);
    entries.splice(index + 1);
    entries.push({ url: href, state });
    __historyIndexes.set(this, entries.length - 1);
  }
  replaceState(data, unused, url = null) {
    if (arguments.length < 2) throw new TypeError('History.replaceState requires data and title');
    String(unused);
    const state = structuredClone(data);
    const href = __callHost('historyUpdateUrl', this, __historyUrl(url));
    const { entries, index } = __historyData(this);
    entries[index] = { url: href, state };
  }
  go(delta = 0) {
    delta = Number(delta);
    if (!Number.isFinite(delta)) delta = 0;
    delta = Math.trunc(delta);
    if (delta === 0) return;
    const { entries, index } = __historyData(this);
    const destination = index + delta;
    if (destination < 0 || destination >= entries.length) return;
    setTimeout(() => {
      const oldURL = location.href;
      __historyIndexes.set(this, destination);
      const entry = entries[destination];
      const newURL = __callHost('historyUpdateUrl', this, entry.url);
      window.dispatchEvent(
        __markTrustedEvent(new PopStateEvent('popstate', { state: entry.state })),
      );
      if (new URL(oldURL).hash !== new URL(newURL).hash) {
        window.dispatchEvent(
          __markTrustedEvent(new HashChangeEvent('hashchange', { oldURL, newURL })),
        );
      }
    }, 0);
  }
  back() {
    this.go(-1);
  }
  forward() {
    this.go(1);
  }
}
