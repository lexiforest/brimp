//! Navigation, load state and response metadata.

use super::Page;
use crate::network::{HeaderList, NetworkError, ResourceRequest};
use brimp_jsc::JsException;
use std::time::{Duration, Instant};

impl Page {
    pub async fn goto(&mut self, url: &str) -> Result<NavigationResponse, NavigationError> {
        self.goto_with_headers(url, HeaderList::new()).await
    }

    pub async fn goto_with_headers(
        &mut self,
        url: &str,
        headers: HeaderList,
    ) -> Result<NavigationResponse, NavigationError> {
        let mut request = self.resource_request(url)?;
        request.headers = headers;
        self.goto_request(request, true, 20).await
    }

    /// Adds the fetch metadata Chromium attaches to a document navigation. The
    /// transport orders these by Chrome position and drops client hints for
    /// non-Chromium impersonation targets.
    fn add_navigation_headers(&self, request: &mut ResourceRequest) {
        if request.method != http::Method::GET && request.method != http::Method::HEAD {
            return;
        }
        let current = self.browsing_context.current_url();
        let site = match current.as_deref().and_then(|url| url::Url::parse(url).ok()) {
            None => "none",
            Some(current) => match url::Url::parse(&request.url) {
                Ok(target) if target.origin() == current.origin() => "same-origin",
                Ok(target) if target.host_str() == current.host_str() => "same-site",
                Ok(_) => "cross-site",
                Err(_) => "none",
            },
        };
        for (name, value) in [
            ("upgrade-insecure-requests", "1"),
            ("sec-fetch-site", site),
            ("sec-fetch-mode", "navigate"),
            ("sec-fetch-user", "?1"),
            ("sec-fetch-dest", "document"),
        ] {
            let Ok(name) = http::HeaderName::from_bytes(name.as_bytes()) else {
                continue;
            };
            if request.headers.contains_key(name.as_str()) {
                continue;
            }
            if let Ok(value) = http::HeaderValue::from_str(value) {
                request.headers.append(name, value);
            }
        }
    }

    pub async fn goto_request(
        &mut self,
        request: ResourceRequest,
        allow_redirects: bool,
        max_redirects: usize,
    ) -> Result<NavigationResponse, NavigationError> {
        if (request.method == http::Method::GET || request.method == http::Method::HEAD)
            && request.body.is_some()
        {
            return Err(NavigationError::InvalidRequest(format!(
                "{} navigation cannot have a body",
                request.method
            )));
        }
        let mut request = request;
        request.resource_type = Some("Document".into());
        self.add_navigation_headers(&mut request);
        self.close_plugins();
        if let Some(network) = &self.network {
            network.routes.cancel_pending();
            network.cancel_streams();
        }
        self.events
            .emit(crate::runtime::events::PageEvent::Lifecycle {
                name: "navigation".into(),
                url: request.url.clone(),
            });
        let started = Instant::now();
        self.load_state = LoadState::Loading;
        let fetched = match crate::runtime::request::fetch_with_redirects(
            self.network_scope.loader.as_ref(),
            &self.browsing_context,
            request,
            crate::runtime::request::RedirectOptions {
                include_credentials: true,
                follow: allow_redirects,
                limit: max_redirects,
            },
        )
        .await
        {
            Ok(response) => response,
            Err(error) => {
                self.load_state = LoadState::Failed;
                return Err(error.into());
            }
        };
        let request = navigation_request_info(&fetched.request);
        let history = fetched
            .history
            .iter()
            .map(|hop| NavigationHistoryEntry {
                status_code: hop.status.as_u16(),
                reason: hop.status.canonical_reason().unwrap_or_default().to_owned(),
                url: hop.url.clone(),
                headers: header_pairs(&hop.headers),
                request: navigation_request_info(&hop.request),
            })
            .collect();
        let response = fetched.response;
        let cross_origin_isolated = response_is_cross_origin_isolated(&response.headers);
        let status_code = response.status.as_u16();
        let reason = response
            .status
            .canonical_reason()
            .unwrap_or_default()
            .to_owned();
        let headers = header_pairs(&response.headers);
        let metadata = response.metadata;
        let is_html = response
            .headers
            .get(http::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.split(';').next())
            .is_none_or(|mime| {
                mime.trim().eq_ignore_ascii_case("text/html")
                    || mime.trim().eq_ignore_ascii_case("application/xhtml+xml")
            });
        let effective_url = response.effective_url;
        let content = response.body;
        let decoded = if is_html {
            Some(
                crate::dom::decode_html(
                    &content,
                    response
                        .headers
                        .get(http::header::CONTENT_TYPE)
                        .and_then(|value| value.to_str().ok()),
                )
                .map_err(|error| {
                    self.load_state = LoadState::Failed;
                    JsException::from_message(error.to_string())
                })?,
            )
        } else {
            None
        };
        let document_encoding = decoded
            .as_ref()
            .map_or(encoding_rs::UTF_8, |(_, encoding)| *encoding);
        if let Err(error) = self.reset_page_at(
            &effective_url,
            cross_origin_isolated,
            document_encoding,
            response
                .headers
                .get(http::header::CONTENT_TYPE)
                .and_then(|value| value.to_str().ok())
                .unwrap_or("text/html"),
        ) {
            self.load_state = LoadState::Failed;
            return Err(error.into());
        }
        self.bindings.script_control(
            &self.js,
            serde_json::json!({"op":"ready", "state":"loading"}),
        )?;
        for (_, source) in &self.preload_scripts {
            self.execute_page_script(source);
        }
        let html = if let Some((source, _)) = decoded {
            if let Err(error) = self
                .parse_navigation_document(&source, &effective_url)
                .await
            {
                self.load_state = LoadState::Failed;
                return Err(error);
            }
            self.document_ready = true;
            self.execute_page_script(
                "document.dispatchEvent(new Event('DOMContentLoaded', {bubbles:true}))",
            );
            self.events
                .emit(crate::runtime::events::PageEvent::Lifecycle {
                    name: "dom_content_loaded".into(),
                    url: effective_url.clone(),
                });
            if self.wait_until == NavigationWait::Load {
                while self.pending_navigation_scripts.get() > 0 {
                    self.run_pending_tasks()?;
                    tokio::task::yield_now().await;
                    self.wait_for_work(Some(Duration::from_millis(5)));
                }
                self.finish_navigation_load()?;
            } else if self.pending_navigation_scripts.get() == 0 {
                // Leave the load milestone for the next owner turn.
                let realm = self.realm_id;
                self.tasks.push(move |page| {
                    if page.realm_id == realm {
                        let _ = page.finish_navigation_load();
                    }
                });
            }
            if self.wait_until == NavigationWait::Load {
                self.process_dynamic_scripts().await;
                let _ = self.run_pending_tasks();
                self.process_dynamic_scripts().await;
            }
            Some(self.document.borrow().outer_html())
        } else {
            None
        };
        let cookies = response_cookie_pairs(&response.headers);
        if !is_html {
            self.finish_navigation_load()?;
        }
        if let Some(destination) = self.browsing_context.take_pending_navigation() {
            return Box::pin(self.goto(&destination)).await;
        }
        Ok(NavigationResponse {
            status_code,
            reason,
            url: effective_url,
            headers,
            content,
            html,
            cookies,
            elapsed: started.elapsed(),
            request,
            history,
            http_version: metadata.http_version,
            downloaded_bytes: metadata.downloaded_bytes,
            uploaded_bytes: metadata.uploaded_bytes,
            header_bytes: metadata.header_bytes,
        })
    }

    pub(super) fn finish_navigation_load(&mut self) -> Result<(), JsException> {
        if self.load_state == LoadState::Complete {
            return Ok(());
        }
        self.bindings.script_control(
            &self.js,
            serde_json::json!({"op":"ready", "state":"complete"}),
        )?;
        self.load_state = LoadState::Complete;
        self.execute_page_script("window.dispatchEvent(new Event('load'))");
        self.events
            .emit(crate::runtime::events::PageEvent::Lifecycle {
                name: "load".into(),
                url: self.url().unwrap_or_default(),
            });
        Ok(())
    }

    pub async fn wait_for_load(&self) -> Result<(), NavigationError> {
        match self.load_state {
            LoadState::Complete => Ok(()),
            LoadState::Failed => Err(NavigationError::LoadFailed),
            state => Err(NavigationError::NotLoaded(state)),
        }
    }

    pub fn load_state(&self) -> LoadState {
        self.load_state
    }

    pub fn url(&self) -> Option<String> {
        self.browsing_context.current_url()
    }
}

fn response_is_cross_origin_isolated(headers: &HeaderList) -> bool {
    let opener = headers
        .get("cross-origin-opener-policy")
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.trim().eq_ignore_ascii_case("same-origin"));
    let embedder = headers
        .get("cross-origin-embedder-policy")
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| {
            let value = value.trim();
            value.eq_ignore_ascii_case("require-corp")
                || value.eq_ignore_ascii_case("credentialless")
        });
    opener && embedder
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LoadState {
    Idle,
    Loading,
    Complete,
    Failed,
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct NavigationResponse {
    pub status_code: u16,
    pub reason: String,
    pub url: String,
    pub headers: Vec<(String, String)>,
    #[serde(skip)]
    pub content: Vec<u8>,
    pub html: Option<String>,
    pub cookies: Vec<(String, String)>,
    pub elapsed: Duration,
    pub request: NavigationRequestInfo,
    pub history: Vec<NavigationHistoryEntry>,
    pub http_version: Option<String>,
    pub downloaded_bytes: u64,
    pub uploaded_bytes: u64,
    pub header_bytes: u64,
}

fn header_pairs(headers: &HeaderList) -> Vec<(String, String)> {
    headers
        .iter()
        .map(|(name, value)| {
            (
                name.as_str().to_owned(),
                String::from_utf8_lossy(value.as_bytes()).into_owned(),
            )
        })
        .collect()
}

fn response_cookie_pairs(headers: &HeaderList) -> Vec<(String, String)> {
    headers
        .get_all(http::header::SET_COOKIE)
        .filter_map(|value| value.to_str().ok())
        .filter_map(|value| value.split(';').next())
        .filter_map(|pair| pair.split_once('='))
        .map(|(name, value)| (name.trim().to_owned(), value.trim().to_owned()))
        .filter(|(name, _)| !name.is_empty())
        .collect()
}

fn navigation_request_info(request: &ResourceRequest) -> NavigationRequestInfo {
    NavigationRequestInfo {
        method: request.method.as_str().to_owned(),
        url: request.url.clone(),
        headers: header_pairs(&request.headers),
        body: request.body.clone(),
    }
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct NavigationRequestInfo {
    pub method: String,
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: Option<Vec<u8>>,
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct NavigationHistoryEntry {
    pub status_code: u16,
    pub reason: String,
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub request: NavigationRequestInfo,
}

#[derive(Debug, thiserror::Error)]
pub enum NavigationError {
    #[error(transparent)]
    Network(#[from] NetworkError),
    #[error("navigation returned HTTP status {0}")]
    HttpStatus(u16),
    #[error("invalid resource URL: {0}")]
    InvalidUrl(#[from] url::ParseError),
    #[error("invalid navigation request: {0}")]
    InvalidRequest(String),
    #[error("script response is not UTF-8: {0}")]
    InvalidScriptUtf8(#[from] std::string::FromUtf8Error),
    #[error("document query failed: {0}")]
    Document(String),
    #[error("invalid cookie header: {0}")]
    Cookie(String),
    #[error("script loading task failed: {0}")]
    ScriptTask(#[from] tokio::task::JoinError),
    #[error(transparent)]
    JavaScript(#[from] JsException),
    #[error("page load failed")]
    LoadFailed,
    #[error("page is not loaded (state: {0:?})")]
    NotLoaded(LoadState),
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum NavigationWait {
    DomContentLoaded,
    #[default]
    Load,
}
