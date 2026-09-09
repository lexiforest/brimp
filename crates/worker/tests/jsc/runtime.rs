use std::{cell::RefCell, rc::Rc};

use brimp_lite_worker::jsc::{JsRuntime, NativeError, NativeValue};

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
fn native_callbacks_exchange_typed_bytes_without_string_encoding() {
    let runtime = JsRuntime::new().unwrap();
    runtime
        .set_global_function("bytes", |call| {
            if call.argument_count() == 0 {
                return Ok(NativeValue::Bytes(vec![0, 127, 255]));
            }
            let bytes = call.argument(0).unwrap().to_bytes()?;
            Ok(NativeValue::Bytes(bytes.into_iter().rev().collect()))
        })
        .unwrap();

    assert_eq!(
        runtime
            .eval(
                "(() => { const value = bytes(); const backing = new Uint8Array([9, 1, 2, 3, 8]); const view = backing.subarray(1, 4); return value instanceof Uint8ClampedArray && String(value) === '0,127,255' && String(bytes(value)) === '255,127,0' && String(bytes(view)) === '3,2,1'; })()",
            )
            .unwrap()
            .to_string()
            .unwrap(),
        "true",
    );
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
fn console_errors_include_the_javascript_stack() {
    let messages = Rc::new(RefCell::new(Vec::new()));
    let captured = Rc::clone(&messages);
    let runtime = JsRuntime::new().unwrap();
    runtime
        .set_console_callback(move |message| captured.borrow_mut().push(message.to_owned()))
        .unwrap();

    runtime
        .eval(
            "function failingFunction() { console.error(new Error('broken')); } failingFunction();",
        )
        .unwrap();

    let message = &messages.borrow()[0];
    assert!(message.contains("Error: broken"), "{message}");
    assert!(message.contains("failingFunction"), "{message}");
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

#[test]
fn owned_handles_and_identities_can_outlive_the_runtime() {
    let (object, identity, promise) = {
        let runtime = JsRuntime::new().unwrap();
        let object = runtime.make_object().unwrap();
        let identity = object.identity();
        let promise = runtime.make_deferred_promise().unwrap();
        (object, identity, promise)
    };
    // Cloning and dropping both require a live retained JSC context.
    let clone = object.clone();
    assert_eq!(clone.identity(), identity);
    drop(object);
    drop(clone);
    drop(identity.clone());
    drop(identity);
    drop(promise);
}

#[test]
fn identity_keeps_a_prototype_alive_after_the_original_handle_is_dropped() {
    let runtime = JsRuntime::new().unwrap();
    let identity = runtime
        .eval("({ answer: 42 })")
        .unwrap()
        .to_object()
        .unwrap()
        .identity();
    runtime.garbage_collect();
    let object = runtime.make_object_with_prototype(identity);
    runtime
        .set_global_object("child", &object.handle(&runtime))
        .unwrap();
    assert_eq!(
        runtime.eval("child.answer").unwrap().to_number().unwrap(),
        42.0
    );
}

#[test]
fn callback_object_identity_survives_collection_and_can_be_returned_later() {
    let runtime = JsRuntime::new().unwrap();
    let saved = Rc::new(RefCell::new(None));
    let captured = Rc::clone(&saved);
    runtime
        .set_global_function("saveObject", move |call| {
            *captured.borrow_mut() = call.argument(0).unwrap().as_object()?;
            Ok(NativeValue::Undefined)
        })
        .unwrap();
    runtime
        .set_global_function("restoreObject", move |_| {
            Ok(NativeValue::Object(
                saved.borrow().as_ref().unwrap().clone(),
            ))
        })
        .unwrap();
    runtime.eval("saveObject({ answer: 42 })").unwrap();
    runtime.garbage_collect();
    assert_eq!(
        runtime
            .eval("restoreObject().answer")
            .unwrap()
            .to_number()
            .unwrap(),
        42.0
    );
}

#[test]
fn typed_bytes_reject_non_views_and_accept_empty_and_offset_views() {
    let runtime = JsRuntime::new().unwrap();
    runtime
        .set_global_function("readBytes", |call| {
            Ok(NativeValue::Bytes(call.argument(0).unwrap().to_bytes()?))
        })
        .unwrap();
    for expression in ["({})", "[]", "new ArrayBuffer(0)", "null", "42"] {
        let error = runtime
            .eval(&format!("readBytes({expression})"))
            .err()
            .unwrap();
        assert!(
            error.message().contains("typed array"),
            "{expression}: {error}"
        );
    }
    assert_eq!(
        runtime
            .eval("readBytes(new Uint8Array(0)).length")
            .unwrap()
            .to_number()
            .unwrap(),
        0.0
    );
    assert_eq!(
        runtime
            .eval("String(readBytes(new Uint8Array([9, 1, 2, 8]).subarray(1, 3)))")
            .unwrap()
            .to_string()
            .unwrap(),
        "1,2"
    );
}

#[test]
fn native_value_arrays_keep_new_objects_and_strings_alive() {
    let runtime = JsRuntime::new().unwrap();
    runtime
        .set_global_function("makeValues", |call| {
            let object = call.make_value_array(vec![NativeValue::Number(42.0)])?;
            let mut values = vec![NativeValue::ProtectedObject(object)];
            values.extend((0..2048).map(|i| NativeValue::String(format!("value-{i}"))));
            Ok(NativeValue::ProtectedObject(call.make_value_array(values)?))
        })
        .unwrap();
    assert_eq!(
        runtime
            .eval(
                "const values = makeValues(); values[0][0] === 42 && values[2048] === 'value-2047'"
            )
            .unwrap()
            .to_string()
            .unwrap(),
        "true"
    );
}

#[test]
fn native_errors_preserve_embedded_nul_characters() {
    let runtime = JsRuntime::new().unwrap();
    runtime
        .set_global_function("fail", |_| Err(NativeError::new("before\0after")))
        .unwrap();
    assert_eq!(
        runtime.eval("fail()").err().unwrap().message(),
        "before\0after"
    );
}

#[test]
fn typed_bytes_support_float16_and_data_view_buffers() {
    let runtime = JsRuntime::new().unwrap();
    runtime
        .set_global_function("readBytes", |call| {
            Ok(NativeValue::Bytes(call.argument(0).unwrap().to_bytes()?))
        })
        .unwrap();
    assert_eq!(
        runtime
            .eval("readBytes(new Float16Array([1, 2])).length")
            .unwrap()
            .to_number()
            .unwrap(),
        4.0
    );
    assert_eq!(
        runtime
            .eval("readBytes(new Float16Array(0)).length")
            .unwrap()
            .to_number()
            .unwrap(),
        0.0
    );
    assert_eq!(
        runtime
            .eval("String(readBytes(new DataView(new Uint8Array([9, 1, 2, 8]).buffer, 1, 2)))")
            .unwrap()
            .to_string()
            .unwrap(),
        "1,2"
    );
    assert_eq!(
        runtime
            .eval("readBytes(new DataView(new ArrayBuffer(0))).length")
            .unwrap()
            .to_number()
            .unwrap(),
        0.0
    );
}
