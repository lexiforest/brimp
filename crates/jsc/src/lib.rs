//! Thread-bound JavaScriptCore ownership and safe native callbacks.
//!
//! Engine values and protected handles stay on their owner thread. Browser APIs
//! and page lifecycle policy belong to the runtime that embeds this crate.

mod exception;
mod ffi;
mod native;
mod object;
mod runtime;
mod string;
mod value;

pub use exception::JsException;
pub use native::{NativeArgument, NativeCall, NativeError, NativeValue};
pub use object::{
    DeferredPromise, JsObject, JsObjectIdentity, PromiseSettlement, ProtectedJsObject,
};
pub use runtime::JsRuntime;
pub use value::JsValue;
