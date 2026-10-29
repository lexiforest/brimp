(() => {
  'use strict';
  const host = globalThis.__brimpStreamingHost;
  const call = (operation, ...arguments_) => host(operation, globalThis, ...arguments_);
  const sockets = new Map();

  Object.defineProperty(Response.prototype, 'body', {
    get() {
      if (this.__bodyStream === undefined) {
        const bytes = this.__bytes.slice();
        this.__bodyStream = new ReadableStream({
          start(controller) {
            controller.enqueue(bytes);
            controller.close();
          },
        });
      }
      return this.__bodyStream;
    },
    enumerable: true,
    configurable: true,
  });
  async function consumeResponseBody(response) {
    if (response.bodyUsed) throw new TypeError('body has already been consumed');
    response.bodyUsed = true;
    const chunks = [];
    let length = 0;
    const reader = response.body.getReader();
    for (;;) {
      const result = await reader.read();
      if (result.done) break;
      const chunk =
        result.value instanceof Uint8Array ? result.value : new Uint8Array(result.value);
      chunks.push(chunk);
      length += chunk.byteLength;
    }
    const bytes = new Uint8Array(length);
    let offset = 0;
    for (const chunk of chunks) {
      bytes.set(chunk, offset);
      offset += chunk.byteLength;
    }
    return bytes;
  }
  const cloneResponse = Response.prototype.clone;
  Object.defineProperties(Response.prototype, {
    clone: {
      value: function clone() {
        if (this.bodyUsed || this.__bodyStream?.locked)
          throw new TypeError('body has already been consumed');
        const result = cloneResponse.call(this);
        if (this.__bodyStream !== undefined) {
          const [original, copy] = this.__bodyStream.tee();
          this.__bodyStream = original;
          result.__bodyStream = copy;
        }
        return result;
      },
      writable: true,
      enumerable: true,
      configurable: true,
    },
    text: {
      value: function text() {
        return consumeResponseBody(this).then((bytes) => new TextDecoder().decode(bytes));
      },
      writable: true,
      enumerable: true,
      configurable: true,
    },
    json: {
      value: function json() {
        return this.text().then(JSON.parse);
      },
      writable: true,
      enumerable: true,
      configurable: true,
    },
    arrayBuffer: {
      value: function arrayBuffer() {
        return consumeResponseBody(this).then((bytes) => bytes.buffer);
      },
      writable: true,
      enumerable: true,
      configurable: true,
    },
    blob: {
      value: function blob() {
        return consumeResponseBody(this).then(
          (bytes) => new Blob([bytes], { type: this.headers.get('content-type') || '' }),
        );
      },
      writable: true,
      enumerable: true,
      configurable: true,
    },
  });

  const fetchStreams = new Map();
  const cancelledFetchStreams = new Set();
  globalThis.fetch = (input, init = {}) => {
    let request;
    try {
      request = new Request(input, init);
    } catch (error) {
      return Promise.reject(error);
    }
    return call(
      'fetchStream',
      request.url,
      request.method,
      JSON.stringify([...request.headers]),
      request.__bodyBytes === null ? null : JSON.stringify(Array.from(request.__bodyBytes)),
      request.referrer === 'about:client' ? null : request.referrer,
    ).then(
      (serialized) => {
        const payload = JSON.parse(serialized);
        const state = fetchStreams.get(payload.streamId) || { events: [] };
        state.id = payload.streamId;
        const stream = new ReadableStream({
          type: 'bytes',
          start(controller) {
            state.controller = controller;
          },
          pull() {
            return new Promise((resolve) => {
              state.resolve = resolve;
              drainFetch(state);
            });
          },
          cancel() {
            state.cancelled = true;
            state.events.length = 0;
            state.resolve?.();
            state.resolve = null;
            fetchStreams.delete(payload.streamId);
            cancelledFetchStreams.add(payload.streamId);
            call('fetchStreamCancel', payload.streamId);
          },
        });
        fetchStreams.set(payload.streamId, state);
        const response = new Response('', payload);
        response.__bodyStream = stream;
        return response;
      },
      (reason) => {
        throw new TypeError(String(reason));
      },
    );
  };

  function deliverFetchEvent(state, event) {
    if (state.cancelled) return;
    state.events.push(event);
    if (event.type === 'error' && state.controller) {
      state.events.length = 0;
      state.controller.error(new TypeError(String(event.message)));
      state.resolve?.();
      state.resolve = null;
      return;
    }
    drainFetch(state);
  }
  function closeFetch(state) {
    state.controller.close();
    // A pending BYOB request needs an explicit zero-byte response at EOF.
    state.controller.byobRequest?.respond(0);
  }
  function drainFetch(state) {
    if (!state.resolve || !state.events.length) return;
    const event = state.events.shift(),
      resolve = state.resolve;
    state.resolve = null;
    if (event.type === 'chunk') {
      state.controller.enqueue(new Uint8Array(event.bytes));
      call('fetchStreamConsumed', state.id, event.bytes.length);
      if (state.events[0]?.type === 'complete') {
        state.events.shift();
        closeFetch(state);
      }
    } else if (event.type === 'complete') closeFetch(state);
    else state.controller.error(new TypeError(String(event.message)));
    resolve();
  }

  class CloseEvent extends Event {
    constructor(type, options = {}) {
      super(type, options);
      this.wasClean = Boolean(options.wasClean);
      this.code = Number(options.code ?? 0);
      this.reason = String(options.reason ?? '');
    }
  }

  class WebSocket extends EventTarget {
    constructor(url, protocols = []) {
      super();
      if (arguments.length === 0) throw new TypeError('WebSocket URL is required');
      const parsed = new URL(String(url), location.href);
      if (parsed.protocol === 'http:') parsed.protocol = 'ws:';
      else if (parsed.protocol === 'https:') parsed.protocol = 'wss:';
      if (parsed.protocol !== 'ws:' && parsed.protocol !== 'wss:')
        throw new DOMException('Invalid WebSocket URL', 'SyntaxError');
      parsed.hash = '';
      this.url = parsed.href;
      this.readyState = WebSocket.CONNECTING;
      this.bufferedAmount = 0;
      this.extensions = '';
      this.protocol = Array.isArray(protocols) ? String(protocols[0] ?? '') : String(protocols);
      this.binaryType = 'blob';
      this.onopen = null;
      this.onmessage = null;
      this.onerror = null;
      this.onclose = null;
      this.__id = call('webSocketCreate', this.url);
      sockets.set(this.__id, this);
    }
    send(data) {
      if (this.readyState === WebSocket.CONNECTING)
        throw new DOMException('WebSocket is connecting', 'InvalidStateError');
      if (this.readyState !== WebSocket.OPEN) return;
      let bytes;
      if (data instanceof Blob) bytes = data.__bytes;
      else if (data instanceof ArrayBuffer) bytes = new Uint8Array(data);
      else if (ArrayBuffer.isView(data))
        bytes = new Uint8Array(data.buffer, data.byteOffset, data.byteLength);
      if (bytes !== undefined)
        call('webSocketSendBinary', this.__id, JSON.stringify(Array.from(bytes)));
      else call('webSocketSend', this.__id, String(data));
    }
    close() {
      if (this.readyState >= WebSocket.CLOSING) return;
      this.readyState = WebSocket.CLOSING;
      call('webSocketClose', this.__id);
    }
  }
  WebSocket.CONNECTING = 0;
  WebSocket.OPEN = 1;
  WebSocket.CLOSING = 2;
  WebSocket.CLOSED = 3;
  Object.assign(WebSocket.prototype, { CONNECTING: 0, OPEN: 1, CLOSING: 2, CLOSED: 3 });

  class EventSource extends EventTarget {
    constructor(url, options = {}) {
      super();
      this.url = new URL(String(url), location.href).href;
      this.withCredentials = Boolean(options.withCredentials);
      this.readyState = EventSource.CONNECTING;
      this.onopen = null;
      this.onmessage = null;
      this.onerror = null;
      this.__closed = false;
      fetch(this.url, { headers: { Accept: 'text/event-stream' } })
        .then(async (response) => {
          if (this.__closed) return;
          this.readyState = EventSource.OPEN;
          this.dispatchEvent(new Event('open'));
          let data = [],
            type = 'message',
            id = '';
          let buffered = '';
          const decoder = new TextDecoder();
          const reader = response.body.getReader();
          this.__reader = reader;
          const processLine = (line) => {
            if (line === '') {
              if (data.length)
                this.dispatchEvent(
                  new MessageEvent(type, {
                    data: data.join('\n'),
                    lastEventId: id,
                    origin: new URL(this.url).origin,
                  }),
                );
              data = [];
              type = 'message';
              return;
            }
            if (line.startsWith('data:')) data.push(line.slice(5).replace(/^ /, ''));
            else if (line.startsWith('event:')) type = line.slice(6).trim();
            else if (line.startsWith('id:')) id = line.slice(3).trim();
          };
          for (;;) {
            const result = await reader.read();
            buffered += decoder.decode(result.value ?? new Uint8Array(), {
              stream: !result.done,
            });
            const lines = buffered.split(/\r?\n/);
            buffered = result.done ? '' : lines.pop();
            for (const line of lines) processLine(line);
            if (result.done) break;
          }
        })
        .catch(() => {
          if (!this.__closed) this.dispatchEvent(new Event('error'));
        });
    }
    close() {
      this.__closed = true;
      this.readyState = EventSource.CLOSED;
      this.__reader?.cancel();
      this.__reader = null;
    }
  }
  EventSource.CONNECTING = 0;
  EventSource.OPEN = 1;
  EventSource.CLOSED = 2;
  Object.assign(EventSource.prototype, { CONNECTING: 0, OPEN: 1, CLOSED: 2 });

  globalThis.WebSocket = WebSocket;
  globalThis.CloseEvent = CloseEvent;
  globalThis.EventSource = EventSource;
  Object.defineProperty(globalThis, '__brimpDeliverWebSocket', {
    value(serialized) {
      const delivery = JSON.parse(serialized);
      const socket = sockets.get(Number(delivery.id));
      if (!socket) return;
      const event = JSON.parse(delivery.event);
      if (event.type === 'open') {
        socket.readyState = WebSocket.OPEN;
        socket.dispatchEvent(new Event('open'));
      } else if (event.type === 'message') {
        const data = event.binary
          ? socket.binaryType === 'arraybuffer'
            ? Uint8Array.from(event.data).buffer
            : new Blob([Uint8Array.from(event.data)])
          : event.data;
        socket.dispatchEvent(
          new MessageEvent('message', { data, origin: new URL(socket.url).origin }),
        );
      } else if (event.type === 'close') {
        socket.readyState = WebSocket.CLOSED;
        sockets.delete(socket.__id);
        socket.dispatchEvent(new CloseEvent('close', event));
      } else socket.dispatchEvent(new Event('error'));
    },
    configurable: true,
  });
  Object.defineProperty(globalThis, '__brimpDeliverFetchStream', {
    value(serialized) {
      const delivery = JSON.parse(serialized);
      const id = Number(delivery.id);
      if (cancelledFetchStreams.has(id)) {
        const event = JSON.parse(delivery.event);
        if (event.type === 'complete' || event.type === 'error') cancelledFetchStreams.delete(id);
        return;
      }
      const state = fetchStreams.get(id) || { events: [] };
      fetchStreams.set(id, state);
      const event = JSON.parse(delivery.event);
      deliverFetchEvent(state, event);
      if ((event.type === 'complete' || event.type === 'error') && state.controller)
        fetchStreams.delete(id);
    },
    configurable: true,
  });
})();
