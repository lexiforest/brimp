const declarationSource = __DECLARATION__;
const declaration = (0, eval)("(" + declarationSource + ")");
const receiverId = __RECEIVER__;
const receiver = receiverId === null ? globalThis : state.objects.get(receiverId);
if (receiverId !== null && receiver === undefined) {
    throw new Error("Unknown remote object: " + receiverId);
}
const specs = __ARGUMENTS__;
const args = specs.map(spec => {
    if ("objectId" in spec) {
        const value = state.objects.get(spec.objectId);
        if (value === undefined) throw new Error("Unknown remote object: " + spec.objectId);
        return value;
    }
    if ("unserializableValue" in spec) {
        if (spec.unserializableValue === "NaN") return NaN;
        if (spec.unserializableValue === "Infinity") return Infinity;
        if (spec.unserializableValue === "-Infinity") return -Infinity;
        if (spec.unserializableValue === "-0") return -0;
        if (spec.unserializableValue.endsWith("n")) {
            return BigInt(spec.unserializableValue.slice(0, -1));
        }
    }
    return spec.value;
});
const value = declaration.apply(receiver, args);
