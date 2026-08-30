use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    fmt,
    rc::Rc,
    sync::{Arc, OnceLock},
};

use blitz_dom::node::SpecialElementData;
use blitz_dom::{BaseDocument, DocumentConfig, Node, NodeData, QualName, local_name, ns};
use blitz_html::HtmlProvider;
use blitz_traits::net::NetProvider;
use blitz_traits::shell::{ColorScheme, Viewport};
use parley::{
    FontContext,
    fontique::{Blob, Collection, CollectionOptions, GenericFamily, SourceCache},
};
use style::{
    properties::{
        LonghandId, PropertyDeclarationBlock, PropertyDeclarationId, PropertyId, property_counts,
    },
    shared_lock::ToCssWithGuard,
    stylesheets::{
        AllowImportRules, CssRule, CssRuleTypes, DocumentStyleSheet, RulesMutateError,
        StylesheetInDocument,
    },
};
use style_traits::ToCss;

use crate::{HtmlParserSession, ParseProgress};

static WENQUANYI_MICRO_HEI: &[u8] = include_bytes!("../assets/fonts/wqy-microhei.ttc");
static NOTO_COLOR_EMOJI: &[u8] = include_bytes!("../assets/fonts/noto-color-emoji.ttf");

const WENQUANYI_FAMILY: &str = "WenQuanYi Micro Hei";
const WENQUANYI_MONO_FAMILY: &str = "WenQuanYi Micro Hei Mono";
const NOTO_COLOR_EMOJI_FAMILY: &str = "Noto Color Emoji";

pub type NodeId = usize;

fn cssom_custom_property_name(name: &style::custom_properties::Name) -> String {
    let name: &str = name.as_ref();
    format!("--{name}")
}

fn cssom_declaration_entries(block: &PropertyDeclarationBlock) -> Vec<(String, String, bool)> {
    block
        .declaration_importance_iter()
        .filter_map(|(declaration, importance)| {
            let mut value = String::new();
            declaration.to_css(&mut value).ok()?;
            let name = match declaration.id() {
                PropertyDeclarationId::Longhand(id) => id.name().to_owned(),
                PropertyDeclarationId::Custom(name) => cssom_custom_property_name(name),
            };
            Some((name, value, importance.important()))
        })
        .collect()
}

fn cssom_declaration_property(
    block: &PropertyDeclarationBlock,
    property: &PropertyId,
) -> Option<String> {
    let mut css = String::new();
    block.property_value_to_css(property, &mut css).ok()?;
    Some(css)
}

fn enabled_longhand_ids() -> impl Iterator<Item = LonghandId> {
    (0..property_counts::LONGHANDS).filter_map(|index| {
        // Stylo generates LonghandId as a contiguous repr(u16) enum with exactly
        // property_counts::LONGHANDS variants.
        let id = unsafe { std::mem::transmute::<u16, LonghandId>(index as u16) };
        PropertyId::parse_enabled_for_all_content(id.name())
            .is_ok()
            .then_some(id)
    })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CssomError {
    Syntax,
    IndexSize,
    HierarchyRequest,
    InvalidState,
    NotAStyleSheet,
}

impl From<RulesMutateError> for CssomError {
    fn from(error: RulesMutateError) -> Self {
        match error {
            RulesMutateError::Syntax => Self::Syntax,
            RulesMutateError::IndexSize => Self::IndexSize,
            RulesMutateError::HierarchyRequest => Self::HierarchyRequest,
            RulesMutateError::InvalidState => Self::InvalidState,
        }
    }
}

/// Owns the one and only DOM tree for a page.
///
/// This is deliberately a thin boundary around Blitz. It stores no parallel
/// tree or copied node state; JavaScript bindings will refer back to these
/// native node identifiers.
pub struct BrowserDocument {
    inner: BaseDocument,
    comment_data: HashMap<NodeId, String>,
    document_fragments: HashSet<NodeId>,
    documents: HashSet<NodeId>,
    node_documents: HashMap<NodeId, NodeId>,
}

impl BrowserDocument {
    pub fn parse(html: &str) -> Self {
        Self::parse_at(html, None)
    }

    pub fn parse_at(html: &str, base_url: Option<&str>) -> Self {
        Self::parse_at_with_net(html, base_url, None)
    }

    pub fn parse_at_with_net(
        html: &str,
        base_url: Option<&str>,
        net_provider: Option<Arc<dyn NetProvider>>,
    ) -> Self {
        let document = Rc::new(RefCell::new(Self::empty_at_with_net(
            base_url,
            net_provider,
        )));
        let mut parser = HtmlParserSession::new(Rc::clone(&document), html);
        while !matches!(parser.resume(), ParseProgress::Done) {}
        drop(parser);
        match Rc::try_unwrap(document) {
            Ok(document) => document.into_inner(),
            Err(_) => unreachable!("the completed parser retains no document references"),
        }
    }

    pub fn empty_at_with_net(
        base_url: Option<&str>,
        net_provider: Option<Arc<dyn NetProvider>>,
    ) -> Self {
        Self {
            inner: BaseDocument::new(Self::config(base_url, net_provider)),
            comment_data: HashMap::new(),
            document_fragments: HashSet::new(),
            documents: HashSet::from([0]),
            node_documents: HashMap::new(),
        }
    }

    pub fn create_comment(&mut self, data: &str) -> NodeId {
        let id = self.inner.mutate().create_comment_node();
        self.comment_data.insert(id, data.to_owned());
        id
    }

    pub fn comment_data(&self, node_id: NodeId) -> Option<&str> {
        self.comment_data.get(&node_id).map(String::as_str)
    }

    pub fn set_comment_data(&mut self, node_id: NodeId, data: &str) -> bool {
        let Some(value) = self.comment_data.get_mut(&node_id) else {
            return false;
        };
        data.clone_into(value);
        true
    }

    pub fn create_document_fragment(&mut self) -> NodeId {
        let id = self.inner.create_node(NodeData::Document);
        self.document_fragments.insert(id);
        id
    }

    pub fn is_document_fragment(&self, node_id: NodeId) -> bool {
        self.document_fragments.contains(&node_id)
    }

    pub fn create_document(&mut self) -> NodeId {
        let id = self.inner.create_node(NodeData::Document);
        self.documents.insert(id);
        id
    }

    pub fn is_document(&self, node_id: NodeId) -> bool {
        self.documents.contains(&node_id)
    }

    pub fn node_document(&self, node_id: NodeId) -> Option<NodeId> {
        if self.documents.contains(&node_id) {
            return Some(node_id);
        }
        if let Some(document) = self.node_documents.get(&node_id) {
            return Some(*document);
        }
        let mut current = self.node(node_id)?.parent;
        while let Some(node_id) = current {
            if self.documents.contains(&node_id) {
                return Some(node_id);
            }
            current = self.node(node_id)?.parent;
        }
        Some(0)
    }

    pub fn set_node_document(&mut self, node_id: NodeId, document_id: NodeId) {
        if document_id == 0 {
            self.node_documents.remove(&node_id);
        } else {
            self.node_documents.insert(node_id, document_id);
        }
    }

    pub fn adopt_subtree(&mut self, node_id: NodeId, document_id: NodeId) {
        self.set_node_document(node_id, document_id);
        let children = self
            .node(node_id)
            .map(|node| node.children.clone())
            .unwrap_or_default();
        for child in children {
            self.adopt_subtree(child, document_id);
        }
    }

    pub fn copy_node_metadata(&mut self, source: NodeId, clone: NodeId, deep: bool) {
        if let Some(data) = self.comment_data.get(&source).cloned() {
            self.comment_data.insert(clone, data);
        }
        if self.document_fragments.contains(&source) {
            self.document_fragments.insert(clone);
        }
        if self.documents.contains(&source) {
            self.documents.insert(clone);
        }
        if let Some(document) = self.node_documents.get(&source).copied() {
            self.node_documents.insert(clone, document);
        }
        if deep {
            let source_children = self
                .node(source)
                .map(|node| node.children.clone())
                .unwrap_or_default();
            let clone_children = self
                .node(clone)
                .map(|node| node.children.clone())
                .unwrap_or_default();
            for (source, clone) in source_children.into_iter().zip(clone_children) {
                self.copy_node_metadata(source, clone, true);
            }
        }
    }

    pub fn remove_node_metadata(&mut self, node_ids: &[NodeId]) {
        for node_id in node_ids {
            self.comment_data.remove(node_id);
            self.document_fragments.remove(node_id);
            self.documents.remove(node_id);
            self.node_documents.remove(node_id);
        }
    }

    fn config(
        base_url: Option<&str>,
        net_provider: Option<Arc<dyn NetProvider>>,
    ) -> DocumentConfig {
        DocumentConfig {
            html_parser_provider: Some(Arc::new(HtmlProvider)),
            base_url: base_url.map(str::to_owned),
            net_provider,
            font_ctx: Some(bundled_font_context()),
            ..DocumentConfig::default()
        }
    }

    pub fn install_author_stylesheet(&mut self, node_id: NodeId, css: &str) {
        let sheet = self
            .inner
            .make_stylesheet(css, style::stylesheets::Origin::Author);
        self.inner.add_stylesheet_for_node(sheet, node_id);
    }

    fn stylesheet_for_node(&self, node_id: NodeId) -> Option<DocumentStyleSheet> {
        let element = self.node(node_id)?.element_data()?;
        match &element.special_data {
            SpecialElementData::Stylesheet(stylesheet) => Some(stylesheet.clone()),
            _ => None,
        }
    }

    pub fn stylesheet_node_ids(&self) -> Vec<NodeId> {
        fn visit(document: &BrowserDocument, node_id: NodeId, output: &mut Vec<NodeId>) {
            if document.stylesheet_for_node(node_id).is_some() {
                output.push(node_id);
            }
            let children = document
                .node(node_id)
                .map(|node| node.children.clone())
                .unwrap_or_default();
            for child in children {
                visit(document, child, output);
            }
        }

        let mut output = Vec::new();
        visit(self, 0, &mut output);
        output
    }

    pub fn stylesheet_rule_texts(&self, node_id: NodeId) -> Option<Vec<String>> {
        let stylesheet = self.stylesheet_for_node(node_id)?;
        Some(self.rule_texts(&stylesheet))
    }

    fn rule_texts(&self, stylesheet: &DocumentStyleSheet) -> Vec<String> {
        let guard = self.inner.guard().read();
        let contents = stylesheet.contents(&guard);
        let rules = contents.rules.read_with(&guard);
        rules
            .0
            .iter()
            .map(|rule| rule.to_css_string(&guard).to_string())
            .collect()
    }

    pub fn parse_stylesheet_text(&self, css: &str) -> Vec<String> {
        let stylesheet = self
            .inner
            .make_stylesheet(css, style::stylesheets::Origin::Author);
        self.rule_texts(&stylesheet)
    }

    pub fn parse_stylesheet_rule(&self, rule: &str) -> Result<String, CssomError> {
        let stylesheet = self
            .inner
            .make_stylesheet(rule, style::stylesheets::Origin::Author);
        let rules = self.rule_texts(&stylesheet);
        if rules.len() != 1 {
            return Err(CssomError::Syntax);
        }
        Ok(rules.into_iter().next().unwrap())
    }

    pub fn style_rule_declarations(&self, rule: &str) -> Option<Vec<(String, String, bool)>> {
        let stylesheet = self
            .inner
            .make_stylesheet(rule, style::stylesheets::Origin::Author);
        let guard = self.inner.guard().read();
        let contents = stylesheet.contents(&guard);
        let rules = contents.rules.read_with(&guard);
        if let Some(CssRule::Style(rule)) = rules.0.first() {
            let rule = rule.read_with(&guard);
            return Some(cssom_declaration_entries(rule.block.read_with(&guard)));
        }
        let wrapped = format!("@keyframes __brimp {{ {rule} }}");
        let stylesheet = self
            .inner
            .make_stylesheet(&wrapped, style::stylesheets::Origin::Author);
        let contents = stylesheet.contents(&guard);
        let rules = contents.rules.read_with(&guard);
        let CssRule::Keyframes(rule) = rules.0.first()? else {
            return None;
        };
        let rule = rule.read_with(&guard);
        let keyframe = rule.keyframes.first()?.read_with(&guard);
        Some(cssom_declaration_entries(keyframe.block.read_with(&guard)))
    }

    pub fn style_rule_property(&self, rule: &str, name: &str) -> Option<String> {
        let property = PropertyId::parse_enabled_for_all_content(name).ok()?;
        let stylesheet = self
            .inner
            .make_stylesheet(rule, style::stylesheets::Origin::Author);
        let guard = self.inner.guard().read();
        let contents = stylesheet.contents(&guard);
        let rules = contents.rules.read_with(&guard);
        if let Some(CssRule::Style(rule)) = rules.0.first() {
            let rule = rule.read_with(&guard);
            return cssom_declaration_property(rule.block.read_with(&guard), &property);
        }
        let wrapped = format!("@keyframes __brimp {{ {rule} }}");
        let stylesheet = self
            .inner
            .make_stylesheet(&wrapped, style::stylesheets::Origin::Author);
        let contents = stylesheet.contents(&guard);
        let rules = contents.rules.read_with(&guard);
        let CssRule::Keyframes(rule) = rules.0.first()? else {
            return None;
        };
        let rule = rule.read_with(&guard);
        let keyframe = rule.keyframes.first()?.read_with(&guard);
        cssom_declaration_property(keyframe.block.read_with(&guard), &property)
    }

    pub fn nested_rule_texts(&self, rule: &str) -> Option<Vec<String>> {
        let stylesheet = self
            .inner
            .make_stylesheet(rule, style::stylesheets::Origin::Author);
        let guard = self.inner.guard().read();
        let contents = stylesheet.contents(&guard);
        let rules = contents.rules.read_with(&guard);
        let nested = match rules.0.first()? {
            CssRule::Media(rule) => rule.rules.read_with(&guard).0.clone(),
            CssRule::Supports(rule) => rule.rules.read_with(&guard).0.clone(),
            CssRule::Keyframes(rule) => {
                return Some(
                    rule.read_with(&guard)
                        .keyframes
                        .iter()
                        .map(|keyframe| {
                            keyframe.read_with(&guard).to_css_string(&guard).to_string()
                        })
                        .collect(),
                );
            }
            _ => return None,
        };
        Some(
            nested
                .iter()
                .map(|rule| rule.to_css_string(&guard).to_string())
                .collect(),
        )
    }

    pub fn replace_stylesheet(
        &mut self,
        node_id: NodeId,
        css: &str,
    ) -> Result<Vec<String>, CssomError> {
        if self.node(node_id).and_then(Node::element_data).is_none() {
            return Err(CssomError::NotAStyleSheet);
        }
        self.install_author_stylesheet(node_id, css);
        self.stylesheet_rule_texts(node_id)
            .ok_or(CssomError::NotAStyleSheet)
    }

    pub fn insert_stylesheet_rule(
        &mut self,
        node_id: NodeId,
        rule: &str,
        index: usize,
    ) -> Result<Vec<String>, CssomError> {
        let stylesheet = self
            .stylesheet_for_node(node_id)
            .ok_or(CssomError::NotAStyleSheet)?;
        let (serialized_rule, mut rules) = {
            let lock = self.inner.guard();
            let guard = lock.read();
            let contents = stylesheet.contents(&guard);
            let rules = contents.rules.read_with(&guard);
            let parsed = rules.parse_rule_for_insert(
                lock,
                rule,
                contents,
                index,
                CssRuleTypes::from_bits(0),
                None,
                None,
                AllowImportRules::Yes,
            )?;
            let serialized_rule = parsed.to_css_string(&guard).to_string();
            let rules = rules
                .0
                .iter()
                .map(|rule| rule.to_css_string(&guard).to_string())
                .collect::<Vec<_>>();
            (serialized_rule, rules)
        };
        rules.insert(index, serialized_rule);
        self.install_author_stylesheet(node_id, &rules.join("\n"));
        self.stylesheet_rule_texts(node_id)
            .ok_or(CssomError::NotAStyleSheet)
    }

    pub fn delete_stylesheet_rule(
        &mut self,
        node_id: NodeId,
        index: usize,
    ) -> Result<Vec<String>, CssomError> {
        let stylesheet = self
            .stylesheet_for_node(node_id)
            .ok_or(CssomError::NotAStyleSheet)?;
        let mut texts = {
            let guard = self.inner.guard().read();
            let contents = stylesheet.contents(&guard);
            let rules = contents.rules.read_with(&guard);
            let Some(rule) = rules.0.get(index) else {
                return Err(CssomError::IndexSize);
            };
            if matches!(rule, CssRule::Namespace(_))
                && rules
                    .0
                    .iter()
                    .any(|rule| !matches!(rule, CssRule::Namespace(_) | CssRule::Import(_)))
            {
                return Err(CssomError::InvalidState);
            }
            rules
                .0
                .iter()
                .map(|rule| rule.to_css_string(&guard).to_string())
                .collect::<Vec<_>>()
        };
        texts.remove(index);
        self.install_author_stylesheet(node_id, &texts.join("\n"));
        self.stylesheet_rule_texts(node_id)
            .ok_or(CssomError::NotAStyleSheet)
    }

    pub fn replace_stylesheet_rule(
        &mut self,
        node_id: NodeId,
        rule: &str,
        index: usize,
    ) -> Result<Vec<String>, CssomError> {
        let stylesheet = self
            .stylesheet_for_node(node_id)
            .ok_or(CssomError::NotAStyleSheet)?;
        let (serialized_rule, mut rules) = {
            let lock = self.inner.guard();
            let guard = lock.read();
            let contents = stylesheet.contents(&guard);
            let rules = contents.rules.read_with(&guard);
            if index >= rules.0.len() {
                return Err(CssomError::IndexSize);
            }
            let parsed = rules.parse_rule_for_insert(
                lock,
                rule,
                contents,
                index,
                CssRuleTypes::from_bits(0),
                None,
                None,
                AllowImportRules::Yes,
            )?;
            let serialized_rule = parsed.to_css_string(&guard).to_string();
            let rules = rules
                .0
                .iter()
                .map(|rule| rule.to_css_string(&guard).to_string())
                .collect::<Vec<_>>();
            (serialized_rule, rules)
        };
        rules[index] = serialized_rule;
        self.install_author_stylesheet(node_id, &rules.join("\n"));
        self.stylesheet_rule_texts(node_id)
            .ok_or(CssomError::NotAStyleSheet)
    }

    pub fn blitz(&self) -> &BaseDocument {
        &self.inner
    }

    pub fn blitz_mut(&mut self) -> &mut BaseDocument {
        &mut self.inner
    }

    pub fn set_viewport(&mut self, width: u32, height: u32, device_pixel_ratio: f32) {
        let physical_width = ((width as f32) * device_pixel_ratio).round() as u32;
        let physical_height = ((height as f32) * device_pixel_ratio).round() as u32;
        self.inner.set_viewport(Viewport::new(
            physical_width,
            physical_height,
            device_pixel_ratio,
            ColorScheme::Light,
        ));
    }

    pub fn viewport_metrics(&self) -> [f64; 3] {
        let viewport = self.inner.viewport();
        let scale = f64::from(viewport.hidpi_scale);
        [
            f64::from(viewport.window_size.0) / scale,
            f64::from(viewport.window_size.1) / scale,
            scale,
        ]
    }

    pub fn resolve(&mut self) {
        self.inner.resolve(0.0);
    }

    pub fn bounding_rect(&self, node_id: NodeId) -> Option<[f64; 4]> {
        self.inner
            .get_client_bounding_rect(node_id)
            .map(|rect| [rect.x, rect.y, rect.width, rect.height])
    }

    pub fn element_at_point(&self, x: f64, y: f64) -> Option<NodeId> {
        let mut node_id = self.inner.hit(x as f32, y as f32)?.node_id;
        loop {
            let node = self.inner.get_node(node_id)?;
            if node.is_element() {
                return Some(node_id);
            }
            node_id = node.parent?;
        }
    }

    pub fn client_size(&self, node_id: NodeId) -> Option<[f64; 2]> {
        let layout = &self.inner.get_node(node_id)?.unrounded_layout;
        Some([
            f64::from(layout.size.width - layout.border.left - layout.border.right),
            f64::from(layout.size.height - layout.border.top - layout.border.bottom),
        ])
    }

    pub fn offset_size(&self, node_id: NodeId) -> Option<[f64; 2]> {
        let size = self.inner.get_node(node_id)?.unrounded_layout.size;
        Some([f64::from(size.width), f64::from(size.height)])
    }

    pub fn root(&self) -> &Node {
        self.inner.root_node()
    }

    pub fn outer_html(&self) -> String {
        let root = self.root();
        let mut html = String::new();
        for child in &root.children {
            if let Some(node) = self.node(*child) {
                node.write_outer_html(&mut html);
            }
        }
        html
    }

    pub fn node(&self, node_id: NodeId) -> Option<&Node> {
        self.inner.get_node(node_id)
    }

    pub fn node_mut(&mut self, node_id: NodeId) -> Option<&mut Node> {
        self.inner.get_node_mut(node_id)
    }

    pub fn get_element_by_id(&self, id: &str) -> Option<NodeId> {
        self.inner.tree().iter().find_map(|(node_id, node)| {
            (node.flags.is_in_document()
                && node
                    .element_data()
                    .and_then(|element| element.attr(local_name!("id")))
                    == Some(id))
            .then_some(node_id)
        })
    }

    pub fn window_named_properties(&self) -> Vec<String> {
        let mut names = HashSet::new();
        for (_, node) in self.inner.tree().iter() {
            if !node.flags.is_in_document() {
                continue;
            }
            let Some(element) = node.element_data() else {
                continue;
            };
            for attribute in [local_name!("id"), local_name!("name")] {
                if let Some(name) = element.attr(attribute)
                    && !name.is_empty()
                {
                    names.insert(name.to_owned());
                }
            }
        }
        let mut names = names.into_iter().collect::<Vec<_>>();
        names.sort_unstable();
        names
    }

    pub fn query_selector(&self, selector: &str) -> Result<Option<NodeId>, SelectorError> {
        self.inner
            .query_selector(selector)
            .map_err(|error| SelectorError(format!("{error:?}")))
    }

    pub fn query_selector_all(&self, selector: &str) -> Result<Vec<NodeId>, SelectorError> {
        self.inner
            .query_selector_all(selector)
            .map(|nodes| nodes.into_iter().collect())
            .map_err(|error| SelectorError(format!("{error:?}")))
    }

    pub fn inline_style_css(&self, node_id: NodeId) -> Option<String> {
        let element = self.inner.get_node(node_id)?.element_data()?;
        let style = element.style_attribute.as_ref()?;
        let guard = self.inner.guard().read();
        let block = style.read_with(&guard);
        let mut css = String::new();
        block.to_css(&mut css).ok()?;
        Some(css)
    }

    pub fn inline_style_property(&self, node_id: NodeId, name: &str) -> Option<String> {
        let property = PropertyId::parse_enabled_for_all_content(name).ok()?;
        let element = self.inner.get_node(node_id)?.element_data()?;
        let style = element.style_attribute.as_ref()?;
        let guard = self.inner.guard().read();
        let block = style.read_with(&guard);
        let mut css = String::new();
        block.property_value_to_css(&property, &mut css).ok()?;
        Some(css)
    }

    pub fn inline_style_declarations(
        &self,
        node_id: NodeId,
    ) -> Option<Vec<(String, String, bool)>> {
        let element = self.inner.get_node(node_id)?.element_data()?;
        let style = element.style_attribute.as_ref()?;
        let guard = self.inner.guard().read();
        let block = style.read_with(&guard);
        Some(
            block
                .declaration_importance_iter()
                .filter_map(|(declaration, importance)| {
                    let mut value = String::new();
                    declaration.to_css(&mut value).ok()?;
                    let name = match declaration.id() {
                        PropertyDeclarationId::Longhand(id) => id.name().to_owned(),
                        PropertyDeclarationId::Custom(name) => cssom_custom_property_name(name),
                    };
                    Some((name, value, importance.important()))
                })
                .collect(),
        )
    }

    pub fn computed_style_property(&self, node_id: NodeId, name: &str) -> Option<String> {
        let styles = self.inner.get_node(node_id)?.primary_styles()?;
        if name == "padding" {
            let padding = styles.get_padding();
            let top = padding.padding_top.to_css_string();
            let right = padding.padding_right.to_css_string();
            let bottom = padding.padding_bottom.to_css_string();
            let left = padding.padding_left.to_css_string();
            return Some(if top == right && top == bottom && top == left {
                top
            } else {
                format!("{top} {right} {bottom} {left}")
            });
        }
        let property = PropertyId::parse_enabled_for_all_content(name).ok()?;
        match property {
            PropertyId::Custom(name) => {
                Some(styles.computed_value_to_string(PropertyDeclarationId::Custom(&name)))
            }
            PropertyId::NonCustom(name) => {
                let name = name.unaliased().as_longhand()?;
                Some(styles.computed_value_to_string(PropertyDeclarationId::Longhand(name)))
            }
        }
    }

    pub fn computed_style_declarations(&self, node_id: NodeId) -> Vec<(String, String, bool)> {
        let Some(styles) = self
            .inner
            .get_node(node_id)
            .and_then(|node| node.primary_styles())
        else {
            return Vec::new();
        };
        let mut declarations = enabled_longhand_ids()
            .map(|id| {
                (
                    id.name().to_owned(),
                    styles.computed_value_to_string(PropertyDeclarationId::Longhand(id)),
                    false,
                )
            })
            .collect::<Vec<_>>();
        declarations.sort_unstable_by(|left, right| {
            left.0
                .starts_with('-')
                .cmp(&right.0.starts_with('-'))
                .then_with(|| left.0.cmp(&right.0))
        });
        let custom_properties = styles.custom_properties();
        let mut custom_declarations = custom_properties
            .inherited
            .iter()
            .chain(custom_properties.non_inherited.iter())
            .filter_map(|(name, value)| {
                Some((
                    cssom_custom_property_name(name),
                    value.as_ref()?.to_css_string(),
                    false,
                ))
            })
            .collect::<Vec<_>>();
        custom_declarations.sort_unstable_by(|left, right| left.0.cmp(&right.0));
        declarations.extend(custom_declarations);
        declarations
    }

    pub fn set_style_property(&mut self, node_id: NodeId, name: &str, value: &str) {
        self.inner.mutate().set_style_property(node_id, name, value);
        self.sync_style_attribute(node_id);
    }

    pub fn set_inline_style_css(&mut self, node_id: NodeId, css: &str) {
        let name = QualName::new(None, ns!(), local_name!("style"));
        self.inner.mutate().set_attribute(node_id, name, css);
        self.sync_style_attribute(node_id);
    }

    pub fn remove_style_property(&mut self, node_id: NodeId, name: &str) {
        self.inner.mutate().remove_style_property(node_id, name);
        self.sync_style_attribute(node_id);
    }

    fn sync_style_attribute(&mut self, node_id: NodeId) {
        let css = self.inline_style_css(node_id).unwrap_or_default();
        let name = QualName::new(None, ns!(), local_name!("style"));
        self.inner.mutate().set_attribute(node_id, name, &css);
    }
}

fn bundled_font_context() -> FontContext {
    static FONT_CONTEXT: OnceLock<FontContext> = OnceLock::new();
    FONT_CONTEXT.get_or_init(build_bundled_font_context).clone()
}

fn build_bundled_font_context() -> FontContext {
    let mut context = FontContext {
        collection: Collection::new(CollectionOptions {
            shared: true,
            system_fonts: false,
        }),
        source_cache: SourceCache::new_shared(),
    };
    context
        .collection
        .register_fonts(Blob::new(Arc::new(WENQUANYI_MICRO_HEI)), None);
    context
        .collection
        .register_fonts(Blob::new(Arc::new(NOTO_COLOR_EMOJI)), None);

    let proportional = context
        .collection
        .family_id(WENQUANYI_FAMILY)
        .expect("bundled WenQuanYi proportional face");
    let monospace = context
        .collection
        .family_id(WENQUANYI_MONO_FAMILY)
        .expect("bundled WenQuanYi monospace face");
    let emoji = context
        .collection
        .family_id(NOTO_COLOR_EMOJI_FAMILY)
        .expect("bundled Noto Color Emoji face");

    for generic in [
        GenericFamily::Serif,
        GenericFamily::SansSerif,
        GenericFamily::Cursive,
        GenericFamily::Fantasy,
        GenericFamily::SystemUi,
        GenericFamily::UiSerif,
        GenericFamily::UiSansSerif,
        GenericFamily::UiRounded,
        GenericFamily::FangSong,
    ] {
        context
            .collection
            .set_generic_families(generic, [proportional].into_iter());
    }
    for generic in [GenericFamily::Monospace, GenericFamily::UiMonospace] {
        context
            .collection
            .set_generic_families(generic, [monospace].into_iter());
    }
    context
        .collection
        .set_generic_families(GenericFamily::Emoji, [emoji].into_iter());
    context
}

#[cfg(test)]
mod font_tests {
    use parley::fontique::{GenericFamily, QueryStatus};

    use super::{
        NOTO_COLOR_EMOJI_FAMILY, WENQUANYI_FAMILY, WENQUANYI_MONO_FAMILY,
        build_bundled_font_context,
    };

    fn family_has_glyph(context: &mut parley::FontContext, family: &str, character: char) -> bool {
        let mut found = false;
        let mut query = context.collection.query(&mut context.source_cache);
        query.set_families([family]);
        query.matches_with(|font| {
            if font
                .charmap()
                .is_some_and(|charmap| charmap.map(character).is_some())
            {
                found = true;
                QueryStatus::Stop
            } else {
                QueryStatus::Continue
            }
        });
        found
    }

    #[test]
    fn deterministic_font_context_contains_only_bundled_families() {
        let mut context = build_bundled_font_context();
        let mut families: Vec<_> = context
            .collection
            .family_names()
            .map(str::to_owned)
            .collect();
        families.sort();
        assert_eq!(
            families,
            [
                NOTO_COLOR_EMOJI_FAMILY.to_owned(),
                WENQUANYI_FAMILY.to_owned(),
                WENQUANYI_MONO_FAMILY.to_owned(),
            ]
        );
    }

    #[test]
    fn bundled_families_cover_western_cjk_monospace_and_emoji_text() {
        let mut context = build_bundled_font_context();
        for character in ['A', 'é', '中', '文', 'あ', '한'] {
            assert!(
                family_has_glyph(&mut context, WENQUANYI_FAMILY, character),
                "WenQuanYi proportional face must cover {character:?}"
            );
        }
        assert!(family_has_glyph(&mut context, WENQUANYI_MONO_FAMILY, '界'));
        assert!(family_has_glyph(
            &mut context,
            NOTO_COLOR_EMOJI_FAMILY,
            '😀'
        ));
    }

    #[test]
    fn generic_families_select_the_bundled_faces() {
        let mut context = build_bundled_font_context();
        let family_name = |context: &mut parley::FontContext, generic| {
            let family = context
                .collection
                .generic_families(generic)
                .next()
                .expect("configured generic family");
            context.collection.family_name(family).map(str::to_owned)
        };
        assert_eq!(
            family_name(&mut context, GenericFamily::SansSerif).as_deref(),
            Some(WENQUANYI_FAMILY)
        );
        assert_eq!(
            family_name(&mut context, GenericFamily::Monospace).as_deref(),
            Some(WENQUANYI_MONO_FAMILY)
        );
        assert_eq!(
            family_name(&mut context, GenericFamily::Emoji).as_deref(),
            Some(NOTO_COLOR_EMOJI_FAMILY)
        );
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectorError(String);

impl fmt::Display for SelectorError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "invalid selector: {}", self.0)
    }
}

impl std::error::Error for SelectorError {}
