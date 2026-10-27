const __shadowRoots = new WeakMap();
const __shadowData = new WeakMap();
const __slotAssignments = new WeakMap();
const __shadowRegistrations = new Set();
const __pendingSlotChanges = new Set();
class ShadowRoot extends DocumentFragment {
  constructor() {
    throw new TypeError('Illegal constructor');
  }
  get host() {
    return __shadowData.get(this).host;
  }
  get mode() {
    return __shadowData.get(this).mode;
  }
  get delegatesFocus() {
    return __shadowData.get(this).delegatesFocus;
  }
  get slotAssignment() {
    return 'named';
  }
  get innerHTML() {
    return [...this.childNodes].map((node) => node.__serialize()).join('');
  }
  set innerHTML(value) {
    const template = this.ownerDocument.createElement('template');
    template.innerHTML = String(value);
    this.replaceChildren(template.content);
  }
  get activeElement() {
    const active = __activeElement;
    return active && active.getRootNode() === this ? active : null;
  }
  get [Symbol.toStringTag]() {
    return 'ShadowRoot';
  }
}
Object.defineProperties(Element.prototype, {
  attachShadow: {
    configurable: true,
    writable: true,
    value(options) {
      const mode = options?.mode;
      if (mode !== 'open' && mode !== 'closed')
        throw new TypeError('Shadow mode must be open or closed');
      if (options.slotAssignment !== undefined && options.slotAssignment !== 'named')
        throw new DOMException('Manual slot assignment is not supported', 'NotSupportedError');
      const allowed = [
        'article',
        'aside',
        'blockquote',
        'body',
        'div',
        'footer',
        'h1',
        'h2',
        'h3',
        'h4',
        'h5',
        'h6',
        'header',
        'main',
        'nav',
        'p',
        'section',
        'span',
      ];
      if (
        this.namespaceURI !== 'http://www.w3.org/1999/xhtml' ||
        (!allowed.includes(this.localName) && !__isValidCustomElementName(this.localName)) ||
        __shadowRoots.has(this)
      )
        throw new DOMException('This element cannot host a shadow root', 'NotSupportedError');
      const root = this.ownerDocument.createDocumentFragment();
      Object.setPrototypeOf(root, ShadowRoot.prototype);
      __shadowRoots.set(this, root);
      __shadowData.set(root, {
        host: this,
        mode,
        delegatesFocus: Boolean(options.delegatesFocus),
      });
      __shadowRegistrations.add(new WeakRef(root));
      return root;
    },
  },
  shadowRoot: {
    configurable: true,
    get() {
      const root = __shadowRoots.get(this);
      return root?.mode === 'open' ? root : null;
    },
  },
  slot: {
    configurable: true,
    get() {
      return this.getAttribute('slot') ?? '';
    },
    set(value) {
      this.setAttribute('slot', String(value));
    },
  },
});
function __assignedSlot(node, openOnly = false) {
  const root = __shadowRoots.get(node.parentNode);
  if (!root || (openOnly && root.mode === 'closed') || (node.nodeType !== 1 && node.nodeType !== 3))
    return null;
  const name = node.nodeType === 1 ? (node.getAttribute('slot') ?? '') : '';
  return [...root.querySelectorAll('slot')].find((slot) => slot.name === name) ?? null;
}
for (const prototype of [Element.prototype, Text.prototype]) {
  Object.defineProperty(prototype, 'assignedSlot', {
    configurable: true,
    get() {
      return __assignedSlot(this, true);
    },
  });
}
Object.defineProperty(HTMLSlotElement.prototype, 'name', {
  configurable: true,
  get() {
    return this.getAttribute('name') ?? '';
  },
  set(value) {
    this.setAttribute('name', String(value));
  },
});
HTMLSlotElement.prototype.assignedNodes = function (options = {}) {
  const root = this.getRootNode();
  let nodes =
    root instanceof ShadowRoot
      ? [...root.host.childNodes].filter((node) => __assignedSlot(node) === this)
      : [];
  if (options.flatten) {
    if (!nodes.length)
      nodes = [...this.childNodes].filter((n) => n.nodeType === 1 || n.nodeType === 3);
    nodes = nodes.flatMap((n) =>
      n instanceof HTMLSlotElement && n.getRootNode() instanceof ShadowRoot
        ? n.assignedNodes({ flatten: true })
        : [n],
    );
  }
  return nodes;
};
HTMLSlotElement.prototype.assignedElements = function (options) {
  return this.assignedNodes(options).filter((n) => n instanceof Element);
};
function __updateSlots() {
  for (const ref of __shadowRegistrations) {
    const root = ref.deref();
    if (!root) {
      __shadowRegistrations.delete(ref);
      continue;
    }
    const data = __shadowData.get(root),
      slots = [...root.querySelectorAll('slot')];
    const changed = new Set([...(data.slots ?? []), ...slots]);
    data.slots = slots;
    for (const slot of changed) {
      const previous = __slotAssignments.get(slot) ?? [],
        next = slot.assignedNodes();
      __slotAssignments.set(slot, next);
      if (previous.length === next.length && previous.every((node, i) => node === next[i]))
        continue;
      if (__pendingSlotChanges.has(slot)) continue;
      __pendingSlotChanges.add(slot);
      queueMicrotask(() => {
        __pendingSlotChanges.delete(slot);
        slot.dispatchEvent(new Event('slotchange', { bubbles: true }));
      });
    }
  }
}
function __eventParent(node, event) {
  const slot = __assignedSlot(node);
  if (slot) return slot;
  if (node instanceof ShadowRoot)
    return event.composed || event.__originalTarget.getRootNode() !== node ? node.host : null;
  if (node instanceof Document) return node.defaultView;
  return node.parentNode ?? null;
}
function __retarget(node, against) {
  while (node instanceof Node) {
    const root = node.getRootNode();
    if (!(root instanceof ShadowRoot)) return node;
    for (
      let current = against;
      current instanceof Node;
      current = current.getRootNode() instanceof ShadowRoot ? current.getRootNode().host : null
    ) {
      if (current.getRootNode() === root) return node;
    }
    node = root.host;
  }
  return node;
}
function __visibleEventPath(event) {
  return (event.__path ?? []).filter((node) => {
    if (!(node instanceof Node)) return true;
    for (
      let root = node.getRootNode();
      root instanceof ShadowRoot;
      root = root.host.getRootNode()
    ) {
      if (root.mode === 'closed' && __retarget(node, event.currentTarget) !== node) return false;
    }
    return true;
  });
}
globalThis.ShadowRoot = ShadowRoot;
