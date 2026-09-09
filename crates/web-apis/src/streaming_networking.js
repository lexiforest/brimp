(() => {
"use strict";
const host = globalThis.__brimpStreamingHost;
const call = (operation, ...arguments_) => host(operation, globalThis, ...arguments_);
const sockets = new Map();

class ReadableStreamDefaultReader {
    constructor(stream) { this.__stream = stream; this.closed = Promise.resolve(); }
    read() {
        const stream = this.__stream;
        if (stream.__chunks.length) return Promise.resolve({ value: stream.__chunks.shift(), done: false });
        if (stream.__error !== undefined) return Promise.reject(stream.__error);
        if (stream.__closed) return Promise.resolve({ value: undefined, done: true });
        return new Promise((resolve, reject) => stream.__pending.push({ resolve, reject }));
    }
    cancel(reason) { this.__stream.__locked = false; return this.__stream.cancel(reason); }
    releaseLock() { this.__stream.__locked = false; }
}

class ReadableStream {
    constructor(source = {}) {
        this.__chunks = [];
        this.__locked = false;
        this.__pending = [];
        this.__closed = false;
        this.__error = undefined;
        this.__cancel = source.cancel;
        const controller = {
            enqueue: chunk => {
                const pending = this.__pending.shift();
                if (pending) pending.resolve({ value: chunk, done: false });
                else this.__chunks.push(chunk);
            },
            close: () => {
                this.__closed = true;
                for (const pending of this.__pending.splice(0)) pending.resolve({ value: undefined, done: true });
            },
            error: error => {
                this.__error = error;
                for (const pending of this.__pending.splice(0)) pending.reject(error);
            },
            get desiredSize() { return 1; },
        };
        source.start?.(controller);
    }
    get locked() { return this.__locked; }
    getReader() {
        if (this.__locked) throw new TypeError("ReadableStream is locked");
        this.__locked = true;
        return new ReadableStreamDefaultReader(this);
    }
    cancel(reason) { this.__chunks.length = 0; this.__closed = true; return Promise.resolve(this.__cancel?.(reason)); }
    tee() {
        const chunks = this.__chunks.slice();
        return [new ReadableStream({ start: controller => chunks.forEach(chunk => controller.enqueue(chunk)) }), new ReadableStream({ start: controller => chunks.forEach(chunk => controller.enqueue(chunk)) })];
    }
    async pipeTo(destination) {
        const writer = destination.getWriter();
        for await (const chunk of this) await writer.write(chunk);
        await writer.close();
    }
    pipeThrough(transform) { this.pipeTo(transform.writable); return transform.readable; }
    async *[Symbol.asyncIterator]() {
        const reader = this.getReader();
        try { for (;;) { const result = await reader.read(); if (result.done) return; yield result.value; } }
        finally { reader.releaseLock(); }
    }
}

class WritableStream {
    constructor(sink = {}) { this.__sink = sink; this.__locked = false; }
    get locked() { return this.__locked; }
    getWriter() {
        if (this.__locked) throw new TypeError("WritableStream is locked");
        this.__locked = true;
        return {
            ready: Promise.resolve(), closed: Promise.resolve(), desiredSize: 1,
            write: chunk => Promise.resolve(this.__sink.write?.(chunk)),
            close: () => Promise.resolve(this.__sink.close?.()),
            abort: reason => Promise.resolve(this.__sink.abort?.(reason)),
            releaseLock: () => { this.__locked = false; },
        };
    }
    abort(reason) { return Promise.resolve(this.__sink.abort?.(reason)); }
    close() { return Promise.resolve(this.__sink.close?.()); }
}

class TransformStream {
    constructor(transformer = {}) {
        const chunks = [];
        const controller = { enqueue: chunk => chunks.push(chunk), error(error) { throw error; }, terminate() {} };
        this.writable = new WritableStream({ write: chunk => transformer.transform ? transformer.transform(chunk, controller) : controller.enqueue(chunk), close: () => transformer.flush?.(controller) });
        this.readable = new ReadableStream({ start: readable => {
            const enqueue = controller.enqueue;
            controller.enqueue = chunk => { chunks.push(chunk); readable.enqueue(chunk); };
            for (const chunk of chunks) readable.enqueue(chunk);
            controller.enqueue = controller.enqueue || enqueue;
        }});
    }
}

globalThis.ReadableStream = ReadableStream;
globalThis.ReadableStreamDefaultReader = ReadableStreamDefaultReader;
globalThis.WritableStream = WritableStream;
globalThis.TransformStream = TransformStream;
Object.defineProperty(Response.prototype, "body", {
    get() {
        if (this.__bodyStream === undefined) {
            const bytes = new TextEncoder().encode(this.__body);
            this.__bodyStream = new ReadableStream({ start(controller) { controller.enqueue(bytes); } });
        }
        return this.__bodyStream;
    }, enumerable: true, configurable: true,
});
async function consumeResponseBody(response) {
    if (response.bodyUsed) throw new TypeError("body has already been consumed");
    response.bodyUsed = true;
    const chunks = [];
    let length = 0;
    const reader = response.body.getReader();
    for (;;) {
        const result = await reader.read();
        if (result.done) break;
        const chunk = result.value instanceof Uint8Array ? result.value : new Uint8Array(result.value);
        chunks.push(chunk); length += chunk.byteLength;
    }
    const bytes = new Uint8Array(length);
    let offset = 0;
    for (const chunk of chunks) { bytes.set(chunk, offset); offset += chunk.byteLength; }
    return bytes;
}
Object.defineProperties(Response.prototype, {
    text: { value: function text() { return consumeResponseBody(this).then(bytes => new TextDecoder().decode(bytes)); }, writable: true, enumerable: true, configurable: true },
    json: { value: function json() { return this.text().then(JSON.parse); }, writable: true, enumerable: true, configurable: true },
    arrayBuffer: { value: function arrayBuffer() { return consumeResponseBody(this).then(bytes => bytes.buffer); }, writable: true, enumerable: true, configurable: true },
    blob: { value: function blob() { return consumeResponseBody(this).then(bytes => new Blob([bytes], { type: this.headers.get("content-type") || "" })); }, writable: true, enumerable: true, configurable: true },
});

const fetchStreams = new Map();
const cancelledFetchStreams = new Set();
globalThis.fetch = (input, init = {}) => {
    let request;
    try { request = new Request(input, init); }
    catch (error) { return Promise.reject(error); }
    return call(
        "fetchStream",
        request.url,
        request.method,
        JSON.stringify([...request.headers]),
        request.__bodyBytes === null ? null : JSON.stringify(Array.from(request.__bodyBytes)),
    ).then(serialized => {
        const payload = JSON.parse(serialized);
        const state = fetchStreams.get(payload.streamId) || { events: [] };
        const stream = new ReadableStream({
            start(controller) {
                state.controller = controller;
                for (const event of state.events.splice(0)) deliverFetchEvent(state, event);
            },
            cancel() {
                state.cancelled = true;
                state.events.length = 0;
                fetchStreams.delete(payload.streamId);
                cancelledFetchStreams.add(payload.streamId);
                call("fetchStreamCancel", payload.streamId);
            },
        });
        fetchStreams.set(payload.streamId, state);
        const response = new Response("", payload);
        response.__bodyStream = stream;
        return response;
    }, reason => { throw new TypeError(String(reason)); });
};

function deliverFetchEvent(state, event) {
    if (state.cancelled) return;
    if (!state.controller) { state.events.push(event); return; }
    if (event.type === "chunk") state.controller.enqueue(new Uint8Array(event.bytes));
    else if (event.type === "complete") state.controller.close();
    else state.controller.error(new TypeError(String(event.message)));
}

class CloseEvent extends Event {
    constructor(type, options = {}) {
        super(type, options);
        this.wasClean = Boolean(options.wasClean);
        this.code = Number(options.code ?? 0);
        this.reason = String(options.reason ?? "");
    }
}

class WebSocket extends EventTarget {
    constructor(url, protocols = []) {
        super();
        if (arguments.length === 0) throw new TypeError("WebSocket URL is required");
        const parsed = new URL(String(url), location.href);
        if (parsed.protocol === "http:") parsed.protocol = "ws:";
        else if (parsed.protocol === "https:") parsed.protocol = "wss:";
        if (parsed.protocol !== "ws:" && parsed.protocol !== "wss:") throw new DOMException("Invalid WebSocket URL", "SyntaxError");
        parsed.hash = "";
        this.url = parsed.href;
        this.readyState = WebSocket.CONNECTING;
        this.bufferedAmount = 0;
        this.extensions = "";
        this.protocol = Array.isArray(protocols) ? String(protocols[0] ?? "") : String(protocols);
        this.binaryType = "blob";
        this.onopen = null;
        this.onmessage = null;
        this.onerror = null;
        this.onclose = null;
        this.__id = call("webSocketCreate", this.url);
        sockets.set(this.__id, this);
    }
    send(data) {
        if (this.readyState === WebSocket.CONNECTING) throw new DOMException("WebSocket is connecting", "InvalidStateError");
        if (this.readyState !== WebSocket.OPEN) return;
        call("webSocketSend", this.__id, String(data));
    }
    close() {
        if (this.readyState >= WebSocket.CLOSING) return;
        this.readyState = WebSocket.CLOSING;
        call("webSocketClose", this.__id);
    }
}
WebSocket.CONNECTING = 0; WebSocket.OPEN = 1; WebSocket.CLOSING = 2; WebSocket.CLOSED = 3;
Object.assign(WebSocket.prototype, { CONNECTING: 0, OPEN: 1, CLOSING: 2, CLOSED: 3 });

class EventSource extends EventTarget {
    constructor(url, options = {}) {
        super();
        this.url = new URL(String(url), location.href).href;
        this.withCredentials = Boolean(options.withCredentials);
        this.readyState = EventSource.CONNECTING;
        this.onopen = null; this.onmessage = null; this.onerror = null;
        this.__closed = false;
        fetch(this.url, { headers: { Accept: "text/event-stream" } }).then(async response => {
            if (this.__closed) return;
            this.readyState = EventSource.OPEN;
            this.dispatchEvent(new Event("open"));
            let data = [], type = "message", id = "";
            let buffered = "";
            const decoder = new TextDecoder();
            const reader = response.body.getReader();
            this.__reader = reader;
            const processLine = line => {
                if (line === "") {
                    if (data.length) this.dispatchEvent(new MessageEvent(type, { data: data.join("\n"), lastEventId: id, origin: new URL(this.url).origin }));
                    data = []; type = "message"; return;
                }
                if (line.startsWith("data:")) data.push(line.slice(5).replace(/^ /, ""));
                else if (line.startsWith("event:")) type = line.slice(6).trim();
                else if (line.startsWith("id:")) id = line.slice(3).trim();
            };
            for (;;) {
                const result = await reader.read();
                buffered += decoder.decode(result.value ?? new Uint8Array(), { stream: !result.done });
                const lines = buffered.split(/\r?\n/);
                buffered = result.done ? "" : lines.pop();
                for (const line of lines) processLine(line);
                if (result.done) break;
            }
        }).catch(() => { if (!this.__closed) this.dispatchEvent(new Event("error")); });
    }
    close() {
        this.__closed = true;
        this.readyState = EventSource.CLOSED;
        this.__reader?.cancel();
        this.__reader = null;
    }
}
EventSource.CONNECTING = 0; EventSource.OPEN = 1; EventSource.CLOSED = 2;
Object.assign(EventSource.prototype, { CONNECTING: 0, OPEN: 1, CLOSED: 2 });

globalThis.WebSocket = WebSocket;
globalThis.CloseEvent = CloseEvent;
globalThis.EventSource = EventSource;
Object.defineProperty(globalThis, "__brimpDeliverWebSocket", {
    value(serialized) {
        const delivery = JSON.parse(serialized);
        const socket = sockets.get(Number(delivery.id));
        if (!socket) return;
        const event = JSON.parse(delivery.event);
        if (event.type === "open") { socket.readyState = WebSocket.OPEN; socket.dispatchEvent(new Event("open")); }
        else if (event.type === "message") socket.dispatchEvent(new MessageEvent("message", { data: event.data, origin: new URL(socket.url).origin }));
        else if (event.type === "close") { socket.readyState = WebSocket.CLOSED; sockets.delete(socket.__id); socket.dispatchEvent(new CloseEvent("close", event)); }
        else socket.dispatchEvent(new Event("error"));
    }, configurable: true,
});
Object.defineProperty(globalThis, "__brimpDeliverFetchStream", {
    value(serialized) {
        const delivery = JSON.parse(serialized);
        const id = Number(delivery.id);
        if (cancelledFetchStreams.has(id)) {
            const event = JSON.parse(delivery.event);
            if (event.type === "complete" || event.type === "error") cancelledFetchStreams.delete(id);
            return;
        }
        const state = fetchStreams.get(id) || { events: [] };
        fetchStreams.set(id, state);
        const event = JSON.parse(delivery.event);
        deliverFetchEvent(state, event);
        if ((event.type === "complete" || event.type === "error") && state.controller) fetchStreams.delete(id);
    }, configurable: true,
});
for (const constructor of [ReadableStream, ReadableStreamDefaultReader, WritableStream, TransformStream, CloseEvent, WebSocket, EventSource]) {
    globalThis.__brimpMarkWebBuiltin?.(constructor);
    for (const key of Reflect.ownKeys(constructor.prototype)) {
        const descriptor = Object.getOwnPropertyDescriptor(constructor.prototype, key);
        if (typeof descriptor?.value === "function") globalThis.__brimpMarkWebBuiltin?.(descriptor.value);
        if (typeof descriptor?.get === "function") globalThis.__brimpMarkWebBuiltin?.(descriptor.get, `function get ${String(key)}() { [native code] }`);
    }
}
})();
