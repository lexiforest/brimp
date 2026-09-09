(() => {
    const state = globalThis.__brimpCdpRemoteObjects
        || (globalThis.__brimpCdpRemoteObjects = {
            next: 1,
            objects: new Map(),
            groups: new Map(),
            objectGroups: new Map(),
        });
    __BODY__
    const type = typeof value;
    const group = __OBJECT_GROUP__;
    let result;
    if (value === null) {
        result = { type: "object", subtype: "null", value: null };
    } else if (type === "undefined") {
        result = { type: "undefined" };
    } else if (type === "number" && (!Number.isFinite(value) || Object.is(value, -0))) {
        result = {
            type: "number",
            unserializableValue: Object.is(value, -0) ? "-0" : String(value),
        };
    } else if (type === "bigint") {
        result = { type: "bigint", unserializableValue: String(value) + "n" };
    } else if (type !== "object" && type !== "function" && type !== "symbol") {
        result = { type, value };
    } else if (__RETURN_BY_VALUE__) {
        result = { type, value };
    } else {
        const objectId = "object-" + state.next++;
        state.objects.set(objectId, value);
        if (group !== null && group !== "") {
            let ids = state.groups.get(group);
            if (ids === undefined) state.groups.set(group, ids = new Set());
            ids.add(objectId);
            state.objectGroups.set(objectId, group);
        }
        result = {
            type,
            subtype: Array.isArray(value)
                ? "array"
                : (value && typeof value.nodeType === "number" ? "node" : undefined),
            className: value && value.constructor ? value.constructor.name : undefined,
            description: type === "function"
                ? String(value)
                : Object.prototype.toString.call(value),
            objectId,
        };
    }
    return JSON.stringify(result);
})()
