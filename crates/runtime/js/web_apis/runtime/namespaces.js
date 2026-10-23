const __htmlNamespace = 'http://www.w3.org/1999/xhtml';
const __xmlNamespace = 'http://www.w3.org/XML/1998/namespace';
const __xmlnsNamespace = 'http://www.w3.org/2000/xmlns/';
const __xmlName = /^[:_\p{L}\p{Nl}][:_\p{L}\p{Nl}\p{N}.\-\u00b7\p{M}\u203f\u2040]*$/u;
function __validateXmlName(name) {
    if (!__xmlName.test(name)) throw new DOMException('Invalid XML name', 'InvalidCharacterError');
}
function __validateQualifiedName(namespace, qualifiedName) {
    namespace = namespace == null || namespace === '' ? null : String(namespace);
    const name = String(qualifiedName);
    __validateXmlName(name);
    const parts = name.split(':');
    if (parts.length > 2 || parts.some(p => !p || !__xmlName.test(p))) throw new DOMException('Invalid qualified name', 'NamespaceError');
    const prefix = parts.length === 2 ? parts[0] : null, localName = parts.at(-1);
    if (prefix !== null && namespace === null || prefix === 'xml' && namespace !== __xmlNamespace ||
        (prefix === 'xmlns' || name === 'xmlns') && namespace !== __xmlnsNamespace ||
        namespace === __xmlnsNamespace && prefix !== 'xmlns' && name !== 'xmlns')
        throw new DOMException('Invalid namespace', 'NamespaceError');
    return {name, localName, prefix, namespaceURI:namespace};
}
function __attributeKey(namespace, localName) { return `${namespace ?? ''}\0${localName}`; }
function __detachAttribute(element, namespace, localName) {
    const map = __attributeMaps.get(element), key = __attributeKey(namespace, localName);
    const attr = map?.__cache.get(key);
    if (attr) { attr.__currentRecord(); attr.__ownerDocument = element.ownerDocument; attr.__element = null; map.__cache.delete(key); }
}
function __attachAttribute(map, attr, namespaced) {
    if (!(attr instanceof Attr)) throw new TypeError('Expected an Attr');
    const element = map.__element;
    if (attr.ownerElement && attr.ownerElement !== element) throw new DOMException('Attribute is in use', 'InUseAttributeError');
    const old = namespaced ? map.getNamedItemNS(attr.namespaceURI, attr.localName) : map.getNamedItem(attr.name);
    if (old === attr) return old;
    if (old) element.removeAttributeNode(old);
    // Attr attachment preserves case and namespaces, including a null namespace.
    __callHost('setAttributeNS', element, attr.namespaceURI ?? '', attr.name, attr.value);
    attr.__element = element; attr.__ownerDocument = element.ownerDocument;
    map.__cache.set(__attributeKey(attr.namespaceURI, attr.localName), attr);
    return old;
}
NamedNodeMap.prototype.setNamedItem = function(attr) { return __attachAttribute(this, attr, false); };
NamedNodeMap.prototype.setNamedItemNS = function(attr) { return __attachAttribute(this, attr, true); };
NamedNodeMap.prototype.getNamedItemNS = function(namespace, localName) {
    namespace = namespace == null || namespace === '' ? null : String(namespace);
    localName = String(localName);
    const record = __attributeRecords(this.__element).find(a => a.namespaceURI === namespace && a.localName === localName);
    return record ? this.__attr(record) : null;
};
NamedNodeMap.prototype.removeNamedItemNS = function(namespace, localName) {
    const attr = this.getNamedItemNS(namespace, localName);
    if (!attr) throw new DOMException('Attribute was not found', 'NotFoundError');
    return this.__element.removeAttributeNode(attr);
};
Element.prototype.setAttributeNode = function(attr) { return this.attributes.setNamedItem(attr); };
Element.prototype.setAttributeNodeNS = function(attr) { return this.attributes.setNamedItemNS(attr); };
Element.prototype.getAttributeNodeNS = function(namespace, name) { return this.attributes.getNamedItemNS(namespace, name); };
Element.prototype.getAttributeNS = function(namespace, name) { return this.getAttributeNodeNS(namespace, name)?.value ?? null; };
Element.prototype.hasAttributeNS = function(namespace, name) { return this.getAttributeNodeNS(namespace, name) !== null; };
Element.prototype.setAttributeNS = function(namespace, name, value) {
    const record = __validateQualifiedName(namespace, name);
    __callHost('setAttributeNS', this, record.namespaceURI ?? '', record.name, String(value));
};
Element.prototype.removeAttributeNS = function(namespace, localName) {
    namespace = namespace == null || namespace === '' ? null : String(namespace);
    localName = String(localName);
    __detachAttribute(this, namespace, localName);
    __callHost('removeAttributeNS', this, namespace ?? '', localName);
};
function __namespaceElements(namespace, localName) {
    namespace = namespace == null || namespace === '' ? null : String(namespace);
    localName = String(localName);
    return new HTMLCollection(() => Array.from(this.querySelectorAll('*')).filter(e =>
        (namespace === '*' || e.namespaceURI === namespace) && (localName === '*' || e.localName === localName)));
}
Element.prototype.getElementsByTagNameNS = __namespaceElements;
Document.prototype.getElementsByTagNameNS = __namespaceElements;
Node.prototype.lookupNamespaceURI = function(prefix) {
    prefix = prefix == null || prefix === '' ? null : String(prefix);
    if (this instanceof Document) return this.documentElement?.lookupNamespaceURI(prefix) ?? null;
    if (this instanceof Attr) return this.ownerElement?.lookupNamespaceURI(prefix) ?? null;
    if (this instanceof Element) {
        if (this.namespaceURI !== null && this.prefix === prefix) return this.namespaceURI;
        for (const a of this.attributes) {
            if (a.namespaceURI === __xmlnsNamespace && (a.prefix === 'xmlns' && a.localName === prefix || a.name === 'xmlns' && prefix === null)) return a.value || null;
        }
    }
    return this.parentElement?.lookupNamespaceURI(prefix) ?? null;
};
Node.prototype.lookupPrefix = function(namespace) {
    namespace = namespace == null || namespace === '' ? null : String(namespace);
    if (namespace === null) return null;
    const element = this instanceof Document ? this.documentElement : this instanceof Attr ? this.ownerElement : this instanceof Element ? this : this.parentElement;
    for (let e = element; e; e = e.parentElement) {
        if (e.namespaceURI === namespace && e.prefix !== null && element.lookupNamespaceURI(e.prefix) === namespace) return e.prefix;
        for (const a of e.attributes) if (a.namespaceURI === __xmlnsNamespace && a.prefix === 'xmlns' && a.value === namespace && element.lookupNamespaceURI(a.localName) === namespace) return a.localName;
    }
    return null;
};
Node.prototype.isDefaultNamespace = function(namespace) { return this.lookupNamespaceURI(null) === (namespace == null || namespace === '' ? null : String(namespace)); };
