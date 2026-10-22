// Reads are queued tasks, including cancellation and reentrant reads from handlers.
class FileReader extends EventTarget {
    #state = 0;
    #result = null;
    #error = null;
    #generation = 0;
    constructor() {
        super();
        for (const name of ['loadstart','progress','load','abort','error','loadend']) this[`on${name}`] = null;
    }
    get readyState() { return this.#state; }
    get result() { return this.#result; }
    get error() { return this.#error; }
    readAsArrayBuffer(blob) { this.#read(blob,'buffer'); }
    readAsText(blob, encoding) { this.#read(blob,'text',encoding); }
    readAsDataURL(blob) { this.#read(blob,'url'); }
    readAsBinaryString(blob) { this.#read(blob,'binary'); }
    abort() {
        this.#result = null;
        if (this.#state !== FileReader.LOADING) return;
        this.#state = FileReader.DONE;
        this.#generation++;
        this.dispatchEvent(__markTrustedEvent(new ProgressEvent('abort')));
        if (this.#state !== FileReader.LOADING) this.dispatchEvent(__markTrustedEvent(new ProgressEvent('loadend')));
    }
    #read(blob, kind, encoding) {
        if (!(blob instanceof Blob)) throw new TypeError('Expected a Blob');
        if (this.#state === FileReader.LOADING) throw new DOMException('A read is in progress','InvalidStateError');
        this.#state = FileReader.LOADING; this.#result = null; this.#error = null;
        const generation = ++this.#generation;
        const bytes = blob.__bytes.slice(), type = blob.type;
        const valid = () => this.#generation === generation && this.#state === FileReader.LOADING;
        const emit = name => this.dispatchEvent(__markTrustedEvent(new ProgressEvent(name,{lengthComputable:true,loaded:name === 'loadstart' ? 0 : bytes.length,total:bytes.length})));
        setTimeout(() => {
            if (!valid()) return;
            emit('loadstart');
            if (!valid()) return;
            setTimeout(() => {
                if (!valid()) return;
                try {
                    let result;
                    if (kind === 'buffer') result = bytes.buffer;
                    else if (kind === 'text') {
                        const label = encoding ?? /charset\s*=\s*([^;]+)/i.exec(type)?.[1]?.replace(/^"|"$/g,'') ?? 'utf-8';
                        let decoder;
                        try { decoder = new TextDecoder(label); } catch { decoder = new TextDecoder(); }
                        // File API text decoding honors BOMs before the requested encoding.
                        if (bytes[0] === 0xff && bytes[1] === 0xfe) decoder = new TextDecoder('utf-16le');
                        else if (bytes[0] === 0xfe && bytes[1] === 0xff) decoder = new TextDecoder('utf-16be');
                        else if (bytes[0] === 0xef && bytes[1] === 0xbb && bytes[2] === 0xbf) decoder = new TextDecoder('utf-8');
                        result = decoder.decode(bytes);
                    } else {
                        let binary='';for(let i=0;i<bytes.length;i+=8192) binary+=String.fromCharCode(...bytes.subarray(i,i+8192));
                        result = kind === 'binary' ? binary : `data:${type || 'application/octet-stream'};base64,${btoa(binary)}`;
                    }
                    emit('progress');
                    if (!valid()) return;
                    this.#state = FileReader.DONE; this.#result = result;
                    emit('load');
                } catch (error) {
                    if (!valid()) return;
                    this.#state = FileReader.DONE; this.#result=null;
                    this.#error = new DOMException(String(error.message),'NotReadableError');emit('error');
                }
                if (this.#generation === generation && this.#state !== FileReader.LOADING) emit('loadend');
            },0);
        },0);
    }
}
for (const [name,value] of Object.entries({EMPTY:0,LOADING:1,DONE:2})) {
    Object.defineProperty(FileReader,name,{value,enumerable:true});
    Object.defineProperty(FileReader.prototype,name,{value,enumerable:true});
}
Object.defineProperty(FileReader.prototype,Symbol.toStringTag,{value:'FileReader',configurable:true});
globalThis.FileReader = FileReader;
