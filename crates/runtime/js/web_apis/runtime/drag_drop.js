const __dataTransferFiles = new WeakMap();
class DataTransferItem {
    constructor(kind, type, value) { this.kind = kind; this.type = type; this.__value = value; }
    getAsFile() { return this.kind === "file" ? this.__value : null; }
    getAsString(callback) { if (this.kind === "string" && typeof callback === "function") setTimeout(() => callback(this.__value), 0); }
}
class DataTransferItemList {
    constructor(owner) { this.__owner = owner; this.__items = []; }
    get length() { return this.__items.length; }
    [Symbol.iterator]() { return this.__items[Symbol.iterator](); }
    __sync() {
        for (const key of Object.keys(this)) if (/^\d+$/.test(key)) delete this[key];
        this.__items.forEach((item, index) => Object.defineProperty(this, index, {value:item, configurable:true, enumerable:true}));
        const files = __dataTransferFiles.get(this.__owner);
        if (files) __replaceFileList(files,this.__items.filter(item=>item.kind==='file').map(item=>item.getAsFile()));
    }
    add(value, type) {
        if (this.__owner.__readOnly) return null;
        const file = value instanceof File;
        type = file ? value.type : String(type).toLowerCase();
        if (!file && this.__items.some(i => i.kind === "string" && i.type === type)) throw new DOMException("Duplicate item type", "NotSupportedError");
        const item = new DataTransferItem(file ? "file" : "string", type, file ? value : String(value));
        this.__items.push(item); this.__sync(); return item;
    }
    remove(index) { if (!this.__owner.__readOnly) { this.__items.splice(Number(index) >>> 0, 1); this.__sync(); } }
    clear() { if (!this.__owner.__readOnly) { this.__items = []; this.__sync(); } }
}
class DataTransfer {
    constructor() { this.dropEffect = "none"; this.effectAllowed = "uninitialized"; this.__readOnly = false; this.items = new DataTransferItemList(this); }
    get types() { return [...new Set(Array.from(this.items, i => i.kind === "file" ? "Files" : i.type))]; }
    get files() {
        if (!__dataTransferFiles.has(this)) __dataTransferFiles.set(this,__fileList(Array.from(this.items).filter(i=>i.kind==='file').map(i=>i.getAsFile())));
        return __dataTransferFiles.get(this);
    }
    __format(type) { type = String(type).toLowerCase(); return type === "text" ? "text/plain" : type === "url" ? "text/uri-list" : type; }
    getData(type) { return this.items.__items.find(i => i.kind === "string" && i.type === this.__format(type))?.__value ?? ""; }
    setData(type, value) {
        if (this.__readOnly) return;
        type = this.__format(type);
        const item = this.items.__items.find(i => i.kind === "string" && i.type === type);
        if (item) item.__value = String(value); else this.items.add(String(value), type);
    }
    clearData(type) {
        if (this.__readOnly) return;
        this.items.__items = this.items.__items.filter(i => i.kind === "file" || (type !== undefined && i.type !== this.__format(type)));
        this.items.__sync();
    }
    setDragImage(element, x, y) { this.__image = {element,x,y}; }
}
class DragEvent extends MouseEvent {
    constructor(type, options = {}) { super(type, options); this.dataTransfer = options.dataTransfer ?? null; }
}
globalThis.DataTransfer = DataTransfer;
globalThis.DataTransferItem = DataTransferItem;
globalThis.DataTransferItemList = DataTransferItemList;
globalThis.DragEvent = DragEvent;
