async url => {
    const bundle = await (await fetch(url + '/xxhash-wasm.js')).text();
    (0, eval)(bundle);
    const hasher = await xxhash();
    const hash32 = hasher.h32ToString('hello');
    const hash64 = hasher.h64ToString('hello');
    const incremental = hasher.create64().update('hel').update('lo').digest();
    const {module, instance} = await WebAssembly.instantiateStreaming(fetch(url + '/xxhash.wasm'));
    const bytes = new TextEncoder().encode('hello');
    new Uint8Array(instance.exports.mem.buffer).set(bytes);
    const streamed32 = instance.exports.xxh32(0, bytes.length, 0) >>> 0;
    const streamed64 = BigInt.asUintN(64, instance.exports.xxh64(0, bytes.length, 0n));
    const importsBytes = await (await fetch(url + '/wasm-import.wasm')).arrayBuffer();
    const imported = await WebAssembly.instantiate(importsBytes, {env:{add:(a,b)=>a+b}});
    let linkError;
    try { await WebAssembly.instantiate(importsBytes, {env:{add:0}}); } catch(error) { linkError = error.name; }
    const compiled = await WebAssembly.compileStreaming(fetch(url + '/xxhash.wasm'));
    const again = await WebAssembly.instantiate(compiled);
    const ordinary = await WebAssembly.compile(await (await fetch(url + '/xxhash.wasm')).arrayBuffer());
    const memory = new WebAssembly.Memory({initial:1, maximum:2});
    const oldPages = memory.grow(1);
    const global = new WebAssembly.Global({value:'i64', mutable:true}, 1n);
    global.value = 2n;
    const table = new WebAssembly.Table({initial:1, element:'anyfunc'});
    table.set(0, instance.exports.xxh32);
    return {hash32, hash64, incremental, streamed32, streamed64, imported:imported.instance.exports.sum(20,22),linkError,
      module: module instanceof WebAssembly.Module && ordinary instanceof WebAssembly.Module,
      memory: [oldPages, memory.buffer.byteLength], global:global.value,
      table: typeof table.get(0), again: typeof again.exports.xxh32};
}
