//! The canonical Blitz-backed document used by the browser runtime.

mod document;
mod parser;

pub use document::{BrowserDocument, NodeId, SelectorError};
pub use parser::{HtmlParserSession, ParseProgress};
