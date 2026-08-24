use std::{marker::PhantomData, ptr};

use jsc_sys::{
    JSContextRef, JSValueProtect, JSValueRef, JSValueToNumber, JSValueToObject, JSValueUnprotect,
};

use crate::{
    JsException, JsRuntime, ProtectedJsObject, context::exception_from_raw, string::JsString,
};

pub struct JsValue<'runtime> {
    runtime: &'runtime JsRuntime,
    raw: JSValueRef,
    _thread_bound: PhantomData<&'runtime JsRuntime>,
}

impl<'runtime> JsValue<'runtime> {
    pub(crate) unsafe fn from_raw(runtime: &'runtime JsRuntime, raw: JSValueRef) -> Self {
        // SAFETY: the caller guarantees `raw` belongs to `runtime`'s context.
        unsafe { JSValueProtect(runtime.as_raw(), raw) };
        Self {
            runtime,
            raw,
            _thread_bound: PhantomData,
        }
    }

    pub fn to_number(&self) -> Result<f64, JsException> {
        let mut exception = ptr::null();
        // SAFETY: both the context and protected value are live and belong together.
        let result = unsafe { JSValueToNumber(self.runtime.as_raw(), self.raw, &mut exception) };
        if exception.is_null() {
            Ok(result)
        } else {
            Err(exception_from_raw(self.runtime.as_raw(), exception))
        }
    }

    pub fn to_string(&self) -> Result<String, JsException> {
        value_to_string(self.runtime.as_raw(), self.raw)
    }

    pub fn to_object(&self) -> Result<ProtectedJsObject, JsException> {
        let mut exception = ptr::null();
        // SAFETY: the protected value and context are live and belong together.
        let object = unsafe { JSValueToObject(self.runtime.as_raw(), self.raw, &mut exception) };
        if !exception.is_null() {
            return Err(exception_from_raw(self.runtime.as_raw(), exception));
        }
        if object.is_null() {
            return Err(JsException::new("JavaScriptCore returned a null object"));
        }
        // SAFETY: `object` is a live object in this runtime.
        Ok(unsafe { ProtectedJsObject::from_raw(self.runtime.as_raw(), object) })
    }
}

impl Drop for JsValue<'_> {
    fn drop(&mut self) {
        // SAFETY: `raw` was protected in this same live context by `from_raw`.
        unsafe { JSValueUnprotect(self.runtime.as_raw(), self.raw) };
    }
}

pub(crate) fn value_to_string(
    context: JSContextRef,
    value: JSValueRef,
) -> Result<String, JsException> {
    let mut exception = ptr::null();
    // SAFETY: the caller supplies a live value from the supplied context.
    let string = unsafe { jsc_sys::JSValueToStringCopy(context, value, &mut exception) };
    if !exception.is_null() {
        return Err(exception_from_raw(context, exception));
    }
    // SAFETY: a successful JSValueToStringCopy returns an owned string reference.
    let string = unsafe { JsString::from_owned_raw(string) }
        .ok_or_else(|| JsException::new("JavaScriptCore returned a null string"))?;
    Ok(string.to_rust_string())
}
