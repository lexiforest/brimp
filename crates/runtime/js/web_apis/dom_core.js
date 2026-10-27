const __dynamicScriptElements = new Map();
const __startedScripts = new WeakSet();
const __forceAsyncScripts = new WeakSet();
const __pendingDynamicScripts = [];
const __scriptMapGet = Function.prototype.call.bind(Map.prototype.get);
const __scriptMapSet = Function.prototype.call.bind(Map.prototype.set);
const __scriptMapDelete = Function.prototype.call.bind(Map.prototype.delete);
const __scriptStartedHas = Function.prototype.call.bind(WeakSet.prototype.has);
const __scriptStartedAdd = Function.prototype.call.bind(WeakSet.prototype.add);
const __scriptPush = Function.prototype.call.bind(Array.prototype.push);
const __scriptSplice = Function.prototype.call.bind(Array.prototype.splice);
let __nextDynamicScriptId = 1;
let __currentScript = null;
let __documentReadyState = 'complete';

function __queueDynamicScript(element) {
  if (
    !(element instanceof HTMLScriptElement) ||
    !element.isConnected ||
    __scriptStartedHas(__startedScripts, element)
  )
    return;
  const type = element.type.trim().toLowerCase();
  if (
    type !== '' &&
    type !== 'text/javascript' &&
    type !== 'application/javascript' &&
    type !== 'module'
  )
    return;
  __scriptStartedAdd(__startedScripts, element);
  const id = __nextDynamicScriptId++;
  __scriptMapSet(__dynamicScriptElements, id, element);
  __scriptPush(__pendingDynamicScripts, {
    id,
    module: type === 'module',
    src: element.getAttribute('src') === null ? null : element.src,
    source: __scriptText(element),
  });
}

// Captured and removed by native initialization before plugins or page code.
Object.defineProperty(globalThis, '__brimpScriptController', {
  configurable: true,
  value: (() => {
    const parse = JSON.parse,
      stringify = JSON.stringify;
    return (serialized) => {
      const { op, id, error, state } = parse(serialized);
      if (op === 'ready') {
        if (__documentReadyState !== state) {
          __documentReadyState = state;
          document.dispatchEvent(new Event('readystatechange'));
        }
        return null;
      }
      if (op === 'take') return stringify(__scriptSplice(__pendingDynamicScripts, 0));
      if (op === 'complete') {
        const element = __scriptMapGet(__dynamicScriptElements, id);
        if (element) {
          __scriptMapDelete(__dynamicScriptElements, id);
          element.dispatchEvent(new Event(error ? 'error' : 'load'));
        }
      } else if (op === 'document') __currentScript = document.scripts.item(id);
      else if (op === 'dynamic')
        __currentScript = __scriptMapGet(__dynamicScriptElements, id) ?? null;
      else if (op === 'clear') __currentScript = null;
      return null;
    };
  })(),
});

const __nodeConstructionToken = {};
function __nodeArgument(value, count) {
  if (!count) throw new TypeError('Not enough arguments');
  if (value == null) return null;
  if (!__attributeNodes.has(value) && !__callHost('isNativeNode', value))
    throw new TypeError('Expected a Node');
  return value;
}
function __equalNodes(first, second) {
  if (second === null) return false;
  const pending = [[first, second]];
  while (pending.length) {
    const [a, b] = pending.pop();
    if (a === b) continue;
    const kind = __attributeNodes.has(a) ? 2 : __callHost('nodeType', a);
    if (kind !== (__attributeNodes.has(b) ? 2 : __callHost('nodeType', b))) return false;
    if (kind === 2) {
      if (a.namespaceURI !== b.namespaceURI || a.localName !== b.localName || a.value !== b.value)
        return false;
      continue;
    }
    if (kind === 1) {
      for (const key of ['namespaceURI', 'prefix', 'localName'])
        if (__callHost(key, a) !== __callHost(key, b)) return false;
      const left = __attributeRecords(a),
        right = __attributeRecords(b);
      if (
        left.length !== right.length ||
        left.some(
          (x) =>
            !right.some(
              (y) =>
                x.namespaceURI === y.namespaceURI &&
                x.localName === y.localName &&
                x.value === y.value,
            ),
        )
      )
        return false;
    } else if (kind === 10) {
      for (const key of ['nodeName', 'doctypePublicId', 'doctypeSystemId'])
        if (__callHost(key, a) !== __callHost(key, b)) return false;
    } else if ([3, 4, 7, 8].includes(kind)) {
      if (__callHost('textContent', a) !== __callHost('textContent', b)) return false;
      if (kind === 7 && __callHost('nodeName', a) !== __callHost('nodeName', b)) return false;
    }
    const left = __callHost('childNodes', a),
      right = __callHost('childNodes', b);
    if (left.length !== right.length) return false;
    for (let i = 0; i < left.length; i++) pending.push([left[i], right[i]]);
  }
  return true;
}
class Node extends EventTarget {
  constructor(token = undefined) {
    super();
    if (token !== __nodeConstructionToken) throw new TypeError('Illegal constructor');
  }
  get nodeType() {
    return __callHost('nodeType', this);
  }
  get nodeName() {
    return __callHost('nodeName', this);
  }
  get parentNode() {
    return __callHost('parentNode', this);
  }
  get ownerDocument() {
    return __callHost('ownerDocument', this);
  }
  get baseURI() {
    return __callHost('nodeBaseURI', __attributeNodes.has(this) ? this.ownerDocument : this);
  }
  get parentElement() {
    const parent = this.parentNode;
    return parent instanceof Element ? parent : null;
  }
  get firstChild() {
    return __callHost('firstChild', this);
  }
  get lastChild() {
    return __callHost('lastChild', this);
  }
  get previousSibling() {
    return __callHost('previousSibling', this);
  }
  get nextSibling() {
    return __callHost('nextSibling', this);
  }
  get childNodes() {
    return __cachedCollection(this, 'childNodes', () =>
      __createNodeList(() => __callHost('childNodes', this)),
    );
  }
  hasChildNodes() {
    return this.firstChild !== null;
  }
  get isConnected() {
    return this.getRootNode({ composed: true }) instanceof Document;
  }
  getRootNode(options = {}) {
    if (options != null && !['object', 'function'].includes(typeof options))
      throw new TypeError('Expected a dictionary');
    const composed = Boolean(options?.composed);
    let node = this;
    while (node.parentNode) node = node.parentNode;
    if (composed && node instanceof ShadowRoot) return node.host.getRootNode({ composed: true });
    return node;
  }
  isSameNode(other) {
    return this === __nodeArgument(other, arguments.length);
  }
  isEqualNode(other) {
    return __equalNodes(this, __nodeArgument(other, arguments.length));
  }
  contains(other) {
    other = __nodeArgument(other, arguments.length);
    while (other) {
      if (other === this) return true;
      if (__attributeNodes.has(other)) return false;
      other = __callHost('parentNode', other);
    }
    return false;
  }
  compareDocumentPosition(other) {
    if (__nodeArgument(other, arguments.length) === null) throw new TypeError('Expected a Node');
    if (other === this) return 0;
    const ancestors = (node) => {
      const nodes = [];
      while (node) {
        nodes.unshift(node);
        node = __attributeNodes.has(node) ? node.ownerElement : __callHost('parentNode', node);
      }
      return nodes;
    };
    const thisAncestors = ancestors(this),
      otherAncestors = ancestors(other);
    if (thisAncestors[0] !== otherAncestors[0]) {
      const left = thisAncestors[0],
        right = otherAncestors[0];
      if (!__disconnectedNodeOrder.has(left))
        __disconnectedNodeOrder.set(left, __nextDisconnectedNodeOrder++);
      if (!__disconnectedNodeOrder.has(right))
        __disconnectedNodeOrder.set(right, __nextDisconnectedNodeOrder++);
      const order =
        __disconnectedNodeOrder.get(left) < __disconnectedNodeOrder.get(right)
          ? Node.DOCUMENT_POSITION_FOLLOWING
          : Node.DOCUMENT_POSITION_PRECEDING;
      return (
        Node.DOCUMENT_POSITION_DISCONNECTED | Node.DOCUMENT_POSITION_IMPLEMENTATION_SPECIFIC | order
      );
    }
    let index = 0;
    while (thisAncestors[index] === otherAncestors[index]) index++;
    if (index === thisAncestors.length) {
      return Node.DOCUMENT_POSITION_FOLLOWING | Node.DOCUMENT_POSITION_CONTAINED_BY;
    }
    if (index === otherAncestors.length) {
      return Node.DOCUMENT_POSITION_PRECEDING | Node.DOCUMENT_POSITION_CONTAINS;
    }
    const left = thisAncestors[index],
      right = otherAncestors[index];
    const leftAttr = __attributeNodes.has(left),
      rightAttr = __attributeNodes.has(right);
    if (leftAttr && rightAttr) {
      const records = __attributeRecords(thisAncestors[index - 1]);
      const position = (attr) =>
        records.findIndex(
          (a) => a.namespaceURI === attr.namespaceURI && a.localName === attr.localName,
        );
      return (
        Node.DOCUMENT_POSITION_IMPLEMENTATION_SPECIFIC |
        (position(left) < position(right)
          ? Node.DOCUMENT_POSITION_FOLLOWING
          : Node.DOCUMENT_POSITION_PRECEDING)
      );
    }
    if (leftAttr || rightAttr)
      return leftAttr ? Node.DOCUMENT_POSITION_FOLLOWING : Node.DOCUMENT_POSITION_PRECEDING;
    const siblings = __callHost('childNodes', thisAncestors[index - 1]);
    return siblings.indexOf(thisAncestors[index]) < siblings.indexOf(otherAncestors[index])
      ? Node.DOCUMENT_POSITION_FOLLOWING
      : Node.DOCUMENT_POSITION_PRECEDING;
  }
  __serialize() {
    return __callHost('serializeNode', this);
  }
  get textContent() {
    return __attributeNodes.has(this) ? this.value : __callHost('textContent', this);
  }
  set textContent(value) {
    value = value == null ? '' : `${value}`;
    if (__attributeNodes.has(this)) this.value = value;
    else if ([1, 3, 4, 7, 8, 11].includes(__callHost('nodeType', this)))
      __callHost('setTextContent', this, value);
  }
  get nodeValue() {
    if (__attributeNodes.has(this)) return this.value;
    return [3, 4, 7, 8].includes(__callHost('nodeType', this))
      ? __callHost('textContent', this)
      : null;
  }
  set nodeValue(value) {
    value = value == null ? '' : `${value}`;
    if (__attributeNodes.has(this)) this.value = value;
    else if ([3, 4, 7, 8].includes(__callHost('nodeType', this)))
      __callHost('setTextContent', this, value);
  }
  appendChild(child) {
    const incoming = __validateInsertion(this, child, null);
    const result = __callHost('appendChild', this, child);
    __finishNodeInsertion(incoming);
    return result;
  }
  removeChild(child) {
    if (__nodeArgument(child, arguments.length) === null) throw new TypeError('Expected a Node');
    if (__attributeNodes.has(child) || __callHost('parentNode', child) !== this)
      throw new DOMException('Node is not a child', 'NotFoundError');
    const result = __callHost('removeChild', this, child);
    __disconnectCustomElementTree(child);
    return result;
  }
  insertBefore(child, reference) {
    if (arguments.length < 2) throw new TypeError('Not enough arguments');
    const incoming = __validateInsertion(this, child, reference);
    const result = __callHost('insertBefore', this, child, reference);
    __finishNodeInsertion(incoming);
    return result;
  }
  replaceChild(node, child) {
    if (arguments.length < 2) throw new TypeError('Not enough arguments');
    const incoming = __validateInsertion(this, node, child, true);
    __callHost('replaceChild', this, node, child);
    __finishNodeInsertion(incoming, child);
    return child;
  }
  cloneNode(deep = false) {
    if (__shadowData.has(this))
      throw new DOMException('Cannot clone a shadow root', 'NotSupportedError');
    if (__attributeNodes.has(this)) {
      const value = this.value;
      return new Attr(
        __attrConstructionToken,
        null,
        { ...this.__record, value },
        this.ownerDocument,
      );
    }
    return __callHost('cloneNode', this, Boolean(deep));
  }
  normalize() {
    let previousText = null;
    for (const child of Array.from(this.childNodes)) {
      if (child.nodeType === Node.TEXT_NODE) {
        if (child.textContent === '') {
          this.removeChild(child);
        } else if (previousText) {
          const length = previousText.length,
            index = Array.prototype.indexOf.call(this.childNodes, child);
          previousText.appendData(child.data);
          __rangeBoundaries((r, n, o) => {
            if (r[n] === child) {
              r[n] = previousText;
              r[o] += length;
            } else if (r[n] === this && r[o] === index) {
              r[n] = previousText;
              r[o] = length;
            }
          });
          this.removeChild(child);
        } else {
          previousText = child;
        }
      } else {
        previousText = null;
        child.normalize();
      }
    }
  }
}
for (const [name, value] of Object.entries({
  ELEMENT_NODE: 1,
  ATTRIBUTE_NODE: 2,
  TEXT_NODE: 3,
  CDATA_SECTION_NODE: 4,
  ENTITY_REFERENCE_NODE: 5,
  ENTITY_NODE: 6,
  PROCESSING_INSTRUCTION_NODE: 7,
  COMMENT_NODE: 8,
  DOCUMENT_NODE: 9,
  DOCUMENT_TYPE_NODE: 10,
  DOCUMENT_FRAGMENT_NODE: 11,
  NOTATION_NODE: 12,
  DOCUMENT_POSITION_DISCONNECTED: 0x01,
  DOCUMENT_POSITION_PRECEDING: 0x02,
  DOCUMENT_POSITION_FOLLOWING: 0x04,
  DOCUMENT_POSITION_CONTAINS: 0x08,
  DOCUMENT_POSITION_CONTAINED_BY: 0x10,
  DOCUMENT_POSITION_IMPLEMENTATION_SPECIFIC: 0x20,
})) {
  Object.defineProperty(Node, name, { value, enumerable: true });
  Object.defineProperty(Node.prototype, name, { value, enumerable: true });
}

const NodeFilter = Object.freeze({
  FILTER_ACCEPT: 1,
  FILTER_REJECT: 2,
  FILTER_SKIP: 3,
  SHOW_ALL: 0xffffffff,
  SHOW_ELEMENT: 0x1,
  SHOW_ATTRIBUTE: 0x2,
  SHOW_TEXT: 0x4,
  SHOW_CDATA_SECTION: 0x8,
  SHOW_ENTITY_REFERENCE: 0x10,
  SHOW_ENTITY: 0x20,
  SHOW_PROCESSING_INSTRUCTION: 0x40,
  SHOW_COMMENT: 0x80,
  SHOW_DOCUMENT: 0x100,
  SHOW_DOCUMENT_TYPE: 0x200,
  SHOW_DOCUMENT_FRAGMENT: 0x400,
  SHOW_NOTATION: 0x800,
});

class TreeWalker {
  constructor(root, whatToShow = NodeFilter.SHOW_ALL, filter = null) {
    this.root = root;
    this.whatToShow = Number(whatToShow) >>> 0;
    this.filter = filter;
    this.currentNode = root;
  }
  __decision(node) {
    const shown = (this.whatToShow & (1 << (node.nodeType - 1))) !== 0;
    if (!shown) return NodeFilter.FILTER_SKIP;
    if (this.filter === null) return NodeFilter.FILTER_ACCEPT;
    if (this.__active) throw new DOMException('Recursive filtering', 'InvalidStateError');
    this.__active = true;
    try {
      return Number(
        typeof this.filter === 'function' ? this.filter(node) : this.filter.acceptNode(node),
      );
    } finally {
      this.__active = false;
    }
  }
  parentNode() {
    let node = this.currentNode;
    while (node !== this.root && (node = node.parentNode)) {
      if (this.__decision(node) === NodeFilter.FILTER_ACCEPT) return (this.currentNode = node);
    }
    return null;
  }
  __child(forward) {
    const first = forward ? 'firstChild' : 'lastChild',
      next = forward ? 'nextSibling' : 'previousSibling';
    const start = this.currentNode;
    let node = start[first];
    while (node) {
      const result = this.__decision(node);
      if (result === NodeFilter.FILTER_ACCEPT) return (this.currentNode = node);
      if (result === NodeFilter.FILTER_SKIP && node[first]) {
        node = node[first];
        continue;
      }
      while (!node[next]) {
        node = node.parentNode;
        if (!node || node === start || node === this.root) return null;
      }
      node = node[next];
    }
    return null;
  }
  firstChild() {
    return this.__child(true);
  }
  lastChild() {
    return this.__child(false);
  }
  __sibling(forward) {
    let node = this.currentNode;
    if (node === this.root) return null;
    const next = forward ? 'nextSibling' : 'previousSibling',
      first = forward ? 'firstChild' : 'lastChild';
    while (node) {
      let sibling = node[next];
      while (sibling) {
        node = sibling;
        const result = this.__decision(node);
        if (result === NodeFilter.FILTER_ACCEPT) return (this.currentNode = node);
        sibling = result === NodeFilter.FILTER_SKIP ? node[first] : null;
        if (!sibling) sibling = node[next];
      }
      node = node.parentNode;
      if (!node || node === this.root || this.__decision(node) === NodeFilter.FILTER_ACCEPT)
        return null;
    }
    return null;
  }
  nextSibling() {
    return this.__sibling(true);
  }
  previousSibling() {
    return this.__sibling(false);
  }
  previousNode() {
    let node = this.currentNode;
    while (node !== this.root) {
      while (node.previousSibling) {
        node = node.previousSibling;
        let result = this.__decision(node);
        while (result !== NodeFilter.FILTER_REJECT && node.lastChild) {
          node = node.lastChild;
          result = this.__decision(node);
        }
        if (result === NodeFilter.FILTER_ACCEPT) return (this.currentNode = node);
      }
      node = node.parentNode;
      if (!node) return null;
      if (this.__decision(node) === NodeFilter.FILTER_ACCEPT) return (this.currentNode = node);
    }
    return null;
  }
  nextNode() {
    let node = this.currentNode;
    let prune = false;
    while (true) {
      if (!prune && node.firstChild) {
        node = node.firstChild;
      } else {
        while (node && node !== this.root && !node.nextSibling) node = node.parentNode;
        if (!node || node === this.root) return null;
        node = node.nextSibling;
      }
      prune = false;
      const decision = this.__decision(node);
      if (decision === NodeFilter.FILTER_ACCEPT) {
        this.currentNode = node;
        return node;
      }
      prune = decision === NodeFilter.FILTER_REJECT;
    }
  }
}

class DOMImplementation {
  hasFeature() {
    return true;
  }
  createHTMLDocument(title) {
    const created = new DOMParser().parseFromString(
      '<!doctype html><html><head></head><body></body></html>',
      'text/html',
    );
    // Safari exposes about:blank as Document.URL here, but resolves element
    // URLs against an empty internal document URL until an absolute base exists.
    __callHost('documentSetURL', created, '');
    if (arguments.length > 0) {
      const titleElement = created.createElement('title');
      titleElement.appendChild(created.createTextNode(String(title)));
      created.head.appendChild(titleElement);
    }
    return created;
  }
}
const __domImplementation = new DOMImplementation();

const __validCustomElementLocalName =
  /^(?:[A-Za-z][^\0\t\n\f\r\u0020\/>]*|[:_\u0080-\u{10FFFF}][A-Za-z0-9-.:_\u0080-\u{10FFFF}]*)$/u;
const __reservedCustomElementNames = new Set([
  'annotation-xml',
  'color-profile',
  'font-face',
  'font-face-src',
  'font-face-uri',
  'font-face-format',
  'font-face-name',
  'missing-glyph',
]);
const __customElementConstructionStacks = new WeakMap();
const __customElementNames = new WeakMap();
const __formAssociatedDefinitions = new Set();
const __upgradedCustomElements = new WeakSet();
const __failedCustomElements = new WeakSet();
const __connectedCustomElements = new WeakSet();

function __isValidCustomElementName(name) {
  if (
    name.length === 0 ||
    name[0] < 'a' ||
    name[0] > 'z' ||
    !name.includes('-') ||
    __reservedCustomElementNames.has(name) ||
    !__validCustomElementLocalName.test(name)
  )
    return false;
  for (const character of name) {
    if (character >= 'A' && character <= 'Z') return false;
  }
  return true;
}

const __customElementRegistries = new WeakMap();
function __customElementRegistryState(registry) {
  const state = __customElementRegistries.get(registry);
  if (!state) throw new TypeError('Illegal invocation');
  return state;
}
class CustomElementRegistry {
  constructor() {
    __customElementRegistries.set(this, {
      definitions: new Map(),
      observedAttributes: new Map(),
      constructorNames: new Map(),
      waiters: new Map(),
    });
  }
  define(name, constructor) {
    name = String(name);
    if (!__isValidCustomElementName(name)) {
      throw new DOMException('Invalid custom element name', 'SyntaxError');
    }
    if (typeof constructor !== 'function') throw new TypeError('constructor must be callable');
    if (
      __customElementRegistryState(this).definitions.has(name) ||
      __customElementRegistryState(this).constructorNames.has(constructor)
    ) {
      throw new DOMException('Custom element definition already exists', 'NotSupportedError');
    }
    const observed = Array.from(constructor.observedAttributes ?? [], String);
    const formAssociated = Boolean(constructor.formAssociated);
    if (formAssociated) __formAssociatedDefinitions.add(name);
    __customElementRegistryState(this).observedAttributes.set(name, observed);
    __customElementRegistryState(this).definitions.set(name, constructor);
    __customElementRegistryState(this).constructorNames.set(constructor, name);
    __upgradeCustomElementTree(document);
    const waiters = __customElementRegistryState(this).waiters.get(name) ?? [];
    __customElementRegistryState(this).waiters.delete(name);
    for (const resolve of waiters) resolve(constructor);
  }
  get(name) {
    return __customElementRegistryState(this).definitions.get(String(name));
  }
  getName(constructor) {
    return __customElementRegistryState(this).constructorNames.get(constructor) ?? null;
  }
  whenDefined(name) {
    name = String(name);
    if (!__isValidCustomElementName(name)) {
      return Promise.reject(new DOMException('Invalid custom element name', 'SyntaxError'));
    }
    const constructor = __customElementRegistryState(this).definitions.get(name);
    if (constructor) return Promise.resolve(constructor);
    return new Promise((resolve) => {
      const waiters = __customElementRegistryState(this).waiters.get(name) ?? [];
      waiters.push(resolve);
      __customElementRegistryState(this).waiters.set(name, waiters);
    });
  }
  upgrade(root) {
    if (!(root instanceof Node)) throw new TypeError('root must be a Node');
    __upgradeCustomElementTree(root);
  }
}
const customElements = new CustomElementRegistry();

function __upgradeCustomElementTree(root) {
  for (const element of __customElementTree(root)) __upgradeCustomElement(element);
}

function __upgradeCustomElement(element, synchronous = false) {
  if (__upgradedCustomElements.has(element) || __failedCustomElements.has(element)) return element;
  if (
    __callHost('nodeType', element) !== 1 ||
    __callHost('namespaceURI', element) !== 'http://www.w3.org/1999/xhtml'
  )
    return element;
  const owner = __callHost('ownerDocument', element);
  const name = __callHost('localName', element);
  const constructor = __customElementRegistryState(customElements).definitions.get(name);
  if (constructor === undefined) return element;
  __customElementNames.set(element, name);
  __upgradedCustomElements.add(element);
  let stack = __customElementConstructionStacks.get(constructor);
  if (!stack) __customElementConstructionStacks.set(constructor, (stack = []));
  const entry = { element, constructed: false };
  stack.push(entry);
  try {
    const result = Reflect.construct(constructor, []);
    if (result !== element)
      throw new TypeError('custom element constructor returned another object');
    if (
      synchronous &&
      (__attributeRecords(element).length ||
        __callHost('childNodes', element).length ||
        __callHost('parentNode', element) ||
        __callHost('ownerDocument', element) !== owner)
    )
      throw new DOMException(
        'Custom element constructor changed its initial tree state',
        'NotSupportedError',
      );
    const observed =
      __customElementRegistryState(customElements).observedAttributes.get(name) ?? [];
    if (typeof element.attributeChangedCallback === 'function') {
      for (const name of observed) {
        if (element.hasAttribute(name)) {
          element.attributeChangedCallback(name, null, element.getAttribute(name));
        }
      }
    }
    if (element.isConnected && typeof element.connectedCallback === 'function') {
      __connectedCustomElements.add(element);
      element.connectedCallback();
    }
  } catch (error) {
    __failedCustomElements.add(element);
    __upgradedCustomElements.delete(element);
    console.error(
      `Custom element upgrade failed for <${element.localName}> (${constructor.name || 'anonymous'})`,
      error,
    );
  } finally {
    stack.pop();
  }
  if (synchronous && __failedCustomElements.has(element)) {
    const failed = __callHost('createElement', owner, name);
    Object.setPrototypeOf(failed, HTMLUnknownElement.prototype);
    __failedCustomElements.add(failed);
    return failed;
  }
  return element;
}

function __customElementTree(root) {
  const nodes = [root];
  for (const child of root.childNodes ?? []) nodes.push(...__customElementTree(child));
  const shadow = __shadowRoots.get(root);
  if (shadow) nodes.push(...__customElementTree(shadow));
  return nodes;
}

function __connectCustomElementTree(root) {
  for (const element of __customElementTree(root)) {
    __upgradeCustomElement(element);
    if (
      element.isConnected &&
      __upgradedCustomElements.has(element) &&
      !__connectedCustomElements.has(element)
    ) {
      __connectedCustomElements.add(element);
      if (typeof element.connectedCallback === 'function') element.connectedCallback();
    }
  }
}

function __disconnectCustomElementTree(root) {
  for (const element of __customElementTree(root)) {
    if (!__connectedCustomElements.delete(element)) continue;
    if (typeof element.disconnectedCallback === 'function') element.disconnectedCallback();
  }
}

function __finishNodeInsertion(incoming, removed = null) {
  // Fragments have already been drained. Keep the original incoming roots so
  // their connection work still runs, including disconnect/connect on moves.
  for (const node of incoming) __disconnectCustomElementTree(node);
  if (removed) __disconnectCustomElementTree(removed);
  for (const node of incoming) {
    if (node instanceof HTMLIFrameElement) node.__connected();
    __connectCustomElementTree(node);
    __queueDynamicScript(node);
  }
}

function __nodesFromArguments(values) {
  return values.map((value) =>
    value instanceof Node ? value : document.createTextNode(String(value)),
  );
}

function __appendNodes(parent, values) {
  for (const node of __nodesFromArguments(values)) parent.appendChild(node);
}

function __prependNodes(parent, values) {
  const reference = parent.firstChild;
  for (const node of __nodesFromArguments(values)) parent.insertBefore(node, reference);
}

function __replaceChildren(parent, values) {
  const nodes = __nodesFromArguments(values);
  while (parent.firstChild) parent.removeChild(parent.firstChild);
  for (const node of nodes) parent.appendChild(node);
}

const __collectionCache = new WeakMap();
function __elementChildren(parent) {
  return __cachedCollection(parent, 'children', () =>
    __createHTMLCollection(() => __callHost('elementChildren', parent)),
  );
}
function __cachedCollection(owner, key, create) {
  let cache = __collectionCache.get(owner);
  if (!cache) __collectionCache.set(owner, (cache = new Map()));
  if (!cache.has(key)) cache.set(key, create());
  return cache.get(key);
}
function __validateInsertion(parent, node, reference, replace = false) {
  if (__nodeArgument(node, 1) === null) throw new TypeError('Expected a Node');
  __nodeArgument(reference, 1);
  if (replace && reference == null) throw new TypeError('Expected a Node');
  const parentType = __attributeNodes.has(parent) ? 2 : __callHost('nodeType', parent);
  const nodeType = __attributeNodes.has(node) ? 2 : __callHost('nodeType', node);
  if (![1, 9, 11].includes(parentType) || ![1, 3, 4, 7, 8, 10, 11].includes(nodeType))
    throw new DOMException('Invalid tree insertion', 'HierarchyRequestError');
  for (
    let ancestor = parent;
    ancestor;
    ancestor = __callHost('parentNode', ancestor) ?? __shadowData.get(ancestor)?.host
  ) {
    if (ancestor === node)
      throw new DOMException('Invalid tree insertion', 'HierarchyRequestError');
  }
  if (
    reference != null &&
    (__attributeNodes.has(reference) || __callHost('parentNode', reference) !== parent)
  )
    throw new DOMException('Node is not a child', 'NotFoundError');
  const incoming = nodeType === 11 ? __callHost('childNodes', node) : [node];
  if (parentType !== 9 && incoming.some((n) => __callHost('nodeType', n) === 10))
    throw new DOMException('DocumentType requires a document', 'HierarchyRequestError');
  if (parentType === 9) {
    const children = __callHost('childNodes', parent).filter((n) => n !== node);
    let index = reference == null ? children.length : children.indexOf(reference);
    if (reference === node) index = __callHost('childNodes', parent).indexOf(node);
    children.splice(index, replace && reference !== node ? 1 : 0, ...incoming);
    const types = children.map((n) => __callHost('nodeType', n));
    if (
      types.some((type) => type === 3 || type === 4) ||
      types.filter((type) => type === 1).length > 1 ||
      types.filter((type) => type === 10).length > 1 ||
      (types.indexOf(10) > types.indexOf(1) && types.includes(1))
    )
      throw new DOMException('Invalid document children', 'HierarchyRequestError');
  }
  return incoming;
}
