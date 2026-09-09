class DOMRect {
    constructor(x = 0, y = 0, width = 0, height = 0) {
        this.x = x; this.y = y; this.width = width; this.height = height;
        this.top = y; this.right = x + width; this.bottom = y + height; this.left = x;
    }
}

const __styleDeclarationTargets = new WeakMap();
const __styleDeclarationProxies = new WeakMap();
const __styleDeclarations = new WeakSet();

function __styleDeclarationTarget(declaration) {
    return __styleDeclarationTargets.get(declaration) ?? declaration;
}

function __requireStyleDeclaration(declaration) {
    const target = __styleDeclarationTarget(declaration);
    if ((typeof target !== "object" && typeof target !== "function") ||
        target === null || !__styleDeclarations.has(target)) {
        throw new TypeError("receiver is not a CSSStyleDeclaration");
    }
    return target;
}

function __requireWritableStyleDeclaration(declaration) {
    const target = __requireStyleDeclaration(declaration);
    if (!__callHost("styleWritable", target)) {
        throw new DOMException("The declaration is read-only", "NoModificationAllowedError");
    }
    return target;
}

function __styleDeclarationProxy(target) {
    let proxy = __styleDeclarationProxies.get(target);
    if (proxy !== undefined) return proxy;
    proxy = new Proxy(target, {
        get(target, property, receiver) {
            if (typeof property === "string" && /^(0|[1-9][0-9]*)$/.test(property)) {
                const name = CSSStyleDeclaration.prototype.item.call(receiver, Number(property));
                return name === "" ? undefined : name;
            }
            if (typeof property === "string" && !(property in target)) {
                return CSSStyleDeclaration.prototype.getPropertyValue.call(
                    receiver,
                    __cssRulePropertyName(property),
                );
            }
            return Reflect.get(target, property, receiver);
        },
        set(target, property, value, receiver) {
            if (typeof property === "string" && !(property in target)) {
                CSSStyleDeclaration.prototype.setProperty.call(
                    receiver,
                    __cssRulePropertyName(property),
                    value,
                );
                return true;
            }
            return Reflect.set(target, property, value, receiver);
        },
        has(target, property) {
            if (Reflect.has(target, property)) return true;
            if (typeof property !== "string") return false;
            if (/^(0|[1-9][0-9]*)$/.test(property)) {
                return Number(property) < Reflect.get(target, "length", receiver);
            }
            return __cssSupportsDeclaration(__cssRulePropertyName(property), "initial");
        },
    });
    __styleDeclarations.add(target);
    __styleDeclarationTargets.set(proxy, target);
    __styleDeclarationProxies.set(target, proxy);
    return proxy;
}

class CSSStyleDeclaration {
    constructor() { throw new TypeError("Illegal constructor"); }
    __entries() {
        return JSON.parse(__callHost("styleDeclarations", __requireStyleDeclaration(this)));
    }
    get cssText() {
        return __callHost("styleCssText", __requireStyleDeclaration(this));
    }
    set cssText(value) {
        const target = __requireWritableStyleDeclaration(this);
        __callHost("styleSetCssText", target, String(value));
    }
    get length() { return this.__entries().length; }
    item(index) {
        if (arguments.length === 0) throw new TypeError("CSSStyleDeclaration.item requires an index");
        return this.__entries()[Number(index)]?.[0] ?? "";
    }
    getPropertyValue(name) {
        if (arguments.length === 0) {
            throw new TypeError("CSSStyleDeclaration.getPropertyValue requires a property");
        }
        return __callHost("styleGetProperty", __requireStyleDeclaration(this), name);
    }
    getPropertyPriority(name) {
        if (arguments.length === 0) {
            throw new TypeError("CSSStyleDeclaration.getPropertyPriority requires a property");
        }
        const entry = this.__entries().find(entry => entry[0] === String(name));
        return entry?.[2] ? "important" : "";
    }
    setProperty(name, value, priority = "") {
        if (arguments.length < 2) {
            throw new TypeError("CSSStyleDeclaration.setProperty requires a property and value");
        }
        const target = __requireWritableStyleDeclaration(this);
        name = String(name);
        value = value === null ? "" : String(value);
        priority = priority === null || priority === undefined ? "" : String(priority);
        if (priority && priority.toLowerCase() !== "important") return;
        if (!value) {
            this.removeProperty(name);
            return;
        }
        if (priority) {
            if (!__cssSupportsDeclaration(name, value)) return;
            this.removeProperty(name);
            this.cssText = `${this.cssText} ${name}: ${value} !important;`;
            return;
        }
        if (!__cssSupportsDeclaration(name, value)) return;
        this.removeProperty(name);
        __callHost("styleSetProperty", target, name, value);
    }
    removeProperty(name) {
        if (arguments.length === 0) {
            throw new TypeError("CSSStyleDeclaration.removeProperty requires a property");
        }
        return __callHost("styleRemoveProperty", __requireWritableStyleDeclaration(this), name);
    }
    get parentRule() {
        __requireStyleDeclaration(this);
        return null;
    }
    [Symbol.iterator]() { return this.__entries().map(entry => entry[0])[Symbol.iterator](); }
}

class CSSStyleProperties extends CSSStyleDeclaration {
    get cssFloat() { return this.getPropertyValue("float"); }
    set cssFloat(value) { this.setProperty("float", value); }
}

Object.setPrototypeOf(CSSRuleStyleDeclaration.prototype, CSSStyleProperties.prototype);
Object.defineProperty(CSSRuleStyleDeclaration.prototype, "parentRule", {
    configurable: true,
    enumerable: true,
    get() { return this.__rule; },
});

let __cssSupportProbe = null;
function __cssSupportsDeclaration(property, value) {
    property = String(property).trim();
    value = String(value).trim();
    if (property === "" || value === "") return false;
    if (__cssSupportProbe === null) __cssSupportProbe = document.createElement("div");
    const style = __styleDeclarationTarget(__cssSupportProbe.style);
    __callHost("styleRemoveProperty", style, property);
    __callHost("styleSetProperty", style, property, value);
    const supported = __callHost("styleGetProperty", style, property) !== "";
    __callHost("styleRemoveProperty", style, property);
    return supported;
}

const CSS = {
    supports(propertyOrCondition, value = undefined) {
        if (arguments.length >= 2) return __cssSupportsDeclaration(propertyOrCondition, value);
        let condition = String(propertyOrCondition).trim();
        if (condition.startsWith("selector(") && condition.endsWith(")")) {
            try {
                document.querySelector(condition.slice(9, -1));
                return true;
            } catch (_) {
                return false;
            }
        }
        if (condition.startsWith("(") && condition.endsWith(")")) {
            condition = condition.slice(1, -1).trim();
        }
        const colon = condition.indexOf(":");
        return colon > 0 && __cssSupportsDeclaration(condition.slice(0, colon), condition.slice(colon + 1));
    },
    escape(value) {
        const input = String(value);
        const characters = Array.from(input);
        let output = "";
        for (let index = 0; index < characters.length; index++) {
            const character = characters[index];
            const code = character.codePointAt(0);
            if (code === 0) {
                output += "\uFFFD";
            } else if ((code >= 1 && code <= 31) || code === 127 ||
                       (index === 0 && code >= 48 && code <= 57) ||
                       (index === 1 && code >= 48 && code <= 57 && characters[0] === "-")) {
                output += `\\${code.toString(16)} `;
            } else if (index === 0 && character === "-" && characters.length === 1) {
                output += "\\-";
            } else if (code >= 128 || character === "-" || character === "_" ||
                       (code >= 48 && code <= 57) || (code >= 65 && code <= 90) ||
                       (code >= 97 && code <= 122)) {
                output += character;
            } else {
                output += `\\${character}`;
            }
        }
        return output;
    },
};
Object.defineProperty(CSS, Symbol.toStringTag, {
    value: "CSS",
    writable: false,
    enumerable: false,
    configurable: true,
});

