//! Host API routing. Both language bindings select an engine at this boundary.
mod transport;
mod webkit;

use crate::runtime::bridge::{BridgeValue, Reference};
use crate::runtime::events::{EventHandler, Subscription};
use crate::runtime::routes::{RouteHandler, RouteRegistration, RouteStage};
use crate::runtime::{AutomationError, CancellationToken, NavigationResponse, native};
use native::{IdentityOptions, RequestOptions, SessionOptions};
use serde_json::Value;
use std::{path::Path, sync::Arc, time::Duration};

type Result<T> = std::result::Result<T, AutomationError>;
fn unsupported<T>(name: &str) -> Result<T> {
    Err(AutomationError::Unsupported(format!(
        "WebKit backend: {name}"
    )))
}

pub enum Session {
    Runtime(Arc<native::Session>),
    WebKit(Arc<webkit::Session>),
}

impl Session {
    pub fn new(
        options: SessionOptions,
        engine: &str,
        executable: Option<&Path>,
    ) -> Result<Arc<Self>> {
        let session = match engine {
            "runtime" => {
                if executable.is_some() {
                    return Err(AutomationError::InvalidInput(
                        "executable_path requires engine='webkit'".into(),
                    ));
                }
                Self::Runtime(native::Session::new(options)?)
            }
            "webkit" => Self::WebKit(webkit::Session::new(
                options,
                executable.unwrap_or(Path::new("brimp-webkit")),
            )?),
            _ => {
                return Err(AutomationError::InvalidInput(
                    "engine must be 'runtime' or 'webkit'".into(),
                ));
            }
        };
        Ok(Arc::new(session))
    }
    pub fn new_page(&self) -> Result<Page> {
        match self {
            Self::Runtime(s) => s.new_page().map(Page::Runtime),
            Self::WebKit(s) => s.new_page().map(Page::WebKit),
        }
    }
    pub fn pages(&self) -> Vec<Page> {
        match self {
            Self::Runtime(s) => s.pages().into_iter().map(Page::Runtime).collect(),
            Self::WebKit(s) => s.pages().into_iter().map(Page::WebKit).collect(),
        }
    }
    pub fn page(&self, id: u64) -> Result<Page> {
        self.pages()
            .into_iter()
            .find(|p| p.id() == id)
            .ok_or(AutomationError::Closed)
    }
    pub fn close(&self) {
        match self {
            Self::Runtime(s) => s.close(),
            Self::WebKit(s) => s.close(),
        }
    }
    pub fn route(
        &self,
        pattern: &str,
        stage: RouteStage,
        handler: RouteHandler,
    ) -> Result<RouteRegistration> {
        match self {
            Self::Runtime(s) => s.route(pattern, stage, handler),
            Self::WebKit(_) => unsupported("session routes"),
        }
    }
    pub fn set_cookie(&self, url: &str, name: &str, value: &str) -> Result<()> {
        match self {
            Self::Runtime(s) => s.cookies().set_cookie(url, name, value),
            Self::WebKit(s) => s.set_cookie(url, name, value),
        }
    }
    pub fn cookies(&self) -> Result<Vec<crate::web_apis::StoredCookie>> {
        match self {
            Self::Runtime(s) => Ok(s.cookies().cookies()),
            Self::WebKit(s) => s.cookies(),
        }
    }
    pub fn clear_cookies(&self) -> Result<()> {
        match self {
            Self::Runtime(s) => {
                s.cookies().clear_cookies();
                Ok(())
            }
            Self::WebKit(s) => s.clear_cookies(),
        }
    }
}

#[derive(Clone)]
pub enum Page {
    Runtime(native::NativePage),
    WebKit(webkit::Page),
}

macro_rules! page_methods {
    ($($name:ident($($arg:ident: $ty:ty),*) -> $result:ty;)*) => {
        $(pub fn $name(&self, $($arg: $ty),*) -> Result<$result> {
            match self { Self::Runtime(page) => page.$name($($arg),*), Self::WebKit(page) => page.$name($($arg),*) }
        })*
    };
}
macro_rules! runtime_page_methods {
    ($($name:ident($($arg:ident: $ty:ty),*) -> $result:ty;)*) => {
        $(pub fn $name(&self, $($arg: $ty),*) -> Result<$result> {
            match self { Self::Runtime(page) => page.$name($($arg),*), Self::WebKit(_) => unsupported(stringify!($name)) }
        })*
    };
}
impl Page {
    pub fn id(&self) -> u64 {
        match self {
            Self::Runtime(p) => p.id(),
            Self::WebKit(p) => p.id(),
        }
    }
    pub fn is_closed(&self) -> bool {
        match self {
            Self::Runtime(p) => p.is_closed(),
            Self::WebKit(p) => p.is_closed(),
        }
    }
    pub fn default_timeout(&self) -> Duration {
        match self {
            Self::Runtime(p) => p.default_timeout(),
            Self::WebKit(p) => p.default_timeout(),
        }
    }
    page_methods! {
        current_url() -> String;
        automate(command: Value, timeout: Option<Duration>) -> BridgeValue;
        request(method: &str, url: &str, options: RequestOptions) -> NavigationResponse;
        html() -> String;
        load_html(html: &str, url: &str) -> ();
        close() -> ();
        on(event: &str, handler: EventHandler) -> Subscription;
        check_callback() -> ();
        wait_for_network_idle(quiet: Duration, timeout: Duration, cancellation: CancellationToken) -> ();
        add_init_script(source: &str) -> ();
        response_body(id: u64) -> Vec<u8>;
        set_extra_headers(headers: Vec<(String, String)>) -> ();
        clear_cache() -> ();
    }
    runtime_page_methods! {
        advance_time(milliseconds: f64, max_tasks: usize) -> Value;
        run_pending_tasks(max_tasks: usize) -> Value;
        profile_contract() -> Value;
        profile_report() -> Value;
        shape_manifest() -> Value;
        take_diagnostics() -> Value;
        peek_diagnostics() -> Value;
        summarize_diagnostics() -> Value;
        stream(method: &str, url: &str, options: RequestOptions) -> native::streaming::StreamingResponse;
        route(pattern: &str, stage: RouteStage, handler: RouteHandler) -> RouteRegistration;
        configure_network(options: crate::runtime::networking::NetworkOptions) -> ();
        set_identity(identity: IdentityOptions) -> ();
        extract(options: crate::runtime::ExtractionOptions) -> crate::runtime::ExtractedDocument;
        set_input_files(selector: &str, files: Vec<native::FileUpload>) -> ();
        drag_to(source: &str, destination: &str) -> ();
    }
    pub fn evaluate_handle(&self, source: &str) -> Result<JSHandle> {
        match self {
            Self::Runtime(p) => p.evaluate_handle(source).map(JSHandle::Runtime),
            Self::WebKit(p) => p.evaluate_handle(source).map(JSHandle::WebKit),
        }
    }
    pub fn query_selector(&self, selector: &str) -> Result<Option<ElementHandle>> {
        match self {
            Self::Runtime(p) => p
                .query_selector(selector)
                .map(|v| v.map(|h| ElementHandle(JSHandle::Runtime(h.0)))),
            Self::WebKit(p) => p
                .query_selector(selector)
                .map(|v| v.map(|h| ElementHandle(JSHandle::WebKit(h)))),
        }
    }
    pub fn query_selector_all(&self, selector: &str) -> Result<Vec<ElementHandle>> {
        match self {
            Self::Runtime(p) => p.query_selector_all(selector).map(|v| {
                v.into_iter()
                    .map(|h| ElementHandle(JSHandle::Runtime(h.0)))
                    .collect()
            }),
            Self::WebKit(p) => p.query_selector_all(selector).map(|v| {
                v.into_iter()
                    .map(|h| ElementHandle(JSHandle::WebKit(h)))
                    .collect()
            }),
        }
    }
    page_methods! {
        click(selector: &str) -> ();
        hover(selector: &str) -> ();
        tap(selector: &str) -> ();
        focus(selector: &str) -> ();
        type_text(selector: &str, text: &str) -> ();
        fill(selector: &str, text: &str) -> ();
        press(selector: &str, key: &str, modifiers: u8) -> ();
        mouse_move(x: f64, y: f64, steps: u32) -> ();
        mouse_down(x: f64, y: f64, button: &str, click_count: u32, modifiers: u8) -> ();
        mouse_up(x: f64, y: f64, button: &str, click_count: u32, modifiers: u8) -> ();
        mouse_click(x: f64, y: f64, button: &str, click_count: u32, modifiers: u8) -> ();
        mouse_wheel(x: f64, y: f64, delta_x: f64, delta_y: f64) -> ();
    }
}

#[derive(Clone)]
pub enum JSHandle {
    Runtime(native::JSHandle),
    WebKit(webkit::Handle),
}
impl From<native::JSHandle> for JSHandle {
    fn from(value: native::JSHandle) -> Self {
        Self::Runtime(value)
    }
}
impl JSHandle {
    pub fn reference(&self) -> Result<Reference> {
        match self {
            Self::Runtime(h) => h.reference(),
            Self::WebKit(h) => h.reference(),
        }
    }
    pub fn peer_handle(&self, token: Reference) -> Self {
        match self {
            Self::Runtime(h) => Self::Runtime(h.peer_handle(token)),
            Self::WebKit(h) => Self::WebKit(h.peer_handle(token)),
        }
    }
    pub fn value(&self) -> Result<BridgeValue> {
        match self {
            Self::Runtime(h) => h.value(),
            Self::WebKit(h) => h.value(),
        }
    }
    pub fn json_value(&self) -> Result<Value> {
        match self {
            Self::Runtime(h) => h.json_value(),
            Self::WebKit(h) => h.json_value(),
        }
    }
    pub fn bridge_command(&self, command: Value) -> Result<BridgeValue> {
        match self {
            Self::Runtime(h) => h.bridge_command(command),
            Self::WebKit(h) => h.bridge_command(command),
        }
    }
    pub fn dispose(&self) -> Result<()> {
        match self {
            Self::Runtime(h) => h.dispose(),
            Self::WebKit(h) => h.dispose(),
        }
    }
}
#[derive(Clone)]
pub struct ElementHandle(pub JSHandle);
impl ElementHandle {
    pub fn focus(&self) -> Result<()> {
        match &self.0 {
            JSHandle::Runtime(h) => native::ElementHandle(h.clone()).focus(),
            JSHandle::WebKit(h) => {
                h.bridge_command(
                    serde_json::json!({"op":"call","source":"node => { node.focus(); }"}),
                )?;
                Ok(())
            }
        }
    }
    pub fn click(&self) -> Result<()> {
        match &self.0 {
            JSHandle::Runtime(h) => native::ElementHandle(h.clone()).click(),
            JSHandle::WebKit(h) => h.click(),
        }
    }
    pub fn set_input_files(&self, files: Vec<native::FileUpload>) -> Result<()> {
        match &self.0 {
            JSHandle::Runtime(h) => native::ElementHandle(h.clone()).set_input_files(files),
            JSHandle::WebKit(_) => unsupported("file upload"),
        }
    }
    pub fn query_selector(&self, selector: &str) -> Result<Option<Self>> {
        match &self.0 {
            JSHandle::Runtime(h) => native::ElementHandle(h.clone())
                .query_selector(selector)
                .map(|v| v.map(|h| Self(h.0.into()))),
            JSHandle::WebKit(h) => h
                .query_selector(selector)
                .map(|v| v.map(|h| Self(JSHandle::WebKit(h)))),
        }
    }
    pub fn query_selector_all(&self, selector: &str) -> Result<Vec<Self>> {
        match &self.0 {
            JSHandle::Runtime(h) => native::ElementHandle(h.clone())
                .query_selector_all(selector)
                .map(|v| v.into_iter().map(|h| Self(h.0.into())).collect()),
            JSHandle::WebKit(h) => h
                .query_selector_all(selector)
                .map(|v| v.into_iter().map(|h| Self(JSHandle::WebKit(h))).collect()),
        }
    }
}
