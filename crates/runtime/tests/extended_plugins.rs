use brimp_runtime::runtime::{
    clock::{ClockOptions, TimeMode},
    native::{Session, SessionOptions},
    plugins::bundled,
};
use serde_json::json;
fn session() -> Session {
    Session::new(SessionOptions {
        clock: ClockOptions {
            mode: TimeMode::Logical,
            epoch_ms: None,
        },
        plugins: vec![
            bundled::geometry(),
            bundled::css_typed_om(),
            bundled::scheduling(),
            bundled::canvas(json!({})),
            bundled::graphics(json!({"rgba":[1,2,3,4],"webgpu":true})),
            bundled::audio(json!({})),
        ],
        ..Default::default()
    })
    .unwrap()
}
#[test]
fn rtc_installer_provides_local_shapes_without_transport() {
    let s = Session::new(SessionOptions {
        plugins: vec![bundled::rtc()],
        ..Default::default()
    })
    .unwrap();
    let p = s.new_page().unwrap();
    assert_eq!(
        p.evaluate("const peer=new RTCPeerConnection(); const channel=peer.createDataChannel('rust'); [channel.label,channel.readyState,peer.sctp,channel.send('ignored')===undefined]").unwrap(),
        json!(["rust", "connecting", null, true])
    );
}
#[test]
fn geometry_and_inline_css_are_functional_without_layout() {
    let s = session();
    let p = s.new_page().unwrap();
    assert_eq!(p.evaluate("const m=new DOMMatrix().translate(10,20).scale(2); const v=m.transformPoint(new DOMPoint(3,4)); const r=DOMQuad.fromRect({x:5,y:6,width:-3,height:2}).getBounds(); [v.x,v.y,m.inverse().transformPoint(v).x,r.left,r.right,typeof getComputedStyle(document.body)]").unwrap(),json!([16,28,3,2,5,"undefined"]));
    assert_eq!(p.evaluate("const el=document.createElement('div'); el.attributeStyleMap.set('width',CSS.cm(2.54).to('px')); [el.style.width,el.attributeStyleMap.get('width').value,CSS.s(2).to('ms').value,CSS.px(2).add(CSS.in(1)).value,new DOMMatrix(new DOMMatrix().toFloat64Array()).isIdentity]").unwrap(),json!(["96px",96,2000,98,true]));
    assert!(p.evaluate("CSS.px(1).to('s')").is_err());
    assert!(p.evaluate("new DOMMatrix().translate(0,0,1)").is_err());
}
#[test]
fn scheduling_priority_cancellation_and_logical_audio() {
    let s = session();
    let p = s.new_page().unwrap();
    assert_eq!(p.evaluate("globalThis.order=[]; Promise.all([scheduler.postTask(()=>order.push('background'),{priority:'background'}),scheduler.postTask(()=>order.push('high'),{priority:'user-blocking'}),scheduler.postTask(()=>order.push('normal'))]).then(()=>order)").unwrap(),json!(["high","normal","background"]));
    assert_eq!(p.evaluate("const control=new TaskController({priority:'background'});const task=scheduler.postTask(()=>99,{signal:control.signal,delay:100});control.abort('cancelled');task.catch(e=>e)").unwrap(),json!("cancelled"));
    assert_eq!(p.evaluate("globalThis.audio=new AudioContext(); audio.resume().then(()=>{const a=audio.createAnalyser();const v=new Uint8Array(4);a.getByteTimeDomainData(v);return [...v]})").unwrap(),json!([128,128,128,128]));
    p.advance_time(500.0, 100).unwrap();
    assert_eq!(p.evaluate("audio.currentTime").unwrap(), json!(0.5));
    p.evaluate("audio.suspend().then(()=>null)").unwrap();
    p.advance_time(500.0, 100).unwrap();
    assert_eq!(p.evaluate("audio.currentTime").unwrap(), json!(0.5));
    assert_eq!(p.evaluate("const b=audio.createBuffer(1,4,48000);b.copyToChannel(new Float32Array([1,2]),0,1);[...b.getChannelData(0)]").unwrap(),json!([0,1,2,0]));
}
#[test]
fn graphics_mocks_have_consistent_buffers_and_explicit_failures() {
    let s = session();
    let p = s.new_page().unwrap();
    assert_eq!(p.evaluate("globalThis.c=document.createElement('canvas');c.width=2;c.height=2;globalThis.gl=c.getContext('webgl');const out=new Uint8Array(16);gl.clearColor(1,0,0,1);gl.clear(gl.COLOR_BUFFER_BIT);gl.readPixels(0,0,2,2,gl.RGBA,gl.UNSIGNED_BYTE,out);[...out]").unwrap(),json!([1,2,3,4,1,2,3,4,1,2,3,4,1,2,3,4]));
    assert_eq!(p.evaluate("[c.getContext('webgl')===gl,c.getContext('2d'),gl.getShaderParameter(gl.createShader(gl.VERTEX_SHADER),gl.COMPILE_STATUS)]").unwrap(),json!([true,null,false]));
    assert_eq!(p.evaluate("navigator.gpu.requestAdapter().then(a=>a.requestDevice()).then(async d=>{const b=d.createBuffer({size:4,usage:9});d.queue.writeBuffer(b,0,new Uint8Array([5,6,7,8]));await b.mapAsync(GPUMapMode.READ);return [...new Uint8Array(b.getMappedRange())]})").unwrap(),json!([5,6,7,8]));
    assert!(
        p.evaluate("new AudioContext().decodeAudioData(new ArrayBuffer(1))")
            .is_err()
    );
}
