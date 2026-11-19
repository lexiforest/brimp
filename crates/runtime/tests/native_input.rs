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
}

#[test]
fn coordinate_pointer_input_hits_the_element_under_the_point() {
    let session = Session::new(SessionOptions::default()).unwrap();
    let page = session.new_page().unwrap();
    page.evaluate(
        r#"document.body.innerHTML=
             '<div id="a" style="margin:20px;width:200px;height:100px">target</div>'+
             '<div id="b" style="margin:20px;width:200px;height:100px"></div>';
           globalThis.log=[];
           for (const name of ['pointermove','pointerdown','pointerup','click','mousedown','mouseup','wheel'])
             document.addEventListener(name, e=>log.push([e.type, e.target.id||e.target.localName, e.clientX, e.clientY, e.buttons, e.isTrusted]), true);
           null"#,
    )
    .unwrap();
    let point = page
        .evaluate(
            r#"(() => { const box = document.getElementById('a').getBoundingClientRect();
                 return [Math.round(box.x + box.width / 2), Math.round(box.y + box.height / 2)]; })()"#,
        )
        .unwrap()
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_f64().unwrap())
        .collect::<Vec<_>>();
    let (x, y) = (point[0], point[1]);
    // Event coordinates are integers for these whole-pixel points.
    let (xi, yi) = (x as i64, y as i64);
    let (start_x, start_y) = (xi - 100, yi - 40);

    // A point resolves to the element the layout places there.
    assert_eq!(
        page.evaluate(&format!(
            "[document.elementFromPoint({x}, {y}).id, document.elementsFromPoint({x}, {y}).map(e=>e.id||e.localName)]"
        ))
        .unwrap(),
        json!(["a", ["a", "body", "html"]])
    );
    assert_eq!(
        page.evaluate("document.elementFromPoint(-5, -5) === null")
            .unwrap(),
        json!(true)
    );

    // Moving in steps reports one sample per step; clicking presses and releases
    // at the point, on the element there, with the pointer's coordinates.
    page.mouse_move(x - 100.0, y - 40.0, 2).unwrap();
    page.mouse_click(x, y, "left", 1, 0).unwrap();
    assert_eq!(
        page.evaluate("log.map(e=>[e[0],e[1],e[2],e[3],e[4]])")
            .unwrap(),
        json!([
            // Two steps from (0,0): the first sample is over the body, the second
            // lands on the target's top-left corner.
            ["pointermove", "body", start_x / 2, start_y / 2, 0],
            ["pointermove", "a", start_x, start_y, 0],
            // The click moves the pointer to its own point before pressing.
            ["pointermove", "a", xi, yi, 0],
            ["pointerdown", "a", xi, yi, 1],
            ["mousedown", "a", xi, yi, 1],
            ["pointerup", "a", xi, yi, 0],
            ["mouseup", "a", xi, yi, 0],
            ["click", "a", xi, yi, 0],
        ])
    );
    assert_eq!(page.evaluate("log.every(e=>e[5])").unwrap(), json!(true));

    // A press and a release are separate commands, so a hold is expressible.
    page.evaluate("log.length = 0; null").unwrap();
    page.mouse_down(x, y, "left", 1, 0).unwrap();
    page.evaluate("globalThis.held = true; null").unwrap();
    page.mouse_up(x, y, "left", 1, 0).unwrap();
    assert_eq!(
        page.evaluate("log.map(e=>e[0])").unwrap(),
        json!(["pointerdown", "mousedown", "pointerup", "mouseup", "click"])
    );

    // Scrolling reports deltas at the point.
    page.evaluate("log.length = 0; null").unwrap();
    page.mouse_wheel(x, y, 0.0, 120.0).unwrap();
    assert_eq!(
        page.evaluate("log.map(e=>[e[0],e[1]])").unwrap(),
        json!([["wheel", "a"]])
    );

    // Selector input now reports the element's centre instead of (0, 0).
    page.evaluate("log.length = 0; null").unwrap();
    page.click("#a").unwrap();
    assert_eq!(
        page.evaluate("[log[0][0], log[0][1], log[0][2] > 0, log[0][3] > 0]")
            .unwrap(),
        json!(["pointermove", "a", true, true])
    );
}
