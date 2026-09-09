use std::{
    fmt,
    hash::{Hash, Hasher},
    marker::PhantomData,
    rc::Rc,
};

use crate::jsc::ffi::{
    JSContextGetGlobalContext, JSContextRef, JSGlobalContextRelease, JSGlobalContextRetain,
    JSObjectMake, JSObjectRef, JSObjectSetPrototype, JSValueProtect, JSValueUnprotect,
};

use crate::jsc::{JsException, JsRuntime, runtime::exception_from_raw};

/// An owned GC root that retains its JSC context until dropped.
pub struct ProtectedJsObject {
    context: JSContextRef,
    raw: JSObjectRef,
    _thread_bound: PhantomData<Rc<()>>,
}

impl Clone for ProtectedJsObject {
    fn clone(&self) -> Self {
        // SAFETY: this object retains its live context and protects the value.
        unsafe { Self::from_raw(self.context, self.raw) }
    }
}

pub struct DeferredPromise {
    promise: ProtectedJsObject,
    resolve: ProtectedJsObject,
    reject: ProtectedJsObject,
}

impl DeferredPromise {
    pub(crate) unsafe fn create(context: JSContextRef) -> Result<Self, JsException> {
        let mut resolve = std::ptr::null_mut();
        let mut reject = std::ptr::null_mut();
        let mut exception = std::ptr::null();
        // SAFETY: the caller supplies a live context and all output pointers are writable.
        let promise = unsafe {
            crate::jsc::ffi::JSObjectMakeDeferredPromise(
                context,
                &mut resolve,
                &mut reject,
                &mut exception,
            )
        };
        if !exception.is_null() {
            return Err(exception_from_raw(context, exception));
        }
        if promise.is_null() || resolve.is_null() || reject.is_null() {
            return Err(JsException::from_message(
                "JavaScriptCore failed to create a deferred Promise",
            ));
        }
        // SAFETY: these three objects were created in the supplied context.
        Ok(Self {
            promise: unsafe { ProtectedJsObject::from_raw(context, promise) },
            resolve: unsafe { ProtectedJsObject::from_raw(context, resolve) },
            reject: unsafe { ProtectedJsObject::from_raw(context, reject) },
        })
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
    pub fn resolve(&self, runtime: &JsRuntime, value: &str) -> Result<(), crate::jsc::JsException> {
        runtime.call_function_with_string(&self.resolve, value)?;
        Ok(())
    }

    pub fn reject(&self, runtime: &JsRuntime, reason: &str) -> Result<(), crate::jsc::JsException> {
        runtime.call_function_with_string(&self.reject, reason)?;
        Ok(())
    }
}

impl ProtectedJsObject {
    pub(crate) unsafe fn from_raw(context: JSContextRef, raw: JSObjectRef) -> Self {
        // SAFETY: the caller guarantees the object belongs to this live context.
        let context = unsafe { JSGlobalContextRetain(JSContextGetGlobalContext(context)) };
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

    pub(crate) fn context(&self) -> JSContextRef {
        self.context
    }
    pub(crate) fn as_raw(&self) -> JSObjectRef {
        self.raw
    }

    pub fn identity(&self) -> JsObjectIdentity {
        // SAFETY: this handle protects the object in a live context.
        JsObjectIdentity {
            object: unsafe { ProtectedJsObject::from_raw(self.context, self.raw) },
        }
    }

    pub(crate) unsafe fn with_prototype(
        context: JSContextRef,
        prototype: JsObjectIdentity,
    ) -> Self {
        assert_eq!(
            context,
            prototype.context(),
            "prototype belongs to another JSC context"
        );
        // SAFETY: the callback context is live and null class/data creates a plain object.
        let raw = unsafe { JSObjectMake(context, std::ptr::null_mut(), std::ptr::null_mut()) };
        assert!(!raw.is_null(), "JavaScriptCore failed to create an object");
        // SAFETY: both object and prototype are live values in this context.
        unsafe { JSObjectSetPrototype(context, raw, prototype.as_raw()) };
        // SAFETY: the object was just created in this context.
        unsafe { Self::from_raw(context, raw) }
    }
}

impl Drop for ProtectedJsObject {
    fn drop(&mut self) {
        // SAFETY: this object was protected in the same still-live context.
        unsafe { JSValueUnprotect(self.context, self.raw) };
        // SAFETY: release the context retained by from_raw after removing the root.
        unsafe { JSGlobalContextRelease(self.context) };
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
        // SAFETY: this handle protects the object in a live context.
        JsObjectIdentity {
            object: unsafe { ProtectedJsObject::from_raw(self.context, self.raw) },
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

/// A rooted object identity that remains valid while the token is held.
#[derive(Clone)]
pub struct JsObjectIdentity {
    object: ProtectedJsObject,
}

impl JsObjectIdentity {
    pub(crate) unsafe fn from_raw(context: JSContextRef, raw: JSObjectRef) -> Self {
        // SAFETY: the caller supplies a live object belonging to context.
        Self {
            object: unsafe { ProtectedJsObject::from_raw(context, raw) },
        }
    }

    pub(crate) fn context(&self) -> JSContextRef {
        self.object.context
    }
    pub(crate) fn as_raw(&self) -> JSObjectRef {
        self.object.raw
    }
}

impl PartialEq for JsObjectIdentity {
    fn eq(&self, other: &Self) -> bool {
        self.context() == other.context() && self.as_raw() == other.as_raw()
    }
}
impl Eq for JsObjectIdentity {}
impl Hash for JsObjectIdentity {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.context().hash(state);
        self.as_raw().hash(state);
    }
}
impl fmt::Debug for JsObjectIdentity {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_tuple("JsObjectIdentity")
            .field(&self.as_raw())
            .finish()
    }
}
