function __insertAdjacentNode(element, position, node) {
    if (position === "beforebegin") {
        if (!element.parentNode) return null;
        element.parentNode.insertBefore(node, element);
    } else if (position === "afterbegin") {
        element.insertBefore(node, element.firstChild);
    } else if (position === "beforeend") {
        element.appendChild(node);
    } else if (position === "afterend") {
        if (!element.parentNode) return null;
        element.parentNode.insertBefore(node, element.nextSibling);
    } else {
        throw new DOMException("Invalid position", "SyntaxError");
    }
    return node;
}

class Element extends Node {
    get tagName() { return __callHost("tagName", this); }
    get localName() { return __callHost("localName", this); }
    get namespaceURI() { return __callHost("namespaceURI", this); }
    get prefix() { return __callHost("prefix", this); }
    get children() { return new HTMLCollection(() => [...this.childNodes].filter(node => node instanceof Element)); }
    get childElementCount() { return this.children.length; }
    get firstElementChild() { return this.children.item(0); }
    get lastElementChild() { return this.children.item(this.children.length - 1); }
    get previousElementSibling() {
        let sibling = this.previousSibling;
        while (sibling !== null && !(sibling instanceof Element)) sibling = sibling.previousSibling;
        return sibling;
    }
    get nextElementSibling() {
        let sibling = this.nextSibling;
        while (sibling !== null && !(sibling instanceof Element)) sibling = sibling.nextSibling;
        return sibling;
    }
    get attributes() {
        let attributes = __attributeMaps.get(this);
        if (!attributes) {
            attributes = new NamedNodeMap(this);
            __attributeMaps.set(this, attributes);
        }
        return attributes;
    }
    get id() { return __callHost("getAttributeOrEmpty", this, "id"); }
    set id(value) { __callHost("setAttribute", this, "id", value); }
    get className() { return __callHost("getAttributeOrEmpty", this, "class"); }
    set className(value) { __callHost("setAttribute", this, "class", value); }
    get classList() {
        let list = __classLists.get(this);
        if (!list) { list = new DOMTokenList(this); __classLists.set(this, list); }
        return list;
    }
    set classList(value) { this.classList.value = value; }
    get innerHTML() { return __callHost("innerHTML", this); }
    set innerHTML(value) { __callHost("setInnerHTML", this, value); }
    get outerHTML() { return __callHost("outerHTML", this); }
    get style() { return __styleDeclarationProxy(__callHost("style", this)); }
    set style(value) { this.style.cssText = value; }
    get clientWidth() { return __callHost("clientWidth", this); }
    get clientHeight() { return __callHost("clientHeight", this); }
    get offsetWidth() { return __callHost("offsetWidth", this); }
    get offsetHeight() { return __callHost("offsetHeight", this); }
    getBoundingClientRect() {
        const rect = __callHost("boundingRect", this);
        return new DOMRect(rect[0], rect[1], rect[2], rect[3]);
    }
    getClientRects() {
        const rect = this.getBoundingClientRect();
        return rect.width > 0 && rect.height > 0 ? [rect] : [];
    }
    getAttribute(name) { return __callHost("getAttribute", this, name); }
    setAttribute(name, value) {
        name = String(name);
        if (name === "") throw new DOMException("attribute name cannot be empty", "InvalidCharacterError");
        __callHost("setAttribute", this, name, value);
    }
    removeAttribute(name) { __callHost("removeAttribute", this, name); }
    hasAttribute(name) { return this.getAttribute(name) !== null; }
    getAttributeNames() { return __attributeRecords(this).map(attribute => attribute.name); }
    toggleAttribute(name, force = undefined) {
        name = String(name);
        if (name === "") throw new DOMException("attribute name cannot be empty", "InvalidCharacterError");
        const present = this.hasAttribute(name);
        if (present && (arguments.length < 2 || !Boolean(force))) {
            this.removeAttribute(name);
            return false;
        }
        if (!present && (arguments.length < 2 || Boolean(force))) this.setAttribute(name, "");
        return true;
    }
    append(...nodes) { __appendNodes(this, nodes); }
    prepend(...nodes) { __prependNodes(this, nodes); }
    replaceChildren(...nodes) { __replaceChildren(this, nodes); }
    replaceWith(...nodes) { __replaceNode(this, nodes); }
    remove() { __removeNode(this); }
    insertAdjacentElement(position, element) {
        if (!(element instanceof Element)) throw new TypeError("element must be an Element");
        position = String(position).toLowerCase();
        return __insertAdjacentNode(this, position, element);
    }
    insertAdjacentText(position, data) {
        __insertAdjacentNode(this, String(position).toLowerCase(), document.createTextNode(String(data)));
    }
    insertAdjacentHTML(position, text) {
        position = String(position).toLowerCase();
        const outside = position === "beforebegin" || position === "afterend";
        if (outside && !this.parentNode) throw new DOMException("The element has no parent", "NoModificationAllowedError");
        if (!["beforebegin", "afterbegin", "beforeend", "afterend"].includes(position)) {
            throw new DOMException("Invalid position", "SyntaxError");
        }
        const container = document.createElement(outside && this.parentElement ? this.parentElement.localName : this.localName);
        container.innerHTML = String(text);
        const nodes = Array.from(container.childNodes);
        if (position === "afterbegin" || position === "afterend") nodes.reverse();
        for (const node of nodes) __insertAdjacentNode(this, position, node);
    }
    getElementsByTagName(name) {
        return new HTMLCollection(() => __callHost("getElementsByTagName", this, name));
    }
    getElementsByClassName(names) {
        return new HTMLCollection(() => __callHost("getElementsByClassName", this, names));
    }
    querySelector(selector) { return __querySelectorWithHas(this, selector); }
    querySelectorAll(selector) { return new NodeList(__querySelectorAllWithHas(this, selector)); }
    matches(selector) { return __matchesSelectorWithHas(this, selector); }
    closest(selector) {
        let element = this;
        while (element) {
            if (element.matches(selector)) return element;
            element = element.parentElement;
        }
        return null;
    }
    scrollIntoView() {}
    focus() {
        const previous = __activeElement;
        if (previous === this) return;
        if (previous) {
            previous.dispatchEvent(new Event("blur"));
            previous.dispatchEvent(new Event("focusout", {bubbles: true}));
        }
        __activeElement = this;
        this.dispatchEvent(new Event("focus"));
        this.dispatchEvent(new Event("focusin", {bubbles: true}));
    }
    blur() {
        if (__activeElement !== this) return;
        __activeElement = null;
        this.dispatchEvent(new Event("blur"));
        this.dispatchEvent(new Event("focusout", {bubbles: true}));
    }
    click() { this.dispatchEvent(new Event("click", { bubbles: true, cancelable: true })); }
}

function __reflectedString(element, name) {
    return element.getAttribute(name) || "";
}

function __setReflectedBoolean(element, name, value) {
    if (Boolean(value)) element.setAttribute(name, "");
    else element.removeAttribute(name);
}

class SVGElement extends Element {}

class HTMLElement extends Element {
    get innerText() { return this.textContent; }
    set innerText(value) { this.textContent = String(value); }
    get title() { return __reflectedString(this, "title"); }
    set title(value) { this.setAttribute("title", value); }
    get lang() { return __reflectedString(this, "lang"); }
    set lang(value) { this.setAttribute("lang", value); }
    get dir() {
        const value = __asciiLowercase(__reflectedString(this, "dir"));
        return value === "ltr" || value === "rtl" || value === "auto" ? value : "";
    }
    set dir(value) { this.setAttribute("dir", value); }
    get autofocus() { return this.hasAttribute("autofocus"); }
    set autofocus(value) { __setReflectedBoolean(this, "autofocus", value); }
    get hidden() { return this.hasAttribute("hidden"); }
    set hidden(value) { __setReflectedBoolean(this, "hidden", value); }
    get accessKey() { return __reflectedString(this, "accesskey"); }
    set accessKey(value) { this.setAttribute("accesskey", value); }
    get tabIndex() {
        const value = this.getAttribute("tabindex");
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
        this.setAttribute("tabindex", String(number));
    }
}
class HTMLAnchorElement extends HTMLElement {
    get href() { return this.hasAttribute("href") ? __callHost("elementUrl", this, "href") : ""; }
    set href(value) {
        value = __toUSVString(value);
        const queryStart = value.indexOf("?");
        if (queryStart !== -1 && __asciiLowercase(document.characterSet) !== "utf-8") {
            const fragmentStart = value.indexOf(String.fromCharCode(35), queryStart);
            const queryEnd = fragmentStart === -1 ? value.length : fragmentStart;
            value = value.slice(0, queryStart + 1)
                + __legacyQueryEncode(document.characterSet, value.slice(queryStart + 1, queryEnd))
                + value.slice(queryEnd);
        }
        this.setAttribute("href", value);
    }
    get origin() { return __callHost("elementUrl", this, "origin"); }
    get protocol() { return new URL(this.href).protocol; }
    set protocol(value) { this.__setUrlComponent("protocol", value); }
    get username() { return new URL(this.href).username; }
    set username(value) { this.__setUrlComponent("username", value); }
    get password() { return new URL(this.href).password; }
    set password(value) { this.__setUrlComponent("password", value); }
    get host() { return new URL(this.href).host; }
    set host(value) { this.__setUrlComponent("host", value); }
    get hostname() { return new URL(this.href).hostname; }
    set hostname(value) { this.__setUrlComponent("hostname", value); }
    get port() { return new URL(this.href).port; }
    set port(value) { this.__setUrlComponent("port", value); }
    get pathname() { return new URL(this.href).pathname; }
    set pathname(value) { this.__setUrlComponent("pathname", value); }
    get search() { return new URL(this.href).search; }
    set search(value) { this.__setUrlComponent("search", value); }
    get hash() { return new URL(this.href).hash; }
    set hash(value) { this.__setUrlComponent("hash", value); }
    toString() { return this.href; }
    __setUrlComponent(component, value) {
        const url = new URL(this.href);
        url[component] = value;
        this.href = url.href;
    }
}
