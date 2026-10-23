//! Thread-bound ownership boundary for the pinned Lexbor DOM.
//!
//! All nodes belong to a document arena. Detaching or replacing children never
//! frees nodes; the arena is released after the document and its parser are gone.
//! Handles contain an owner identity and cannot be dereferenced by another document.
use std::{
    cell::RefCell,
    collections::HashSet,
    ffi::c_void,
    ptr::NonNull,
    rc::Rc,
    sync::atomic::{AtomicU64, Ordering},
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Lexbor allocation or parser failure")]
    Native,
    #[error("node belongs to another document or an expired document")]
    ForeignNode,
    #[error("document parsing has already started")]
    AlreadyParsed,
    #[error("invalid CSS selector")]
    Selector,
    #[error("invalid DOM insertion")]
    Hierarchy,
}

#[repr(C)]
struct RawDocument {
    _opaque: [u8; 0],
}
#[repr(C)]
struct RawParser {
    _opaque: [u8; 0],
}
#[repr(C)]
struct RawNode {
    _opaque: [u8; 0],
}
#[repr(C)]
#[derive(Default)]
struct Bytes {
    data: *mut u8,
    len: usize,
    capacity: usize,
}
impl Bytes {
    fn string(&self) -> String {
        // SAFETY: C initialized this buffer, and it remains owned until Drop.
        String::from_utf8_lossy(unsafe { slice(self.data, self.len) }).into_owned()
    }
}
impl Drop for Bytes {
    fn drop(&mut self) {
        // SAFETY: the buffer is either zeroed or allocated by the shim.
        unsafe { br_bytes_free(self) }
    }
}
#[repr(C)]
struct RawMutation {
    target: *mut RawNode,
    node: *mut RawNode,
    previous: *mut RawNode,
    next: *mut RawNode,
    removed: bool,
    character_data: bool,
    old_data: *const u8,
    old_length: usize,
}
pub struct ParserMutation {
    pub target: Node,
    pub node: Node,
    pub previous: Option<Node>,
    pub next: Option<Node>,
    pub removed: bool,
    pub old_value: Option<String>,
}
unsafe extern "C" {
    fn br_document_new() -> *mut RawDocument;
    fn br_document_free(doc: *mut RawDocument);
    fn br_document_root(doc: *mut RawDocument) -> *mut RawNode;
    fn br_writer_new(root: *mut RawNode) -> *mut RawParser;
    fn br_writer_feed(
        parser: *mut RawParser,
        input: *const u8,
        len: usize,
        offset: *mut usize,
        script: *mut *mut RawNode,
    ) -> i32;
    fn br_parser_new(root: *mut RawNode, scripting: bool) -> *mut RawParser;
    fn br_parser_free(parser: *mut RawParser);
    fn br_parser_mutation_count(parser: *mut RawParser) -> usize;
    fn br_parser_mutation_at(parser: *mut RawParser, index: usize, out: *mut RawMutation) -> bool;
    fn br_parser_mutations_clear(parser: *mut RawParser);

    fn br_parser_resume(
        parser: *mut RawParser,
        input: *const u8,
        len: usize,
        offset: *mut usize,
        script: *mut *mut RawNode,
    ) -> i32;
    fn br_fragment(
        doc: *mut RawDocument,
        context: *mut RawNode,
        input: *const u8,
        len: usize,
    ) -> *mut RawNode;
    fn br_node_type(node: *mut RawNode) -> u32;
    fn br_node_parent(node: *mut RawNode) -> *mut RawNode;
    fn br_node_first(node: *mut RawNode) -> *mut RawNode;
    fn br_node_next(node: *mut RawNode) -> *mut RawNode;
    fn br_node_template_content(node: *mut RawNode) -> *mut RawNode;
    fn br_node_name(node: *mut RawNode, part: u32, out: *mut Bytes) -> i32;
    fn br_node_text(node: *mut RawNode, out: *mut Bytes) -> i32;
    fn br_node_set_text(node: *mut RawNode, input: *const u8, len: usize) -> i32;
    fn br_node_serialize(node: *mut RawNode, inner: bool, out: *mut Bytes) -> i32;
    fn br_node_create(
        doc: *mut RawDocument,
        kind: u32,
        input: *const u8,
        len: usize,
        ns: *const u8,
        ns_len: usize,
        prefix: *const u8,
        prefix_len: usize,
    ) -> *mut RawNode;
    fn br_node_clone(node: *mut RawNode, deep: bool) -> *mut RawNode;
    fn br_node_detach(node: *mut RawNode);
    fn br_node_clear(node: *mut RawNode);
    fn br_node_insert(parent: *mut RawNode, child: *mut RawNode, before: *mut RawNode) -> i32;
    fn br_node_attribute(node: *mut RawNode, name: *const u8, len: usize, out: *mut Bytes) -> i32;
    fn br_node_set_attribute(
        node: *mut RawNode,
        name: *const u8,
        len: usize,
        value: *const u8,
        value_len: usize,
    ) -> i32;
    fn br_node_remove_attribute(node: *mut RawNode, name: *const u8, len: usize) -> i32;
    fn br_node_set_attribute_ns(
        node: *mut RawNode,
        ns: *const u8,
        ns_len: usize,
        name: *const u8,
        name_len: usize,
        value: *const u8,
        value_len: usize,
    ) -> i32;
    fn br_node_remove_attribute_ns(
        node: *mut RawNode,
        ns: *const u8,
        ns_len: usize,
        local: *const u8,
        local_len: usize,
    ) -> i32;
    fn br_node_attributes(
        node: *mut RawNode,
        callback: unsafe extern "C" fn(
            *const u8,
            usize,
            *const u8,
            usize,
            *const u8,
            usize,
            *const u8,
            usize,
            *const u8,
            usize,
            *mut c_void,
        ),
        ctx: *mut c_void,
    );
    fn br_select(
        node: *mut RawNode,
        selector: *const u8,
        len: usize,
        matches: bool,
        callback: unsafe extern "C" fn(*mut RawNode, *mut c_void),
        ctx: *mut c_void,
    ) -> i32;
    fn br_stylesheet(
        css: *const u8,
        len: usize,
        callback: unsafe extern "C" fn(u32, u32, *const u8, usize, *const u8, usize, *mut c_void),
        ctx: *mut c_void,
    ) -> i32;
    fn br_declarations(
        css: *const u8,
        len: usize,
        callback: unsafe extern "C" fn(*const u8, usize, *const u8, usize, bool, *mut c_void),
        ctx: *mut c_void,
    ) -> i32;
    fn br_bytes_free(bytes: *mut Bytes);
}

/// This token never dereferences itself, and carries no ownership of its arena.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct Node {
    owner: u64,
    raw: NonNull<RawNode>,
}

struct DocumentInner {
    raw: NonNull<RawDocument>,
    identity: u64,
    parsing_started: RefCell<HashSet<Node>>,
}
impl Drop for DocumentInner {
    fn drop(&mut self) {
        // SAFETY: this is the final Rc, so no parser or operation can use the arena.
        unsafe { br_document_free(self.raw.as_ptr()) }
    }
}

/// Rc prevents sending a document or parser to another thread. No mirrored tree
/// or cached attribute/text values are kept in Rust.
#[derive(Clone)]
pub struct Document(Rc<DocumentInner>);

impl Document {
    pub fn new() -> Result<Self, Error> {
        static NEXT_OWNER: AtomicU64 = AtomicU64::new(1);
        // SAFETY: constructor takes no borrowed pointers.
        let raw = NonNull::new(unsafe { br_document_new() }).ok_or(Error::Native)?;
        Ok(Self(Rc::new(DocumentInner {
            raw,
            identity: NEXT_OWNER.fetch_add(1, Ordering::Relaxed),
            parsing_started: RefCell::new(HashSet::new()),
        })))
    }
    fn token(&self, raw: *mut RawNode) -> Result<Node, Error> {
        Ok(Node {
            owner: self.0.identity,
            raw: NonNull::new(raw).ok_or(Error::Native)?,
        })
    }
    fn raw(&self, node: Node) -> Result<*mut RawNode, Error> {
        if node.owner != self.0.identity {
            return Err(Error::ForeignNode);
        }
        Ok(node.raw.as_ptr())
    }
    pub fn root(&self) -> Node {
        // SAFETY: the document arena is alive and always has a document node.
        self.token(unsafe { br_document_root(self.0.raw.as_ptr()) })
            .expect("Lexbor document root")
    }
    pub fn parser(&self, html: &str, scripting: bool) -> Result<Parser, Error> {
        self.parser_at(self.root(), html, scripting)
    }
    pub fn parser_at(&self, root: Node, html: &str, scripting: bool) -> Result<Parser, Error> {
        let raw_root = self.raw(root)?;
        if !self.0.parsing_started.borrow_mut().insert(root) {
            return Err(Error::AlreadyParsed);
        }
        // SAFETY: the resulting parser retains this document through its Rc.
        let raw =
            NonNull::new(unsafe { br_parser_new(raw_root, scripting) }).ok_or(Error::Native)?;
        Ok(Parser {
            raw,
            document: self.clone(),
            source: html.as_bytes().to_vec(),
            offset: 0,
            done: false,
        })
    }
    pub fn writer_at(&self, root: Node) -> Result<HtmlWriter, Error> {
        let root = self.raw(root)?;
        // SAFETY: the shim requires an empty document, retained by the parser.
        let raw = NonNull::new(unsafe { br_writer_new(root) }).ok_or(Error::Native)?;
        Ok(HtmlWriter {
            parser: Parser {
                raw,
                document: self.clone(),
                source: Vec::new(),
                offset: 0,
                done: false,
            },
            chunks: Vec::new(),
            pending: Vec::new(),
        })
    }
    pub fn parse(html: &str) -> Result<Self, Error> {
        let document = Self::new()?;
        let mut parser = document.parser(html, true)?;
        while parser.resume()?.is_some() {}
        Ok(document)
    }
    pub fn fragment(&self, context: Node, html: &str) -> Result<Node, Error> {
        let context = self.raw(context)?;
        // SAFETY: the context belongs to this live arena; the shim uses an
        // independent parser, so a suspended document parser is unaffected.
        let temporary = self.token(unsafe {
            br_fragment(self.0.raw.as_ptr(), context, html.as_ptr(), html.len())
        })?;
        let fragment = self.create_fragment()?;
        for child in self.children(temporary)? {
            self.insert(fragment, child, None)?;
        }
        Ok(fragment)
    }
    pub fn node_type(&self, node: Node) -> Result<u32, Error> {
        let node = self.raw(node)?;
        // SAFETY: validated live arena membership.
        Ok(unsafe { br_node_type(node) })
    }
    pub fn parent(&self, node: Node) -> Result<Option<Node>, Error> {
        let node = self.raw(node)?;
        // SAFETY: validated live arena membership; a null parent is normal.
        Ok(
            NonNull::new(unsafe { br_node_parent(node) }).map(|raw| Node {
                owner: self.0.identity,
                raw,
            }),
        )
    }
    pub fn children(&self, node: Node) -> Result<Vec<Node>, Error> {
        let node = self.raw(node)?;
        let mut children = Vec::new();
        // SAFETY: all linked siblings belong to the validated arena. No mutation
        // is possible during this synchronous owner-thread traversal.
        unsafe {
            let mut child = br_node_first(node);
            while !child.is_null() {
                children.push(self.token(child)?);
                child = br_node_next(child);
            }
        }
        Ok(children)
    }
    pub fn template_content(&self, node: Node) -> Result<Option<Node>, Error> {
        let node = self.raw(node)?;
        // SAFETY: the shim checks both tag and namespace before accessing template data.
        Ok(
            NonNull::new(unsafe { br_node_template_content(node) }).map(|raw| Node {
                owner: self.0.identity,
                raw,
            }),
        )
    }
    pub fn name(&self, node: Node) -> Result<String, Error> {
        self.name_part(node, 0)
    }
    pub fn local_name(&self, node: Node) -> Result<String, Error> {
        self.name_part(node, 1)
    }
    pub fn namespace(&self, node: Node) -> Result<String, Error> {
        self.name_part(node, 2)
    }
    pub fn prefix(&self, node: Node) -> Result<String, Error> {
        self.name_part(node, 3)
    }
    fn name_part(&self, node: Node, part: u32) -> Result<String, Error> {
        let node = self.raw(node)?;
        let mut out = Bytes::default();
        // SAFETY: node is live and out is writable; the shim copies borrowed names.
        status(unsafe { br_node_name(node, part, &mut out) })?;
        Ok(out.string())
    }
    pub fn text(&self, node: Node) -> Result<String, Error> {
        let node = self.raw(node)?;
        let mut out = Bytes::default();
        // SAFETY: node is live and out is writable.
        status(unsafe { br_node_text(node, &mut out) })?;
        Ok(out.string())
    }
    pub fn set_text(&self, node: Node, text: &str) -> Result<(), Error> {
        let node = self.raw(node)?;
        // SAFETY: the input is borrowed only for this call. Existing children
        // are detached, never destroyed, so all other tokens remain valid.
        status(unsafe { br_node_set_text(node, text.as_ptr(), text.len()) })
    }
    pub fn serialize(&self, node: Node, inner: bool) -> Result<String, Error> {
        let node = self.raw(node)?;
        let mut out = Bytes::default();
        // SAFETY: the serializer reads the arena synchronously and owns out's bytes.
        status(unsafe { br_node_serialize(node, inner, &mut out) })?;
        Ok(out.string())
    }
    pub fn create_element(&self, name: &str, namespace: &str, prefix: &str) -> Result<Node, Error> {
        self.create(1, name, namespace, prefix)
    }
    pub fn create_text(&self, text: &str) -> Result<Node, Error> {
        self.create(3, text, "", "")
    }
    pub fn create_comment(&self, text: &str) -> Result<Node, Error> {
        self.create(8, text, "", "")
    }
    pub fn create_fragment(&self) -> Result<Node, Error> {
        self.create(11, "", "", "")
    }
    /// A detached HTML document sharing this arena's lifetime.
    pub fn create_document(&self) -> Result<Node, Error> {
        self.create(9, "", "", "")
    }
    fn create(&self, kind: u32, input: &str, namespace: &str, prefix: &str) -> Result<Node, Error> {
        // SAFETY: all strings are borrowed for the call and copied into this arena.
        self.token(unsafe {
            br_node_create(
                self.0.raw.as_ptr(),
                kind,
                input.as_ptr(),
                input.len(),
                namespace.as_ptr(),
                namespace.len(),
                prefix.as_ptr(),
                prefix.len(),
            )
        })
    }
    pub fn clone_node(&self, node: Node, deep: bool) -> Result<Node, Error> {
        let node = self.raw(node)?;
        // SAFETY: cloning allocates new nodes in the same live arena.
        self.token(unsafe { br_node_clone(node, deep) })
    }
    pub fn detach(&self, node: Node) -> Result<(), Error> {
        let node = self.raw(node)?;
        // SAFETY: detachment does not release any arena storage.
        unsafe { br_node_detach(node) };
        Ok(())
    }
    pub fn clear(&self, node: Node) -> Result<(), Error> {
        let node = self.raw(node)?;
        // SAFETY: children are detached, not deallocated.
        unsafe { br_node_clear(node) };
        Ok(())
    }
    pub fn insert(&self, parent: Node, child: Node, before: Option<Node>) -> Result<(), Error> {
        let parent = self.raw(parent)?;
        let child = self.raw(child)?;
        let before = before
            .map(|node| self.raw(node))
            .transpose()?
            .unwrap_or(std::ptr::null_mut());
        // SAFETY: every node was validated against this arena; the shim rejects
        // cycles and invalid reference children before mutating links.
        if unsafe { br_node_insert(parent, child, before) } == 0 {
            Ok(())
        } else {
            Err(Error::Hierarchy)
        }
    }
    pub fn attribute(&self, node: Node, name: &str) -> Result<Option<String>, Error> {
        let node = self.raw(node)?;
        let mut out = Bytes::default();
        // SAFETY: all pointers are live for this call; returned text is copied.
        match unsafe { br_node_attribute(node, name.as_ptr(), name.len(), &mut out) } {
            0 => Ok(Some(out.string())),
            1 => Ok(None),
            _ => Err(Error::Native),
        }
    }
    pub fn set_attribute(&self, node: Node, name: &str, value: &str) -> Result<(), Error> {
        let node = self.raw(node)?;
        // SAFETY: the strings are copied into this validated node's arena.
        status(unsafe {
            br_node_set_attribute(node, name.as_ptr(), name.len(), value.as_ptr(), value.len())
        })
    }
    pub fn remove_attribute(&self, node: Node, name: &str) -> Result<(), Error> {
        let node = self.raw(node)?;
        // SAFETY: the shim validates the node kind; no node storage is released.
        status(unsafe { br_node_remove_attribute(node, name.as_ptr(), name.len()) })
    }
    pub fn set_attribute_ns(
        &self,
        node: Node,
        ns: &str,
        name: &str,
        value: &str,
    ) -> Result<(), Error> {
        let node = self.raw(node)?;
        // SAFETY: validated arena node, borrowed strings are copied by the shim.
        status(unsafe {
            br_node_set_attribute_ns(
                node,
                ns.as_ptr(),
                ns.len(),
                name.as_ptr(),
                name.len(),
                value.as_ptr(),
                value.len(),
            )
        })
    }
    pub fn remove_attribute_ns(&self, node: Node, ns: &str, local: &str) -> Result<(), Error> {
        let node = self.raw(node)?;
        // SAFETY: validated arena node and strings live for this synchronous call.
        status(unsafe {
            br_node_remove_attribute_ns(node, ns.as_ptr(), ns.len(), local.as_ptr(), local.len())
        })
    }
    pub fn attributes(&self, node: Node) -> Result<Vec<Attribute>, Error> {
        let node = self.raw(node)?;
        let mut output = Vec::<Attribute>::new();
        // SAFETY: callback/context match, the call is synchronous, and callbacks
        // only copy bytes into output without touching the arena.
        unsafe {
            br_node_attributes(
                node,
                attr_callback,
                (&mut output as *mut Vec<Attribute>).cast(),
            )
        };
        Ok(output)
    }
    pub fn select(&self, root: Node, selector: &str) -> Result<Vec<Node>, Error> {
        self.query(root, selector, false)
    }
    pub fn matches(&self, node: Node, selector: &str) -> Result<bool, Error> {
        Ok(!self.query(node, selector, true)?.is_empty())
    }
    fn query(&self, root: Node, selector: &str, matches: bool) -> Result<Vec<Node>, Error> {
        let root = self.raw(root)?;
        let mut output = Vec::<*mut RawNode>::new();
        // SAFETY: the callback only collects nodes into the live output vector.
        let result = unsafe {
            br_select(
                root,
                selector.as_ptr(),
                selector.len(),
                matches,
                node_callback,
                (&mut output as *mut Vec<*mut RawNode>).cast(),
            )
        };
        if result != 0 {
            return Err(Error::Selector);
        }
        output.into_iter().map(|node| self.token(node)).collect()
    }
}

pub struct Parser {
    raw: NonNull<RawParser>,
    document: Document,
    source: Vec<u8>,
    offset: usize,
    done: bool,
}
impl Parser {
    pub fn take_mutations(&mut self) -> Result<Vec<ParserMutation>, Error> {
        let mut result = Vec::new();
        // SAFETY: the parser and its arena are alive on this owner thread. Pointers
        // are converted to arena tokens before the native queue is cleared.
        unsafe {
            for index in 0..br_parser_mutation_count(self.raw.as_ptr()) {
                let mut raw = std::mem::MaybeUninit::uninit();
                if !br_parser_mutation_at(self.raw.as_ptr(), index, raw.as_mut_ptr()) {
                    return Err(Error::Native);
                }
                let raw = raw.assume_init();
                result.push(ParserMutation {
                    target: self.document.token(raw.target)?,
                    node: self.document.token(raw.node)?,
                    previous: if raw.previous.is_null() {
                        None
                    } else {
                        Some(self.document.token(raw.previous)?)
                    },
                    next: if raw.next.is_null() {
                        None
                    } else {
                        Some(self.document.token(raw.next)?)
                    },
                    removed: raw.removed,
                    old_value: if raw.character_data {
                        Some(
                            String::from_utf8_lossy(std::slice::from_raw_parts(
                                raw.old_data,
                                raw.old_length,
                            ))
                            .into_owned(),
                        )
                    } else {
                        None
                    },
                });
            }
            br_parser_mutations_clear(self.raw.as_ptr());
        }
        Ok(result)
    }

    /// Returns a completed script node, before any subsequent markup is consumed.
    pub fn resume(&mut self) -> Result<Option<Node>, Error> {
        if self.done {
            return Ok(None);
        }
        let mut script = std::ptr::null_mut();
        // SAFETY: parser and document remain alive; source stays at the same
        // address throughout parsing, including across suspended script execution.
        match unsafe {
            br_parser_resume(
                self.raw.as_ptr(),
                self.source.as_ptr(),
                self.source.len(),
                &mut self.offset,
                &mut script,
            )
        } {
            0 => {
                self.done = true;
                Ok(None)
            }
            1 => self.document.token(script).map(Some),
            _ => {
                self.done = true;
                Err(Error::Native)
            }
        }
    }
}
impl Drop for Parser {
    fn drop(&mut self) {
        // SAFETY: destroy the parser before dropping its document Rc or source.
        unsafe { br_parser_free(self.raw.as_ptr()) }
    }
}

pub type Declaration = (String, String, bool);
pub fn declarations(css: &str) -> Result<Vec<Declaration>, Error> {
    // Lexbor's declaration-list entry point expects the first declaration token.
    let css = css.trim_start_matches([' ', '\t', '\n', '\r', '\x0c']);
    let mut output = Vec::<Declaration>::new();
    // SAFETY: callback/context match and are valid throughout this synchronous call.
    status(unsafe {
        br_declarations(
            css.as_ptr(),
            css.len(),
            declaration_callback,
            (&mut output as *mut Vec<Declaration>).cast(),
        )
    })?;
    Ok(output)
}
fn status(code: i32) -> Result<(), Error> {
    if code == 0 {
        Ok(())
    } else {
        Err(Error::Native)
    }
}
unsafe fn slice<'a>(data: *const u8, len: usize) -> &'a [u8] {
    if len == 0 {
        return &[];
    }
    // SAFETY: callers supply a C-owned readable buffer with the stated length.
    unsafe { std::slice::from_raw_parts(data, len) }
}
unsafe extern "C" fn node_callback(node: *mut RawNode, context: *mut c_void) {
    // SAFETY: query supplies this exact vector type and a live mutable pointer.
    unsafe { (&mut *context.cast::<Vec<*mut RawNode>>()).push(node) };
}
#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Attribute {
    pub name: String,
    pub local_name: String,
    #[serde(rename = "namespaceURI")]
    pub namespace: Option<String>,
    pub prefix: Option<String>,
    pub value: String,
}
unsafe extern "C" fn attr_callback(
    name: *const u8,
    name_len: usize,
    local: *const u8,
    local_len: usize,
    namespace: *const u8,
    namespace_len: usize,
    prefix: *const u8,
    prefix_len: usize,
    value: *const u8,
    value_len: usize,
    context: *mut c_void,
) {
    // SAFETY: attributes() supplies this vector; all C buffers live for the callback.
    unsafe {
        let string = |data, len| String::from_utf8_lossy(slice(data, len)).into_owned();
        (&mut *context.cast::<Vec<Attribute>>()).push(Attribute {
            name: string(name, name_len),
            local_name: string(local, local_len),
            namespace: (namespace_len != 0).then(|| string(namespace, namespace_len)),
            prefix: (prefix_len != 0).then(|| string(prefix, prefix_len)),
            value: string(value, value_len),
        });
    }
}
unsafe extern "C" fn declaration_callback(
    name: *const u8,
    name_len: usize,
    value: *const u8,
    value_len: usize,
    important: bool,
    context: *mut c_void,
) {
    // SAFETY: declarations() supplies this vector and the shim borrows live buffers.
    unsafe {
        (&mut *context.cast::<Vec<Declaration>>()).push((
            String::from_utf8_lossy(slice(name, name_len)).into_owned(),
            String::from_utf8_lossy(slice(value, value_len)).into_owned(),
            important,
        ))
    };
}

/// A flattened, preorder CSS rule tree. Depth identifies a rule's parent.
#[derive(Debug, serde::Serialize)]
pub struct CssRule {
    pub depth: u32,
    pub kind: u32,
    pub prelude: String,
    pub declarations: String,
}
pub fn stylesheet(css: &str) -> Result<Vec<CssRule>, Error> {
    let mut rules = Vec::<CssRule>::new();
    // SAFETY: callback copies parser-owned data synchronously into the live vector.
    status(unsafe {
        br_stylesheet(
            css.as_ptr(),
            css.len(),
            rule_callback,
            (&mut rules as *mut Vec<CssRule>).cast(),
        )
    })?;
    Ok(rules)
}
unsafe extern "C" fn rule_callback(
    depth: u32,
    kind: u32,
    prelude: *const u8,
    prelude_len: usize,
    declarations: *const u8,
    declarations_len: usize,
    ctx: *mut c_void,
) {
    // SAFETY: br_stylesheet supplies live byte spans and the matching vector.
    let (rules, prelude, declarations) = unsafe {
        (
            &mut *ctx.cast::<Vec<CssRule>>(),
            slice(prelude, prelude_len),
            slice(declarations, declarations_len),
        )
    };
    rules.push(CssRule {
        depth,
        kind,
        prelude: String::from_utf8_lossy(prelude).trim().to_owned(),
        declarations: String::from_utf8_lossy(declarations).into_owned(),
    });
}

/// Incremental HTML document writer. Buffers remain pinned until parser teardown,
/// including incomplete tokens split across writes and reentrant script writes.
pub struct HtmlWriter {
    parser: Parser,
    chunks: Vec<(Box<[u8]>, usize)>,
    pending: Vec<usize>,
}
impl HtmlWriter {
    pub fn write(&mut self, html: &str) {
        self.pending.push(self.chunks.len());
        self.chunks.push((html.as_bytes().into(), 0));
    }
    pub fn resume(&mut self) -> Result<Option<Node>, Error> {
        let Some(&index) = self.pending.last() else {
            return Ok(None);
        };
        let (source, offset) = &mut self.chunks[index];
        let mut script = std::ptr::null_mut();
        // SAFETY: the parser retains the arena and chunks retain stable input buffers.
        match unsafe {
            br_writer_feed(
                self.parser.raw.as_ptr(),
                source.as_ptr(),
                source.len(),
                offset,
                &mut script,
            )
        } {
            0 => {
                self.pending.pop();
                Ok(None)
            }
            1 => self.parser.document.token(script).map(Some),
            _ => Err(Error::Native),
        }
    }
    pub fn finish(&mut self) -> Result<(), Error> {
        self.parser.resume().map(|_| ())
    }
    pub fn take_mutations(&mut self) -> Result<Vec<ParserMutation>, Error> {
        self.parser.take_mutations()
    }
}
