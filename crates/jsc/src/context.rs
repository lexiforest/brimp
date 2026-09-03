use std::{
    cell::RefCell,
    collections::HashMap,
    marker::PhantomData,
    panic::{AssertUnwindSafe, catch_unwind},
    ptr::{self, NonNull},
    rc::Rc,
    slice,
    sync::OnceLock,
    time::Duration,
};

use jsc_sys::{
    JSClassCreate, JSClassDefinition, JSClassRelease, JSContextGetGlobalObject, JSContextGetGroup,
    JSContextGroupClearExecutionTimeLimit, JSContextGroupSetExecutionTimeLimit, JSContextRef,
    JSEvaluateScript, JSGarbageCollect, JSGlobalContextCreate, JSGlobalContextRef,
    JSGlobalContextRelease, JSObjectCallAsFunction, JSObjectGetProperty, JSObjectMake,
    JSObjectMakeDeferredPromise, JSObjectMakeFunctionWithCallback, JSObjectSetProperty,
    JSValueIsNull, JSValueIsObject, JSValueIsUndefined, JSValueMakeString, JSValueMakeUndefined,
    JSValueRef, JSValueToObject, K_JS_PROPERTY_ATTRIBUTE_NONE,
};

use crate::{
    DeferredPromise, JsException, JsObject, JsValue, NativeCall, NativeError, NativeValue,
    ProtectedJsObject, native::arguments_from_raw, string::JsString, value::value_to_string,
};

type ConsoleCallback = Rc<dyn Fn(&str)>;
type NativeCallback = Rc<dyn for<'call> Fn(NativeCall<'call>) -> Result<NativeValue, NativeError>>;

thread_local! {
    static CONSOLE_CALLBACKS: RefCell<HashMap<usize, ConsoleCallback>> = RefCell::new(HashMap::new());
    static NATIVE_CALLBACKS: RefCell<HashMap<(usize, usize), NativeCallback>> = RefCell::new(HashMap::new());
}

pub struct JsRuntime {
    context: NonNull<jsc_sys::OpaqueJSContext>,
    _thread_bound: PhantomData<Rc<()>>,
}

impl JsRuntime {
    pub fn new() -> Result<Self, JsException> {
        static OPTIONS_READY: OnceLock<bool> = OnceLock::new();
        let options_ready = OPTIONS_READY.get_or_init(|| {
            // SAFETY: JSC initialization is process-global and internally guarded. Options are
            // changed before the first VM is created and remain immutable afterward.
            unsafe {
                #[cfg(target_os = "macos")]
                {
                    if !jsc_sys::allow_mach_exception_handlers() {
                        return false;
                    }
                }
                jsc_sys::JSCInitialize();
                jsc_sys::JSCSetOptions(c"useSharedArrayBuffer=true".as_ptr())
            }
        });
        if !options_ready {
            return Err(JsException::new(
                "JavaScriptCore rejected the SharedArrayBuffer runtime option",
            ));
        }
        let global_definition = JSClassDefinition {
            version: 0,
            attributes: 0,
            class_name: c"Window".as_ptr(),
            parent_class: ptr::null_mut(),
            static_values: ptr::null(),
            static_functions: ptr::null(),
            initialize: ptr::null(),
            finalize: ptr::null(),
            has_property: ptr::null(),
            get_property: ptr::null(),
            set_property: ptr::null(),
            delete_property: ptr::null(),
            get_property_names: ptr::null(),
            call_as_function: ptr::null(),
            call_as_constructor: ptr::null(),
            has_instance: ptr::null(),
            convert_to_type: ptr::null(),
        };
        // SAFETY: the definition contains only null callbacks and static data. The
        // context retains the class, so the caller's reference can be released.
        let global_class = unsafe { JSClassCreate(&global_definition) };
        if global_class.is_null() {
            return Err(JsException::new(
                "JavaScriptCore failed to create the global object class",
            ));
        }
        // SAFETY: `global_class` is live and describes a callback-free global object.
        let context = unsafe { JSGlobalContextCreate(global_class) };
        // SAFETY: the context retained its own reference during creation.
        unsafe { JSClassRelease(global_class) };
        let context = NonNull::new(context.cast_mut())
            .ok_or_else(|| JsException::new("JavaScriptCore failed to create a global context"))?;
        let runtime = Self {
            context,
            _thread_bound: PhantomData,
        };
        runtime.set_console_callback(|message| eprintln!("{message}"))?;
        Ok(runtime)
    }

    pub fn eval(&self, source: &str) -> Result<JsValue<'_>, JsException> {
        let script = JsString::new(source)?;
        let mut exception = ptr::null();
        // SAFETY: the context and script are live; null optional objects use JSC defaults.
        let result = unsafe {
            JSEvaluateScript(
                self.as_raw(),
                script.as_raw(),
                ptr::null_mut(),
                ptr::null_mut(),
                1,
                &mut exception,
            )
        };
        if !exception.is_null() {
            return Err(exception_from_raw(self.as_raw(), exception));
        }
        if result.is_null() {
            return Err(JsException::new(
                "JavaScriptCore returned no value and no exception",
            ));
        }
        // SAFETY: `result` is a live value in this runtime's context.
        Ok(unsafe { JsValue::from_raw(self, result) })
    }

    pub fn garbage_collect(&self) {
        // SAFETY: the context remains live for the duration of this call.
        unsafe { JSGarbageCollect(self.as_raw()) };
    }

    pub fn set_execution_time_limit(&self, limit: Duration) {
        let seconds = limit.as_secs_f64().max(0.001);
        // SAFETY: the context owns a live group. A null callback requests
        // unconditional termination when the watchdog expires.
        unsafe {
            JSContextGroupSetExecutionTimeLimit(
                JSContextGetGroup(self.as_raw()),
                seconds,
                ptr::null(),
                ptr::null_mut(),
            )
        };
    }

    pub fn clear_execution_time_limit(&self) {
        // SAFETY: the context and its group remain live for this call.
        unsafe { JSContextGroupClearExecutionTimeLimit(JSContextGetGroup(self.as_raw())) };
    }

    pub fn make_object(&self) -> Result<ProtectedJsObject, JsException> {
        // SAFETY: the context is live and null class/data creates a plain object.
        let object = unsafe { JSObjectMake(self.as_raw(), ptr::null_mut(), ptr::null_mut()) };
        if object.is_null() {
            return Err(JsException::new(
                "JavaScriptCore failed to create an object",
            ));
        }
        // SAFETY: `object` was just created in this live context.
        Ok(unsafe { ProtectedJsObject::from_raw(self.as_raw(), object) })
    }

    pub fn make_deferred_promise(&self) -> Result<DeferredPromise, JsException> {
        let mut resolve = ptr::null_mut();
        let mut reject = ptr::null_mut();
        let mut exception = ptr::null();
        // SAFETY: all output pointers are writable and the context is live.
        let promise = unsafe {
            JSObjectMakeDeferredPromise(self.as_raw(), &mut resolve, &mut reject, &mut exception)
        };
        if !exception.is_null() {
            return Err(exception_from_raw(self.as_raw(), exception));
        }
        if promise.is_null() || resolve.is_null() || reject.is_null() {
            return Err(JsException::new(
                "JavaScriptCore failed to create a deferred Promise",
            ));
        }
        // SAFETY: all objects were created in this live context.
        Ok(DeferredPromise::new(
            unsafe { ProtectedJsObject::from_raw(self.as_raw(), promise) },
            unsafe { ProtectedJsObject::from_raw(self.as_raw(), resolve) },
            unsafe { ProtectedJsObject::from_raw(self.as_raw(), reject) },
        ))
    }

    pub fn call_function(&self, function: &ProtectedJsObject) -> Result<JsValue<'_>, JsException> {
        let function = function.handle(self);
        let mut exception = ptr::null();
        // SAFETY: the protected object is a callable object in this live context.
        let result = unsafe {
            JSObjectCallAsFunction(
                self.as_raw(),
                function.as_raw(self),
                ptr::null_mut(),
                0,
                ptr::null(),
                &mut exception,
            )
        };
        if !exception.is_null() {
            return Err(exception_from_raw(self.as_raw(), exception));
        }
        if result.is_null() {
            return Err(JsException::new(
                "JavaScriptCore returned no value from function call",
            ));
        }
        // SAFETY: the call result is live in this runtime.
        Ok(unsafe { JsValue::from_raw(self, result) })
    }

    pub fn call_function_with_string(
        &self,
        function: &ProtectedJsObject,
        value: &str,
    ) -> Result<JsValue<'_>, JsException> {
        let function = function.handle(self);
        let value = JsString::new(value)?;
        // SAFETY: the temporary string and context are live for this conversion.
        let argument = unsafe { JSValueMakeString(self.as_raw(), value.as_raw()) };
        let mut exception = ptr::null();
        // SAFETY: function and argument belong to this live context.
        let result = unsafe {
            JSObjectCallAsFunction(
                self.as_raw(),
                function.as_raw(self),
                ptr::null_mut(),
                1,
                &argument,
                &mut exception,
            )
        };
        if !exception.is_null() {
            return Err(exception_from_raw(self.as_raw(), exception));
        }
        if result.is_null() {
            return Err(JsException::new(
                "JavaScriptCore returned no value from function call",
            ));
        }
        // SAFETY: the call result is live in this runtime.
        Ok(unsafe { JsValue::from_raw(self, result) })
    }

    pub fn call_function_with_string_and_object(
        &self,
        function: &ProtectedJsObject,
        value: &str,
        object: &ProtectedJsObject,
    ) -> Result<JsValue<'_>, JsException> {
        let function = function.handle(self);
        let object = object.handle(self);
        let value = JsString::new(value)?;
        // SAFETY: the temporary string and context are live for this conversion.
        let string_argument = unsafe { JSValueMakeString(self.as_raw(), value.as_raw()) };
        let arguments = [string_argument, object.as_raw(self).cast_const()];
        let mut exception = ptr::null();
        // SAFETY: the function and both arguments belong to this live context.
        let result = unsafe {
            JSObjectCallAsFunction(
                self.as_raw(),
                function.as_raw(self),
                ptr::null_mut(),
                arguments.len(),
                arguments.as_ptr(),
                &mut exception,
            )
        };
        if !exception.is_null() {
            return Err(exception_from_raw(self.as_raw(), exception));
        }
        if result.is_null() {
            return Err(JsException::new(
                "JavaScriptCore returned no value from function call",
            ));
        }
        // SAFETY: the call result is live in this runtime.
        Ok(unsafe { JsValue::from_raw(self, result) })
    }

    pub fn make_object_with_prototype(
        &self,
        prototype: crate::JsObjectIdentity,
    ) -> ProtectedJsObject {
        // SAFETY: `prototype` is validated against this runtime by the constructor.
        unsafe { ProtectedJsObject::with_prototype(self.as_raw(), prototype) }
    }

    pub fn set_global_object(&self, name: &str, object: &JsObject<'_>) -> Result<(), JsException> {
        let name = JsString::new(name)?;
        // SAFETY: this context owns its live global object.
        let global = unsafe { JSContextGetGlobalObject(self.as_raw()) };
        set_property(self.as_raw(), global, &name, object.as_raw(self))
    }

    pub fn set_global_function<F>(&self, name: &str, callback: F) -> Result<(), JsException>
    where
        F: for<'call> Fn(NativeCall<'call>) -> Result<NativeValue, NativeError> + 'static,
    {
        let function_name = JsString::new(name)?;
        // SAFETY: the context and function name are live and the dispatcher has C ABI.
        let function = unsafe {
            JSObjectMakeFunctionWithCallback(
                self.as_raw(),
                function_name.as_raw(),
                Some(native_dispatch),
            )
        };
        if function.is_null() {
            return Err(JsException::new(
                "JavaScriptCore failed to create a native function",
            ));
        }
        let key = (self.as_raw() as usize, function as usize);
        NATIVE_CALLBACKS.with(|callbacks| {
            callbacks.borrow_mut().insert(key, Rc::new(callback));
        });

        // SAFETY: the context owns its live global object.
        let global = unsafe { JSContextGetGlobalObject(self.as_raw()) };
        if let Err(error) = set_property(self.as_raw(), global, &function_name, function) {
            NATIVE_CALLBACKS.with(|callbacks| {
                callbacks.borrow_mut().remove(&key);
            });
            return Err(error);
        }
        Ok(())
    }

    pub fn set_console_callback<F>(&self, callback: F) -> Result<(), JsException>
    where
        F: Fn(&str) + 'static,
    {
        CONSOLE_CALLBACKS.with(|callbacks| {
            callbacks
                .borrow_mut()
                .insert(self.as_raw() as usize, Rc::new(callback));
        });

        if let Err(error) = self.install_console_object() {
            CONSOLE_CALLBACKS.with(|callbacks| {
                callbacks.borrow_mut().remove(&(self.as_raw() as usize));
            });
            return Err(error);
        }
        Ok(())
    }

    pub(crate) fn as_raw(&self) -> JSGlobalContextRef {
        self.context.as_ptr()
    }

    fn install_console_object(&self) -> Result<(), JsException> {
        let console_name = JsString::new("console")?;
        // SAFETY: the context is live and null class/data creates a plain object.
        let console = unsafe { JSObjectMake(self.as_raw(), ptr::null_mut(), ptr::null_mut()) };
        if console.is_null() {
            return Err(JsException::new(
                "JavaScriptCore failed to create the console object",
            ));
        }

        for method in ["debug", "error", "info", "log", "warn"] {
            let name = JsString::new(method)?;
            // SAFETY: the context and function name are live and the callback has C ABI.
            let function = unsafe {
                JSObjectMakeFunctionWithCallback(self.as_raw(), name.as_raw(), Some(console_log))
            };
            if function.is_null() {
                return Err(JsException::new(format!(
                    "JavaScriptCore failed to create console.{method}"
                )));
            }
            set_property(self.as_raw(), console, &name, function)?;
        }
        // SAFETY: the context has a live standard global object.
        let global = unsafe { JSContextGetGlobalObject(self.as_raw()) };
        set_property(self.as_raw(), global, &console_name, console)
    }
}

impl Drop for JsRuntime {
    fn drop(&mut self) {
        CONSOLE_CALLBACKS.with(|callbacks| {
            callbacks.borrow_mut().remove(&(self.as_raw() as usize));
        });
        NATIVE_CALLBACKS.with(|callbacks| {
            callbacks
                .borrow_mut()
                .retain(|(context, _), _| *context != self.as_raw() as usize);
        });
        // SAFETY: this runtime uniquely owns the global context.
        unsafe { JSGlobalContextRelease(self.as_raw()) };
    }
}

fn set_property(
    context: JSContextRef,
    object: jsc_sys::JSObjectRef,
    name: &JsString,
    value: JSValueRef,
) -> Result<(), JsException> {
    let mut exception = ptr::null();
    // SAFETY: all references are live and belong to `context`.
    unsafe {
        JSObjectSetProperty(
            context,
            object,
            name.as_raw(),
            value,
            K_JS_PROPERTY_ATTRIBUTE_NONE,
            &mut exception,
        )
    };
    if exception.is_null() {
        Ok(())
    } else {
        Err(exception_from_raw(context, exception))
    }
}

unsafe extern "C" fn console_log(
    context: JSContextRef,
    _function: jsc_sys::JSObjectRef,
    _this_object: jsc_sys::JSObjectRef,
    argument_count: usize,
    arguments: *const JSValueRef,
    _exception: *mut JSValueRef,
) -> JSValueRef {
    let arguments = if argument_count == 0 {
        &[]
    } else {
        // SAFETY: JSC supplies `argument_count` readable values for this callback invocation.
        unsafe { slice::from_raw_parts(arguments, argument_count) }
    };
    let message = arguments
        .iter()
        .map(|value| {
            console_value_to_string(context, *value).unwrap_or_else(|error| error.to_string())
        })
        .collect::<Vec<_>>()
        .join(" ");
    let callback =
        CONSOLE_CALLBACKS.with(|callbacks| callbacks.borrow().get(&(context as usize)).cloned());
    if let Some(callback) = callback {
        let _ = catch_unwind(AssertUnwindSafe(|| callback(&message)));
    }
    // SAFETY: the callback's context remains live throughout the invocation.
    unsafe { JSValueMakeUndefined(context) }
}

fn console_value_to_string(
    context: JSContextRef,
    value: JSValueRef,
) -> Result<String, JsException> {
    // Browser consoles render Error objects with their stack. Preserve that diagnostic across
    // the native callback instead of reducing errors to only `name: message`.
    if unsafe { JSValueIsObject(context, value) } {
        let mut exception = ptr::null();
        let object = unsafe { JSValueToObject(context, value, &mut exception) };
        if exception.is_null() && !object.is_null() {
            let name = JsString::new("stack")?;
            let stack =
                unsafe { JSObjectGetProperty(context, object, name.as_raw(), &mut exception) };
            if exception.is_null()
                && !stack.is_null()
                && !unsafe { JSValueIsNull(context, stack) || JSValueIsUndefined(context, stack) }
            {
                let stack = value_to_string(context, stack)?;
                if !stack.is_empty() {
                    let summary = value_to_string(context, value)?;
                    return Ok(if stack.starts_with(&summary) {
                        stack
                    } else {
                        format!("{summary}\n{stack}")
                    });
                }
            }
        }
    }
    value_to_string(context, value)
}

unsafe extern "C" fn native_dispatch(
    context: JSContextRef,
    function: jsc_sys::JSObjectRef,
    this_object: jsc_sys::JSObjectRef,
    argument_count: usize,
    arguments: *const JSValueRef,
    exception: *mut JSValueRef,
) -> JSValueRef {
    let callback = NATIVE_CALLBACKS.with(|callbacks| {
        callbacks
            .borrow()
            .get(&(context as usize, function as usize))
            .cloned()
    });
    let result = match callback {
        Some(callback) => catch_unwind(AssertUnwindSafe(|| {
            callback(NativeCall {
                context,
                this_object,
                arguments: arguments_from_raw(argument_count, arguments),
                _lifetime: PhantomData,
            })
        }))
        .unwrap_or_else(|_| Err(NativeError::new("native callback panicked"))),
        None => Err(NativeError::new("native callback is no longer registered")),
    };

    match result.and_then(|value| value.into_raw(context)) {
        Ok(value) => value,
        Err(error) => {
            let message = JsString::new(&error.to_string())
                .expect("native error messages cannot contain embedded NUL bytes");
            // SAFETY: the context and temporary string are live for this conversion.
            let value = unsafe { jsc_sys::JSValueMakeString(context, message.as_raw()) };
            if !exception.is_null() {
                // SAFETY: JSC supplied a writable exception result pointer.
                unsafe { *exception = value };
            }
            // SAFETY: the callback context remains live.
            unsafe { JSValueMakeUndefined(context) }
        }
    }
}

pub(crate) fn exception_from_raw(context: JSContextRef, exception: JSValueRef) -> JsException {
    match value_to_string_without_exception(context, exception) {
        Some(message) => JsException::new(message),
        None => JsException::new("JavaScript exception could not be converted to text"),
    }
}

fn value_to_string_without_exception(context: JSContextRef, value: JSValueRef) -> Option<String> {
    // Avoid recursive exception conversion by deliberately ignoring a conversion exception here.
    let raw = unsafe { jsc_sys::JSValueToStringCopy(context, value, ptr::null_mut()) };
    // SAFETY: a non-null result is an owned string returned by JSC.
    unsafe { JsString::from_owned_raw(raw) }.map(|string| string.to_rust_string())
}
