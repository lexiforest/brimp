//! Interface-owned profiles shared by transport, JavaScript and browser capture.

mod config;
mod error;

pub use config::BrowserProfile;
pub(crate) use config::{CORE_SOURCE, REGISTRY_SOURCE};
pub use error::ProfileError;

#[cfg(test)]
mod tests;
