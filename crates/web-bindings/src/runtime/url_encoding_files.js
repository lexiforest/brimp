function __urlRecord(input, base = undefined) {
    try {
        return JSON.parse(__callHost("urlParse", window, String(input), base === undefined ? "" : String(base)));
    } catch (_) {
        throw new TypeError("Invalid URL");
    }
}

class URLSearchParams {
    constructor(init = "", owner = null) {
        this.__owner = owner;
        if (typeof init === "string") {
            this.__pairs = JSON.parse(__callHost("urlSearchParamsParse", window, init));
        } else if (init != null && typeof init[Symbol.iterator] === "function") {
            this.__pairs = Array.from(init, pair => {
                if (pair == null || typeof pair[Symbol.iterator] !== "function") throw new TypeError("each query pair must be iterable");
                const values = Array.from(pair);
                if (values.length !== 2) throw new TypeError("each query pair must have two items");
                return [String(values[0]), String(values[1])];
            });
        } else if (init != null) {
            this.__pairs = Object.keys(init).map(name => [name, String(init[name])]);
        } else {
            this.__pairs = [];
        }
    }
    get size() { return this.__pairs.length; }
    append(name, value) { this.__pairs.push([String(name), String(value)]); this.__changed(); }
    delete(name, value = undefined) {
        name = String(name);
        this.__pairs = this.__pairs.filter(pair => pair[0] !== name || (value !== undefined && pair[1] !== String(value)));
        this.__changed();
    }
    get(name) { name = String(name); const pair = this.__pairs.find(pair => pair[0] === name); return pair ? pair[1] : null; }
    getAll(name) { name = String(name); return this.__pairs.filter(pair => pair[0] === name).map(pair => pair[1]); }
    has(name, value = undefined) {
        name = String(name);
        return this.__pairs.some(pair => pair[0] === name && (value === undefined || pair[1] === String(value)));
    }
    set(name, value) {
        name = String(name); value = String(value);
        const index = this.__pairs.findIndex(pair => pair[0] === name);
        if (index === -1) this.__pairs.push([name, value]);
        else {
            this.__pairs[index][1] = value;
            this.__pairs = this.__pairs.filter((pair, item) => pair[0] !== name || item === index);
        }
        this.__changed();
    }
    sort() {
        this.__pairs = this.__pairs.map((pair, index) => [pair, index])
            .sort((a, b) => a[0][0] < b[0][0] ? -1 : a[0][0] > b[0][0] ? 1 : a[1] - b[1])
            .map(item => item[0]);
        this.__changed();
    }
    entries() { return this.__pairs.map(pair => pair.slice())[Symbol.iterator](); }
    keys() { return this.__pairs.map(pair => pair[0])[Symbol.iterator](); }
    values() { return this.__pairs.map(pair => pair[1])[Symbol.iterator](); }
    forEach(callback, thisArg = undefined) {
        for (const [name, value] of this.__pairs) callback.call(thisArg, value, name, this);
    }
    toString() { return __callHost("urlSearchParamsSerialize", window, JSON.stringify(this.__pairs)); }
    [Symbol.iterator]() { return this.entries(); }
    __changed() { if (this.__owner !== null) this.__owner.search = this.toString(); }
}

class URL {
    constructor(input, base = undefined) {
        this.__href = __urlRecord(input, base).href;
        this.__searchParams = new URLSearchParams(this.search, this);
    }
    static canParse(input, base = undefined) { try { __urlRecord(input, base); return true; } catch (_) { return false; } }
    static parse(input, base = undefined) { try { return new URL(input, base); } catch (_) { return null; } }
    get href() { return this.__href; }
    set href(value) { this.__set("href", value); }
    get origin() { return __urlRecord(this.__href).origin; }
    get protocol() { return __urlRecord(this.__href).protocol; }
    set protocol(value) { this.__set("protocol", value); }
    get username() { return __urlRecord(this.__href).username; }
    set username(value) { this.__set("username", value); }
    get password() { return __urlRecord(this.__href).password; }
    set password(value) { this.__set("password", value); }
    get host() { return __urlRecord(this.__href).host; }
    set host(value) { this.__set("host", value); }
    get hostname() { return __urlRecord(this.__href).hostname; }
    set hostname(value) { this.__set("hostname", value); }
    get port() { return __urlRecord(this.__href).port; }
    set port(value) { this.__set("port", value); }
    get pathname() { return __urlRecord(this.__href).pathname; }
    set pathname(value) { this.__set("pathname", value); }
    get search() { return __urlRecord(this.__href).search; }
    set search(value) { this.__set("search", value); }
    get searchParams() { return this.__searchParams; }
    get hash() { return __urlRecord(this.__href).hash; }
    set hash(value) { this.__set("hash", value); }
    toString() { return this.href; }
    toJSON() { return this.href; }
    __set(component, value) {
        this.__href = __callHost("urlSet", window, this.__href, component, String(value));
        if (this.__searchParams) {
            this.__searchParams.__pairs = JSON.parse(__callHost("urlSearchParamsParse", window, this.search));
        }
    }
}
globalThis.URL = URL;
globalThis.URLSearchParams = URLSearchParams;

function __encodingBytes(input) {
    if (input === undefined) return [];
    if (input instanceof ArrayBuffer) return Array.from(new Uint8Array(input));
    if (typeof SharedArrayBuffer !== "undefined" && input instanceof SharedArrayBuffer) {
        return Array.from(new Uint8Array(input));
    }
    if (ArrayBuffer.isView(input)) {
        if (input.buffer instanceof ArrayBuffer && input.buffer.byteLength === 0) return [];
        return Array.from(new Uint8Array(input.buffer, input.byteOffset, input.byteLength));
    }
    throw new TypeError("TextDecoder input must be an ArrayBuffer or an ArrayBufferView");
}

function __iso2022JpStatePrefix(bytes) {
    let state = [];
    for (let index = 0; index < bytes.length; index++) {
        if (bytes[index] !== 0x1B) continue;
        if (bytes[index + 1] === 0x28 && [0x42, 0x49, 0x4A].includes(bytes[index + 2])) {
            state = bytes.slice(index, index + 3);
            index += 2;
        } else if (bytes[index + 1] === 0x24 && [0x40, 0x42].includes(bytes[index + 2])) {
            state = bytes.slice(index, index + 3);
            index += 2;
        } else if (bytes[index + 1] === 0x24 && bytes[index + 2] === 0x28 && bytes[index + 3] === 0x44) {
            state = bytes.slice(index, index + 4);
            index += 3;
        }
    }
    return state;
}

class TextDecoder {
    constructor(label = "utf-8", options = {}) {
        const encoding = __callHost("encodingCanonical", window, String(label));
        if (encoding === null) throw new RangeError("The encoding label is invalid");
        this.__encoding = encoding;
        this.__fatal = Boolean(options.fatal);
        this.__ignoreBOM = Boolean(options.ignoreBOM);
        this.__bytes = [];
        this.__emitted = 0;
        this.__streaming = false;
    }
    get encoding() { return this.__encoding; }
    get fatal() { return this.__fatal; }
    get ignoreBOM() { return this.__ignoreBOM; }
    decode(input = undefined, options = {}) {
        const stream = Boolean(options.stream);
        const bytes = __encodingBytes(input);
        if (this.__streaming) this.__bytes.push(...bytes);
        else this.__bytes = bytes;
        const decoded = __callHost(
            "decodeBytes",
            window,
            this.encoding,
            JSON.stringify(this.__bytes),
            this.fatal,
            this.ignoreBOM,
            stream,
        );
        if (decoded === null) {
            const preserveIso2022JpState = stream && this.encoding === "iso-2022-jp";
            this.__bytes = preserveIso2022JpState ? __iso2022JpStatePrefix(this.__bytes) : [];
            this.__emitted = 0;
            this.__streaming = preserveIso2022JpState;
            throw new TypeError("The encoded data is not valid");
        }
        const output = decoded.slice(this.__emitted);
        if (stream) {
            this.__emitted = decoded.length;
            this.__streaming = true;
        } else {
            this.__bytes = [];
            this.__emitted = 0;
            this.__streaming = false;
        }
        return output;
    }
}

class TextEncoder {
    get encoding() { return "utf-8"; }
    encode(input = "") {
        return new Uint8Array(JSON.parse(__callHost("encodeUtf8", window, __toUSVString(input))));
    }
    encodeInto(source, destination) {
        source = __toUSVString(source);
        if (!(destination instanceof Uint8Array)) {
            throw new TypeError("TextEncoder destination must be a Uint8Array");
        }
        let read = 0;
        let written = 0;
        for (const scalar of source) {
            const bytes = this.encode(scalar);
            if (written + bytes.length > destination.length) break;
            destination.set(bytes, written);
            written += bytes.length;
            read += scalar.length;
        }
        return { read, written };
    }
}

function __toUSVString(value) {
    value = String(value);
    let output = "";
    for (let index = 0; index < value.length; index++) {
        const first = value.charCodeAt(index);
        if (first >= 0xD800 && first <= 0xDBFF) {
            const second = value.charCodeAt(index + 1);
            if (second >= 0xDC00 && second <= 0xDFFF) {
                output += value[index] + value[index + 1];
                index++;
            } else {
                output += "\uFFFD";
            }
        } else if (first >= 0xDC00 && first <= 0xDFFF) {
            output += "\uFFFD";
        } else {
            output += value[index];
        }
    }
    return output;
}

globalThis.TextDecoder = TextDecoder;
globalThis.TextEncoder = TextEncoder;

globalThis.btoa = input => {
    input = String(input);
    const bytes = [];
    for (let index = 0; index < input.length; index++) {
        const value = input.charCodeAt(index);
        if (value > 255) {
            throw new DOMException("The string contains characters outside of Latin1", "InvalidCharacterError");
        }
        bytes.push(value);
    }
    return __callHost("base64Encode", window, JSON.stringify(bytes));
};

globalThis.atob = input => {
    const encoded = __callHost("base64Decode", window, String(input));
    if (encoded === null) {
        throw new DOMException("The string is not correctly encoded", "InvalidCharacterError");
    }
    const bytes = JSON.parse(encoded);
    let output = "";
    for (const byte of bytes) output += String.fromCharCode(byte);
    return output;
};

function __blobPartBytes(part, endings) {
    if (part instanceof Blob) return part.__bytes;
    if (part instanceof ArrayBuffer) return new Uint8Array(part);
    if (typeof SharedArrayBuffer !== "undefined" && part instanceof SharedArrayBuffer) {
        return new Uint8Array(part);
    }
    if (ArrayBuffer.isView(part)) {
        return new Uint8Array(part.buffer, part.byteOffset, part.byteLength);
    }
    let text = String(part);
    if (endings === "native") text = text.replace(/\r\n|\r/g, "\n");
    return new TextEncoder().encode(text);
}

function __isHttpToken(value) {
    if (value.length === 0) return false;
    for (let index = 0; index < value.length; index++) {
        const code = value.charCodeAt(index);
        if (code < 0x21 || code > 0x7E || "()<>@,;:\"/[]?={}\\".includes(value[index])) return false;
    }
    return true;
}

function __isHttpQuotedString(value) {
    for (let index = 0; index < value.length; index++) {
        const code = value.charCodeAt(index);
        if (code !== 0x09 && !(code >= 0x20 && code <= 0x7E) && !(code >= 0x80 && code <= 0xFF)) {
            return false;
        }
    }
    return true;
}

function __parseMimeType(input) {
    input = String(input).replace(/^[\t\n\r ]+|[\t\n\r ]+$/g, "");
    const slash = input.indexOf("/");
    if (slash <= 0) return "";
    const type = input.slice(0, slash);
    if (!__isHttpToken(type)) return "";
    let position = slash + 1;
    let semicolon = input.indexOf(";", position);
    if (semicolon === -1) semicolon = input.length;
    const subtype = input.slice(position, semicolon).replace(/[\t\n\r ]+$/g, "");
    if (!__isHttpToken(subtype)) return "";
    const parameters = new Map();
    position = semicolon;
    while (position < input.length) {
        position++;
        while (/[\t\n\r ]/.test(input[position] ?? "")) position++;
        const nameStart = position;
        while (position < input.length && input[position] !== ";" && input[position] !== "=") position++;
        let name = input.slice(nameStart, position).toLowerCase();
        if (position >= input.length || input[position] === ";") continue;
        position++;
        let value = "";
        if (input[position] === '"') {
            position++;
            while (position < input.length) {
                const character = input[position++];
                if (character === '"') break;
                if (character === "\\" && position < input.length) value += input[position++];
                else value += character;
            }
            while (position < input.length && input[position] !== ";") position++;
        } else {
            const valueStart = position;
            while (position < input.length && input[position] !== ";") position++;
            value = input.slice(valueStart, position).replace(/[\t\n\r ]+$/g, "");
            if (value.length === 0) continue;
        }
        if (!parameters.has(name) && __isHttpToken(name) && __isHttpQuotedString(value)) {
            parameters.set(name, value);
        }
    }
    let output = type.toLowerCase() + "/" + subtype.toLowerCase();
    for (const [name, value] of parameters) {
        output += ";" + name + "=";
        output += __isHttpToken(value)
            ? value
            : '"' + value.replace(/(["\\])/g, "\\$1") + '"';
    }
    return output;
}

class Blob {
    constructor(blobParts = [], options = {}) {
        if (blobParts === null || (typeof blobParts !== "object" && typeof blobParts !== "function")) {
            throw new TypeError("blobParts must be a sequence");
        }
        const iterator = blobParts[Symbol.iterator];
        if (typeof iterator !== "function") throw new TypeError("blobParts must be a sequence");
        if (options == null) options = {};
        else if (typeof options !== "object" && typeof options !== "function") {
            throw new TypeError("options must be a dictionary");
        }
        const endings = options.endings === undefined ? "transparent" : String(options.endings);
        if (endings !== "transparent" && endings !== "native") {
            throw new TypeError("endings must be 'transparent' or 'native'");
        }
        const chunks = [];
        let size = 0;
        for (const part of blobParts) {
            const bytes = __blobPartBytes(part, endings);
            chunks.push(bytes);
            size += bytes.byteLength;
        }
        this.__bytes = new Uint8Array(size);
        let offset = 0;
        for (const chunk of chunks) {
            this.__bytes.set(chunk, offset);
            offset += chunk.byteLength;
        }
        this.__type = __parseMimeType(options.type === undefined ? "" : options.type);
    }
    get size() { return this.__bytes.byteLength; }
    get type() { return this.__type; }
    slice(start = 0, end = this.size, contentType = "") {
        const size = this.size;
        start = Number(start);
        end = Number(end);
        start = Number.isNaN(start) ? 0 : Math.trunc(start);
        end = Number.isNaN(end) ? 0 : Math.trunc(end);
        const first = start < 0 ? Math.max(size + start, 0) : Math.min(start, size);
        const last = end < 0 ? Math.max(size + end, 0) : Math.min(end, size);
        return new Blob([this.__bytes.slice(first, Math.max(first, last))], { type: contentType });
    }
    arrayBuffer() { return Promise.resolve(this.__bytes.slice().buffer); }
    bytes() { return Promise.resolve(this.__bytes.slice()); }
    text() { return Promise.resolve(new TextDecoder().decode(this.__bytes)); }
    get [Symbol.toStringTag]() { return "Blob"; }
}
globalThis.Blob = Blob;

class File extends Blob {
    constructor(fileBits, fileName, options = {}) {
        if (arguments.length < 2) throw new TypeError("File requires fileBits and fileName");
        if (options == null) options = {};
        super(fileBits, options);
        this.__name = __toUSVString(fileName);
        const lastModified = options.lastModified === undefined ? Date.now() : Number(options.lastModified);
        this.__lastModified = Number.isFinite(lastModified) ? Math.trunc(lastModified) : 0;
    }
    get name() { return this.__name; }
    get lastModified() { return this.__lastModified; }
    get webkitRelativePath() { return ""; }
    get [Symbol.toStringTag]() { return "File"; }
}
globalThis.File = File;

