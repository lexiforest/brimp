use std::{
    borrow::Cow,
    cell::{Cell, Ref, RefCell},
    rc::Rc,
};

use blitz_dom::{Attribute, DocumentMutator, QualName};
use html5ever::{
    ParseOpts,
    tendril::StrTendril,
    tokenizer::TokenizerOpts,
    tree_builder::{ElementFlags, NodeOrText, QuirksMode, TreeBuilderOpts, TreeSink},
};

use crate::{BrowserDocument, NodeId};

pub enum ParseProgress {
    Script(NodeId),
    Done,
}

pub struct HtmlParserSession {
    parser: html5ever::Parser<SharedDocumentSink>,
    done: bool,
}

impl HtmlParserSession {
    pub fn new(document: Rc<RefCell<BrowserDocument>>, html: &str) -> Self {
        let sink = SharedDocumentSink::new(document);
        let options = ParseOpts {
            tokenizer: TokenizerOpts::default(),
            tree_builder: TreeBuilderOpts {
                exact_errors: false,
                scripting_enabled: true,
                iframe_srcdoc: false,
                drop_doctype: true,
                quirks_mode: QuirksMode::NoQuirks,
            },
        };
        let parser = html5ever::parse_document(sink, options);
        parser.input_buffer.push_back(StrTendril::from(html));
        Self {
            parser,
            done: false,
        }
    }

    pub fn resume(&mut self) -> ParseProgress {
        assert!(!self.done, "cannot resume a completed HTML parser");
        loop {
            match self.parser.tokenizer.feed(&self.parser.input_buffer) {
                html5ever::TokenizerResult::Script(node_id) => {
                    return ParseProgress::Script(node_id);
                }
                html5ever::TokenizerResult::EncodingIndicator(_) => {}
                html5ever::TokenizerResult::Done => {
                    assert!(self.parser.input_buffer.is_empty());
                    self.parser.tokenizer.end();
                    self.done = true;
                    return ParseProgress::Done;
                }
            }
        }
    }
}

struct SharedDocumentSink {
    document: Rc<RefCell<BrowserDocument>>,
    errors: RefCell<Vec<Cow<'static, str>>>,
    quirks_mode: Cell<QuirksMode>,
}

impl SharedDocumentSink {
    fn new(document: Rc<RefCell<BrowserDocument>>) -> Self {
        Self {
            document,
            errors: RefCell::new(Vec::new()),
            quirks_mode: Cell::new(QuirksMode::NoQuirks),
        }
    }

    fn mutate<R>(&self, operation: impl FnOnce(&mut DocumentMutator<'_>) -> R) -> R {
        let mut document = self.document.borrow_mut();
        let mut mutator = document.blitz_mut().mutate();
        operation(&mut mutator)
    }

    fn convert_attrs(attrs: Vec<html5ever::Attribute>) -> Vec<Attribute> {
        attrs
            .into_iter()
            .map(|attr| Attribute {
                name: attr.name,
                value: attr.value.to_string(),
            })
            .collect()
    }
}

impl TreeSink for SharedDocumentSink {
    type Output = ();
    type Handle = NodeId;
    type ElemName<'a>
        = Ref<'a, QualName>
    where
        Self: 'a;

    fn finish(self) {}

    fn parse_error(&self, message: Cow<'static, str>) {
        self.errors.borrow_mut().push(message);
    }

    fn get_document(&self) -> Self::Handle {
        0
    }

    fn elem_name<'a>(&'a self, target: &'a Self::Handle) -> Self::ElemName<'a> {
        Ref::map(self.document.borrow(), |document| {
            &document
                .node(*target)
                .and_then(|node| node.element_data())
                .expect("TreeSink::elem_name requires an element")
                .name
        })
    }

    fn create_element(
        &self,
        name: QualName,
        attrs: Vec<html5ever::Attribute>,
        _flags: ElementFlags,
    ) -> Self::Handle {
        self.mutate(|mutator| mutator.create_element(name, Self::convert_attrs(attrs)))
    }

    fn create_comment(&self, _text: StrTendril) -> Self::Handle {
        self.mutate(|mutator| mutator.create_comment_node())
    }

    fn create_pi(&self, _target: StrTendril, _data: StrTendril) -> Self::Handle {
        self.mutate(|mutator| mutator.create_comment_node())
    }

    fn append(&self, parent_id: &Self::Handle, child: NodeOrText<Self::Handle>) {
        self.mutate(|mutator| match child {
            NodeOrText::AppendNode(id) => mutator.append_children(*parent_id, &[id]),
            NodeOrText::AppendText(text) => {
                let last_child_id = mutator.last_child_id(*parent_id);
                let appended =
                    last_child_id.is_some_and(|id| mutator.append_text_to_node(id, &text).is_ok());
                if !appended {
                    let id = mutator.create_text_node(&text);
                    mutator.append_children(*parent_id, &[id]);
                }
            }
        });
    }

    fn append_before_sibling(&self, sibling_id: &Self::Handle, new_node: NodeOrText<Self::Handle>) {
        self.mutate(|mutator| match new_node {
            NodeOrText::AppendNode(id) => mutator.insert_nodes_before(*sibling_id, &[id]),
            NodeOrText::AppendText(text) => {
                let previous_id = mutator.previous_sibling_id(*sibling_id);
                let appended =
                    previous_id.is_some_and(|id| mutator.append_text_to_node(id, &text).is_ok());
                if !appended {
                    let id = mutator.create_text_node(&text);
                    mutator.insert_nodes_before(*sibling_id, &[id]);
                }
            }
        });
    }

    fn append_based_on_parent_node(
        &self,
        element: &Self::Handle,
        previous_element: &Self::Handle,
        child: NodeOrText<Self::Handle>,
    ) {
        let has_parent = self.mutate(|mutator| mutator.node_has_parent(*element));
        if has_parent {
            self.append_before_sibling(element, child);
        } else {
            self.append(previous_element, child);
        }
    }

    fn append_doctype_to_document(
        &self,
        _name: StrTendril,
        _public_id: StrTendril,
        _system_id: StrTendril,
    ) {
    }

    fn get_template_contents(&self, target: &Self::Handle) -> Self::Handle {
        *target
    }

    fn same_node(&self, left: &Self::Handle, right: &Self::Handle) -> bool {
        left == right
    }

    fn set_quirks_mode(&self, mode: QuirksMode) {
        self.quirks_mode.set(mode);
    }

    fn add_attrs_if_missing(&self, target: &Self::Handle, attrs: Vec<html5ever::Attribute>) {
        self.mutate(|mutator| {
            mutator.add_attrs_if_missing(*target, Self::convert_attrs(attrs));
        });
    }

    fn remove_from_parent(&self, target: &Self::Handle) {
        self.mutate(|mutator| mutator.remove_node(*target));
    }

    fn reparent_children(&self, old_parent: &Self::Handle, new_parent: &Self::Handle) {
        self.mutate(|mutator| mutator.reparent_children(*old_parent, *new_parent));
    }
}
