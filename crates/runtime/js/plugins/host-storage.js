// Explicit asynchronous host storage. This does not implement a browser Storage
// or IndexedDB interface and cannot be mistaken for synchronous localStorage.
({global,host}) => {
    const transaction = operations => host.call('transaction',operations);
    const api=Object.freeze({
        transaction,
        get:key=>transaction([{type:'get',key:String(key)}]).then(values=>values[0]),
        set:(key,value)=>transaction([{type:'set',key:String(key),value}]).then(()=>null),
        delete:key=>transaction([{type:'delete',key:String(key)}]).then(values=>values[0]),
        clear:()=>transaction([{type:'clear'}]).then(()=>null),
    });
    Object.defineProperty(global,'hostStorage',{value:api,configurable:true});
    return ()=>{delete global.hostStorage;};
}
