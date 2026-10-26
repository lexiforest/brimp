(realm) => {
    const roots = new Map();
    const references = new WeakMap();
    let next = 0, callbackOperations = 0;
    const retain = value => {
        const token = {realm, id: ++next};
        roots.set(token.id, value);
        return token;
    };
    const get = token => {
        if (token?.realm !== realm || !roots.has(token.id)) throw new Error('stale_handle: reference belongs to another realm or was disposed');
        return roots.get(token.id);
    };
    function ref(value) {
        const wrapper = Object.freeze(Object.create(null));
        references.set(wrapper, value);
        return wrapper;
    }
    function encodeNode(value, ancestors, born) {
        if (value === undefined) return {type: 'undefined'};
        if (value === null) return {type: 'null'};
        if (typeof value === 'boolean' || typeof value === 'string') return {type: typeof value, value};
        if (typeof value === 'number') return Number.isFinite(value) && !Object.is(value, -0)
            ? {type: 'number', value} : {type: 'specialNumber', value: Object.is(value, -0) ? '-0' : String(value)};
        if (typeof value === 'bigint') return {type: 'bigInt', value: String(value)};
        if (references.has(value)) { const token = retain(references.get(value)); born.push(token); return {type: 'reference', value: token}; }
        if (value instanceof ArrayBuffer || ArrayBuffer.isView(value)) {
            const bytes = value instanceof ArrayBuffer ? new Uint8Array(value) : new Uint8Array(value.buffer, value.byteOffset, value.byteLength);
            return {type: 'binary', value: Array.from(bytes)};
        }
        if (typeof value !== 'object' || ancestors.has(value) ||
            (!Array.isArray(value) && Object.getPrototypeOf(value) !== Object.prototype && Object.getPrototypeOf(value) !== null))
            throw new TypeError('Value requires a JS reference; use evaluate_handle/evaluateHandle or host.ref(value)');
        ancestors.add(value);
        try {
            return Array.isArray(value)
                ? {type: 'array', value: Array.from(value, item => encodeNode(item, ancestors, born))}
                : {type: 'object', value: Object.fromEntries(Object.entries(value).map(([key, item]) => [key, encodeNode(item, ancestors, born)]))};
        } finally { ancestors.delete(value); }
    }
    function encode(value) {
        const born = [];
        try { return encodeNode(value, new Set(), born); }
        catch (error) { for (const token of born) roots.delete(token.id); throw error; }
    }
    function release(wire) {
        if (wire.type === 'reference' && wire.value.realm === realm) roots.delete(wire.value.id);
        else if (wire.type === 'array') wire.value.forEach(release);
        else if (wire.type === 'object') Object.values(wire.value).forEach(release);
    }
    function decode(wire, hostFunction) {
        switch (wire.type) {
            case 'undefined': return undefined;
            case 'null': return null;
            case 'boolean': case 'string': case 'number': return wire.value;
            case 'specialNumber': return {'NaN': NaN, 'Infinity': Infinity, '-Infinity': -Infinity, '-0': -0}[wire.value];
            case 'bigInt': return BigInt(wire.value);
            case 'binary': return Uint8Array.from(wire.value);
            case 'reference': return get(wire.value);
            case 'hostFunction': if (!hostFunction) throw new TypeError('HostFunction requires a plugin result'); return hostFunction(wire.value);
            case 'array': return wire.value.map(item => decode(item, hostFunction));
            case 'object': return Object.fromEntries(Object.entries(wire.value).map(([key, value]) => [key, decode(value, hostFunction)]));
            default: throw new TypeError('Invalid bridge value');
        }
    }
    function controller(serialized, box) {
        const command = JSON.parse(serialized);
        if (command.op === 'retain') return JSON.stringify(retain(box.value));
        if (command.op === 'get') return {value: get(command.token)};
        if (command.op === 'dispose') {
            if (command.token.realm === realm) roots.delete(command.token.id);
            return {done: true, wire: {type: 'undefined'}};
        }
        const result = {done: false};
        const callback = !!command.callback;
        if (callback) ++callbackOperations;
        let cancelled = false;
        const finish = () => { if (!result.done && callback) --callbackOperations; result.done = true; };
        result.cancel = () => { cancelled = true; if (result.wire) release(result.wire); finish(); };
        try {
            const value = get(command.token);
            const args = (command.args ?? []).map(item => decode(item));
            let output;
            switch (command.op) {
                case 'value': output = value; break;
                case 'json': output = JSON.stringify(value); if (output === undefined) throw new TypeError('Value is not JSON serializable'); break;
                case 'call': output = (0, eval)('(' + command.source + ')')(value, ...args); break;
                case 'invoke': output = Reflect.apply(value, decode(command.receiver ?? {type: 'undefined'}), args); break;
                case 'property': output = value[command.name]; break;
                case 'set': if (!Reflect.set(value, command.name, args[0])) throw new TypeError('Property is not writable'); break;
                case 'delete': output = Reflect.deleteProperty(value, command.name); break;
                case 'names': output = Object.getOwnPropertyNames(value); break;
                default: throw new TypeError('Unknown bridge operation');
            }
            Promise.resolve(output).then(value => {
                if (cancelled) return;
                try { result.wire = command.handle ? {type: 'reference', value: retain(value)} : encode(value); }
                catch (error) { result.error = String(error); }
            }, error => { result.error = String(error); }).finally(finish);
        } catch (error) {
            result.error = String(error);
            finish();
        }
        return result;
    }
    controller.codec = Object.freeze({encode, decode, ref, release, checkCall() {
        if (callbackOperations) throw new Error('reentry: a callback handle operation cannot call a host handler');
    }});
    return controller;
}
