//! Canonical Lexbor DOM with document-owned node identities.
mod document;
mod encoding;
mod parser;
pub use document::{BrowserDocument, DomError, HTML_NAMESPACE, NodeId};
pub(crate) use encoding::decode_html;
pub use parser::{HtmlParserSession, ParseProgress};
