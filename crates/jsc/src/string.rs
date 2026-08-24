use std::{ffi::CString, ptr::NonNull};

use jsc_sys::{
    JSStringCreateWithUTF8CString, JSStringGetMaximumUTF8CStringSize, JSStringGetUTF8CString,
    JSStringRef, JSStringRelease,
};

use crate::JsException;

pub(crate) struct JsString {
    raw: NonNull<jsc_sys::OpaqueJSString>,
}

impl JsString {
    pub(crate) fn new(value: &str) -> Result<Self, JsException> {
        let value = CString::new(value)
            .map_err(|_| JsException::new("JavaScript strings cannot contain an embedded NUL"))?;
        // SAFETY: `value` is a live, NUL-terminated string for the duration of the call.
        let raw = unsafe { JSStringCreateWithUTF8CString(value.as_ptr()) };
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
        // SAFETY: `self.raw` is owned and remains valid until `drop`.
        let capacity = unsafe { JSStringGetMaximumUTF8CStringSize(self.as_raw()) };
        let mut buffer = vec![0_u8; capacity];
        // SAFETY: the buffer is writable for `capacity` bytes and the JSC string is valid.
        let written = unsafe {
            JSStringGetUTF8CString(self.as_raw(), buffer.as_mut_ptr().cast(), buffer.len())
        };
        buffer.truncate(written.saturating_sub(1));
        String::from_utf8_lossy(&buffer).into_owned()
    }
}

impl Drop for JsString {
    fn drop(&mut self) {
        // SAFETY: this wrapper uniquely owns the retained JSStringRef.
        unsafe { JSStringRelease(self.as_raw()) };
    }
}
