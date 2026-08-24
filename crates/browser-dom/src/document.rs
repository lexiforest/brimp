use std::{fmt, sync::Arc};

use blitz_dom::{BaseDocument, DocumentConfig, Node, QualName, local_name, ns};
use blitz_html::{HtmlDocument, HtmlProvider};
use blitz_traits::net::NetProvider;
use blitz_traits::shell::{ColorScheme, Viewport};
use style_traits::ToCss;

pub type NodeId = usize;

/// Owns the one and only DOM tree for a page.
///
/// This is deliberately a thin boundary around Blitz. It stores no parallel
/// tree or copied node state; JavaScript bindings will refer back to these
/// native node identifiers.
pub struct BrowserDocument {
    inner: BaseDocument,
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
        let config = Self::config(base_url, net_provider);
        Self {
            inner: HtmlDocument::from_html(html, config).into_inner(),
        }
    }

    pub fn empty_at_with_net(
        base_url: Option<&str>,
        net_provider: Option<Arc<dyn NetProvider>>,
    ) -> Self {
        Self {
            inner: BaseDocument::new(Self::config(base_url, net_provider)),
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
            ..DocumentConfig::default()
        }
    }

    pub fn install_author_stylesheet(&mut self, node_id: NodeId, css: &str) {
        let sheet = self
            .inner
            .make_stylesheet(css, style::stylesheets::Origin::Author);
        self.inner.add_stylesheet_for_node(sheet, node_id);
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

    pub fn computed_style_property(&self, node_id: NodeId, name: &str) -> Option<String> {
        let styles = self.inner.get_node(node_id)?.primary_styles()?;
        let value = match name {
            "width" => styles.get_position().width.to_css_string(),
            "height" => styles.get_position().height.to_css_string(),
            "padding-top" => styles.get_padding().padding_top.to_css_string(),
            "padding-right" => styles.get_padding().padding_right.to_css_string(),
            "padding-bottom" => styles.get_padding().padding_bottom.to_css_string(),
            "padding-left" => styles.get_padding().padding_left.to_css_string(),
            "padding" => {
                let padding = styles.get_padding();
                let top = padding.padding_top.to_css_string();
                let right = padding.padding_right.to_css_string();
                let bottom = padding.padding_bottom.to_css_string();
                let left = padding.padding_left.to_css_string();
                if top == right && top == bottom && top == left {
                    top
                } else {
                    format!("{top} {right} {bottom} {left}")
                }
            }
            "background-color" => styles.get_background().background_color.to_css_string(),
            _ => return None,
        };
        Some(value)
    }

    pub fn set_style_property(&mut self, node_id: NodeId, name: &str, value: &str) {
        self.inner.mutate().set_style_property(node_id, name, value);
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectorError(String);

impl fmt::Display for SelectorError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "invalid selector: {}", self.0)
    }
}

impl std::error::Error for SelectorError {}
