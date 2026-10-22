use std::ffi::{CStr, CString, c_char, c_int, c_long, c_void};
use std::fs;
use std::ptr;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use bytes::Bytes;
use http::{
    HeaderMap, HeaderName, HeaderValue, Method, Request, Response, StatusCode,
    response::Builder as ResponseBuilder,
};
use tokio::sync::{mpsc, oneshot};

use crate::body::Body;
use crate::config::{Config, RedirectPolicy};
use crate::error::Error;
use crate::ffi::{
    CURLE_OK, CURLINFO_EFFECTIVE_URL, CURLINFO_HTTP_VERSION, CURLINFO_LOCAL_IP,
    CURLINFO_LOCAL_PORT, CURLINFO_NUM_CONNECTS, CURLINFO_PRIMARY_IP, CURLINFO_PRIMARY_PORT,
    CURLINFO_RESPONSE_CODE, CURLOPT_ACCEPT_ENCODING, CURLOPT_CONNECTTIMEOUT_MS,
    CURLOPT_CUSTOMREQUEST, CURLOPT_FOLLOWLOCATION, CURLOPT_HEADERDATA, CURLOPT_HEADERFUNCTION,
    CURLOPT_HTTPHEADER, CURLOPT_NOBODY, CURLOPT_POSTFIELDS, CURLOPT_POSTFIELDSIZE, CURLOPT_PROXY,
    CURLOPT_PROXYTYPE, CURLOPT_TIMEOUT_MS, CURLOPT_URL, CURLOPT_WRITEDATA, CURLOPT_WRITEFUNCTION,
    Curl, CurlSlist, curl_easy_impersonate, curl_easy_perform, curl_error, curl_slist_append,
    curl_slist_free_all, getinfo, setopt,
};
use crate::fingerprint;
use crate::headers::{final_header_block, normalize_decoded_response_headers, parse_status_line};
use crate::pool::{CurlHandlePool, trace_enabled};
use crate::response::CollectedResponse;

static CAPTURE_SEQUENCE: AtomicU64 = AtomicU64::new(0);

/// Owns one curl easy handle and the state needed to execute a request.
pub(crate) struct CurlTransfer {
    metadata: crate::TransferInfo,
    auth: crate::Credentials,
    handle: *mut Curl,
    handle_pool: Arc<CurlHandlePool>,
    request_url: CString,
    request_url_string: String,
    proxy_url: Option<CString>,
    request_headers: *mut CurlSlist,
    request_body: Option<Vec<u8>>,
    method: Method,
    method_string: Option<CString>,
    config: Config,
    response_status: Option<StatusCode>,
    response_headers: HeaderMap,
    response_sent: bool,
    response_sender: Option<oneshot::Sender<Result<Response<Body>, Error>>>,
    body_sender: Option<mpsc::Sender<Bytes>>,
    body_receiver: Option<mpsc::Receiver<Bytes>>,
    collected_headers: Vec<String>,
    collected_body: Vec<u8>,
    completion_sender: Option<oneshot::Sender<Result<(), Error>>>,
    completion_receiver: Option<oneshot::Receiver<Result<(), Error>>>,
    paused: bool,
    streaming: bool,
    capture_body: bool,
    wake: Option<Arc<crate::multi::Wake>>,
}

impl CurlTransfer {
    /// Creates and configures a curl transfer from an HTTP request.
    pub(crate) fn new(
        request: Request<Option<Vec<u8>>>,
        config: Config,
        handle_pool: Arc<CurlHandlePool>,
        named_fingerprint: Option<&fingerprint::Fingerprint>,
    ) -> Result<Self, Error> {
        let (mut parts, request_body) = request.into_parts();
        if transfer_default_headers(&config, named_fingerprint) {
            fingerprint::add_default_headers(
                named_fingerprint.expect("custom fingerprint is present"),
                &mut parts.headers,
            )?;
        }
        let is_navigation = parts.headers.contains_key("upgrade-insecure-requests")
            || parts.headers.contains_key("sec-fetch-user");
        let ordered = parts.extensions.remove::<crate::OrderedHeaders>();
        let auth = parts
            .extensions
            .remove::<crate::Credentials>()
            .unwrap_or_default();
        let request_headers = prepare_request_headers(
            parts.headers,
            &config.impersonation_target,
            is_navigation,
            named_fingerprint.and_then(fingerprint::header_order),
        );
        let request_headers = ordered.map(|headers| headers.0).unwrap_or(request_headers);
        let request_url_string = parts.uri.to_string();
        if trace_enabled() {
            let client_hints = request_headers
                .iter()
                .filter(|(name, _)| name.as_str().starts_with("sec-ch-"))
                .filter_map(|(name, value)| {
                    value
                        .to_str()
                        .ok()
                        .map(|value| format!("{}={value}", name.as_str()))
                })
                .collect::<Vec<_>>()
                .join(" ");
            eprintln!(
                "brimp-net request method={} navigation={is_navigation} url={request_url_string} client_hints=[{client_hints}]",
                parts.method
            );
        }
        let request_url = CString::new(request_url_string.as_str())
            .map_err(|error| Error::InvalidRequest(error.to_string()))?;
        let handle = handle_pool.acquire()?;
        let mut transfer = Self {
            metadata: Default::default(),
            auth,
            handle,
            handle_pool,
            request_url,
            request_url_string,
            proxy_url: None,
            request_headers: ptr::null_mut(),
            request_body,
            method: parts.method,
            method_string: None,
            config,
            response_status: None,
            response_headers: HeaderMap::new(),
            response_sent: false,
            response_sender: None,
            body_sender: None,
            body_receiver: None,
            collected_headers: Vec::new(),
            collected_body: Vec::new(),
            completion_sender: None,
            completion_receiver: None,
            paused: false,
            streaming: false,
            capture_body: std::env::var_os("BRIMP_NET_CAPTURE_DIR").is_some(),
            wake: None,
        };

        if let Some(profile) = named_fingerprint {
            fingerprint::apply(transfer.handle, profile)?;
        } else {
            let target = CString::new(transfer.config.impersonation_target.as_str())
                .map_err(|error| Error::Initialization(error.to_string()))?;
            let result = unsafe {
                curl_easy_impersonate(
                    transfer.handle,
                    target.as_ptr(),
                    transfer.config.default_headers as c_int,
                )
            };
            if result != CURLE_OK {
                let message = curl_error(transfer.handle, result);
                return Err(Error::Initialization(format!(
                    "libcurl-impersonate could not select `{}`: {message}",
                    transfer.config.impersonation_target
                )));
            }
        }

        for (name, value) in request_headers {
            transfer.append_request_header(name, value)?;
        }

        Ok(transfer)
    }

    /// Performs the transfer and sends body bytes through a channel as they arrive.
    pub(crate) fn prepare_streaming(
        &mut self,
        response_sender: oneshot::Sender<Result<Response<Body>, Error>>,
        body_sender: mpsc::Sender<Bytes>,
        body_receiver: mpsc::Receiver<Bytes>,
        completion_sender: oneshot::Sender<Result<(), Error>>,
        completion_receiver: oneshot::Receiver<Result<(), Error>>,
        wake: Arc<crate::multi::Wake>,
    ) -> Result<(), Error> {
        self.wake = Some(wake);
        self.streaming = true;
        self.completion_sender = Some(completion_sender);
        self.completion_receiver = Some(completion_receiver);
        self.response_sender = Some(response_sender);
        self.body_sender = Some(body_sender);
        self.body_receiver = Some(body_receiver);
        self.configure_common()?;
        unsafe {
            setopt(
                self.handle,
                CURLOPT_WRITEFUNCTION,
                write_body as extern "C" fn(*mut c_char, usize, usize, *mut c_void) -> usize,
            )?;
            setopt(
                self.handle,
                CURLOPT_WRITEDATA,
                self as *mut Self as *mut c_void,
            )?;
            setopt(
                self.handle,
                CURLOPT_HEADERFUNCTION,
                write_header as extern "C" fn(*mut c_char, usize, usize, *mut c_void) -> usize,
            )?;
            setopt(
                self.handle,
                CURLOPT_HEADERDATA,
                self as *mut Self as *mut c_void,
            )?;
        }
        Ok(())
    }

    pub(crate) fn finish_streaming(&mut self, mut result: Result<(), Error>) {
        self.update_metadata();
        self.trace_connection(None);
        if result.is_ok() && !self.response_sent {
            result = self.send_streaming_response();
        }
        self.capture_exchange();
        if let Some(sender) = self.response_sender.take() {
            let _ = sender.send(Err(result.err().unwrap_or_else(|| {
                Error::Response("transfer completed without response headers".into())
            })));
        } else if let Some(sender) = self.completion_sender.take() {
            let _ = sender.send(result);
        }
        self.body_sender = None;
    }

    pub(crate) fn is_cancelled(&self) -> bool {
        if self.response_sent {
            self.body_sender
                .as_ref()
                .is_some_and(mpsc::Sender::is_closed)
        } else {
            self.response_sender
                .as_ref()
                .is_some_and(oneshot::Sender::is_closed)
        }
    }

    pub(crate) fn resume_handle(&mut self) -> Option<*mut Curl> {
        if self.paused && self.body_sender.as_ref().is_some_and(|s| s.capacity() > 0) {
            // Unpausing may immediately invoke the write callback, so release
            // the Rust borrow before calling curl_easy_pause in the executor.
            self.paused = false;
            Some(self.handle)
        } else {
            None
        }
    }

    pub(crate) fn handle(&self) -> *mut Curl {
        self.handle
    }

    /// Performs the transfer and collects the entire response body into memory.
    pub(crate) fn perform_collect(mut self) -> Result<CollectedResponse, Error> {
        self.configure_common()?;
        unsafe {
            setopt(
                self.handle,
                CURLOPT_WRITEFUNCTION,
                write_body as extern "C" fn(*mut c_char, usize, usize, *mut c_void) -> usize,
            )?;
            setopt(
                self.handle,
                CURLOPT_WRITEDATA,
                &mut self as *mut Self as *mut c_void,
            )?;
            setopt(
                self.handle,
                CURLOPT_HEADERFUNCTION,
                write_header as extern "C" fn(*mut c_char, usize, usize, *mut c_void) -> usize,
            )?;
            setopt(
                self.handle,
                CURLOPT_HEADERDATA,
                &mut self as *mut Self as *mut c_void,
            )?;
            let code = curl_easy_perform(self.handle);
            if code != CURLE_OK {
                self.trace_connection(Some(code));
                return Err(Error::Network(curl_error(self.handle, code)));
            }
        }
        self.trace_connection(None);
        self.capture_exchange();

        let status = self.response_code()?;
        let effective_url = self.effective_url()?;
        Ok(CollectedResponse {
            status,
            headers: normalize_decoded_response_headers(final_header_block(
                &self.collected_headers,
            )),
            body: std::mem::take(&mut self.collected_body),
            effective_url,
        })
    }

    /// Applies libcurl options common to streaming and collected transfers.
    fn configure_common(&mut self) -> Result<(), Error> {
        unsafe {
            setopt(self.handle, crate::ffi::CURLOPT_NOSIGNAL, 1 as c_long)?;
            setopt(
                self.handle,
                crate::ffi::CURLOPT_SUPPRESS_CONNECT_HEADERS,
                1 as c_long,
            )?;
            setopt(self.handle, CURLOPT_URL, self.request_url.as_ptr())?;
            if self.config.prefer_http3 {
                setopt(self.handle, crate::ffi::CURLOPT_HTTP_VERSION, 30 as c_long)?;
            }
            if let Some(path) = &self.config.ca_bundle {
                let path = CString::new(path.to_string_lossy().as_bytes())
                    .map_err(|e| Error::InvalidRequest(e.to_string()))?;
                setopt(self.handle, crate::ffi::CURLOPT_CAINFO, path.as_ptr())?;
            }
            for auth in &self.auth.0 {
                let username = CString::new(auth.username.as_bytes())
                    .map_err(|e| Error::InvalidRequest(e.to_string()))?;
                let password = CString::new(auth.password.as_bytes())
                    .map_err(|e| Error::InvalidRequest(e.to_string()))?;
                let (user, pass, mode) = if auth.proxy {
                    (
                        crate::ffi::CURLOPT_PROXYUSERNAME,
                        crate::ffi::CURLOPT_PROXYPASSWORD,
                        crate::ffi::CURLOPT_PROXYAUTH,
                    )
                } else {
                    (
                        crate::ffi::CURLOPT_USERNAME,
                        crate::ffi::CURLOPT_PASSWORD,
                        crate::ffi::CURLOPT_HTTPAUTH,
                    )
                };
                setopt(self.handle, user, username.as_ptr())?;
                setopt(self.handle, pass, password.as_ptr())?;
                setopt(self.handle, mode, !(1 << 4) as c_long)?;
            }

            setopt(self.handle, CURLOPT_ACCEPT_ENCODING, c"".as_ptr())?;
            setopt(
                self.handle,
                CURLOPT_CONNECTTIMEOUT_MS,
                duration_ms(self.config.connect_timeout)?,
            )?;
            setopt(
                self.handle,
                CURLOPT_TIMEOUT_MS,
                duration_ms(self.config.request_timeout)?,
            )?;
            setopt(
                self.handle,
                CURLOPT_FOLLOWLOCATION,
                match self.config.redirect_policy {
                    RedirectPolicy::None => 0 as c_long,
                    RedirectPolicy::Follow => 1 as c_long,
                },
            )?;
            if let Some(proxy) = &self.config.proxy {
                let proxy_url = CString::new(proxy.url())
                    .map_err(|error| Error::InvalidRequest(error.to_string()))?;
                setopt(self.handle, CURLOPT_PROXY, proxy_url.as_ptr())?;
                setopt(
                    self.handle,
                    CURLOPT_PROXYTYPE,
                    proxy.curl_proxy_type() as c_long,
                )?;
                self.proxy_url = Some(proxy_url);
            }
            if !self.request_headers.is_null() {
                setopt(self.handle, CURLOPT_HTTPHEADER, self.request_headers)?;
            }
            if self.method == Method::HEAD {
                setopt(self.handle, CURLOPT_NOBODY, 1 as c_long)?;
            } else if self.method != Method::GET || self.request_body.is_some() {
                let method_string = CString::new(self.method.as_str())
                    .map_err(|error| Error::InvalidRequest(error.to_string()))?;
                setopt(self.handle, CURLOPT_CUSTOMREQUEST, method_string.as_ptr())?;
                self.method_string = Some(method_string);
            }
            if let Some(body) = &self.request_body {
                setopt(
                    self.handle,
                    CURLOPT_POSTFIELDS,
                    body.as_ptr().cast::<c_void>(),
                )?;
                setopt(self.handle, CURLOPT_POSTFIELDSIZE, body.len() as c_long)?;
            }
        }
        Ok(())
    }

    /// Adds one request header to libcurl's request header list.
    fn append_request_header(&mut self, name: HeaderName, value: HeaderValue) -> Result<(), Error> {
        let Ok(value) = value.to_str() else {
            return Ok(());
        };
        let header = if value.is_empty() {
            format!("{}:", name.as_str())
        } else {
            format!("{}: {value}", name.as_str())
        };
        let header =
            CString::new(header).map_err(|error| Error::InvalidRequest(error.to_string()))?;
        let next = unsafe { curl_slist_append(self.request_headers, header.as_ptr()) };
        if next.is_null() {
            return Err(Error::Network(format!(
                "failed to append curl header `{}`",
                name.as_str()
            )));
        }
        self.request_headers = next;
        Ok(())
    }

    /// Parses one raw header callback line and updates response state.
    fn process_header_bytes(&mut self, bytes: &[u8]) -> Result<(), Error> {
        let line = String::from_utf8_lossy(bytes).trim_end().to_string();
        self.collected_headers.push(line.clone());
        if let Some(status) = parse_status_line(&line)? {
            self.response_status = Some(status);
            self.response_headers = HeaderMap::new();
            return Ok(());
        }
        if line.is_empty() {
            if self
                .response_status
                .is_some_and(|status| status.is_informational())
            {
                self.response_status = None;
                self.response_headers = HeaderMap::new();
                return Ok(());
            }
            let following_redirect = self.config.redirect_policy == RedirectPolicy::Follow
                && self
                    .response_status
                    .is_some_and(|status| status.is_redirection())
                && self.response_headers.contains_key(http::header::LOCATION);
            let authenticating = self.response_status.is_some_and(|status| {
                (status.as_u16() == 401 && self.auth.0.iter().any(|a| !a.proxy))
                    || (status.as_u16() == 407 && self.auth.0.iter().any(|a| a.proxy))
            });
            if !self.response_sent && !following_redirect && !authenticating {
                self.send_streaming_response()?;
            }
            return Ok(());
        }
        if let Some((name, value)) = line.split_once(':') {
            let Ok(name) = HeaderName::from_bytes(name.trim().as_bytes()) else {
                return Ok(());
            };
            let Ok(value) = HeaderValue::from_str(value.trim()) else {
                return Ok(());
            };
            self.response_headers.append(name, value);
        }
        Ok(())
    }

    /// Sends the final response head and body receiver to the async caller.
    fn send_streaming_response(&mut self) -> Result<(), Error> {
        let Some(response_sender) = self.response_sender.take() else {
            return Ok(());
        };
        let status = self.response_status.unwrap_or(StatusCode::OK);
        let Some(body_receiver) = self.body_receiver.take() else {
            let _ =
                response_sender.send(Err(Error::Response("body channel unavailable".to_string())));
            return Ok(());
        };
        self.update_metadata();
        let version = match self.metadata.snapshot().http_version.as_deref() {
            Some("HTTP/1.0") => http::Version::HTTP_10,
            Some("HTTP/2") => http::Version::HTTP_2,
            Some("HTTP/3") => http::Version::HTTP_3,
            _ => http::Version::HTTP_11,
        };
        let mut builder = Response::builder()
            .status(status)
            .version(version)
            .extension(self.metadata.clone());
        *builder
            .headers_mut()
            .ok_or_else(|| Error::Response("response headers unavailable".to_string()))? =
            normalize_decoded_response_headers(self.response_headers.clone());
        self.response_sent = true;
        let response = build_response(
            builder,
            Body::new(
                body_receiver,
                self.completion_receiver.take().expect("stream completion"),
                self.wake.take().expect("stream wake"),
            ),
        );
        let _ = response_sender.send(response);
        Ok(())
    }

    /// Pause without consuming bytes when the bounded channel is full. Curl
    /// replays that chunk after resume, so never append it before acceptance.
    fn send_body_bytes(&mut self, bytes: &[u8]) -> usize {
        if self.streaming && !self.response_sent && self.send_streaming_response().is_err() {
            return 0;
        }
        if let Some(sender) = &self.body_sender {
            match sender.try_send(Bytes::copy_from_slice(bytes)) {
                Ok(()) => {}
                Err(mpsc::error::TrySendError::Full(_)) => {
                    self.paused = true;
                    return crate::ffi::CURL_WRITEFUNC_PAUSE;
                }
                Err(mpsc::error::TrySendError::Closed(_)) => return 0,
            }
        }
        if !self.streaming || self.capture_body {
            self.collected_body.extend_from_slice(bytes);
        }
        bytes.len()
    }

    fn update_metadata(&self) {
        let mut data = self.metadata.0.lock().unwrap();
        data.effective_url = self
            .effective_url()
            .unwrap_or_else(|_| self.request_url_string.clone());
        let (mut version, mut headers) = (0 as c_long, 0 as c_long);
        let (mut downloaded, mut uploaded) = (0i64, 0i64);
        unsafe {
            let _ = getinfo(self.handle, CURLINFO_HTTP_VERSION, &mut version);
            let _ = getinfo(self.handle, crate::ffi::CURLINFO_HEADER_SIZE, &mut headers);
            let _ = getinfo(
                self.handle,
                crate::ffi::CURLINFO_SIZE_DOWNLOAD_T,
                &mut downloaded,
            );
            let _ = getinfo(
                self.handle,
                crate::ffi::CURLINFO_SIZE_UPLOAD_T,
                &mut uploaded,
            );
        }
        data.http_version = match version {
            1 => Some("HTTP/1.0".into()),
            2 => Some("HTTP/1.1".into()),
            3 => Some("HTTP/2".into()),
            30 | 31 => Some("HTTP/3".into()),
            _ => None,
        };
        data.header_bytes = headers.max(0) as u64;
        data.downloaded_bytes = downloaded.max(0) as u64;
        data.uploaded_bytes = uploaded.max(0) as u64;
    }

    /// Returns the final response status from libcurl.
    fn response_code(&self) -> Result<StatusCode, Error> {
        let mut status = 0 as c_long;
        unsafe {
            getinfo(self.handle, CURLINFO_RESPONSE_CODE, &mut status)?;
        }
        StatusCode::from_u16(status as u16).map_err(|error| Error::Response(error.to_string()))
    }

    /// Returns the effective URL reported by libcurl.
    fn effective_url(&self) -> Result<String, Error> {
        let mut effective_url: *mut c_char = ptr::null_mut();
        unsafe {
            getinfo(self.handle, CURLINFO_EFFECTIVE_URL, &mut effective_url)?;
            if effective_url.is_null() {
                return Ok(self.request_url_string.clone());
            }
            Ok(CStr::from_ptr(effective_url).to_string_lossy().to_string())
        }
    }

    fn trace_connection(&self, curl_code: Option<u32>) {
        if !trace_enabled() {
            return;
        }
        let mut response_code = 0 as c_long;
        let mut new_connections = 0 as c_long;
        let mut primary_port = 0 as c_long;
        let mut local_port = 0 as c_long;
        let mut http_version = 0 as c_long;
        let mut primary_ip: *mut c_char = ptr::null_mut();
        let mut local_ip: *mut c_char = ptr::null_mut();
        unsafe {
            let _ = getinfo(self.handle, CURLINFO_RESPONSE_CODE, &mut response_code);
            let _ = getinfo(self.handle, CURLINFO_NUM_CONNECTS, &mut new_connections);
            let _ = getinfo(self.handle, CURLINFO_PRIMARY_PORT, &mut primary_port);
            let _ = getinfo(self.handle, CURLINFO_LOCAL_PORT, &mut local_port);
            let _ = getinfo(self.handle, CURLINFO_HTTP_VERSION, &mut http_version);
            let _ = getinfo(self.handle, CURLINFO_PRIMARY_IP, &mut primary_ip);
            let _ = getinfo(self.handle, CURLINFO_LOCAL_IP, &mut local_ip);
        }
        eprintln!(
            "brimp-net handle={:p} curl_code={} status={response_code} new_connections={new_connections} http_version={http_version} local={}:{} remote={}:{} url={}",
            self.handle,
            curl_code.map_or_else(|| "ok".to_string(), |code| code.to_string()),
            display_curl_string(local_ip),
            local_port,
            display_curl_string(primary_ip),
            primary_port,
            self.request_url_string
        );
    }

    /// Optionally preserves raw request/response bodies without injecting any
    /// JavaScript into the page. This is intended for protocol diagnostics and
    /// is disabled unless `BRIMP_NET_CAPTURE_DIR` is set. An optional
    /// `BRIMP_NET_CAPTURE_HOST` substring limits which URLs are retained.
    fn capture_exchange(&self) {
        let Some(directory) = std::env::var_os("BRIMP_NET_CAPTURE_DIR") else {
            return;
        };
        if let Some(host_filter) = std::env::var_os("BRIMP_NET_CAPTURE_HOST") {
            let host_filter = host_filter.to_string_lossy();
            if !self.request_url_string.contains(host_filter.as_ref()) {
                return;
            }
        }

        let sequence = CAPTURE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let host = url::Url::parse(&self.request_url_string)
            .ok()
            .and_then(|url| url.host_str().map(ToOwned::to_owned))
            .unwrap_or_else(|| "unknown-host".to_string())
            .replace(|character: char| !character.is_ascii_alphanumeric(), "-");
        let prefix = format!("{sequence:04}-{host}");
        let directory = std::path::PathBuf::from(directory);
        let result = (|| -> std::io::Result<()> {
            fs::create_dir_all(&directory)?;
            fs::write(
                directory.join(format!("{prefix}.url.txt")),
                &self.request_url_string,
            )?;
            fs::write(
                directory.join(format!("{prefix}.request.body")),
                self.request_body.as_deref().unwrap_or_default(),
            )?;
            fs::write(
                directory.join(format!("{prefix}.response.headers.txt")),
                self.collected_headers.join("\n"),
            )?;
            fs::write(
                directory.join(format!("{prefix}.response.body")),
                &self.collected_body,
            )
        })();
        if let Err(error) = result {
            eprintln!(
                "brimp-net could not capture exchange for {}: {error}",
                self.request_url_string
            );
        }
    }
}

fn display_curl_string(value: *mut c_char) -> String {
    if value.is_null() {
        return "?".to_string();
    }
    unsafe { CStr::from_ptr(value).to_string_lossy().into_owned() }
}

/// Restores the request-header portion of the selected impersonation profile.
/// Supplying `CURLOPT_HTTPHEADER` replaces curl-impersonate's default header
/// list, so profile-specific omissions, additions, and ordering must be
/// reproduced before the list is handed to libcurl.
fn prepare_request_headers(
    headers: HeaderMap,
    target: &str,
    is_navigation: bool,
    custom_header_order: Option<&str>,
) -> Vec<(HeaderName, HeaderValue)> {
    let mut headers = headers
        .iter()
        .map(|(name, value)| (name.clone(), value.clone()))
        .collect::<Vec<_>>();

    if target.starts_with("firefox") {
        headers.retain(|(name, _)| !name.as_str().starts_with("sec-ch-"));
        if !headers.iter().any(|(name, _)| name == "te") {
            headers.push((
                HeaderName::from_static("te"),
                HeaderValue::from_static("trailers"),
            ));
        }
        headers.sort_by_key(|(name, _)| {
            custom_header_order
                .map(|order| header_rank(order, name))
                .unwrap_or_else(|| firefox_request_header_rank(name))
        });
    } else if target.starts_with("chrome") {
        headers.sort_by_key(|(name, _)| chrome_request_header_rank(name, is_navigation));
    }

    headers
}

fn transfer_default_headers(config: &Config, profile: Option<&fingerprint::Fingerprint>) -> bool {
    config.default_headers && profile.is_some()
}

fn header_rank(order: &str, name: &HeaderName) -> usize {
    let field_count = order.split(',').count();
    order
        .split(',')
        .position(|candidate| candidate == name.as_str())
        .unwrap_or(field_count)
}

/// Orders custom headers like Chromium before handing them to libcurl. Supplying
/// `CURLOPT_HTTPHEADER` replaces curl-impersonate's default header list, so the
/// order must be restored here as part of the HTTP/2 fingerprint.
fn chrome_request_header_rank(name: &HeaderName, is_navigation: bool) -> usize {
    let name = name.as_str();
    let order: &[&str] = if is_navigation {
        &[
            "sec-ch-ua",
            "sec-ch-ua-mobile",
            "sec-ch-ua-platform",
            "upgrade-insecure-requests",
            "user-agent",
            "accept",
            "sec-fetch-site",
            "sec-fetch-mode",
            "sec-fetch-user",
            "sec-fetch-dest",
            "referer",
            "accept-encoding",
            "accept-language",
            "priority",
            "cookie",
        ]
    } else {
        &[
            "sec-ch-ua-platform",
            "user-agent",
            "sec-ch-ua",
            "sec-ch-ua-mobile",
            "accept",
            "sec-fetch-site",
            "sec-fetch-mode",
            "sec-fetch-dest",
            "referer",
            "accept-encoding",
            "accept-language",
            "priority",
            "cookie",
        ]
    };
    order
        .iter()
        .position(|candidate| *candidate == name)
        .unwrap_or(order.len())
}

fn firefox_request_header_rank(name: &HeaderName) -> usize {
    const ORDER: &[&str] = &[
        "user-agent",
        "accept",
        "accept-language",
        "accept-encoding",
        "upgrade-insecure-requests",
        "sec-fetch-dest",
        "sec-fetch-mode",
        "sec-fetch-site",
        "sec-fetch-user",
        "referer",
        "origin",
        "accept-encoding",
        "accept-language",
        "priority",
        "te",
        "cookie",
    ];
    ORDER
        .iter()
        .position(|candidate| *candidate == name.as_str())
        .unwrap_or(ORDER.len())
}

impl Drop for CurlTransfer {
    fn drop(&mut self) {
        unsafe {
            if !self.request_headers.is_null() {
                curl_slist_free_all(self.request_headers);
            }
        }
        self.handle_pool.release(self.handle);
    }
}

unsafe impl Send for CurlTransfer {}

/// Builds an HTTP response from parsed headers and a streaming body.
fn build_response(builder: ResponseBuilder, body: Body) -> Result<Response<Body>, Error> {
    builder
        .body(body)
        .map_err(|error| Error::Response(error.to_string()))
}

/// Converts a Rust duration to the millisecond integer expected by libcurl.
fn duration_ms(duration: Duration) -> Result<c_long, Error> {
    c_long::try_from(duration.as_millis())
        .map_err(|_| Error::InvalidRequest("timeout is too large".to_string()))
}

/// libcurl body callback that forwards received bytes into transfer state.
extern "C" fn write_body(
    data: *mut c_char,
    size: usize,
    nmemb: usize,
    user_data: *mut c_void,
) -> usize {
    let byte_count = size.saturating_mul(nmemb);
    if user_data.is_null() || data.is_null() {
        return 0;
    }
    let transfer = unsafe { &mut *(user_data as *mut CurlTransfer) };
    let bytes = unsafe { std::slice::from_raw_parts(data.cast::<u8>(), byte_count) };
    transfer.send_body_bytes(bytes)
}

/// libcurl header callback that forwards received header lines into transfer state.
extern "C" fn write_header(
    data: *mut c_char,
    size: usize,
    nmemb: usize,
    user_data: *mut c_void,
) -> usize {
    let byte_count = size.saturating_mul(nmemb);
    if user_data.is_null() || data.is_null() {
        return 0;
    }
    let transfer = unsafe { &mut *(user_data as *mut CurlTransfer) };
    let bytes = unsafe { std::slice::from_raw_parts(data.cast::<u8>(), byte_count) };
    if transfer.process_header_bytes(bytes).is_ok() {
        byte_count
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use http::{HeaderMap, HeaderValue, header::HeaderName};

    use super::{chrome_request_header_rank, prepare_request_headers};

    #[test]
    fn chrome_navigation_headers_follow_profile_order() {
        let names = [
            "accept-language",
            "sec-fetch-dest",
            "accept",
            "sec-ch-ua",
            "priority",
            "user-agent",
        ];
        let mut headers = names.map(HeaderName::from_static);

        headers.sort_by_key(|name| chrome_request_header_rank(name, true));

        assert_eq!(
            headers.map(|name| name.as_str().to_owned()),
            [
                "sec-ch-ua",
                "user-agent",
                "accept",
                "sec-fetch-dest",
                "accept-language",
                "priority",
            ]
            .map(str::to_owned)
        );
    }

    #[test]
    fn chrome_subresource_order_places_platform_first() {
        assert!(
            chrome_request_header_rank(&HeaderName::from_static("sec-ch-ua-platform"), false)
                < chrome_request_header_rank(&HeaderName::from_static("user-agent"), false)
        );
    }

    #[test]
    fn firefox_headers_drop_client_hints_and_restore_te_and_profile_order() {
        let mut input = HeaderMap::new();
        input.insert("sec-fetch-dest", HeaderValue::from_static("document"));
        input.insert("accept", HeaderValue::from_static("text/html"));
        input.insert("sec-ch-ua-mobile", HeaderValue::from_static("?0"));
        input.insert("user-agent", HeaderValue::from_static("Firefox"));

        let headers = prepare_request_headers(input, "firefox147", true, None);
        let names = headers
            .iter()
            .map(|(name, _)| name.as_str())
            .collect::<Vec<_>>();

        assert_eq!(names, ["user-agent", "accept", "sec-fetch-dest", "te"]);
        assert_eq!(headers.last().unwrap().1, "trailers");
    }

    #[test]
    fn catalog_firefox_headers_follow_catalog_order() {
        let mut input = HeaderMap::new();
        input.insert("accept-language", HeaderValue::from_static("en-US"));
        input.insert("upgrade-insecure-requests", HeaderValue::from_static("1"));
        input.insert("sec-fetch-dest", HeaderValue::from_static("document"));
        input.insert("accept", HeaderValue::from_static("text/html"));
        input.insert("user-agent", HeaderValue::from_static("Firefox"));

        let headers = prepare_request_headers(
            input,
            "firefox_152_macos_26.0",
            true,
            Some("upgrade-insecure-requests,user-agent,accept,sec-fetch-dest,accept-language,te"),
        );
        let names = headers
            .iter()
            .map(|(name, _)| name.as_str())
            .collect::<Vec<_>>();

        assert_eq!(
            names,
            [
                "upgrade-insecure-requests",
                "user-agent",
                "accept",
                "sec-fetch-dest",
                "accept-language",
                "te"
            ]
        );
    }
}
