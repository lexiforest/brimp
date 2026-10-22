//! Canonical Lexbor DOM with document-owned node identities.
mod document;
mod parser;
pub use document::{BrowserDocument, DomError, HTML_NAMESPACE, NodeId};
pub use parser::{HtmlParserSession, ParseProgress};
