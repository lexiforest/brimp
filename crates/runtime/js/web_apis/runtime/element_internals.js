const __elementInternals = new WeakMap(), __internalsToken = {};
class CustomStateSet extends Set {
    constructor(token) { if (token !== __internalsToken) throw new TypeError('Illegal constructor'); super(); }
    add(value) { return super.add(String(value)); }
}
class ElementInternals {
    #element; #aria = new Map(); #states = new CustomStateSet(__internalsToken);
    constructor(token, element) { if (token !== __internalsToken) throw new TypeError('Illegal constructor'); this.#element = element; }
    get shadowRoot() { return __shadowRoots.get(this.#element) ?? null; }
    get states() { return this.#states; }
    __getAria(name) { return this.#aria.get(name) ?? null; }
    __setAria(name, value) { this.#aria.set(name, value == null ? null : String(value)); }
}
// Default accessibility semantics are independent of content attributes.
for (const name of ['role','ariaAtomic','ariaAutoComplete','ariaBrailleLabel','ariaBrailleRoleDescription','ariaBusy','ariaChecked','ariaColCount','ariaColIndex','ariaColIndexText','ariaColSpan','ariaCurrent','ariaDescription','ariaDisabled','ariaExpanded','ariaHasPopup','ariaHidden','ariaInvalid','ariaKeyShortcuts','ariaLabel','ariaLevel','ariaLive','ariaModal','ariaMultiLine','ariaMultiSelectable','ariaOrientation','ariaPlaceholder','ariaPosInSet','ariaPressed','ariaReadOnly','ariaRelevant','ariaRequired','ariaRoleDescription','ariaRowCount','ariaRowIndex','ariaRowIndexText','ariaRowSpan','ariaSelected','ariaSetSize','ariaSort','ariaValueMax','ariaValueMin','ariaValueNow','ariaValueText']) {
    Object.defineProperty(ElementInternals.prototype, name, {configurable:true,get() { return this.__getAria(name); },set(value) { this.__setAria(name,value); }});
}
HTMLElement.prototype.attachInternals = function() {
    const name = __customElementNames.get(this) ?? this.localName, definition = customElements.get(name);
    if (!definition || customElements.__localNames.get(name) !== name || __elementInternals.has(this) || definition.disabledFeatures?.includes('internals'))
        throw new DOMException('Element internals are unavailable','NotSupportedError');
    const internals = new ElementInternals(__internalsToken,this);
    __elementInternals.set(this,internals); return internals;
};
