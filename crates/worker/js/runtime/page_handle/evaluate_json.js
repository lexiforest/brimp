(() => {
    const value = (0, eval)(__EXPRESSION__);
    const kind = typeof value;
    if (["undefined", "function", "symbol", "bigint"].includes(kind)) {
        throw new Error("BRIMP_UNSUPPORTED_RESULT:" + kind);
    }
    let json;
    try {
        json = JSON.stringify(value);
    } catch (error) {
        throw new Error("BRIMP_UNSUPPORTED_RESULT:" + error.message);
    }
    if (json === undefined) throw new Error("BRIMP_UNSUPPORTED_RESULT:unserializable");
    return json;
})()
