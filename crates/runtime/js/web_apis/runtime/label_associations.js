// Safari label lists cache by document. Element-tree changes and changed
// null-namespace `for`/`type` attributes invalidate them; ID changes do not.
const __labelVersions = new WeakMap();
function __labelable(element) {
    if (!(element instanceof HTMLElement)) return false;
    const name = __callHost('localName', element);
    if (['button','input','meter','output','progress','select','textarea'].includes(name))
        return !(element instanceof HTMLInputElement) || __inputType(element) !== 'hidden';
    return __upgradedCustomElements.has(element) && !__failedCustomElements.has(element)
        && __formAssociatedDefinitions.has(__customElementNames.get(element));
}
function __labelControl(label) {
    const id = __getContentAttribute(label, 'for');
    if (id !== null) {
        if (id === '' || !__nodeIsConnected(label)) return null;
        const control = __callHost('getElementById', __nodeTreeRoot(label), id);
        return __labelable(control) ? control : null;
    }
    return __callHost('querySelectorAll', label, '*').find(__labelable) ?? null;
}
function __labelForm(label) {
    const control = __labelControl(label);
    if (!control || control instanceof HTMLMeterElement || control instanceof HTMLProgressElement) return null;
    return __formOwner(control);
}
function __labelsFor(control) {
    return __cachedCollection(control, 'labels', () => {
        let previousDocument, previousVersion = -1, labels;
        return __createNodeList(() => {
            const owner = __callHost('ownerDocument', control), version = __labelVersions.get(owner) ?? 0;
            __labelVersions.set(owner, version);
            if (owner !== previousDocument || version !== previousVersion) {
                const root = __nodeTreeRoot(control);
                labels = root instanceof Document || root instanceof ShadowRoot
                    ? __callHost('querySelectorAll', root, 'label').filter(label => label instanceof HTMLLabelElement && __labelControl(label) === control)
                    : [];
                previousDocument = owner; previousVersion = version;
            }
            return labels;
        });
    });
}
function __labelMutationBefore(operation, args) {
    const target = args[0];
    let changed;
    if (['setAttribute','removeAttribute','setAttributeNS','removeAttributeNS'].includes(operation)) {
        const namespaced = operation.endsWith('NS');
        if (namespaced && args[1]) return null;
        const name = namespaced ? args[2] : __asciiLowercase(args[1]);
        if (!['for','type'].includes(name)) return null;
        const previous = __getContentAttribute(target, name);
        changed = () => __getContentAttribute(target, name) !== previous;
    } else if (['setInnerHTML','setTextContent'].includes(operation)) {
        if (!(target instanceof Element || target instanceof DocumentFragment || target instanceof Document)) return null;
        const parent = target instanceof HTMLTemplateElement ? __callHost('templateContent', target) : target;
        const hadElements = __callHost('childNodes', parent).some(node => node instanceof Element);
        changed = () => hadElements || __callHost('childNodes', parent).some(node => node instanceof Element);
    } else if (['appendChild','removeChild','insertBefore','replaceChild'].includes(operation)) {
        const containsElement = node => node instanceof Element || node instanceof DocumentFragment && __callHost('childNodes', node).some(child => child instanceof Element);
        if (!containsElement(args[1]) && !(operation === 'replaceChild' && containsElement(args[2]))) return null;
        changed = () => true;
    } else return null;
    const owners = new Set(args.filter(node => node instanceof Node).map(node => node instanceof Document ? node : __callHost('ownerDocument', node)));
    return () => {
        if (!changed()) return;
        for (const node of args) if (node instanceof Node) owners.add(node instanceof Document ? node : __callHost('ownerDocument', node));
        for (const owner of owners) if (__labelVersions.has(owner)) __labelVersions.set(owner, __labelVersions.get(owner) + 1);
    };
}
Object.defineProperties(HTMLLabelElement.prototype, {
    control: Object.getOwnPropertyDescriptor({get control() { return __labelControl(this); }}, 'control'),
    form: Object.getOwnPropertyDescriptor({get form() { return __labelForm(this); }}, 'form'),
});
for (const cls of [HTMLInputElement,HTMLButtonElement,HTMLSelectElement,HTMLTextAreaElement,HTMLMeterElement,HTMLProgressElement,HTMLOutputElement]) {
    Object.defineProperty(cls.prototype, 'labels', Object.getOwnPropertyDescriptor({
        get labels() { return __labelable(this) ? __labelsFor(this) : null; },
    }, 'labels'));
}
