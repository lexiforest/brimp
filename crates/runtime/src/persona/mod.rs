//! Browser persona configuration shared by transport and JavaScript bindings.

mod config;
mod error;
mod options;

pub use config::PersonaConfig;
pub use error::PersonaConfigError;
pub use options::*;

#[cfg(test)]
mod tests;
