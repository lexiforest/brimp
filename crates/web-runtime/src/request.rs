use network::{NetworkError, ResourceLoader, ResourceRequest, ResourceResponse};
use std::sync::Arc;
use web_bindings::BrowsingContext;

const REDIRECT_LIMIT: usize = 20;

#[derive(Clone, Copy, Debug)]
pub(crate) struct RedirectOptions {
    pub follow: bool,
    pub limit: usize,
}

impl Default for RedirectOptions {
    fn default() -> Self {
        Self {
            follow: true,
            limit: REDIRECT_LIMIT,
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct RedirectHop {
    pub request: ResourceRequest,
    pub status: http::StatusCode,
    pub headers: network::HeaderList,
    pub url: String,
}

#[derive(Debug)]
pub(crate) struct FetchResult {
    pub response: ResourceResponse,
    pub request: ResourceRequest,
    pub history: Vec<RedirectHop>,
}

/// Applies browser-owned redirect and cookie policy around the transport.
pub(crate) async fn fetch(
    loader: &dyn ResourceLoader,
    context: &BrowsingContext,
    request: ResourceRequest,
) -> Result<ResourceResponse, NetworkError> {
    fetch_with_redirects(loader, context, request, RedirectOptions::default())
        .await
        .map(|result| result.response)
}

pub(crate) async fn fetch_with_redirects(
    loader: &dyn ResourceLoader,
    context: &BrowsingContext,
    mut request: ResourceRequest,
    options: RedirectOptions,
) -> Result<FetchResult, NetworkError> {
    let mut history = Vec::new();
    for redirects in 0..=options.limit {
        refresh_cookie_header(context, &mut request)?;
        let sent_request = request.clone();
        let requested_url = request.url.clone();
        let mut response = loader.fetch(request.clone()).await?;
        for cookie in response.headers.get_all(http::header::SET_COOKIE) {
            if let Ok(cookie) = cookie.to_str() {
                context.store_response_cookie(&requested_url, cookie);
            }
        }
        if !response.status.is_redirection() || !options.follow {
            response.effective_url = requested_url;
            return Ok(FetchResult {
                response,
                request: sent_request,
                history,
            });
        }
        if redirects == options.limit {
            return Err(NetworkError::Transport(format!(
                "redirect limit of {} exceeded",
                options.limit
            )));
        }
        let Some(location) = response.headers.get(http::header::LOCATION) else {
            response.effective_url = requested_url;
            return Ok(FetchResult {
                response,
                request: sent_request,
                history,
            });
        };
        history.push(RedirectHop {
            request: sent_request,
            status: response.status,
            headers: response.headers.clone(),
            url: requested_url.clone(),
        });
        let location = location
            .to_str()
            .map_err(|_| NetworkError::InvalidRequest("redirect Location is not ASCII".into()))?;
        let base = url::Url::parse(&requested_url)
            .map_err(|error| NetworkError::InvalidRequest(error.to_string()))?;
        let target = base
            .join(location)
            .map_err(|error| NetworkError::InvalidRequest(error.to_string()))?;
        let rewrite_to_get = ((response.status == http::StatusCode::MOVED_PERMANENTLY
            || response.status == http::StatusCode::FOUND)
            && request.method == http::Method::POST)
            || (response.status == http::StatusCode::SEE_OTHER
                && request.method != http::Method::HEAD);
        if rewrite_to_get {
            request.method = http::Method::GET;
            request.body = None;
            request.headers.remove(http::header::CONTENT_LENGTH);
            request.headers.remove(http::header::CONTENT_TYPE);
        }
        if origin(&base) != origin(&target) {
            request.headers.remove(http::header::AUTHORIZATION);
            request.headers.remove(http::header::PROXY_AUTHORIZATION);
            request.headers.remove(http::header::COOKIE);
        }
        request.url = target.to_string();
    }
    unreachable!()
}

fn refresh_cookie_header(
    context: &BrowsingContext,
    request: &mut ResourceRequest,
) -> Result<(), NetworkError> {
    context.apply_request_identity(&mut request.headers);
    let supplied = request
        .headers
        .get(http::header::COOKIE)
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned);
    request.headers.remove(http::header::COOKIE);
    let cookies = match (context.cookie_header(&request.url), supplied) {
        (Some(stored), Some(supplied)) => Some(format!("{stored}; {supplied}")),
        (Some(stored), None) => Some(stored),
        (None, Some(supplied)) => Some(supplied),
        (None, None) => None,
    };
    if let Some(cookies) = cookies {
        let value = http::HeaderValue::from_str(&cookies)
            .map_err(|error| NetworkError::InvalidRequest(error.to_string()))?;
        request.headers.insert(http::header::COOKIE, value);
    }
    Ok(())
}

fn origin(url: &url::Url) -> (&str, Option<&str>, Option<u16>) {
    (url.scheme(), url.host_str(), url.port_or_known_default())
}

pub(crate) fn fetch_callback(
    loader: Arc<dyn ResourceLoader>,
    context: Arc<BrowsingContext>,
    request: ResourceRequest,
    callback: network::ResourceCallback,
) -> Result<(), NetworkError> {
    let callback = Arc::new(std::sync::Mutex::new(Some(callback)));
    RedirectTask {
        loader,
        context,
        request,
        redirects: 0,
        callback,
    }
    .submit()
}

struct RedirectTask {
    loader: Arc<dyn ResourceLoader>,
    context: Arc<BrowsingContext>,
    request: ResourceRequest,
    redirects: usize,
    callback: Arc<std::sync::Mutex<Option<network::ResourceCallback>>>,
}
impl RedirectTask {
    fn submit(mut self) -> Result<(), NetworkError> {
        refresh_cookie_header(&self.context, &mut self.request)?;
        let loader = Arc::clone(&self.loader);
        loader.fetch_callback(
            self.request.clone(),
            Box::new(move |result| self.complete(result)),
        )
    }
    fn complete(mut self, result: Result<ResourceResponse, NetworkError>) {
        let result = result.and_then(|mut response| {
            let requested_url = self.request.url.clone();
            for cookie in response.headers.get_all(http::header::SET_COOKIE) {
                if let Ok(cookie) = cookie.to_str() {
                    self.context.store_response_cookie(&requested_url, cookie);
                }
            }
            if !response.status.is_redirection() {
                response.effective_url = requested_url;
                return Ok(Some(response));
            }
            if self.redirects == REDIRECT_LIMIT {
                return Err(NetworkError::Transport(format!(
                    "redirect limit of {REDIRECT_LIMIT} exceeded"
                )));
            }
            let Some(location) = response.headers.get(http::header::LOCATION) else {
                response.effective_url = requested_url;
                return Ok(Some(response));
            };
            let location = location.to_str().map_err(|_| {
                NetworkError::InvalidRequest("redirect Location is not ASCII".into())
            })?;
            let base = url::Url::parse(&requested_url)
                .map_err(|error| NetworkError::InvalidRequest(error.to_string()))?;
            let target = base
                .join(location)
                .map_err(|error| NetworkError::InvalidRequest(error.to_string()))?;
            let rewrite = ((response.status == http::StatusCode::MOVED_PERMANENTLY
                || response.status == http::StatusCode::FOUND)
                && self.request.method == http::Method::POST)
                || (response.status == http::StatusCode::SEE_OTHER
                    && self.request.method != http::Method::HEAD);
            if rewrite {
                self.request.method = http::Method::GET;
                self.request.body = None;
                self.request.headers.remove(http::header::CONTENT_LENGTH);
                self.request.headers.remove(http::header::CONTENT_TYPE);
            }
            if origin(&base) != origin(&target) {
                self.request.headers.remove(http::header::AUTHORIZATION);
                self.request
                    .headers
                    .remove(http::header::PROXY_AUTHORIZATION);
                self.request.headers.remove(http::header::COOKIE);
            }
            self.request.url = target.to_string();
            self.redirects += 1;
            Ok(None)
        });
        match result {
            Ok(Some(response)) => finish_callback(&self.callback, Ok(response)),
            Ok(None) => {
                let callback = Arc::clone(&self.callback);
                if let Err(error) = self.submit() {
                    finish_callback(&callback, Err(error));
                }
            }
            Err(error) => finish_callback(&self.callback, Err(error)),
        }
    }
}
fn finish_callback(
    callback: &std::sync::Mutex<Option<network::ResourceCallback>>,
    result: Result<ResourceResponse, NetworkError>,
) {
    if let Some(callback) = callback
        .lock()
        .expect("resource callback lock poisoned")
        .take()
    {
        callback(result);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use http::{HeaderValue, Method, StatusCode};
    use network::{HeaderList, ResourceLoader};
    use std::sync::Mutex;

    struct RedirectLoader {
        requests: Mutex<Vec<ResourceRequest>>,
        cross_origin: bool,
        status: StatusCode,
    }
    #[async_trait]
    impl ResourceLoader for RedirectLoader {
        async fn fetch(&self, request: ResourceRequest) -> Result<ResourceResponse, NetworkError> {
            let index = self.requests.lock().unwrap().len();
            let mut headers = HeaderList::new();
            let status = if index == 0 {
                headers.append(
                    http::header::SET_COOKIE,
                    HeaderValue::from_static("hop=yes; Path=/"),
                );
                headers.append(
                    http::header::LOCATION,
                    HeaderValue::from_str(if self.cross_origin {
                        "https://other.test/final"
                    } else {
                        "/final"
                    })
                    .unwrap(),
                );
                self.status
            } else {
                StatusCode::OK
            };
            self.requests.lock().unwrap().push(request.clone());
            Ok(ResourceResponse {
                status,
                headers,
                body: b"done".to_vec(),
                effective_url: request.url,
                metadata: network::ResponseMetadata::default(),
            })
        }
    }

    #[test]
    fn follows_relative_redirect_and_applies_intermediate_cookie() {
        let loader = RedirectLoader {
            requests: Mutex::new(Vec::new()),
            cross_origin: false,
            status: StatusCode::FOUND,
        };
        let context = BrowsingContext::default();
        let runtime = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap();
        let response = runtime
            .block_on(fetch(
                &loader,
                &context,
                ResourceRequest::get("https://example.test/start"),
            ))
            .unwrap();
        assert_eq!(response.effective_url, "https://example.test/final");
        let requests = loader.requests.lock().unwrap();
        assert_eq!(
            requests[1].headers.get(http::header::COOKIE).unwrap(),
            "hop=yes"
        );
    }

    #[test]
    fn rewrites_post_and_removes_sensitive_cross_origin_headers() {
        let loader = RedirectLoader {
            requests: Mutex::new(Vec::new()),
            cross_origin: true,
            status: StatusCode::FOUND,
        };
        let context = BrowsingContext::default();
        context.store_response_cookie("https://example.test/", "session=yes; Path=/");
        let mut request = ResourceRequest::new(Method::POST, "https://example.test/start");
        request.body = Some(b"body".to_vec());
        request.headers.insert(
            http::header::AUTHORIZATION,
            HeaderValue::from_static("secret"),
        );
        request.headers.insert(
            http::header::CONTENT_TYPE,
            HeaderValue::from_static("text/plain"),
        );
        let runtime = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap();
        runtime.block_on(fetch(&loader, &context, request)).unwrap();
        let requests = loader.requests.lock().unwrap();
        assert_eq!(requests[1].method, Method::GET);
        assert!(requests[1].body.is_none());
        assert!(
            !requests[1]
                .headers
                .contains_key(http::header::AUTHORIZATION)
        );
        assert!(!requests[1].headers.contains_key(http::header::COOKIE));
        assert!(!requests[1].headers.contains_key(http::header::CONTENT_TYPE));
    }

    #[test]
    fn disabled_redirects_return_the_first_response() {
        let loader = RedirectLoader {
            requests: Mutex::new(Vec::new()),
            cross_origin: false,
            status: StatusCode::FOUND,
        };
        let runtime = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap();
        let result = runtime
            .block_on(fetch_with_redirects(
                &loader,
                &BrowsingContext::default(),
                ResourceRequest::get("https://example.test/start"),
                RedirectOptions {
                    follow: false,
                    limit: 0,
                },
            ))
            .unwrap();
        assert_eq!(result.response.status, StatusCode::FOUND);
        assert_eq!(result.response.effective_url, "https://example.test/start");
        assert!(result.history.is_empty());
        assert_eq!(loader.requests.lock().unwrap().len(), 1);
    }

    #[test]
    fn temporary_redirect_preserves_method_and_body() {
        let loader = RedirectLoader {
            requests: Mutex::new(Vec::new()),
            cross_origin: false,
            status: StatusCode::TEMPORARY_REDIRECT,
        };
        let mut request = ResourceRequest::new(Method::PATCH, "https://example.test/start");
        request.body = Some(b"body".to_vec());
        let runtime = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap();
        let result = runtime
            .block_on(fetch_with_redirects(
                &loader,
                &BrowsingContext::default(),
                request,
                RedirectOptions {
                    follow: true,
                    limit: 1,
                },
            ))
            .unwrap();
        assert_eq!(result.history.len(), 1);
        assert_eq!(result.request.method, Method::PATCH);
        assert_eq!(result.request.body.as_deref(), Some(b"body".as_slice()));
    }

    #[test]
    fn redirect_method_rewrites_match_browser_navigation_rules() {
        let cases = [
            (StatusCode::MOVED_PERMANENTLY, Method::POST, Method::GET),
            (StatusCode::MOVED_PERMANENTLY, Method::PUT, Method::PUT),
            (StatusCode::FOUND, Method::POST, Method::GET),
            (StatusCode::FOUND, Method::PATCH, Method::PATCH),
            (StatusCode::SEE_OTHER, Method::DELETE, Method::GET),
            (StatusCode::SEE_OTHER, Method::HEAD, Method::HEAD),
            (StatusCode::TEMPORARY_REDIRECT, Method::POST, Method::POST),
            (StatusCode::PERMANENT_REDIRECT, Method::PATCH, Method::PATCH),
        ];
        let runtime = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap();
        for (status, method, expected) in cases {
            let loader = RedirectLoader {
                requests: Mutex::new(Vec::new()),
                cross_origin: false,
                status,
            };
            let mut request = ResourceRequest::new(method.clone(), "https://example.test/start");
            if method != Method::HEAD {
                request.body = Some(b"body".to_vec());
            }
            let result = runtime
                .block_on(fetch_with_redirects(
                    &loader,
                    &BrowsingContext::default(),
                    request,
                    RedirectOptions {
                        follow: true,
                        limit: 1,
                    },
                ))
                .unwrap();
            assert_eq!(result.request.method, expected, "status {status}");
            if expected == Method::GET {
                assert!(result.request.body.is_none(), "status {status}");
            } else if expected != Method::HEAD {
                assert_eq!(
                    result.request.body.as_deref(),
                    Some(b"body".as_slice()),
                    "status {status}"
                );
            }
        }
    }

    struct EndlessRedirect;
    #[async_trait]
    impl ResourceLoader for EndlessRedirect {
        async fn fetch(&self, request: ResourceRequest) -> Result<ResourceResponse, NetworkError> {
            let mut headers = HeaderList::new();
            headers.append(http::header::LOCATION, HeaderValue::from_static("/again"));
            Ok(ResourceResponse {
                status: StatusCode::TEMPORARY_REDIRECT,
                headers,
                body: Vec::new(),
                effective_url: request.url,
                metadata: network::ResponseMetadata::default(),
            })
        }
    }
    #[test]
    fn enforces_redirect_limit() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap();
        let error = runtime
            .block_on(fetch(
                &EndlessRedirect,
                &BrowsingContext::default(),
                ResourceRequest::get("https://example.test/start"),
            ))
            .unwrap_err();
        assert!(error.to_string().contains("redirect limit"));

        let error = runtime
            .block_on(fetch_with_redirects(
                &EndlessRedirect,
                &BrowsingContext::default(),
                ResourceRequest::get("https://example.test/start"),
                RedirectOptions {
                    follow: true,
                    limit: 2,
                },
            ))
            .unwrap_err();
        assert!(error.to_string().contains("redirect limit of 2"));
    }
}
