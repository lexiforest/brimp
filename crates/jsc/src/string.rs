use std::ptr::NonNull;

use jsc_sys::{
    JSStringCreateWithCharacters, JSStringGetCharactersPtr, JSStringGetLength, JSStringRef,
    JSStringRelease,
};

use crate::JsException;

pub(crate) struct JsString {
    raw: NonNull<jsc_sys::OpaqueJSString>,
}

impl JsString {
    pub(crate) fn new(value: &str) -> Result<Self, JsException> {
        let value = value.encode_utf16().collect::<Vec<_>>();
        // SAFETY: `value` is a live UTF-16 buffer for the duration of the call.
        let raw = unsafe { JSStringCreateWithCharacters(value.as_ptr(), value.len()) };
        let raw = NonNull::new(raw)
            .ok_or_else(|| JsException::new("JavaScriptCore failed to allocate a string"))?;
        Ok(Self { raw })
    }

    pub(crate) unsafe fn from_owned_raw(raw: JSStringRef) -> Option<Self> {
        NonNull::new(raw).map(|raw| Self { raw })
    }

    pub(crate) fn as_raw(&self) -> JSStringRef {
        self.raw.as_ptr()
    }

    pub(crate) fn to_rust_string(&self) -> String {
        // SAFETY: these values borrow the live JSStringRef owned by self.
        let length = unsafe { JSStringGetLength(self.as_raw()) };
        let characters = unsafe { JSStringGetCharactersPtr(self.as_raw()) };
        if length == 0 || characters.is_null() {
            return String::new();
        }
        // SAFETY: JavaScriptCore guarantees `length` UTF-16 code units at this pointer.
        let utf16 = unsafe { std::slice::from_raw_parts(characters, length) };
        String::from_utf16_lossy(utf16)
    }
}

impl Drop for JsString {
    fn drop(&mut self) {
        // SAFETY: this wrapper uniquely owns the retained JSStringRef.
        unsafe { JSStringRelease(self.as_raw()) };
    }
}
