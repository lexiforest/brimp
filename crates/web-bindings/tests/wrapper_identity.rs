use jsc::JsRuntime;
use web_bindings::WrapperCache;

#[test]
fn wrapping_a_node_reuses_the_same_javascript_object() {
    let runtime = JsRuntime::new().unwrap();
    let cache = WrapperCache::default();

    let first = cache.wrap(&runtime, 7).unwrap();
    let second = cache.wrap(&runtime, 7).unwrap();
    let different = cache.wrap(&runtime, 8).unwrap();

    assert_eq!(first, second);
    assert_ne!(first, different);
    assert_eq!(cache.len(), 2);

    runtime.set_global_object("first", &first).unwrap();
    runtime.set_global_object("second", &second).unwrap();
    runtime.set_global_object("different", &different).unwrap();
    assert_eq!(
        runtime
            .eval("Number(first === second && first !== different)")
            .unwrap()
            .to_number()
            .unwrap(),
        1.0
    );
}
