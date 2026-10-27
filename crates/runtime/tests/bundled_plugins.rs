use brimp_runtime::runtime::{
    ExtractionOptions, PageOptions,
    native::{Session, SessionOptions},
    plugins::bundled,
};
use serde_json::json;

#[test]
fn core_only_selection_omits_subsystem_surfaces() {
    let session = Session::new(SessionOptions {
        plugins: vec![],
        ..Default::default()
    })
    .unwrap();
    let page = session.new_page().unwrap();
    assert_eq!(page.evaluate("['Worker','SharedWorker','Notification','CanvasRenderingContext2D'].map(name => typeof globalThis[name])").unwrap(), json!(vec!["undefined";4]));
    assert_eq!(
        page.evaluate(
            "['serviceWorker','mediaDevices','bluetooth'].map(name => typeof navigator[name])"
        )
        .unwrap(),
        json!(vec!["undefined"; 3])
    );
    assert!(
        page.extract(ExtractionOptions::default())
            .unwrap_err()
            .to_string()
            .contains("requires the defuddle plugin")
    );
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
        const dataUrl=canvas.toDataURL(),bytes=atob(dataUrl.split(',')[1]);
        const encoded={prefix:dataUrl.slice(0,22),signature:[bytes.charCodeAt(1),bytes.charCodeAt(2),bytes.charCodeAt(3)],
          width:(bytes.charCodeAt(16)<<24|bytes.charCodeAt(17)<<16|bytes.charCodeAt(18)<<8|bytes.charCodeAt(19))>>>0,
          height:(bytes.charCodeAt(20)<<24|bytes.charCodeAt(21)<<16|bytes.charCodeAt(22)<<8|bytes.charCodeAt(23))>>>0};
        const metrics=c.measureText('abcd');
        return {width:image.width,height:image.height,length:image.data.length,pixel:Array.from(image.data.slice(0,4)),encoded,fill:c.fillStyle,transform:c.getTransform().e,textWidth:metrics.width,hit:c.isPointInPath(0,0)};
    })()"#).unwrap();
    assert_eq!(
        value,
        json!({"width":2,"height":3,"length":24,"pixel":[255,0,0,255],"encoded":{"prefix":"data:image/png;base64,","signature":[80,78,71],"width":2,"height":3},"fill":"#ff0000","transform":0,"textWidth":20,"hit":false})
    );
    assert_eq!(
        page.evaluate("typeof getComputedStyle(document.body)")
            .unwrap(),
        json!("undefined")
    );
}

#[test]
fn services_plugin_provides_deterministic_device_mocks() {
    let session = Session::new(SessionOptions {
        plugins: vec![
            bundled::environment(json!({})),
            bundled::permissions(json!({})),
            bundled::devices(json!({})),
            bundled::media(json!({})),
            bundled::workers(json!({})),
            bundled::trusted_types(json!({})),
        ],
        ..Default::default()
    })
    .unwrap();
    let page = session.new_page().unwrap();
    assert_eq!(
        page.evaluate("navigator.bluetooth.getAvailability()")
            .unwrap(),
        json!(false)
    );
    assert_eq!(
        page.evaluate("Notification.requestPermission()").unwrap(),
        json!("default")
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
        page.eval("typeof getComputedStyle(document.body) + ':' + typeof Defuddle")
            .unwrap()
            .to_string()
            .unwrap(),
        "undefined:undefined"
    );
}

#[test]
fn default_browser_subsystems_are_installed() {
    let session = Session::new(Default::default()).unwrap();
    let page = session.new_page().unwrap();
    assert_eq!(page.evaluate("[typeof Worker,typeof AudioContext,typeof MediaEncryptedEvent,typeof SpeechSynthesisUtterance,typeof RTCPeerConnection,!!document.createElement('canvas').getContext('2d'),typeof hostStorage,typeof Defuddle]").unwrap(), json!(["function","function","function","function","function",true,"undefined","undefined"]));
}
