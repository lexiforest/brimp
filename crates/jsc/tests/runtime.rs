use std::{cell::RefCell, rc::Rc};

use jsc::{JsRuntime, NativeError, NativeValue};

#[test]
fn independent_runtime_can_run_on_an_owner_thread() {
    let main = JsRuntime::new().unwrap();
    assert_eq!(main.eval("21 * 2").unwrap().to_number().unwrap(), 42.0);
    let result = std::thread::spawn(|| {
        let worker = JsRuntime::new().unwrap();
        worker.eval("6 * 7").unwrap().to_number().unwrap()
    })
    .join()
    .unwrap();
    assert_eq!(result, 42.0);
}

#[test]
fn evaluates_arithmetic() {
    let runtime = JsRuntime::new().unwrap();
    let result = runtime.eval("1 + 2").unwrap();

    assert_eq!(result.to_number().unwrap(), 3.0);
}

#[test]
fn strings_preserve_embedded_nul_and_non_bmp_characters() {
    let runtime = JsRuntime::new().unwrap();
    runtime
        .set_global_function("nativeString", |_| {
            Ok(NativeValue::String("before\0after 🦀".into()))
        })
        .unwrap();

    let result = runtime.eval("nativeString()").unwrap().to_string().unwrap();

    assert_eq!(result, "before\0after 🦀");
}

#[test]
fn strings_replace_unpaired_utf16_surrogates() {
    let runtime = JsRuntime::new().unwrap();
    let result = runtime
        .eval("'before ' + String.fromCharCode(0xD83C) + ' after'")
        .unwrap()
        .to_string()
        .unwrap();

    assert_eq!(result, "before \u{FFFD} after");
}

#[test]
fn reports_javascript_exceptions() {
    let runtime = JsRuntime::new().unwrap();
    let error = runtime.eval("throw new Error('broken')").err().unwrap();

    assert!(error.message().contains("broken"), "{error}");
}

#[test]
fn console_log_invokes_rust_callback() {
    let messages = Rc::new(RefCell::new(Vec::new()));
    let captured = Rc::clone(&messages);
    let runtime = JsRuntime::new().unwrap();
    runtime
        .set_console_callback(move |message| captured.borrow_mut().push(message.to_owned()))
        .unwrap();

    runtime.eval("console.log('hello', 42)").unwrap();

    assert_eq!(&*messages.borrow(), &["hello 42"]);
}

#[test]
fn safe_native_function_receives_arguments_and_returns_a_value() {
    let runtime = JsRuntime::new().unwrap();
    runtime
        .set_global_function("nativeAdd", |call| {
            let left = call
                .argument(0)
                .ok_or_else(|| NativeError::new("missing left operand"))?
                .to_number()?;
            let right = call
                .argument(1)
                .ok_or_else(|| NativeError::new("missing right operand"))?
                .to_number()?;
            Ok(NativeValue::Number(left + right))
        })
        .unwrap();

    assert_eq!(
        runtime
            .eval("nativeAdd(20, 22)")
            .unwrap()
            .to_number()
            .unwrap(),
        42.0
    );
}

#[test]
fn deferred_promises_resolve_through_public_jsc_api() {
    let runtime = JsRuntime::new().unwrap();
    let (promise, settlement) = runtime.make_deferred_promise().unwrap().into_parts();
    runtime
        .set_global_object("pending", &promise.handle(&runtime))
        .unwrap();
    runtime
        .eval("globalThis.promiseValue = 'waiting'; pending.then(value => { promiseValue = value; });")
        .unwrap();

    settlement.resolve(&runtime, "done").unwrap();

    assert_eq!(
        runtime.eval("promiseValue").unwrap().to_string().unwrap(),
        "done"
    );
}

#[test]
fn protected_objects_survive_forced_garbage_collection() {
    let runtime = JsRuntime::new().unwrap();
    let object = runtime.make_object().unwrap();
    runtime
        .set_global_object("temporary", &object.handle(&runtime))
        .unwrap();
    runtime
        .eval("temporary.answer = 42; delete globalThis.temporary")
        .unwrap();

    runtime.garbage_collect();
    runtime
        .set_global_object("restored", &object.handle(&runtime))
        .unwrap();

    assert_eq!(
        runtime
            .eval("restored.answer")
            .unwrap()
            .to_number()
            .unwrap(),
        42.0
    );
}
