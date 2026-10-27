const __elementInternals = new WeakMap(),
  __internalsToken = {};
class CustomStateSet extends Set {
  constructor(token) {
    if (token !== __internalsToken) throw new TypeError('Illegal constructor');
    super();
  }
  add(value) {
    return super.add(String(value));
  }
}
class ElementInternals {
  #element;
  #aria = new Map();
  #states = new CustomStateSet(__internalsToken);
  constructor(token, element) {
    if (token !== __internalsToken) throw new TypeError('Illegal constructor');
    this.#element = element;
  }
  get shadowRoot() {
    return __shadowRoots.get(this.#element) ?? null;
  }
  get states() {
    return this.#states;
  }
  __getAria(name) {
    return this.#aria.get(name) ?? null;
  }
  __setAria(name, value) {
    this.#aria.set(name, value == null ? null : `${value}`);
  }
}
for (const name of [
  'role',
  'ariaAtomic',
  'ariaAutoComplete',
  'ariaBrailleLabel',
  'ariaBrailleRoleDescription',
  'ariaBusy',
  'ariaChecked',
  'ariaColCount',
  'ariaColIndex',
  'ariaColIndexText',
  'ariaColSpan',
  'ariaCurrent',
  'ariaDescription',
  'ariaDisabled',
  'ariaExpanded',
  'ariaHasPopup',
  'ariaHidden',
  'ariaInvalid',
  'ariaKeyShortcuts',
  'ariaLabel',
  'ariaLevel',
  'ariaLive',
  'ariaModal',
  'ariaMultiLine',
  'ariaMultiSelectable',
  'ariaOrientation',
  'ariaPlaceholder',
  'ariaPosInSet',
  'ariaPressed',
  'ariaReadOnly',
  'ariaRelevant',
  'ariaRequired',
  'ariaRoleDescription',
  'ariaRowCount',
  'ariaRowIndex',
  'ariaRowIndexText',
  'ariaRowSpan',
  'ariaSelected',
  'ariaSetSize',
  'ariaSort',
  'ariaValueMax',
  'ariaValueMin',
  'ariaValueNow',
  'ariaValueText',
]) {
  const attribute = name === 'role' ? 'role' : `aria-${name.slice(4).toLowerCase()}`;
  Object.defineProperty(
    Element.prototype,
    name,
    Object.getOwnPropertyDescriptor(
      {
        get [name]() {
          return __getContentAttribute(this, attribute);
        },
        set [name](value) {
          __setContentAttribute(this, attribute, value);
        },
      },
      name,
    ),
  );
  // Default accessibility semantics stay independent of content attributes.
  // Safari exposes these strings on Element but not ElementInternals.
  if (['ariaColIndexText', 'ariaDescription', 'ariaRowIndexText'].includes(name)) continue;
  Object.defineProperty(
    ElementInternals.prototype,
    name,
    Object.getOwnPropertyDescriptor(
      {
        get [name]() {
          return this.__getAria(name);
        },
        set [name](value) {
          this.__setAria(name, value);
        },
      },
      name,
    ),
  );
}
HTMLElement.prototype.attachInternals = function () {
  const name = __customElementNames.get(this) ?? this.localName,
    definition = __customElementRegistryState(customElements).definitions.get(name);
  if (
    !definition ||
    !__upgradedCustomElements.has(this) ||
    __elementInternals.has(this) ||
    definition.disabledFeatures?.includes('internals')
  )
    throw new DOMException('Element internals are unavailable', 'NotSupportedError');
  const internals = new ElementInternals(__internalsToken, this);
  __elementInternals.set(this, internals);
  return internals;
};
