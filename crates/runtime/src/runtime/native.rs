//! Typed native API. JavaScript objects never leave the page owner thread.
//!
//! Sessions and page methods submit owned commands; handles refer to values rooted
//! by the owner-thread bridge. Child modules keep these responsibilities separate.

mod handles;
mod operations;
mod options;
mod page;
mod session;
pub mod streaming;

pub use handles::{ElementHandle, JSHandle};
pub use options::{FileUpload, IdentityOptions, RequestOptions, SessionOptions};
pub use page::NativePage;
pub use session::Session;

pub(crate) use operations::{HandleToken, Operation, OperationResult, execute_operation};
