function __insertAdjacentNode(element, position, node) {
  if (position === 'beforebegin') {
    if (!element.parentNode) return null;
    element.parentNode.insertBefore(node, element);
  } else if (position === 'afterbegin') {
    element.insertBefore(node, element.firstChild);
  } else if (position === 'beforeend') {
    element.appendChild(node);
  } else if (position === 'afterend') {
    if (!element.parentNode) return null;
    element.parentNode.insertBefore(node, element.nextSibling);
  } else {
    throw new DOMException('Invalid position', 'SyntaxError');
  }
  return node;
}

const __datasetConstructorToken = {};
const __datasetMaps = new WeakMap();
class DOMStringMap {
  constructor(token, element) {
    if (token !== __datasetConstructorToken) throw new TypeError('Illegal constructor');
    Object.defineProperty(this, '__element', { value: element });
  }
}

function __datasetPropertyFromAttribute(name) {
  return name.slice(5).replace(/-([a-z])/g, (_, letter) => letter.toUpperCase());
}

function __datasetAttributeFromProperty(property) {
  if (/-[a-z]/.test(property)) {
    throw new DOMException(
      'dataset property contains a hyphen followed by a lowercase letter',
      'SyntaxError',
    );
  }
  return 'data-' + property.replace(/[A-Z]/g, (letter) => '-' + letter.toLowerCase());
}

function __datasetFor(element) {
  let dataset = __datasetMaps.get(element);
  if (dataset) return dataset;
  const target = new DOMStringMap(__datasetConstructorToken, element);
  dataset = new Proxy(target, {
    get(target, property, receiver) {
      if (typeof property !== 'string' || Reflect.has(target, property)) {
        return Reflect.get(target, property, receiver);
      }
      const value = element.getAttribute(__datasetAttributeFromProperty(property));
      return value === null ? undefined : value;
    },
    set(target, property, value, receiver) {
      if (typeof property !== 'string' || Reflect.has(target, property)) {
        return Reflect.set(target, property, value, receiver);
      }
      element.setAttribute(__datasetAttributeFromProperty(property), String(value));
      return true;
    },
    deleteProperty(target, property) {
      if (typeof property !== 'string' || Reflect.has(target, property)) {
        return Reflect.deleteProperty(target, property);
      }
      element.removeAttribute(__datasetAttributeFromProperty(property));
      return true;
    },
    has(target, property) {
      if (typeof property !== 'string' || Reflect.has(target, property)) return true;
      return element.hasAttribute(__datasetAttributeFromProperty(property));
    },
    ownKeys(target) {
      const keys = element
        .getAttributeNames()
        .filter((name) => name.startsWith('data-') && name.length > 5)
        .map(__datasetPropertyFromAttribute);
      return [...Reflect.ownKeys(target), ...new Set(keys)];
    },
    getOwnPropertyDescriptor(target, property) {
      const own = Reflect.getOwnPropertyDescriptor(target, property);
      if (own || typeof property !== 'string') return own;
      const value = element.getAttribute(__datasetAttributeFromProperty(property));
      return value === null
        ? undefined
        : {
            value,
            writable: true,
            enumerable: true,
            configurable: true,
          };
    },
    defineProperty(target, property, descriptor) {
      if (typeof property !== 'string' || Reflect.has(target, property)) {
        return Reflect.defineProperty(target, property, descriptor);
      }
      if (!('value' in descriptor)) return false;
      element.setAttribute(__datasetAttributeFromProperty(property), String(descriptor.value));
      return true;
    },
  });
  __datasetMaps.set(element, dataset);
  return dataset;
}

const __attributeReactions = new WeakMap();
const __clickInProgress = new WeakSet();
function __invokeAttributeReactions(element) {
  const pending = __attributeReactions.get(element);
  if (!pending) return;
  while (pending.callbacks.length) {
    const reaction = pending.callbacks.shift();
    try {
      reaction();
    } catch (error) {
      setTimeout(() => {
        throw error;
      }, 0);
    }
  }
}
function __notifyAttributeChange(
  element,
  name,
  oldValue,
  value,
  deferred = false,
  namespace = null,
) {
  if (
    element instanceof HTMLScriptElement &&
    namespace === null &&
    name === 'async' &&
    oldValue === null &&
    value !== null
  ) {
    __forceAsyncScripts.delete(element);
  }
  const observed = __customElementRegistryState(customElements).observedAttributes.get(
    __customElementNames.get(element) ?? __callHost('localName', element),
  );
  if (
    __upgradedCustomElements.has(element) &&
    Array.isArray(observed) &&
    observed.map(String).includes(name) &&
    typeof element.attributeChangedCallback === 'function'
  ) {
    let pending = __attributeReactions.get(element);
    if (!pending)
      __attributeReactions.set(element, (pending = { callbacks: [], scheduled: false }));
    const callback = element.attributeChangedCallback;
    pending.callbacks.push(() => callback.call(element, name, oldValue, value, namespace));
    if (!deferred) __invokeAttributeReactions(element);
    else if (!pending.scheduled) {
      pending.scheduled = true;
      queueMicrotask(() => {
        pending.scheduled = false;
        __invokeAttributeReactions(element);
      });
    }
  }
}

// Reflected IDL properties use only null-namespace content attributes. Use the canonical
// attribute store rather than overridable public get/setAttribute methods.
function __getContentAttribute(element, attribute) {
  return (
    __attributeRecords(element).find((a) => a.namespaceURI === null && a.localName === attribute)
      ?.value ?? null
  );
}
function __setContentAttribute(element, attribute, value, deferred = false) {
  value = value == null ? null : `${value}`;
  const oldValue = __getContentAttribute(element, attribute);
  if (value === null) {
    if (oldValue === null) return;
    __detachAttribute(element, null, attribute);
    __callHost('removeAttributeNS', element, '', attribute);
  } else {
    __callHost('setAttributeNS', element, '', attribute, value);
  }
  __notifyAttributeChange(element, attribute, oldValue, value, deferred);
}

function __selectorString(value, argumentCount) {
  if (!argumentCount) throw new TypeError('A selector is required');
  return `${value}`;
}
function __selectorCall(operation, element, selector) {
  try {
    return __callHost(operation, element, selector);
  } catch (error) {
    // The native Lexbor boundary reports parser failures as strings.
    if (error === 'invalid CSS selector')
      throw new DOMException('Invalid CSS selector', 'SyntaxError');
    throw error;
  }
}

class Element extends Node {
  get tagName() {
    return __callHost('tagName', this);
  }
  get localName() {
    return __callHost('localName', this);
  }
  get namespaceURI() {
    return __callHost('namespaceURI', this);
  }
  get prefix() {
    return __callHost('prefix', this);
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
  get previousElementSibling() {
    return __callHost('previousElementSibling', this);
  }
  get nextElementSibling() {
    return __callHost('nextElementSibling', this);
  }
  get attributes() {
    let attributes = __attributeMaps.get(this);
    if (!attributes) {
      attributes = new NamedNodeMap(this);
      __attributeMaps.set(this, attributes);
    }
    return attributes;
  }
  get dataset() {
    return __datasetFor(this);
  }
  get id() {
    return __callHost('getAttributeOrEmpty', this, 'id');
  }
  set id(value) {
    __callHost('setAttribute', this, 'id', value);
  }
  get className() {
    return __callHost('getAttributeOrEmpty', this, 'class');
  }
  set className(value) {
    __callHost('setAttribute', this, 'class', value);
  }
  get classList() {
    let list = __classLists.get(this);
    if (!list) {
      list = __createTokenList(this);
      __classLists.set(this, list);
    }
    return list;
  }
  set classList(value) {
    this.classList.value = value;
  }
  get innerHTML() {
    return __callHost('innerHTML', this);
  }
  set innerHTML(value) {
    __callHost('setInnerHTML', this, value);
  }
  get outerHTML() {
    return __callHost('outerHTML', this);
  }
  get style() {
    const target = __callHost('style', this);
    __styleOwners.set(target, this);
    return __styleDeclarationProxy(target);
  }
  set style(value) {
    this.style.cssText = value;
  }
  getAttribute(name) {
    return __callHost('getAttribute', this, name);
  }
  setAttribute(name, value) {
    name = String(name);
    value = `${value}`;
    if (name === '')
      throw new DOMException('attribute name cannot be empty', 'InvalidCharacterError');
    const oldValue = __callHost('getAttribute', this, name);
    __callHost('setAttribute', this, name, value);
    __notifyAttributeChange(this, name, oldValue, value);
  }
  removeAttribute(name) {
    name = String(name);
    const oldValue = __callHost('getAttribute', this, name);
    const attr = this.getAttributeNode(name);
    if (attr) __detachAttribute(this, attr.namespaceURI, attr.localName);
    __callHost('removeAttribute', this, name);
    if (oldValue !== null) __notifyAttributeChange(this, name, oldValue, null);
  }
  getAttributeNode(name) {
    return this.attributes.getNamedItem(name);
  }
  removeAttributeNode(attr) {
    if (!(attr instanceof Attr)) throw new TypeError('Expected an Attr');
    if (attr.ownerElement !== this)
      throw new DOMException('Attribute does not belong to this element', 'NotFoundError');
    this.removeAttributeNS(attr.namespaceURI, attr.localName);
    return attr;
  }
  hasAttributes() {
    return this.attributes.length !== 0;
  }
  hasAttribute(name) {
    return this.getAttribute(name) !== null;
  }
  getAttributeNames() {
    return __attributeRecords(this).map((attribute) => attribute.name);
  }
  toggleAttribute(name, force = undefined) {
    name = String(name);
    if (name === '')
      throw new DOMException('attribute name cannot be empty', 'InvalidCharacterError');
    const present = this.hasAttribute(name);
    if (present && (arguments.length < 2 || !Boolean(force))) {
      this.removeAttribute(name);
      return false;
    }
    if (!present && (arguments.length < 2 || Boolean(force))) this.setAttribute(name, '');
    return true;
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
  insertAdjacentElement(position, element) {
    if (!(element instanceof Element)) throw new TypeError('element must be an Element');
    position = String(position).toLowerCase();
    return __insertAdjacentNode(this, position, element);
  }
  insertAdjacentText(position, data) {
    __insertAdjacentNode(
      this,
      String(position).toLowerCase(),
      document.createTextNode(String(data)),
    );
  }
  insertAdjacentHTML(position, text) {
    position = String(position).toLowerCase();
    const outside = position === 'beforebegin' || position === 'afterend';
    if (outside && !this.parentNode)
      throw new DOMException('The element has no parent', 'NoModificationAllowedError');
    if (!['beforebegin', 'afterbegin', 'beforeend', 'afterend'].includes(position)) {
      throw new DOMException('Invalid position', 'SyntaxError');
    }
    const container = document.createElement(
      outside && this.parentElement ? this.parentElement.localName : this.localName,
    );
    container.innerHTML = String(text);
    const nodes = Array.from(container.childNodes);
    if (position === 'afterbegin' || position === 'afterend') nodes.reverse();
    for (const node of nodes) __insertAdjacentNode(this, position, node);
  }
  getElementsByTagName(name) {
    return __createHTMLCollection(() => __callHost('getElementsByTagName', this, name));
  }
  getElementsByClassName(names) {
    return __createHTMLCollection(() => __callHost('getElementsByClassName', this, names));
  }
  querySelector(selector) {
    return __selectorCall('querySelector', this, __selectorString(selector, arguments.length));
  }
  querySelectorAll(selector) {
    return __createNodeList(
      __selectorCall('querySelectorAll', this, __selectorString(selector, arguments.length)),
    );
  }
  matches(selector) {
    return __selectorCall('matches', this, __selectorString(selector, arguments.length));
  }
  webkitMatchesSelector(selector) {
    return __selectorCall('matches', this, __selectorString(selector, arguments.length));
  }
  closest(selector) {
    selector = __selectorString(selector, arguments.length);
    let element = this;
    while (element) {
      if (__selectorCall('matches', element, selector)) return element;
      const parent = __callHost('parentNode', element);
      element = parent !== null && __callHost('nodeType', parent) === 1 ? parent : null;
    }
    return null;
  }
  focus() {
    if (
      this instanceof HTMLInputElement &&
      __inputTemporalType(__inputType(this)) &&
      __controlConnected(this)
    )
      __inputTemporalEditors.add(this);
    const previous = __activeElement;
    if (previous === this) return;
    if (previous) {
      previous.dispatchEvent(new Event('blur'));
      previous.dispatchEvent(new Event('focusout', { bubbles: true }));
    }
    __activeElement = this;
    this.dispatchEvent(new Event('focus'));
    this.dispatchEvent(new Event('focusin', { bubbles: true }));
  }
  blur() {
    if (__activeElement !== this) return;
    __activeElement = null;
    this.dispatchEvent(new Event('blur'));
    this.dispatchEvent(new Event('focusout', { bubbles: true }));
  }
  click() {
    if (__clickInProgress.has(this)) return;
    if (
      [HTMLInputElement, HTMLButtonElement, HTMLSelectElement, HTMLTextAreaElement].some(
        (type) => this instanceof type,
      ) &&
      __isDisabledFormControl(this)
    )
      return;
    __clickInProgress.add(this);
    try {
      Reflect.apply(__dispatchEvent, this, [
        new PointerEvent('click', {
          bubbles: true,
          cancelable: true,
          composed: true,
          view: window,
        }),
      ]);
    } finally {
      __clickInProgress.delete(this);
    }
  }
}

function __reflectedString(element, name) {
  return __getContentAttribute(element, name) ?? '';
}

function __setReflectedBoolean(element, name, value) {
  if (Boolean(value)) element.setAttribute(name, '');
  else element.removeAttribute(name);
}

const __svgClassNames = new WeakMap();
class SVGElement extends Element {
  get className() {
    let value = __svgClassNames.get(this);
    if (!value) {
      const element = this;
      value = {
        get baseVal() {
          return __reflectedString(element, 'class');
        },
        set baseVal(value) {
          __setContentAttribute(element, 'class', String(value));
        },
        get animVal() {
          return __reflectedString(element, 'class');
        },
        [Symbol.toStringTag]: 'SVGAnimatedString',
      };
      __svgClassNames.set(this, value);
    }
    return value;
  }
  get ownerSVGElement() {
    for (let parent = this.parentElement; parent; parent = parent.parentElement) {
      if (parent.namespaceURI === 'http://www.w3.org/2000/svg' && parent.localName === 'svg')
        return parent;
    }
    return null;
  }
  get viewportElement() {
    return this.ownerSVGElement;
  }
}
class SVGGraphicsElement extends SVGElement {
  // Legacy placeholders: no SVG geometry, transforms or rendering backend.
  get requiredExtensions() {
    return '';
  }
  get systemLanguage() {
    return '';
  }
  get transform() {
    return '';
  }
  getBBox() {}
  getCTM() {}
  getScreenCTM() {}
}

class HTMLElement extends Element {
  constructor() {
    super(__nodeConstructionToken);
    if (new.target === HTMLElement) throw new TypeError('Illegal constructor');
    const name = __customElementRegistryState(customElements).constructorNames.get(new.target);
    if (name === undefined) throw new TypeError('Illegal constructor');
    const stack = __customElementConstructionStacks.get(new.target);
    const entry = stack?.[stack.length - 1];
    if (entry) {
      if (entry.constructed) throw new TypeError('Custom element already constructed');
      entry.constructed = true;
      const prototype = new.target.prototype;
      Object.setPrototypeOf(
        entry.element,
        (typeof prototype === 'object' && prototype !== null) || typeof prototype === 'function'
          ? prototype
          : HTMLElement.prototype,
      );
      return entry.element;
    }
    const element = __callHost('createElement', document, name);
    __customElementNames.set(element, name);
    const prototype = new.target.prototype;
    Object.setPrototypeOf(
      element,
      (typeof prototype === 'object' && prototype !== null) || typeof prototype === 'function'
        ? prototype
        : HTMLElement.prototype,
    );
    __upgradedCustomElements.add(element);
    return element;
  }

  get innerText() {
    return this.textContent;
  }
  set innerText(value) {
    this.textContent = String(value);
  }
  get outerText() {
    return this.innerText;
  }
  set outerText(value) {
    value = String(value);
    if (!this.parentNode)
      throw new DOMException('The element has no parent', 'NoModificationAllowedError');
    const nodes = [];
    for (const [index, text] of value.split(/\r\n|\r|\n/).entries()) {
      if (index) nodes.push(this.ownerDocument.createElement('br'));
      nodes.push(this.ownerDocument.createTextNode(text));
    }
    __replaceNode(this, nodes);
  }
  get isContentEditable() {
    for (let element = this; element; element = element.parentElement) {
      const value = __getContentAttribute(element, 'contenteditable');
      if (value === null) continue;
      const keyword = __asciiLowercase(value);
      if (keyword === '' || keyword === 'true' || keyword === 'plaintext-only') return true;
      if (keyword === 'false') return false;
    }
    return this.ownerDocument?.designMode === 'on';
  }
  // Legacy compatibility placeholders: Brimp does not perform box layout.
  get offsetHeight() {
    return 0;
  }
  get offsetLeft() {
    return 0;
  }
  get offsetParent() {
    return null;
  }
  get offsetTop() {
    return 0;
  }
  get offsetWidth() {
    return 0;
  }
  get title() {
    return __reflectedString(this, 'title');
  }
  set title(value) {
    __setContentAttribute(this, 'title', `${value}`);
  }
  get lang() {
    return __reflectedString(this, 'lang');
  }
  set lang(value) {
    __setContentAttribute(this, 'lang', `${value}`);
  }
  get dir() {
    const value = __asciiLowercase(__reflectedString(this, 'dir'));
    return value === 'ltr' || value === 'rtl' || value === 'auto' ? value : '';
  }
  set dir(value) {
    __setContentAttribute(this, 'dir', `${value}`);
  }
  get autofocus() {
    return __getContentAttribute(this, 'autofocus') !== null;
  }
  // Safari delivers this IDL setter's custom-element reactions at a microtask.
  set autofocus(value) {
    __setContentAttribute(this, 'autofocus', value ? '' : null, true);
  }
  get hidden() {
    const value = __getContentAttribute(this, 'hidden');
    return value !== null && (__asciiLowercase(value) === 'until-found' ? 'until-found' : true);
  }
  set hidden(value) {
    if (value != null && typeof value !== 'boolean' && typeof value !== 'number')
      value = `${value}`;
    __setContentAttribute(
      this,
      'hidden',
      typeof value === 'string' && __asciiLowercase(value) === 'until-found'
        ? 'until-found'
        : value
          ? ''
          : null,
    );
  }
  get accessKey() {
    return __reflectedString(this, 'accesskey');
  }
  set accessKey(value) {
    __setContentAttribute(this, 'accesskey', `${value}`);
  }
  get contentEditable() {
    const value = __getContentAttribute(this, 'contenteditable');
    if (value === null) return 'inherit';
    const keyword = __asciiLowercase(value);
    if (keyword === '') return 'true';
    return ['true', 'false', 'plaintext-only'].includes(keyword) ? keyword : 'inherit';
  }
  set contentEditable(value) {
    value = __asciiLowercase(`${value}`);
    if (!['true', 'false', 'plaintext-only', 'inherit'].includes(value))
      throw new DOMException('Invalid contentEditable value', 'SyntaxError');
    __setContentAttribute(this, 'contenteditable', value === 'inherit' ? null : value);
  }
  get draggable() {
    const value = __asciiLowercase(__getContentAttribute(this, 'draggable') ?? '');
    if (value === 'true' || value === 'false') return value === 'true';
    // Safari includes area hyperlinks as well as anchors in the auto state.
    return (
      this.localName === 'img' ||
      (['a', 'area'].includes(this.localName) && __getContentAttribute(this, 'href') !== null)
    );
  }
  set draggable(value) {
    __setContentAttribute(this, 'draggable', value ? 'true' : 'false');
  }
  get spellcheck() {
    for (
      let element = this;
      element;
      element =
        element.parentElement ??
        (element.parentNode instanceof ShadowRoot ? element.parentNode.host : null)
    ) {
      const value = __getContentAttribute(element, 'spellcheck');
      if (value === null) continue;
      const keyword = __asciiLowercase(value);
      if (keyword === '' || keyword === 'true') return true;
      if (keyword === 'false') return false;
    }
    return true;
  }
  set spellcheck(value) {
    __setContentAttribute(this, 'spellcheck', value ? 'true' : 'false');
  }
  get translate() {
    for (let element = this; element; element = element.parentElement) {
      if (!(element instanceof HTMLElement)) continue;
      const value = __getContentAttribute(element, 'translate');
      if (value === null) continue;
      const keyword = __asciiLowercase(value);
      if (keyword === '' || keyword === 'yes') return true;
      if (keyword === 'no') return false;
    }
    return true;
  }
  set translate(value) {
    __setContentAttribute(this, 'translate', value ? 'yes' : 'no');
  }
  get tabIndex() {
    const value = this.getAttribute('tabindex');
    if (value === null) return -1;
    const match = /^[\t\n\f\r ]*([+-]?\d+)/.exec(value);
    if (!match) return 0;
    const parsed = Number(match[1]);
    if (parsed === 0) return 0;
    return parsed >= -2147483648 && parsed <= 2147483647 ? parsed : 0;
  }
  set tabIndex(value) {
    let number = Number(value);
    if (!Number.isFinite(number) || number === 0) number = 0;
    else number = Math.trunc(number);
    number = ((number % 4294967296) + 4294967296) % 4294967296;
    if (number >= 2147483648) number -= 4294967296;
    this.setAttribute('tabindex', String(number));
  }
}
class HTMLUnknownElement extends HTMLElement {
  constructor() {
    throw new TypeError('Illegal constructor');
  }
}

class HTMLAnchorElement extends HTMLElement {
  constructor() {
    throw new TypeError('Illegal constructor');
  }
  get text() {
    return __callHost('textContent', this);
  }
  set text(value) {
    __callHost('setTextContent', this, `${value}`);
  }
  get href() {
    return __getContentAttribute(this, 'href') !== null ? __anchorHref(this) : '';
  }
  set href(value) {
    __setContentAttribute(this, 'href', __toUSVString(`${value}`));
  }
  get origin() {
    return __anchorURLComponent(this, 'origin');
  }
  get protocol() {
    return __anchorURLComponent(this, 'protocol');
  }
  set protocol(value) {
    __setAnchorURLComponent(this, 'protocol', value);
  }
  get username() {
    return __anchorURLComponent(this, 'username');
  }
  set username(value) {
    __setAnchorURLComponent(this, 'username', value);
  }
  get password() {
    return __anchorURLComponent(this, 'password');
  }
  set password(value) {
    __setAnchorURLComponent(this, 'password', value);
  }
  get host() {
    return __anchorURLComponent(this, 'host');
  }
  set host(value) {
    __setAnchorURLComponent(this, 'host', value);
  }
  get hostname() {
    return __anchorURLComponent(this, 'hostname');
  }
  set hostname(value) {
    __setAnchorURLComponent(this, 'hostname', value);
  }
  get port() {
    return __anchorURLComponent(this, 'port');
  }
  set port(value) {
    __setAnchorURLComponent(this, 'port', value);
  }
  get pathname() {
    return __anchorURLComponent(this, 'pathname');
  }
  set pathname(value) {
    __setAnchorURLComponent(this, 'pathname', value);
  }
  get search() {
    return __anchorURLComponent(this, 'search');
  }
  set search(value) {
    __setAnchorURLComponent(this, 'search', value);
  }
  get hash() {
    return __anchorURLComponent(this, 'hash');
  }
  set hash(value) {
    __setAnchorURLComponent(this, 'hash', value);
  }
  toString() {
    return __getContentAttribute(this, 'href') !== null ? __anchorHref(this) : '';
  }
}

function __anchorURLComponent(anchor, component) {
  const href = __getContentAttribute(anchor, 'href');
  if (href !== null) {
    try {
      const record = __urlRecord(__anchorHref(anchor));
      // Safari anchor origins use SecurityOrigin, including file URLs.
      if (
        component === 'origin' &&
        (record.protocol === 'file:' ||
          (record.protocol === 'blob:' && /^file:/i.test(record.pathname)))
      )
        return 'file://';
      return record[component];
    } catch (_) {}
  }
  return component === 'protocol' ? ':' : '';
}

function __setAnchorURLComponent(anchor, component, value) {
  value = __toUSVString(`${value}`);
  if (__getContentAttribute(anchor, 'href') === null) return;
  const href = __anchorHref(anchor);
  try {
    __urlRecord(href);
  } catch (_) {
    // Safari rewrites the unchanged invalid URL for these three setters.
    if (component === 'pathname' || component === 'search' || component === 'hash')
      __setContentAttribute(anchor, 'href', href);
    return;
  }
  const next = __callHost('urlSetAnchor', window, href, component, value);
  if (next !== null) __setContentAttribute(anchor, 'href', next);
}

function __anchorHref(anchor) {
  return __elementURL(anchor, 'href');
}
