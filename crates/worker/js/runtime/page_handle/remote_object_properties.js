(() => {
    const state = globalThis.__brimpCdpRemoteObjects;
    const objectId = __OBJECT_ID__;
    if (!state || !state.objects.has(objectId)) {
        throw new Error("Unknown remote object: " + objectId);
    }
    const object = state.objects.get(objectId);
    const group = state.objectGroups.get(objectId) ?? null;
    const remote = value => {
        const type = typeof value;
        if (value === null) return { type: "object", subtype: "null", value: null };
        if (type === "undefined") return { type: "undefined" };
        if (type === "number" && (!Number.isFinite(value) || Object.is(value, -0))) {
            return {
                type: "number",
                unserializableValue: Object.is(value, -0) ? "-0" : String(value),
            };
        }
        if (type === "bigint") {
            return { type: "bigint", unserializableValue: String(value) + "n" };
        }
        if (type !== "object" && type !== "function" && type !== "symbol") {
            return { type, value };
        }
        const childId = "object-" + state.next++;
        state.objects.set(childId, value);
        if (group !== null) {
            let ids = state.groups.get(group);
            if (ids === undefined) state.groups.set(group, ids = new Set());
            ids.add(childId);
            state.objectGroups.set(childId, group);
        }
        return {
            type,
            subtype: Array.isArray(value)
                ? "array"
                : (value && typeof value.nodeType === "number" ? "node" : undefined),
            className: value && value.constructor ? value.constructor.name : undefined,
            description: type === "function" || type === "symbol"
                ? String(value)
                : Object.prototype.toString.call(value),
            objectId: childId,
        };
    };
    const result = [];
    const seen = new Set();
    for (
        let current = object;
        current !== null;
        current = __OWN_PROPERTIES__ ? null : Object.getPrototypeOf(current)
    ) {
        for (const key of Reflect.ownKeys(Object(current))) {
            const name = typeof key === "symbol" ? String(key) : key;
            if (seen.has(name)) continue;
            seen.add(name);
            const descriptor = Object.getOwnPropertyDescriptor(current, key);
            if (__ACCESSOR_PROPERTIES_ONLY__ && !descriptor.get && !descriptor.set) continue;
            const property = {
                name,
                configurable: descriptor.configurable,
                enumerable: descriptor.enumerable,
                isOwn: current === object,
            };
            if ("value" in descriptor) {
                property.value = remote(descriptor.value);
                property.writable = descriptor.writable;
            } else {
                if (descriptor.get) property.get = remote(descriptor.get);
                if (descriptor.set) property.set = remote(descriptor.set);
            }
            result.push(property);
        }
    }
    return JSON.stringify({ result, internalProperties: [], privateProperties: [] });
})()
