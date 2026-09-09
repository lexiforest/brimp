#![allow(non_camel_case_types)]

use std::ffi::{c_char, c_double, c_int, c_uint, c_void};

pub enum OpaqueJSContext {}
pub enum OpaqueJSContextGroup {}
pub enum OpaqueJSValue {}
pub enum OpaqueJSString {}
pub enum OpaqueJSClass {}

pub type JSContextRef = *const OpaqueJSContext;
pub type JSGlobalContextRef = *const OpaqueJSContext;
pub type JSContextGroupRef = *const OpaqueJSContextGroup;
pub type JSValueRef = *const OpaqueJSValue;
pub type JSObjectRef = *mut OpaqueJSValue;
pub type JSStringRef = *mut OpaqueJSString;
pub type JSClassRef = *mut OpaqueJSClass;
pub type JSPropertyAttributes = c_uint;
pub type JSTypedArrayType = c_uint;

#[repr(C)]
pub struct JSClassDefinition {
    pub version: c_int,
    pub attributes: c_uint,
    pub class_name: *const c_char,
    pub parent_class: JSClassRef,
    pub static_values: *const c_void,
    pub static_functions: *const c_void,
    pub initialize: *const c_void,
    pub finalize: *const c_void,
    pub has_property: *const c_void,
    pub get_property: *const c_void,
    pub set_property: *const c_void,
    pub delete_property: *const c_void,
    pub get_property_names: *const c_void,
    pub call_as_function: *const c_void,
    pub call_as_constructor: *const c_void,
    pub has_instance: *const c_void,
    pub convert_to_type: *const c_void,
}

pub const K_JS_PROPERTY_ATTRIBUTE_NONE: JSPropertyAttributes = 0;
pub const K_JS_TYPED_ARRAY_TYPE_INT32_ARRAY: JSTypedArrayType = 2;
pub const K_JS_TYPED_ARRAY_TYPE_UINT8_CLAMPED_ARRAY: JSTypedArrayType = 4;
pub const K_JS_TYPED_ARRAY_TYPE_FLOAT32_ARRAY: JSTypedArrayType = 7;

/// Configures JavaScriptCore's Darwin exception-handler policy before initialization.
///
/// # Safety
///
/// This must run before any thread initializes JavaScriptCore or creates a VM.
#[cfg(target_os = "macos")]
pub unsafe fn allow_mach_exception_handlers() -> bool {
    // Resolve the private data symbol dynamically. A direct relocation against this mutable
    // symbol produces invalid chained fixups in Python extension dylibs on macOS.
    let address = unsafe {
        libc::dlsym(
            libc::RTLD_DEFAULT,
            c"_ZN3JSC7Options33machExceptionHandlerSandboxPolicyE".as_ptr(),
        )
    };
    if address.is_null() {
        return false;
    }
    // `SandboxPolicy::Allow` is the second u8 enum value in the pinned WebKit API.
    unsafe { address.cast::<u8>().write(1) };
    true
}

pub type JSObjectCallAsFunctionCallback = Option<
    unsafe extern "C" fn(
        ctx: JSContextRef,
        function: JSObjectRef,
        this_object: JSObjectRef,
        argument_count: usize,
        arguments: *const JSValueRef,
        exception: *mut JSValueRef,
    ) -> JSValueRef,
>;

#[cfg_attr(target_os = "macos", link(name = "JavaScriptCore", kind = "framework"))]
#[cfg_attr(not(target_os = "macos"), link(name = "JavaScriptCore"))]
unsafe extern "C" {
    #[cfg_attr(target_env = "msvc", link_name = "?initialize@JSC@@YAXXZ")]
    #[cfg_attr(not(target_env = "msvc"), link_name = "_ZN3JSC10initializeEv")]
    pub fn JSCInitialize();
    #[cfg_attr(target_env = "msvc", link_name = "?setOptions@Options@JSC@@SA_NPEBD@Z")]
    #[cfg_attr(
        not(target_env = "msvc"),
        link_name = "_ZN3JSC7Options10setOptionsEPKc"
    )]
    pub fn JSCSetOptions(options: *const c_char) -> bool;

    pub fn JSGlobalContextCreate(global_object_class: JSClassRef) -> JSGlobalContextRef;
    pub fn JSGlobalContextRelease(ctx: JSGlobalContextRef);
    pub fn JSClassCreate(definition: *const JSClassDefinition) -> JSClassRef;
    pub fn JSClassRelease(class: JSClassRef);
    pub fn JSContextGetGlobalObject(ctx: JSContextRef) -> JSObjectRef;
    pub fn JSContextGetGroup(ctx: JSContextRef) -> JSContextGroupRef;
    pub fn JSContextGroupSetExecutionTimeLimit(
        group: JSContextGroupRef,
        limit: c_double,
        callback: *const c_void,
        context: *mut c_void,
    );
    pub fn JSContextGroupClearExecutionTimeLimit(group: JSContextGroupRef);
    pub fn JSGarbageCollect(ctx: JSContextRef);

    pub fn JSEvaluateScript(
        ctx: JSContextRef,
        script: JSStringRef,
        this_object: JSObjectRef,
        source_url: JSStringRef,
        starting_line_number: c_int,
        exception: *mut JSValueRef,
    ) -> JSValueRef;

    pub fn JSValueMakeUndefined(ctx: JSContextRef) -> JSValueRef;
    pub fn JSValueMakeNull(ctx: JSContextRef) -> JSValueRef;
    pub fn JSValueMakeBoolean(ctx: JSContextRef, boolean: bool) -> JSValueRef;
    pub fn JSValueMakeNumber(ctx: JSContextRef, number: c_double) -> JSValueRef;
    pub fn JSValueMakeString(ctx: JSContextRef, string: JSStringRef) -> JSValueRef;
    pub fn JSValueIsUndefined(ctx: JSContextRef, value: JSValueRef) -> bool;
    pub fn JSValueIsNull(ctx: JSContextRef, value: JSValueRef) -> bool;
    pub fn JSValueIsObject(ctx: JSContextRef, value: JSValueRef) -> bool;
    pub fn JSValueToBoolean(ctx: JSContextRef, value: JSValueRef) -> bool;
    pub fn JSValueToNumber(
        ctx: JSContextRef,
        value: JSValueRef,
        exception: *mut JSValueRef,
    ) -> c_double;
    pub fn JSValueToStringCopy(
        ctx: JSContextRef,
        value: JSValueRef,
        exception: *mut JSValueRef,
    ) -> JSStringRef;
    pub fn JSValueToObject(
        ctx: JSContextRef,
        value: JSValueRef,
        exception: *mut JSValueRef,
    ) -> JSObjectRef;
    pub fn JSValueProtect(ctx: JSContextRef, value: JSValueRef);
    pub fn JSValueUnprotect(ctx: JSContextRef, value: JSValueRef);

    pub fn JSObjectMake(ctx: JSContextRef, class: JSClassRef, data: *mut c_void) -> JSObjectRef;
    pub fn JSObjectMakeFunctionWithCallback(
        ctx: JSContextRef,
        name: JSStringRef,
        callback: JSObjectCallAsFunctionCallback,
    ) -> JSObjectRef;
    pub fn JSObjectMakeArray(
        ctx: JSContextRef,
        argument_count: usize,
        arguments: *const JSValueRef,
        exception: *mut JSValueRef,
    ) -> JSObjectRef;
    pub fn JSObjectMakeTypedArray(
        ctx: JSContextRef,
        array_type: JSTypedArrayType,
        length: usize,
        exception: *mut JSValueRef,
    ) -> JSObjectRef;
    pub fn JSObjectGetTypedArrayBytesPtr(
        ctx: JSContextRef,
        object: JSObjectRef,
        exception: *mut JSValueRef,
    ) -> *mut c_void;
    pub fn JSObjectGetTypedArrayByteLength(
        ctx: JSContextRef,
        object: JSObjectRef,
        exception: *mut JSValueRef,
    ) -> usize;
    pub fn JSObjectGetTypedArrayByteOffset(
        ctx: JSContextRef,
        object: JSObjectRef,
        exception: *mut JSValueRef,
    ) -> usize;
    pub fn JSObjectMakeDeferredPromise(
        ctx: JSContextRef,
        resolve: *mut JSObjectRef,
        reject: *mut JSObjectRef,
        exception: *mut JSValueRef,
    ) -> JSObjectRef;
    pub fn JSObjectIsFunction(ctx: JSContextRef, object: JSObjectRef) -> bool;
    pub fn JSObjectCallAsFunction(
        ctx: JSContextRef,
        object: JSObjectRef,
        this_object: JSObjectRef,
        argument_count: usize,
        arguments: *const JSValueRef,
        exception: *mut JSValueRef,
    ) -> JSValueRef;
    pub fn JSObjectGetProperty(
        ctx: JSContextRef,
        object: JSObjectRef,
        property_name: JSStringRef,
        exception: *mut JSValueRef,
    ) -> JSValueRef;
    pub fn JSObjectSetProperty(
        ctx: JSContextRef,
        object: JSObjectRef,
        property_name: JSStringRef,
        value: JSValueRef,
        attributes: JSPropertyAttributes,
        exception: *mut JSValueRef,
    );
    pub fn JSObjectSetPrototype(ctx: JSContextRef, object: JSObjectRef, value: JSValueRef);

    pub fn JSStringCreateWithCharacters(characters: *const u16, num_chars: usize) -> JSStringRef;
    pub fn JSStringGetLength(string: JSStringRef) -> usize;
    pub fn JSStringGetCharactersPtr(string: JSStringRef) -> *const u16;
    pub fn JSStringRelease(string: JSStringRef);
}
