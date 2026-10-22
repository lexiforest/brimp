use brimp_runtime::runtime::{
    ExtractionOptions, PageOptions,
    native::{Session, SessionOptions},
    plugins::bundled,
};
use serde_json::json;

#[test]
fn optional_surfaces_absent_by_default() {
    let session = Session::new(SessionOptions::default()).unwrap();
    let page = session.new_page().unwrap();
    assert_eq!(page.evaluate("['localStorage','sessionStorage','indexedDB','caches','Worker','SharedWorker','Notification','screen','CanvasRenderingContext2D'].map(name => typeof globalThis[name])").unwrap(), json!(vec!["undefined";9]));
    assert_eq!(page.evaluate("['locks','storage','serviceWorker','mediaDevices','bluetooth'].map(name => typeof navigator[name])").unwrap(), json!(vec!["undefined";5]));
    assert!(
        page.extract(ExtractionOptions::default())
            .unwrap_err()
            .to_string()
            .contains("requires the defuddle plugin")
    );
}

#[test]
fn storage_is_functional_in_memory_and_isolated() {
    let session = Session::new(SessionOptions {
        plugins: vec![bundled::storage()],
        ..Default::default()
    })
    .unwrap();
    let page = session.new_page().unwrap();
    assert_eq!(page.evaluate("localStorage.answer = 42; sessionStorage.setItem('a', 'b'); [localStorage.getItem('answer'), localStorage.length, Object.keys(localStorage), sessionStorage.a]").unwrap(), json!(["42",1,["answer"],"b"]));
    assert_eq!(
        page.evaluate("navigator.storage.persisted()").unwrap(),
        json!(false)
    );
    assert_eq!(
        page.evaluate("typeof navigator.storage.getDirectory")
            .unwrap(),
        json!("undefined")
    );
    assert_eq!(page.evaluate("caches.open('test').then(c => c.put('https://example.test/', new Response('cached'))).then(() => caches.match('https://example.test/')).then(r => r.text())").unwrap(), json!("cached"));
    assert_eq!(page.evaluate("new Promise((resolve,reject) => { const r=indexedDB.open('db',1); r.onupgradeneeded=()=>r.result.createObjectStore('values'); r.onerror=()=>reject(r.error); r.onsuccess=()=>{ const store=r.result.transaction('values','readwrite').objectStore('values'); store.put({ok:42},'key').onsuccess=()=>{store.get('key').onsuccess=e=>resolve(e.target.result);}; }; })").unwrap(), json!({"ok":42}));
    let other = session.new_page().unwrap();
    assert_eq!(other.evaluate("localStorage.length").unwrap(), json!(0));
}

#[test]
fn canvas_mock_preserves_state_and_consistent_dimensions() {
    let session = Session::new(SessionOptions {
        plugins: vec![bundled::canvas(json!({"rgba":[1,2,3,4]}))],
        ..Default::default()
    })
    .unwrap();
    let page = session.new_page().unwrap();
    let value = page.evaluate(r#"(() => {
        const canvas=document.createElement('canvas'); canvas.width=2; canvas.height=3;
        const c=canvas.getContext('2d'); c.fillStyle='#ff0000'; c.save(); c.translate(2,3); c.restore(); c.fillRect(0,0,2,3);
        const image=c.getImageData(0,0,2,3);
        const encoded=JSON.parse(atob(canvas.toDataURL().split(',')[1]));
        const metrics=c.measureText('abcd');
        return {width:image.width,height:image.height,length:image.data.length,pixel:Array.from(image.data.slice(0,4)),encoded,fill:c.fillStyle,transform:c.getTransform().e,textWidth:metrics.width,hit:c.isPointInPath(0,0)};
    })()"#).unwrap();
    assert_eq!(
        value,
        json!({"width":2,"height":3,"length":24,"pixel":[1,2,3,4],"encoded":{"width":2,"height":3,"rgba":[1,2,3,4]},"fill":"#ff0000","transform":0,"textWidth":20,"hit":false})
    );
    assert_eq!(
        page.evaluate("typeof getComputedStyle").unwrap(),
        json!("undefined")
    );
}

#[test]
fn services_plugin_provides_locks_and_deterministic_device_mocks() {
    let session = Session::new(SessionOptions {
        plugins: vec![bundled::services(json!({}))],
        ..Default::default()
    })
    .unwrap();
    let page = session.new_page().unwrap();
    assert_eq!(
        page.evaluate(
            "navigator.locks.request('key', () => navigator.bluetooth.getAvailability())"
        )
        .unwrap(),
        json!(false)
    );
    assert_eq!(
        page.evaluate("Notification.requestPermission()").unwrap(),
        json!("denied")
    );
    assert_eq!(
        page.evaluate("navigator.mediaDevices.enumerateDevices()")
            .unwrap(),
        json!([])
    );
}

#[test]
fn extraction_plugin_uses_live_dom_without_layout() {
    let browser = Session::new(Default::default()).unwrap();
    let mut page = browser
        .new_local_page(
            PageOptions::builder()
                .plugins(vec![bundled::extraction()])
                .build(),
        )
        .unwrap();
    page.set_content("<html><head><title>Article</title></head><body><article><h1>Article</h1><p>This content is extracted directly from the live DOM without a layout engine.</p></article></body></html>").unwrap();
    let result = page
        .extract(ExtractionOptions {
            content_selector: Some("article".into()),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(result.title, "Article");
    assert!(result.content_markdown.unwrap().contains("live DOM"));
    assert_eq!(
        page.eval("typeof getComputedStyle + ':' + typeof Defuddle")
            .unwrap()
            .to_string()
            .unwrap(),
        "undefined:undefined"
    );
}
