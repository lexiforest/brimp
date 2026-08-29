use std::{error::Error, fmt, marker::PhantomData, ptr, slice};

use jsc_sys::{
    JSContextRef, JSObjectGetTypedArrayByteLength, JSObjectGetTypedArrayByteOffset,
    JSObjectGetTypedArrayBytesPtr, JSObjectIsFunction, JSObjectMakeArray, JSObjectMakeTypedArray,
    JSObjectRef, JSValueIsNull, JSValueIsObject, JSValueIsUndefined, JSValueMakeBoolean,
    JSValueMakeNull, JSValueMakeNumber, JSValueMakeString, JSValueMakeUndefined, JSValueRef,
    JSValueToBoolean, JSValueToNumber, JSValueToObject, K_JS_TYPED_ARRAY_TYPE_FLOAT32_ARRAY,
    K_JS_TYPED_ARRAY_TYPE_INT32_ARRAY, K_JS_TYPED_ARRAY_TYPE_UINT8_CLAMPED_ARRAY,
};

use crate::{
    DeferredPromise, JsObjectIdentity, ProtectedJsObject, context::exception_from_raw,
    string::JsString, value::value_to_string,
};

pub struct NativeCall<'call> {
    pub(crate) context: JSContextRef,
    pub(crate) this_object: JSObjectRef,
    pub(crate) arguments: &'call [JSValueRef],
    pub(crate) _lifetime: PhantomData<&'call ()>,
}

impl<'call> NativeCall<'call> {
    pub fn this_object(&self) -> JsObjectIdentity {
        JsObjectIdentity {
            context: self.context,
            raw: self.this_object,
        }
    }

    pub fn argument(&self, index: usize) -> Option<NativeArgument<'call>> {
        self.arguments
            .get(index)
            .copied()
            .map(|value| NativeArgument {
                context: self.context,
                value,
                _lifetime: PhantomData,
            })
    }

    pub fn argument_count(&self) -> usize {
        self.arguments.len()
    }

    pub fn make_object_with_prototype(&self, prototype: JsObjectIdentity) -> ProtectedJsObject {
        // SAFETY: this call and the prototype belong to the live callback context.
        unsafe { ProtectedJsObject::with_prototype(self.context, prototype) }
    }

    pub fn make_deferred_promise(&self) -> Result<DeferredPromise, NativeError> {
        let mut resolve = ptr::null_mut();
        let mut reject = ptr::null_mut();
        let mut exception = ptr::null();
        // SAFETY: all output pointers are writable and the callback context is live.
        let promise = unsafe {
            jsc_sys::JSObjectMakeDeferredPromise(
                self.context,
                &mut resolve,
                &mut reject,
                &mut exception,
            )
        };
        if !exception.is_null() {
            return Err(NativeError::new(exception_from_raw(
                self.context,
                exception,
            )));
        }
        if promise.is_null() || resolve.is_null() || reject.is_null() {
            return Err(NativeError::new(
                "JavaScriptCore failed to create a deferred Promise",
            ));
        }
        // SAFETY: all objects were created in the live callback context.
        Ok(DeferredPromise::new(
            unsafe { ProtectedJsObject::from_raw(self.context, promise) },
            unsafe { ProtectedJsObject::from_raw(self.context, resolve) },
            unsafe { ProtectedJsObject::from_raw(self.context, reject) },
        ))
    }

    pub fn make_array(
        &self,
        values: &[JsObjectIdentity],
    ) -> Result<ProtectedJsObject, NativeError> {
        if values.iter().any(|value| value.context != self.context) {
            return Err(NativeError::new(
                "array element belongs to another JSC context",
            ));
        }
        let values = values
            .iter()
            .map(|value| value.raw as JSValueRef)
            .collect::<Vec<_>>();
        self.make_array_from_raw(&values)
    }

    pub fn make_value_array(
        &self,
        values: Vec<NativeValue>,
    ) -> Result<ProtectedJsObject, NativeError> {
        let values = values
            .into_iter()
            .map(|value| value.into_raw(self.context))
            .collect::<Result<Vec<_>, _>>()?;
        self.make_array_from_raw(&values)
    }

    fn make_array_from_raw(&self, values: &[JSValueRef]) -> Result<ProtectedJsObject, NativeError> {
        let mut exception = ptr::null();
        // SAFETY: every element belongs to the live callback context.
        let array = unsafe {
            JSObjectMakeArray(self.context, values.len(), values.as_ptr(), &mut exception)
        };
        if !exception.is_null() {
            return Err(NativeError::new(exception_from_raw(
                self.context,
                exception,
            )));
        }
        // SAFETY: JSC returned a live array in this context.
        Ok(unsafe { ProtectedJsObject::from_raw(self.context, array) })
    }
}

#[derive(Clone, Copy)]
pub struct NativeArgument<'call> {
    context: JSContextRef,
    value: JSValueRef,
    _lifetime: PhantomData<&'call ()>,
}

impl NativeArgument<'_> {
    pub fn is_null_or_undefined(&self) -> bool {
        // SAFETY: the callback value and context remain live for this argument's lifetime.
        unsafe {
            JSValueIsNull(self.context, self.value) || JSValueIsUndefined(self.context, self.value)
        }
    }

    pub fn to_boolean(&self) -> bool {
        // SAFETY: the callback value and context remain live.
        unsafe { JSValueToBoolean(self.context, self.value) }
    }

    pub fn to_number(&self) -> Result<f64, NativeError> {
        let mut exception = ptr::null();
        // SAFETY: the callback value and context remain live.
        let number = unsafe { JSValueToNumber(self.context, self.value, &mut exception) };
        if exception.is_null() {
            Ok(number)
        } else {
            Err(NativeError::new(exception_from_raw(
                self.context,
                exception,
            )))
        }
    }

    pub fn to_string(&self) -> Result<String, NativeError> {
        value_to_string(self.context, self.value).map_err(NativeError::new)
    }

    pub fn as_object(&self) -> Result<Option<JsObjectIdentity>, NativeError> {
        // SAFETY: the callback value and context remain live.
        if !unsafe { JSValueIsObject(self.context, self.value) } {
            return Ok(None);
        }
        let mut exception = ptr::null();
        // SAFETY: object conversion operates on the live callback value.
        let raw = unsafe { JSValueToObject(self.context, self.value, &mut exception) };
        if !exception.is_null() {
            return Err(NativeError::new(exception_from_raw(
                self.context,
                exception,
            )));
        }
        Ok(Some(JsObjectIdentity {
            context: self.context,
            raw,
        }))
    }

    pub fn to_function(&self) -> Result<ProtectedJsObject, NativeError> {
        let object = self
            .as_object()?
            .ok_or_else(|| NativeError::new("callback must be a function"))?;
        // SAFETY: the object and callback context are live and belong together.
        if !unsafe { JSObjectIsFunction(self.context, object.raw) } {
            return Err(NativeError::new("callback must be a function"));
        }
        // SAFETY: the object is live in this callback context and is protected for later use.
        Ok(unsafe { ProtectedJsObject::from_raw(self.context, object.raw) })
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, NativeError> {
        let object = self
            .as_object()?
            .ok_or_else(|| NativeError::new("value must be a typed array"))?;
        let mut exception = ptr::null();
        // SAFETY: the object and callback context remain live and no JSC calls occur while the
        // temporary bytes pointer is being copied.
        let length =
            unsafe { JSObjectGetTypedArrayByteLength(self.context, object.raw, &mut exception) };
        if !exception.is_null() {
            return Err(NativeError::new(exception_from_raw(
                self.context,
                exception,
            )));
        }
        // SAFETY: the object and callback context remain live for this operation.
        let bytes =
            unsafe { JSObjectGetTypedArrayBytesPtr(self.context, object.raw, &mut exception) };
        if !exception.is_null() {
            return Err(NativeError::new(exception_from_raw(
                self.context,
                exception,
            )));
        }
        if length == 0 {
            return Ok(Vec::new());
        }
        if bytes.is_null() {
            return Err(NativeError::new("value must be a typed array"));
        }
        // SAFETY: the object and callback context remain live for this operation.
        let offset =
            unsafe { JSObjectGetTypedArrayByteOffset(self.context, object.raw, &mut exception) };
        if !exception.is_null() {
            return Err(NativeError::new(exception_from_raw(
                self.context,
                exception,
            )));
        }
        // SAFETY: JSC returns the backing buffer start and guarantees the view's
        // `offset..offset + length` range remains readable until the next JSC API call.
        Ok(unsafe { slice::from_raw_parts(bytes.cast::<u8>().add(offset), length) }.to_vec())
    }
}

pub enum NativeValue {
    Undefined,
    Null,
    Boolean(bool),
    Number(f64),
    String(String),
    Bytes(Vec<u8>),
    Int32Array(Vec<i32>),
    Float32Array(Vec<f32>),
    Object(JsObjectIdentity),
    ProtectedObject(ProtectedJsObject),
}

impl NativeValue {
    pub(crate) fn into_raw(self, context: JSContextRef) -> Result<JSValueRef, NativeError> {
        // SAFETY: constructors receive the live callback context.
        Ok(unsafe {
            match self {
                Self::Undefined => JSValueMakeUndefined(context),
                Self::Null => JSValueMakeNull(context),
                Self::Boolean(value) => JSValueMakeBoolean(context, value),
                Self::Number(value) => JSValueMakeNumber(context, value),
                Self::String(value) => {
                    let value = JsString::new(&value).map_err(NativeError::new)?;
                    JSValueMakeString(context, value.as_raw())
                }
                Self::Bytes(value) => {
                    let mut exception = ptr::null();
                    let object = JSObjectMakeTypedArray(
                        context,
                        K_JS_TYPED_ARRAY_TYPE_UINT8_CLAMPED_ARRAY,
                        value.len(),
                        &mut exception,
                    );
                    if !exception.is_null() {
                        return Err(NativeError::new(exception_from_raw(context, exception)));
                    }
                    if object.is_null() {
                        return Err(NativeError::new(
                            "JavaScriptCore failed to create a Uint8ClampedArray",
                        ));
                    }
                    if !value.is_empty() {
                        let bytes = JSObjectGetTypedArrayBytesPtr(context, object, &mut exception);
                        if !exception.is_null() {
                            return Err(NativeError::new(exception_from_raw(context, exception)));
                        }
                        if bytes.is_null() {
                            return Err(NativeError::new(
                                "JavaScriptCore returned no typed-array storage",
                            ));
                        }
                        ptr::copy_nonoverlapping(value.as_ptr(), bytes.cast::<u8>(), value.len());
                    }
                    object
                }
                Self::Int32Array(value) => {
                    let mut exception = ptr::null();
                    let object = JSObjectMakeTypedArray(
                        context,
                        K_JS_TYPED_ARRAY_TYPE_INT32_ARRAY,
                        value.len(),
                        &mut exception,
                    );
                    if !exception.is_null() {
                        return Err(NativeError::new(exception_from_raw(context, exception)));
                    }
                    if object.is_null() {
                        return Err(NativeError::new(
                            "JavaScriptCore failed to create an Int32Array",
                        ));
                    }
                    if !value.is_empty() {
                        let bytes = JSObjectGetTypedArrayBytesPtr(context, object, &mut exception);
                        if !exception.is_null() {
                            return Err(NativeError::new(exception_from_raw(context, exception)));
                        }
                        if bytes.is_null() {
                            return Err(NativeError::new(
                                "JavaScriptCore returned no typed-array storage",
                            ));
                        }
                        ptr::copy_nonoverlapping(value.as_ptr(), bytes.cast::<i32>(), value.len());
                    }
                    object
                }
                Self::Float32Array(value) => {
                    let mut exception = ptr::null();
                    let object = JSObjectMakeTypedArray(
                        context,
                        K_JS_TYPED_ARRAY_TYPE_FLOAT32_ARRAY,
                        value.len(),
                        &mut exception,
                    );
                    if !exception.is_null() {
                        return Err(NativeError::new(exception_from_raw(context, exception)));
                    }
                    if object.is_null() {
                        return Err(NativeError::new(
                            "JavaScriptCore failed to create a Float32Array",
                        ));
                    }
                    if !value.is_empty() {
                        let bytes = JSObjectGetTypedArrayBytesPtr(context, object, &mut exception);
                        if !exception.is_null() {
                            return Err(NativeError::new(exception_from_raw(context, exception)));
                        }
                        if bytes.is_null() {
                            return Err(NativeError::new(
                                "JavaScriptCore returned no typed-array storage",
                            ));
                        }
                        ptr::copy_nonoverlapping(value.as_ptr(), bytes.cast::<f32>(), value.len());
                    }
                    object
                }
                Self::Object(object) => {
                    if object.context != context {
                        return Err(NativeError::new("object belongs to another JSC context"));
                    }
                    object.raw
                }
                Self::ProtectedObject(object) => {
                    if object.identity().context != context {
                        return Err(NativeError::new("object belongs to another JSC context"));
                    }
                    object.identity().raw
                }
            }
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NativeError(String);

impl NativeError {
    pub fn new(message: impl ToString) -> Self {
        Self(message.to_string())
    }
}

impl fmt::Display for NativeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl Error for NativeError {}

pub(crate) fn arguments_from_raw<'call>(
    argument_count: usize,
    arguments: *const JSValueRef,
) -> &'call [JSValueRef] {
    if argument_count == 0 {
        &[]
    } else {
        // SAFETY: JSC supplies this many readable arguments for the callback duration.
        unsafe { slice::from_raw_parts(arguments, argument_count) }
    }
}
