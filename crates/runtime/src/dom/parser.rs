use brimp_lexbor as lexbor;

use super::{BrowserDocument, DomError, NodeId};
use std::{cell::RefCell, rc::Rc};

pub enum ParseProgress {
    Script(NodeId),
    Done,
}

pub struct HtmlParserSession {
    parser: lexbor::Parser,
    document: Rc<RefCell<BrowserDocument>>,
}
impl HtmlParserSession {
    pub fn new(document: Rc<RefCell<BrowserDocument>>, html: &str) -> Result<Self, DomError> {
        Self::new_at(document, html, 0, true)
    }
    pub fn new_at_root(
        document: Rc<RefCell<BrowserDocument>>,
        html: &str,
        root: NodeId,
    ) -> Result<Self, DomError> {
        Self::new_at(document, html, root, false)
    }
    fn new_at(
        document: Rc<RefCell<BrowserDocument>>,
        html: &str,
        root: NodeId,
        scripting: bool,
    ) -> Result<Self, DomError> {
        let parser = {
            let doc = document.borrow();
            doc.arena.parser_at(doc.token(root)?, html, scripting)?
        };
        Ok(Self { parser, document })
    }
    pub fn take_mutations(&mut self) -> Result<Vec<serde_json::Value>, DomError> {
        let records = self.parser.take_mutations()?;
        let document = self.document.borrow();
        Ok(records.into_iter().map(|record|serde_json::json!({"target":document.register(record.target),"node":document.register(record.node),"previous":record.previous.map(|node|document.register(node)),"next":record.next.map(|node|document.register(node)),"removed":record.removed,"oldValue":record.old_value})).collect())
    }
    pub fn resume(&mut self) -> Result<ParseProgress, DomError> {
        Ok(match self.parser.resume()? {
            Some(node) => ParseProgress::Script(self.document.borrow().register(node)),
            None => ParseProgress::Done,
        })
    }
}
