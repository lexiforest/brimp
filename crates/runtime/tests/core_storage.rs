use async_trait::async_trait;
use brimp_runtime::network::{NetworkError, ResourceLoader, ResourceRequest, ResourceResponse};
use brimp_runtime::runtime::native::{NativePage, Session};
use serde_json::json;
use std::sync::Arc;
struct Loader;
#[async_trait]
impl ResourceLoader for Loader {
    async fn fetch(&self, request: ResourceRequest) -> Result<ResourceResponse, NetworkError> {
        Ok(ResourceResponse {
            status: http::StatusCode::OK,
            headers: Default::default(),
            body: b"<!doctype html><body></body>".to_vec(),
            effective_url: request.url,
            metadata: Default::default(),
        })
    }
}
fn session() -> Session {
    Session::with_resource_loader(
        brimp_runtime::runtime::native::SessionOptions {
            plugins: vec![],
            ..Default::default()
        },
        Arc::new(Loader),
    )
    .unwrap()
}
fn page(s: &Session, url: &str) -> NativePage {
    let p = s.new_page().unwrap();
    p.request("GET", url, Default::default()).unwrap();
    p
}
#[test]
fn origin_storage_and_page_storage_survive_navigation() {
    let s = session();
    let a = page(&s, "https://a.test/");
    let b = page(&s, "https://a.test/other");
    let c = page(&s, "https://b.test/");
    a.evaluate("localStorage.value='shared'; sessionStorage.value='page'; null")
        .unwrap();
    assert_eq!(
        b.evaluate("[localStorage.value, sessionStorage.getItem('value')]")
            .unwrap(),
        json!(["shared", null])
    );
    assert_eq!(
        c.evaluate("localStorage.getItem('value')").unwrap(),
        json!(null)
    );
    a.request("GET", "https://a.test/next", Default::default())
        .unwrap();
    assert_eq!(
        a.evaluate("[localStorage.value,sessionStorage.value]")
            .unwrap(),
        json!(["shared", "page"])
    );
    let other = session();
    let d = page(&other, "https://a.test/");
    assert_eq!(d.evaluate("localStorage.length").unwrap(), json!(0));
}
#[test]
fn indexeddb_commits_survive_realms_and_pages() {
    let s = session();
    let a = page(&s, "https://a.test/");
    a.evaluate(r#"new Promise((resolve,reject)=>{const r=indexedDB.open('db',1);r.onupgradeneeded=()=>r.result.createObjectStore('items');r.onerror=()=>reject(r.error);r.onsuccess=()=>{const tx=r.result.transaction('items','readwrite');tx.objectStore('items').put({n:42,bytes:new Uint8Array([0,255]),date:new Date(100)},'key');tx.oncomplete=()=>{r.result.close();resolve(null)};tx.onabort=()=>reject(tx.error)}})"#).unwrap();
    let b = page(&s, "https://a.test/other");
    assert_eq!(b.evaluate(r#"new Promise((resolve,reject)=>{const r=indexedDB.open('db');r.onerror=()=>reject(r.error);r.onsuccess=()=>{const tx=r.result.transaction('items');const q=tx.objectStore('items').get('key');q.onsuccess=()=>resolve([q.result.n,[...q.result.bytes],q.result.date.getTime()]);q.onerror=()=>reject(q.error)}})"#).unwrap(),json!([42,[0,255],100]));
}
#[test]
fn cookie_store_uses_session_cookie_jar() {
    let s = session();
    let a = page(&s, "https://a.test/");
    a.evaluate("cookieStore.set('key','value').then(()=>null)")
        .unwrap();
    assert_eq!(a.evaluate("document.cookie").unwrap(), json!("key=value"));
    let b = page(&s, "https://a.test/");
    assert_eq!(
        b.evaluate("cookieStore.get('key').then(c=>c.value)")
            .unwrap(),
        json!("value")
    );
    b.evaluate("cookieStore.delete('key').then(()=>null)")
        .unwrap();
    assert_eq!(a.evaluate("document.cookie").unwrap(), json!(""));
}
#[test]
fn locks_coordinate_pages_and_release_on_close() {
    let s = session();
    let a = page(&s, "https://a.test/");
    let b = page(&s, "https://a.test/other");
    a.evaluate("globalThis.held=false; navigator.locks.request('key',()=>{held=true;return new Promise(()=>{})}); null").unwrap();
    a.run_pending_tasks(100).unwrap();
    assert_eq!(a.evaluate("held").unwrap(), json!(true));
    assert_eq!(
        b.evaluate("navigator.locks.request('key',{ifAvailable:true},lock=>lock===null)")
            .unwrap(),
        json!(true)
    );
    a.close().unwrap();
    assert_eq!(
        b.evaluate("navigator.locks.request('key',lock=>lock.name)")
            .unwrap(),
        json!("key")
    );
}
#[test]
fn storage_events_notify_other_same_origin_pages() {
    let s = session();
    let a = page(&s, "https://a.test/");
    let b = page(&s, "https://a.test/other");
    let c = page(&s, "https://b.test/");
    for p in [&a, &b, &c] {
        p.evaluate("globalThis.changes=[];addEventListener('storage',e=>changes.push([e.key,e.oldValue,e.newValue,e.url,e.storageArea===localStorage]));null").unwrap();
    }
    a.evaluate("localStorage.setItem('key','first');localStorage.setItem('key','second');localStorage.setItem('key','second');null").unwrap();
    b.run_pending_tasks(100).unwrap();
    c.run_pending_tasks(100).unwrap();
    assert_eq!(a.evaluate("changes").unwrap(), json!([]));
    assert_eq!(c.evaluate("changes").unwrap(), json!([]));
    assert_eq!(
        b.evaluate("changes").unwrap(),
        json!([
            ["key", null, "first", "https://a.test/", true],
            ["key", "first", "second", "https://a.test/", true]
        ])
    );
}
#[test]
fn configured_storage_file_survives_session_reopen() {
    let path = std::env::temp_dir().join(format!("brimp-core-storage-{}.json", std::process::id()));
    let _ = std::fs::remove_file(&path);
    let open = || {
        Session::with_resource_loader(
            brimp_runtime::runtime::native::SessionOptions {
                storage_path: Some(path.clone()),
                ..Default::default()
            },
            Arc::new(Loader),
        )
        .unwrap()
    };
    {
        let s = open();
        let a = page(&s, "https://a.test/");
        a.evaluate("localStorage.setItem('z','last');localStorage.setItem('a','first');sessionStorage.x='temporary';null").unwrap();
        a.evaluate("caches.open('data').then(c=>c.put('https://a.test/value',new Response('persisted'))).then(()=>null)").unwrap();
        a.evaluate("new Promise((resolve,reject)=>{const r=indexedDB.open('file-db',1);r.onupgradeneeded=()=>{const store=r.result.createObjectStore('people',{keyPath:'id',autoIncrement:true});store.createIndex('byName','name')};r.onsuccess=()=>{const tx=r.result.transaction('people','readwrite');tx.objectStore('people').put({name:'Ada'});tx.oncomplete=()=>{r.result.close();resolve(null)};tx.onabort=()=>reject(tx.error)};r.onerror=()=>reject(r.error)})").unwrap();
        assert_eq!(
            a.evaluate("navigator.storage.persisted()").unwrap(),
            json!(true)
        );
        assert!(
            Session::with_resource_loader(
                brimp_runtime::runtime::native::SessionOptions {
                    storage_path: Some(path.clone()),
                    ..Default::default()
                },
                Arc::new(Loader)
            )
            .is_err()
        );
    }
    {
        let s = open();
        let a = page(&s, "https://a.test/");
        assert_eq!(
            a.evaluate("[Object.keys(localStorage),localStorage.z,sessionStorage.length]")
                .unwrap(),
            json!([["z", "a"], "last", 0])
        );
        assert_eq!(
            a.evaluate("caches.match('https://a.test/value').then(r=>r.text())")
                .unwrap(),
            json!("persisted")
        );
        assert_eq!(a.evaluate("new Promise((resolve,reject)=>{const r=indexedDB.open('file-db');r.onsuccess=()=>{const tx=r.result.transaction('people','readwrite'),store=tx.objectStore('people');let name,key;const get=store.index('byName').get('Ada');get.onsuccess=()=>{name=get.result.name;const put=store.put({name:'Grace'});put.onsuccess=()=>key=put.result};tx.oncomplete=()=>{r.result.close();resolve([name,key])};tx.onabort=()=>reject(tx.error)};r.onerror=()=>reject(r.error)})").unwrap(),json!(["Ada",2]));
    }
    std::fs::remove_file(&path).unwrap();
    let _ = std::fs::remove_file(format!("{}.lock", path.display()));
}
#[test]
fn canvas_backend_order_and_core_value_identity() {
    use brimp_runtime::runtime::{native::SessionOptions, plugins::bundled};
    for plugins in [
        vec![bundled::canvas(json!({})), bundled::graphics(json!({}))],
        vec![bundled::graphics(json!({})), bundled::canvas(json!({}))],
    ] {
        let s = Session::new(SessionOptions {
            plugins,
            ..Default::default()
        })
        .unwrap();
        let p = s.new_page().unwrap();
        assert_eq!(p.evaluate("const a=document.createElement('canvas'),b=document.createElement('canvas');const gl=a.getContext('webgl'),ctx=b.getContext('2d');[a.getContext('webgl')===gl,a.getContext('2d'),b.getContext('webgl'),ctx.createImageData(1,1) instanceof ImageData,new DOMPoint(1,2).x]").unwrap(),json!([true,null,null,true,1]));
    }
    let s = session();
    let p = s.new_page().unwrap();
    assert_eq!(p.evaluate("[typeof DOMMatrix,typeof CSSUnitValue,typeof ImageData,typeof MediaEncryptedEvent,typeof SpeechSynthesisUtterance,typeof TaskController,document.createElement('canvas').getContext('2d')]").unwrap(),json!(["function","function","function","undefined","undefined","function",null]));
}
#[test]
fn indexeddb_serializes_concurrent_page_transactions_and_rolls_back() {
    let s = session();
    let a = page(&s, "https://a.test/");
    let b = page(&s, "https://a.test/other");
    let open = "new Promise((resolve,reject)=>{const r=indexedDB.open('counter');r.onupgradeneeded=()=>r.result.createObjectStore('values');r.onerror=()=>reject(r.error);r.onsuccess=()=>{globalThis.db=r.result;resolve(null)}})";
    a.evaluate(open).unwrap();
    b.evaluate(open).unwrap();
    let increment = "(async()=>{for(let i=0;i<10;i++)await new Promise((resolve,reject)=>{const tx=db.transaction('values','readwrite'),store=tx.objectStore('values'),get=store.get('count');get.onsuccess=()=>store.put((get.result??0)+1,'count');tx.oncomplete=()=>resolve(null);tx.onabort=()=>reject(tx.error)});return null})()";
    std::thread::scope(|scope| {
        let first = scope.spawn(|| a.evaluate(increment).unwrap());
        let second = scope.spawn(|| b.evaluate(increment).unwrap());
        first.join().unwrap();
        second.join().unwrap();
    });
    a.evaluate("new Promise(resolve=>{const tx=db.transaction('values','readwrite');tx.objectStore('values').put(999,'count');tx.onabort=()=>resolve(null);tx.abort()})").unwrap();
    assert_eq!(b.evaluate("new Promise((resolve,reject)=>{const r=db.transaction('values').objectStore('values').get('count');r.onsuccess=async()=>resolve([r.result,(await navigator.locks.query()).held.length]);r.onerror=()=>reject(r.error)})").unwrap(),json!([20,0]));
}
