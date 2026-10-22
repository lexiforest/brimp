(descriptors) => {
    const native = globalThis.__brimpPluginHost;
    delete globalThis.__brimpPluginHost;
    const PublicError = DOMException;
    let ready = false, error = null, closed = false, disposed = false, disposing = false;
    const subscriptions = descriptors.map(() => new Map());
    const disposers = [];
    function encode(value, seen = new Set()) {
        if (value === null) return {type: 'null'};
        if (typeof value === 'boolean') return {type: 'boolean', value};
        if (typeof value === 'string') return {type: 'string', value};
        if (typeof value === 'number' && Number.isFinite(value)) return {type: 'number', value};
        if (value instanceof ArrayBuffer || ArrayBuffer.isView(value)) {
            const bytes = value instanceof ArrayBuffer ? new Uint8Array(value) : new Uint8Array(value.buffer, value.byteOffset, value.byteLength);
            return {type: 'binary', value: Array.from(bytes)};
        }
        if (typeof value !== 'object' || value === null || seen.has(value)) throw new TypeError('host values must be acyclic JSON-like values or binary buffers');
        if (!Array.isArray(value) && Object.getPrototypeOf(value) !== Object.prototype && Object.getPrototypeOf(value) !== null) throw new TypeError('host values cannot contain DOM nodes or class instances');
        seen.add(value);
        const result = Array.isArray(value)
            ? {type: 'array', value: value.map(item => encode(item, seen))}
            : {type: 'object', value: Object.fromEntries(Object.entries(value).map(([key, item]) => [key, encode(item, seen)]))};
        seen.delete(value);
        return result;
    }
    function decode(wire) {
        switch (wire.type) {
            case 'null': return null;
            case 'binary': return Uint8Array.from(wire.value);
            case 'array': return wire.value.map(decode);
            case 'object': return Object.fromEntries(Object.entries(wire.value).map(([key, value]) => [key, decode(value)]));
            case 'string': case 'number': case 'boolean': return wire.value;
            default: throw new TypeError('invalid host value');
        }
    }
    (async () => {
        for (let index = 0; index < descriptors.length; index++) {
            const descriptor = descriptors[index];
            try {
                if (closed) throw new Error('plugin realm was closed');
                const allowed = new Set(descriptor.handlers);
                const host = Object.freeze({
                    call(name, ...args) {
                        return Promise.resolve().then(() => {
                            if (closed && !disposing) throw new Error('plugin realm was closed');
                            if (!allowed.has(name)) throw new Error(`unregistered host handler: ${name}`);
                            return native(index, name, JSON.stringify(args.map(value => encode(value))));
                        }).then(wire => decode(JSON.parse(wire)), () => {
                            // Native code records the full failure for the host.
                            // Never pass adapter exceptions or host stacks to pages.
                            throw new PublicError('The operation failed.', 'OperationError');
                        });
                    },
                    subscribe(channel, listener) {
                        if (closed) throw new Error('plugin realm was closed');
                        if (typeof listener !== 'function') throw new TypeError('subscription listener must be callable');
                        channel = String(channel);
                        let listeners = subscriptions[index].get(channel);
                        if (!listeners) subscriptions[index].set(channel, listeners = new Set());
                        listeners.add(listener);
                        return () => { listeners.delete(listener); if (!listeners.size) subscriptions[index].delete(channel); };
                    },
                });
                // The final directive overrides file/source-map paths in trusted
                // installers. Keep their original locations only in host errors.
                const installer = (0, eval)(`(${descriptor.source}\n)\n//# sourceURL=web-api`);
                if (typeof installer !== 'function') throw new TypeError('plugin source must evaluate to an installer');
                const installed = await installer({global: globalThis, host});
                const dispose = typeof installed === 'function' ? installed : installed?.dispose?.bind(installed);
                if (dispose) {
                    if (closed) await dispose(); else disposers.push(dispose);
                }
            } catch (cause) {
                const locations = Array.from(descriptor.source.matchAll(/\/\/[#@]\s*sourceURL=([^\r\n]+)/g), match => match[1]);
                throw new Error(`Plugin ${descriptor.name}${locations.length ? ` (${locations.join(', ')})` : ''}: ${String(cause)}\n${cause?.stack || ""}`);
            }
        }
        ready = true;
    })().catch(cause => { error = `${String(cause)}\n${cause?.stack || ""}`; ready = true; });
    return serialized => {
        const command = JSON.parse(serialized);
        if (command.op === 'dispose' && !closed) {
            closed = true;
            disposing = true;
            for (const channels of subscriptions) channels.clear();
            (async () => {
                for (const dispose of disposers.reverse()) {
                    try { await dispose(); } catch (cause) { error = `${String(cause)}\n${cause?.stack || ""}`; }
                }
                disposers.length = 0;
                disposing = false;
                disposed = true;
            })();
        } else if (command.op === 'notify' && !closed) {
            const {plugin, channel, value} = command.notification;
            for (const listener of subscriptions[plugin]?.get(channel) ?? []) {
                try { listener(decode(value)); } catch (cause) { setTimeout(() => { throw cause; }, 0); }
            }
        }
        return JSON.stringify({ready, error, disposed});
    };
}
