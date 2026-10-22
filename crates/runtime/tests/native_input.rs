use brimp_runtime::runtime::native::{FileUpload, Session, SessionOptions};
use serde_json::json;
#[test]
fn direct_input_order_defaults_cancellation_and_no_geometry() {
    let session = Session::new(SessionOptions::default()).unwrap();
    let page = session.new_page().unwrap();
    page.evaluate(r#"document.body.innerHTML='<input id="edit"><input id="check" type="checkbox"><div id="drag" draggable="true">source</div><div id="drop">target</div><input id="file" type="file">'; globalThis.events=[]; for (const name of ['pointerdown','mousedown','focus','pointerup','mouseup','click','keydown','keypress','beforeinput','input','keyup','change']) document.addEventListener(name,e=>events.push([e.type,e.target.id,e.isTrusted]),true); null"#).unwrap();
    page.click("#check").unwrap();
    assert_eq!(
        page.evaluate("document.querySelector('#check').checked")
            .unwrap(),
        json!(true)
    );
    assert_eq!(
        page.evaluate("events.map(e=>e[0])").unwrap(),
        json!([
            "pointerdown",
            "mousedown",
            "focus",
            "pointerup",
            "mouseup",
            "click",
            "input",
            "change"
        ])
    );
    assert_eq!(page.evaluate("events.every(e=>e[2])").unwrap(), json!(true));
    page.evaluate("document.querySelector('#check').onclick=e=>e.preventDefault(); null")
        .unwrap();
    page.click("#check").unwrap();
    assert_eq!(
        page.evaluate("document.querySelector('#check').checked")
            .unwrap(),
        json!(true)
    );
    page.fill("#edit", "ab").unwrap();
    page.type_text("#edit", "c").unwrap();
    page.press("#edit", "Backspace", 0).unwrap();
    assert_eq!(
        page.evaluate("document.querySelector('#edit').value")
            .unwrap(),
        json!("ab")
    );
    page.evaluate("document.querySelector('#edit').addEventListener('beforeinput',e=>e.preventDefault()); null").unwrap();
    page.fill("#edit", "blocked").unwrap();
    assert_eq!(
        page.evaluate("document.querySelector('#edit').value")
            .unwrap(),
        json!("ab")
    );
    page.set_input_files(
        "#file",
        vec![FileUpload {
            name: "test.bin".into(),
            bytes: vec![1, 0, 255],
            mime_type: "application/octet-stream".into(),
            last_modified: 0,
        }],
    )
    .unwrap();
    assert_eq!(page.evaluate("document.querySelector('#file').files[0].arrayBuffer().then(b=>Array.from(new Uint8Array(b)))").unwrap(), json!([1,0,255]));
    page.evaluate("globalThis.dragEvents=[]; for (const name of ['dragstart','dragenter','dragover','drop','dragend']) document.addEventListener(name,e=>{ dragEvents.push(name); if(name==='dragover')e.preventDefault(); }); null").unwrap();
    page.drag_to("#drag", "#drop").unwrap();
    assert_eq!(
        page.evaluate("dragEvents").unwrap(),
        json!(["dragstart", "dragenter", "dragover", "drop", "dragend"])
    );
    page.tap("#check").unwrap();
    let element = page.query_selector("#edit").unwrap().unwrap();
    element.focus().unwrap();
    assert_eq!(
        page.evaluate("document.activeElement.id").unwrap(),
        json!("edit")
    );
    assert_eq!(
        page.evaluate("typeof document.elementFromPoint").unwrap(),
        json!("undefined")
    );
}
