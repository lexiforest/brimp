use std::{fmt, marker::PhantomData, rc::Rc};

use jsc_sys::{
    JSContextRef, JSObjectMake, JSObjectRef, JSObjectSetPrototype, JSValueProtect, JSValueUnprotect,
};

use crate::JsRuntime;

pub struct ProtectedJsObject {
    context: JSContextRef,
    raw: JSObjectRef,
    _thread_bound: PhantomData<Rc<()>>,
}

pub struct DeferredPromise {
    promise: ProtectedJsObject,
    resolve: ProtectedJsObject,
    reject: ProtectedJsObject,
}

impl DeferredPromise {
    pub(crate) fn new(
        promise: ProtectedJsObject,
        resolve: ProtectedJsObject,
        reject: ProtectedJsObject,
    ) -> Self {
        Self {
            promise,
            resolve,
            reject,
        }
    }

    pub fn into_parts(self) -> (ProtectedJsObject, PromiseSettlement) {
        (
            self.promise,
            PromiseSettlement {
                resolve: self.resolve,
                reject: self.reject,
            },
        )
    }
}

pub struct PromiseSettlement {
    resolve: ProtectedJsObject,
    reject: ProtectedJsObject,
}

impl PromiseSettlement {
    pub fn resolve(&self, runtime: &JsRuntime, value: &str) -> Result<(), crate::JsException> {
        runtime.call_function_with_string(&self.resolve, value)?;
        Ok(())
    }

    pub fn reject(&self, runtime: &JsRuntime, reason: &str) -> Result<(), crate::JsException> {
        runtime.call_function_with_string(&self.reject, reason)?;
        Ok(())
    }
}

impl ProtectedJsObject {
    pub(crate) unsafe fn from_raw(context: JSContextRef, raw: JSObjectRef) -> Self {
        // SAFETY: the caller guarantees the object belongs to this live context.
        unsafe { JSValueProtect(context, raw) };
        Self {
            context,
            raw,
            _thread_bound: PhantomData,
        }
    }

    pub fn handle<'runtime>(&self, runtime: &'runtime JsRuntime) -> JsObject<'runtime> {
        assert_eq!(
            self.context,
            runtime.as_raw(),
            "object belongs to another JSC runtime"
        );
        // SAFETY: this adds an independent root in the same live context.
        unsafe { JSValueProtect(self.context, self.raw) };
        JsObject {
            context: self.context,
            raw: self.raw,
            _runtime: PhantomData,
        }
    }

    pub fn identity(&self) -> JsObjectIdentity {
        JsObjectIdentity {
            context: self.context,
            raw: self.raw,
        }
    }

    pub(crate) unsafe fn with_prototype(
        context: JSContextRef,
        prototype: JsObjectIdentity,
    ) -> Self {
        assert_eq!(
            context, prototype.context,
            "prototype belongs to another JSC context"
        );
        // SAFETY: the callback context is live and null class/data creates a plain object.
        let raw = unsafe { JSObjectMake(context, std::ptr::null_mut(), std::ptr::null_mut()) };
        assert!(!raw.is_null(), "JavaScriptCore failed to create an object");
        // SAFETY: both object and prototype are live values in this context.
        unsafe { JSObjectSetPrototype(context, raw, prototype.raw) };
        // SAFETY: the object was just created in this context.
        unsafe { Self::from_raw(context, raw) }
    }
}

impl Drop for ProtectedJsObject {
    fn drop(&mut self) {
        // SAFETY: this object was protected in the same still-live context.
        unsafe { JSValueUnprotect(self.context, self.raw) };
    }
}

pub struct JsObject<'runtime> {
    context: JSContextRef,
    raw: JSObjectRef,
    _runtime: PhantomData<&'runtime JsRuntime>,
}

impl JsObject<'_> {
    pub(crate) fn as_raw(&self, runtime: &JsRuntime) -> JSObjectRef {
        assert_eq!(
            self.context,
            runtime.as_raw(),
            "object belongs to another JSC runtime"
        );
        self.raw
    }

    pub fn identity(&self) -> JsObjectIdentity {
        JsObjectIdentity {
            context: self.context,
            raw: self.raw,
        }
    }
}

impl PartialEq for JsObject<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.raw == other.raw
    }
}

impl Eq for JsObject<'_> {}

impl fmt::Debug for JsObject<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_tuple("JsObject").field(&self.raw).finish()
    }
}

impl Drop for JsObject<'_> {
    fn drop(&mut self) {
        // SAFETY: `handle` added this object's independent protection.
        unsafe { JSValueUnprotect(self.context, self.raw) };
    }
}

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct JsObjectIdentity {
    pub(crate) context: JSContextRef,
    pub(crate) raw: JSObjectRef,
}

impl fmt::Debug for JsObjectIdentity {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_tuple("JsObjectIdentity")
            .field(&self.raw)
            .finish()
    }
}
