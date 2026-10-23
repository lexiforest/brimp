use brimp_lexbor::{self as lexbor, Document, Node};
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
};

pub type NodeId = usize;
pub use brimp_lexbor::Error as DomError;
pub const HTML_NAMESPACE: &str = "http://www.w3.org/1999/xhtml";

/// The only DOM is Lexbor's arena. These tables assign stable wrapper identities;
/// they contain no copies of DOM links, attributes, text, or style state.
pub struct BrowserDocument {
    pub(super) arena: Document,
    handles: RefCell<Handles>,
    // Detached nodes retain their logical ownerDocument. Lexbor's memory owner
    // is the shared realm arena, including detached HTML documents.
    owners: RefCell<HashMap<NodeId, NodeId>>,
    template_owners: RefCell<HashMap<NodeId, NodeId>>,
}
#[derive(Default)]
struct Handles {
    nodes: Vec<Node>,
    ids: HashMap<Node, NodeId>,
}
impl BrowserDocument {
    pub fn empty() -> Self {
        let arena = Document::new().expect("allocate Lexbor document");
        let result = Self {
            arena,
            handles: RefCell::new(Handles::default()),
            owners: RefCell::new(HashMap::new()),
            template_owners: RefCell::new(HashMap::new()),
        };
        result.register(result.arena.root());
        result
    }
    pub fn parse(html: &str) -> Self {
        let document = Self::empty();
        let mut parser = document
            .arena
            .parser(html, true)
            .expect("initialize HTML parser");
        while parser.resume().expect("parse HTML").is_some() {}
        document
    }
    pub fn root(&self) -> NodeId {
        0
    }
    pub(super) fn register(&self, node: Node) -> NodeId {
        let mut handles = self.handles.borrow_mut();
        if let Some(id) = handles.ids.get(&node) {
            return *id;
        }
        let id = handles.nodes.len();
        handles.nodes.push(node);
        handles.ids.insert(node, id);
        id
    }
    pub(super) fn token(&self, id: NodeId) -> Result<Node, DomError> {
        self.handles
            .borrow()
            .nodes
            .get(id)
            .copied()
            .ok_or(DomError::ForeignNode)
    }
    pub fn contains(&self, id: NodeId) -> bool {
        self.token(id).is_ok()
    }
    pub fn kind(&self, id: NodeId) -> Result<u32, DomError> {
        self.arena.node_type(self.token(id)?)
    }
    pub fn name(&self, id: NodeId) -> Result<String, DomError> {
        self.arena.name(self.token(id)?)
    }
    pub fn local_name(&self, id: NodeId) -> Result<String, DomError> {
        self.arena.local_name(self.token(id)?)
    }
    pub fn namespace(&self, id: NodeId) -> Result<String, DomError> {
        self.arena.namespace(self.token(id)?)
    }
    pub fn prefix(&self, id: NodeId) -> Result<String, DomError> {
        self.arena.prefix(self.token(id)?)
    }
    pub fn is_element(&self, id: NodeId) -> bool {
        self.kind(id).ok() == Some(1)
    }
    pub fn is_document(&self, id: NodeId) -> bool {
        self.kind(id).ok() == Some(9)
    }
    pub fn is_document_fragment(&self, id: NodeId) -> bool {
        self.kind(id).ok() == Some(11)
    }
    pub fn parent(&self, id: NodeId) -> Result<Option<NodeId>, DomError> {
        Ok(self
            .arena
            .parent(self.token(id)?)?
            .map(|node| self.register(node)))
    }
    pub fn children(&self, id: NodeId) -> Result<Vec<NodeId>, DomError> {
        Ok(self
            .arena
            .children(self.token(id)?)?
            .into_iter()
            .map(|node| self.register(node))
            .collect())
    }
    pub fn descendants(&self, id: NodeId) -> Result<Vec<NodeId>, DomError> {
        let mut output = Vec::new();
        let mut pending = self.children(id)?;
        pending.reverse();
        while let Some(id) = pending.pop() {
            output.push(id);
            let mut children = self.children(id)?;
            children.reverse();
            pending.extend(children);
        }
        Ok(output)
    }
    pub fn template_content(&self, id: NodeId) -> Result<Option<NodeId>, DomError> {
        let Some(node) = self.arena.template_content(self.token(id)?)? else {
            return Ok(None);
        };
        let content = self.register(node);
        let owner = self.node_document(id).unwrap_or(0);
        let inert = self.template_owners.borrow().get(&owner).copied();
        let inert = match inert {
            Some(inert) => inert,
            None => {
                let inert = self.create_document()?;
                let mut owners = self.template_owners.borrow_mut();
                owners.insert(owner, inert);
                owners.insert(inert, inert);
                inert
            }
        };
        self.owners.borrow_mut().insert(content, inert);
        Ok(Some(content))
    }
    pub fn node_document(&self, id: NodeId) -> Option<NodeId> {
        let mut current = id;
        loop {
            if self.is_document(current) {
                return Some(current);
            }
            match self.parent(current).ok()? {
                Some(parent) => current = parent,
                None => return Some(*self.owners.borrow().get(&current).unwrap_or(&0)),
            }
        }
    }
    pub fn adopt_subtree(&self, id: NodeId, owner: NodeId) -> Result<(), DomError> {
        self.token(id)?;
        self.owners.borrow_mut().insert(id, owner);
        // Template content is outside the child links and has an inert owner.
        // Update it eagerly so retained content wrappers see adoption immediately.
        let mut pending = vec![id];
        while let Some(node) = pending.pop() {
            pending.extend(self.children(node)?);
            if let Some(content) = self.template_content(node)? {
                pending.push(content);
            }
        }
        Ok(())
    }
    pub fn writer(&self, id: NodeId) -> Result<lexbor::HtmlWriter, DomError> {
        self.arena.writer_at(self.token(id)?)
    }
    pub fn writer_node_id(&self, node: lexbor::Node) -> NodeId {
        self.register(node)
    }
    pub fn create_document(&self) -> Result<NodeId, DomError> {
        Ok(self.register(self.arena.create_document()?))
    }
    pub fn create_element(
        &self,
        owner: NodeId,
        name: &str,
        namespace: &str,
        prefix: &str,
    ) -> Result<NodeId, DomError> {
        let id = self.register(self.arena.create_element(name, namespace, prefix)?);
        self.owners.borrow_mut().insert(id, owner);
        Ok(id)
    }
    pub fn create_text(&self, owner: NodeId, text: &str) -> Result<NodeId, DomError> {
        let id = self.register(self.arena.create_text(text)?);
        self.owners.borrow_mut().insert(id, owner);
        Ok(id)
    }
    pub fn create_comment(&self, owner: NodeId, text: &str) -> Result<NodeId, DomError> {
        let id = self.register(self.arena.create_comment(text)?);
        self.owners.borrow_mut().insert(id, owner);
        Ok(id)
    }
    pub fn create_document_fragment(&self, owner: NodeId) -> Result<NodeId, DomError> {
        let id = self.register(self.arena.create_fragment()?);
        self.owners.borrow_mut().insert(id, owner);
        Ok(id)
    }
    pub fn clone_node(&self, id: NodeId, deep: bool, owner: NodeId) -> Result<NodeId, DomError> {
        if self.is_document(id) {
            let clone = self.create_document()?;
            if deep {
                for child in self.children(id)? {
                    let child = self.clone_node(child, true, clone)?;
                    self.insert(clone, child, None)?;
                }
            }
            return Ok(clone);
        }
        let clone = self.register(self.arena.clone_node(self.token(id)?, deep)?);
        self.adopt_subtree(clone, owner)?;
        Ok(clone)
    }
    pub fn insert(
        &self,
        parent: NodeId,
        child: NodeId,
        before: Option<NodeId>,
    ) -> Result<(), DomError> {
        let owner = self.node_document(parent).ok_or(DomError::ForeignNode)?;
        let children = if self.is_document_fragment(child) {
            self.children(child)?
        } else {
            vec![child]
        };
        self.arena.insert(
            self.token(parent)?,
            self.token(child)?,
            before.map(|id| self.token(id)).transpose()?,
        )?;
        for child in children {
            self.adopt_subtree(child, owner)?;
        }
        Ok(())
    }
    pub fn detach(&self, id: NodeId) -> Result<(), DomError> {
        let owner = self.node_document(id).ok_or(DomError::ForeignNode)?;
        self.adopt_subtree(id, owner)?;
        self.arena.detach(self.token(id)?)
    }
    pub fn text(&self, id: NodeId) -> Result<String, DomError> {
        self.arena.text(self.token(id)?)
    }
    pub fn set_text(&self, id: NodeId, text: &str) -> Result<(), DomError> {
        // Preserve the owner of children that will be detached by replacement.
        for child in self.children(id)? {
            self.adopt_subtree(child, self.node_document(id).unwrap_or(0))?;
        }
        self.arena.set_text(self.token(id)?, text)
    }
    pub fn set_attribute_ns(
        &self,
        id: NodeId,
        ns: &str,
        name: &str,
        value: &str,
    ) -> Result<(), DomError> {
        self.arena
            .set_attribute_ns(self.token(id)?, ns, name, value)
    }
    pub fn remove_attribute_ns(&self, id: NodeId, ns: &str, local: &str) -> Result<(), DomError> {
        self.arena.remove_attribute_ns(self.token(id)?, ns, local)
    }
    pub fn attribute(&self, id: NodeId, name: &str) -> Option<String> {
        self.arena
            .attribute(self.token(id).ok()?, name)
            .ok()
            .flatten()
    }
    pub fn attributes(&self, id: NodeId) -> Result<Vec<lexbor::Attribute>, DomError> {
        self.arena.attributes(self.token(id)?)
    }
    pub fn set_attribute(&self, id: NodeId, name: &str, value: &str) -> Result<(), DomError> {
        self.arena.set_attribute(self.token(id)?, name, value)
    }
    pub fn remove_attribute(&self, id: NodeId, name: &str) -> Result<(), DomError> {
        self.arena.remove_attribute(self.token(id)?, name)
    }
    pub fn serialize(&self, id: NodeId, inner: bool) -> Result<String, DomError> {
        self.arena.serialize(self.token(id)?, inner)
    }
    pub fn outer_html(&self) -> String {
        self.serialize(0, true).expect("serialize document")
    }
    pub fn set_inner_html(&self, id: NodeId, html: &str) -> Result<(), DomError> {
        let fragment = self.arena.fragment(self.token(id)?, html)?;
        let target = self.template_content(id)?.unwrap_or(id);
        for child in self.children(target)? {
            self.detach(child)?;
        }
        self.insert(target, self.register(fragment), None)
    }
    pub fn select(&self, root: NodeId, selector: &str) -> Result<Vec<NodeId>, DomError> {
        Ok(self
            .arena
            .select(self.token(root)?, selector)?
            .into_iter()
            .map(|node| self.register(node))
            .filter(|id| *id != root)
            .collect())
    }
    pub fn matches(&self, id: NodeId, selector: &str) -> Result<bool, DomError> {
        self.arena.matches(self.token(id)?, selector)
    }
    pub fn query_selector(&self, selector: &str) -> Result<Option<NodeId>, DomError> {
        Ok(self.select(0, selector)?.into_iter().next())
    }
    pub fn query_selector_all(&self, selector: &str) -> Result<Vec<NodeId>, DomError> {
        self.select(0, selector)
    }
    pub fn get_element_by_id(&self, id: &str) -> Option<NodeId> {
        self.descendants(0)
            .ok()?
            .into_iter()
            .find(|node| self.attribute(*node, "id").as_deref() == Some(id))
    }
    pub fn window_named_properties(&self) -> Vec<String> {
        let mut names = HashSet::new();
        for id in self.descendants(0).unwrap_or_default() {
            for attribute in ["id", "name"] {
                if let Some(value) = self.attribute(id, attribute).filter(|s| !s.is_empty()) {
                    names.insert(value);
                }
            }
        }
        let mut names: Vec<_> = names.into_iter().collect();
        names.sort();
        names
    }
    pub fn inline_style_declarations(&self, id: NodeId) -> Option<Vec<lexbor::Declaration>> {
        let css = self.attribute(id, "style").unwrap_or_default();
        let mut result: Vec<lexbor::Declaration> = Vec::new();
        for declaration in lexbor::declarations(&css).ok()? {
            if let Some(existing) = result
                .iter_mut()
                .find(|(name, _, _)| *name == declaration.0)
            {
                if !existing.2 || declaration.2 {
                    *existing = declaration;
                }
            } else {
                result.push(declaration);
            }
        }
        Some(result)
    }
    pub fn inline_style_css(&self, id: NodeId) -> Option<String> {
        Some(serialize_declarations(&self.inline_style_declarations(id)?))
    }
    pub fn inline_style_property(&self, id: NodeId, name: &str) -> Option<String> {
        self.inline_style_declarations(id)?
            .into_iter()
            .find(|(key, _, _)| key == name)
            .map(|(_, value, _)| value)
    }
    pub fn set_inline_style_css(&self, id: NodeId, css: &str) {
        if let Ok(declarations) = lexbor::declarations(css) {
            let _ = self.set_attribute(id, "style", &serialize_declarations(&declarations));
        }
    }
    pub fn set_style_property(&self, id: NodeId, name: &str, value: &str) {
        let Ok(mut parsed) = lexbor::declarations(&format!("{name}: {value}")) else {
            return;
        };
        if parsed.len() != 1 || parsed[0].0 != name {
            return;
        }
        let mut entries = self.inline_style_declarations(id).unwrap_or_default();
        if let Some(entry) = entries.iter_mut().find(|(key, _, _)| key == name) {
            *entry = parsed.remove(0);
        } else {
            entries.extend(parsed);
        }
        let _ = self.set_attribute(id, "style", &serialize_declarations(&entries));
    }
    pub fn remove_style_property(&self, id: NodeId, name: &str) {
        let mut entries = self.inline_style_declarations(id).unwrap_or_default();
        entries.retain(|(key, _, _)| key != name);
        let _ = self.set_attribute(id, "style", &serialize_declarations(&entries));
    }
}
fn serialize_declarations(entries: &[lexbor::Declaration]) -> String {
    entries
        .iter()
        .map(|(name, value, important)| {
            format!(
                "{name}: {value}{};",
                if *important { " !important" } else { "" }
            )
        })
        .collect::<Vec<_>>()
        .join(" ")
}
