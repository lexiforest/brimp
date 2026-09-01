use std::collections::HashMap;
use std::ffi::{CStr, CString, c_char, c_int, c_long, c_void};
use std::ptr;
use std::sync::{
    Mutex,
    atomic::{AtomicUsize, Ordering},
    mpsc::{self, Receiver, SyncSender, TryRecvError, TrySendError},
};
use std::thread::JoinHandle;

use http::{HeaderName, HeaderValue, Method, StatusCode};
use tokio::sync::oneshot;

use crate::ffi::*;
use crate::{
    CurlConfig, HeaderList, NetworkError, ResourceCallback, ResourceRequest, ResourceResponse,
    ResourceStreamCallback, ResourceStreamDirective, ResourceStreamEvent, ResourceStreamHandle,
};

static EXECUTOR_THREADS: AtomicUsize = AtomicUsize::new(0);

enum Completion {
    Future(oneshot::Sender<Result<ResourceResponse, NetworkError>>),
    Callback(Option<ResourceCallback>),
    Stream {
        callback: Option<ResourceStreamCallback>,
        handle: ResourceStreamHandle,
    },
}
impl Completion {
    fn is_closed(&self) -> bool {
        match self {
            Self::Future(sender) => sender.is_closed(),
            Self::Stream { handle, .. } => handle.is_cancelled(),
            Self::Callback(_) => false,
        }
    }
    fn send(&mut self, result: Result<ResourceResponse, NetworkError>) {
        match self {
            Self::Future(_) => {
                if let Self::Future(sender) = std::mem::replace(self, Self::Callback(None)) {
                    let _ = sender.send(result);
                }
            }
            Self::Callback(callback) => {
                if let Some(callback) = callback.take() {
                    callback(result);
                }
            }
            Self::Stream { callback, handle } => {
                if let Some(callback) = callback.as_mut() {
                    match result {
                        Ok(_) => {
                            let _ = callback(ResourceStreamEvent::Complete, handle);
                        }
                        Err(error) => {
                            let _ = callback(ResourceStreamEvent::Error(error), handle);
                        }
                    }
                }
                *callback = None;
            }
        }
    }

    fn stream_event(&mut self, event: ResourceStreamEvent) -> ResourceStreamDirective {
        if let Self::Stream {
            callback: Some(callback),
            handle,
        } = self
        {
            let directive = callback(event, handle);
            match directive {
                ResourceStreamDirective::Continue => {}
                ResourceStreamDirective::Pause => handle.pause(),
                ResourceStreamDirective::Cancel => handle.cancel(),
            }
            directive
        } else {
            ResourceStreamDirective::Continue
        }
    }

    fn is_stream(&self) -> bool {
        matches!(self, Self::Stream { .. })
    }

    fn stream_handle(&self) -> Option<&ResourceStreamHandle> {
        match self {
            Self::Stream { handle, .. } => Some(handle),
            _ => None,
        }
    }
}

enum Command {
    Submit(ResourceRequest, Completion),
    Shutdown,
}

pub(crate) struct MultiExecutor {
    sender: SyncSender<Command>,
    worker: Mutex<Option<JoinHandle<()>>>,
}
impl std::fmt::Debug for MultiExecutor {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("MultiExecutor")
            .finish_non_exhaustive()
    }
}

impl MultiExecutor {
    pub(crate) fn new(config: CurlConfig) -> Result<Self, NetworkError> {
        if config.queue_capacity == 0 {
            return Err(NetworkError::InvalidRequest(
                "queue capacity must be positive".into(),
            ));
        }
        if config.max_response_bytes == 0 {
            return Err(NetworkError::InvalidRequest(
                "response body limit must be positive".into(),
            ));
        }
        let (sender, receiver) = mpsc::sync_channel(config.queue_capacity);
        let worker = std::thread::Builder::new()
            .name("brimp-curl-multi".into())
            .spawn(move || run(config, receiver))
            .map_err(|error| NetworkError::WorkerStart(error.to_string()))?;
        Ok(Self {
            sender,
            worker: Mutex::new(Some(worker)),
        })
    }

    pub(crate) async fn fetch(
        &self,
        request: ResourceRequest,
    ) -> Result<ResourceResponse, NetworkError> {
        let (sender, receiver) = oneshot::channel();
        match self
            .sender
            .try_send(Command::Submit(request, Completion::Future(sender)))
        {
            Ok(()) => receiver.await.unwrap_or(Err(NetworkError::Closed)),
            Err(TrySendError::Full(_)) => Err(NetworkError::QueueFull),
            Err(TrySendError::Disconnected(_)) => Err(NetworkError::Closed),
        }
    }
    pub(crate) fn fetch_callback(
        &self,
        request: ResourceRequest,
        callback: ResourceCallback,
    ) -> Result<(), NetworkError> {
        match self.sender.try_send(Command::Submit(
            request,
            Completion::Callback(Some(callback)),
        )) {
            Ok(()) => Ok(()),
            Err(TrySendError::Full(_)) => Err(NetworkError::QueueFull),
            Err(TrySendError::Disconnected(_)) => Err(NetworkError::Closed),
        }
    }
    pub(crate) fn fetch_stream_callback(
        &self,
        request: ResourceRequest,
        callback: ResourceStreamCallback,
    ) -> Result<ResourceStreamHandle, NetworkError> {
        let handle = ResourceStreamHandle::new();
        match self.sender.try_send(Command::Submit(
            request,
            Completion::Stream {
                callback: Some(callback),
                handle: handle.clone(),
            },
        )) {
            Ok(()) => Ok(handle),
            Err(TrySendError::Full(_)) => Err(NetworkError::QueueFull),
            Err(TrySendError::Disconnected(_)) => Err(NetworkError::Closed),
        }
    }
}
impl Drop for MultiExecutor {
    fn drop(&mut self) {
        let worker = self
            .worker
            .get_mut()
            .expect("curl worker lock poisoned")
            .take();
        let Some(worker) = worker else { return };

        // A completion callback can own the final ResourceLoader clone. In that
        // case the executor is dropped by its own worker; dropping the sender
        // disconnects the queue and lets the loop exit, while joining here
        // would panic because a thread cannot join itself.
        if worker.thread().id() == std::thread::current().id() {
            return;
        }

        let _ = self.sender.send(Command::Shutdown);
        {
            let _ = worker.join();
        }
    }
}

fn run(config: CurlConfig, receiver: Receiver<Command>) {
    struct ThreadCountGuard;
    impl Drop for ThreadCountGuard {
        fn drop(&mut self) {
            EXECUTOR_THREADS.fetch_sub(1, Ordering::SeqCst);
        }
    }
    EXECUTOR_THREADS.fetch_add(1, Ordering::SeqCst);
    let _thread_count_guard = ThreadCountGuard;
    global_init();
    let multi = unsafe { curl_multi_init() };
    if multi.is_null() {
        reject_remaining(
            &receiver,
            NetworkError::Transport("failed to initialize curl multi handle".into()),
        );
        return;
    }
    let mut active = HashMap::<usize, Box<Transfer>>::new();
    let mut idle = Vec::new();
    let mut shutdown = false;
    loop {
        if active.is_empty() && !shutdown {
            match receiver.recv() {
                Ok(command) => process(
                    command,
                    multi,
                    &config,
                    &mut active,
                    &mut idle,
                    &mut shutdown,
                ),
                Err(_) => shutdown = true,
            }
        }
        loop {
            match receiver.try_recv() {
                Ok(command) => process(
                    command,
                    multi,
                    &config,
                    &mut active,
                    &mut idle,
                    &mut shutdown,
                ),
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    shutdown = true;
                    break;
                }
            }
        }
        cancel_dropped(multi, &mut active, &mut idle);
        resume_streams(&mut active);
        if shutdown {
            cancel_all(multi, &mut active, &mut idle, NetworkError::Closed);
            reject_remaining(&receiver, NetworkError::Closed);
            break;
        }
        drive(multi, &mut active, &mut idle);
        if !active.is_empty() {
            wait_for_activity(multi, &mut active, &mut idle);
            drive(multi, &mut active, &mut idle);
        }
    }
    for handle in idle {
        unsafe {
            curl_easy_cleanup(handle);
        }
    }
    unsafe {
        let _ = curl_multi_cleanup(multi);
    }
}

#[cfg(test)]
pub(crate) fn executor_thread_count() -> usize {
    EXECUTOR_THREADS.load(Ordering::SeqCst)
}

fn process(
    command: Command,
    multi: *mut CurlMulti,
    config: &CurlConfig,
    active: &mut HashMap<usize, Box<Transfer>>,
    idle: &mut Vec<*mut Curl>,
    shutdown: &mut bool,
) {
    match command {
        Command::Shutdown => *shutdown = true,
        Command::Submit(request, mut completion) => {
            if completion.is_closed() {
                return;
            }
            let handle = idle.pop().unwrap_or_else(|| unsafe { curl_easy_init() });
            if handle.is_null() {
                completion.send(Err(NetworkError::Transport(
                    "failed to initialize curl easy handle".into(),
                )));
                return;
            }
            unsafe {
                curl_easy_reset(handle);
            }
            match Transfer::new(handle, request, completion, config.clone()) {
                Ok(mut transfer) => {
                    if let Err(error) = transfer.configure() {
                        let _ = transfer.complete(Err(error));
                        idle.push(transfer.take_handle());
                        return;
                    }
                    let result = unsafe { curl_multi_add_handle(multi, handle) };
                    if result != CURLM_OK {
                        let _ = transfer.complete(Err(NetworkError::Transport(format!(
                            "curl multi add failed ({result})"
                        ))));
                        idle.push(transfer.take_handle());
                        return;
                    }
                    active.insert(handle as usize, transfer);
                }
                Err((error, mut completion)) => {
                    idle.push(handle);
                    completion.send(Err(error));
                }
            }
        }
    }
}

fn drive(
    multi: *mut CurlMulti,
    active: &mut HashMap<usize, Box<Transfer>>,
    idle: &mut Vec<*mut Curl>,
) {
    let mut running = 0;
    let result = unsafe { curl_multi_perform(multi, &mut running) };
    if result != CURLM_OK {
        cancel_all(
            multi,
            active,
            idle,
            NetworkError::Transport(format!("curl multi perform failed ({result})")),
        );
        return;
    }
    drain_completions(multi, active, idle);
}

fn drain_completions(
    multi: *mut CurlMulti,
    active: &mut HashMap<usize, Box<Transfer>>,
    idle: &mut Vec<*mut Curl>,
) {
    loop {
        let mut queued = 0;
        let message = unsafe { curl_multi_info_read(multi, &mut queued) };
        if message.is_null() {
            break;
        }
        let message = unsafe { &*message };
        if message.message != CURLMSG_DONE {
            continue;
        }
        let handle = message.easy_handle;
        unsafe {
            let _ = curl_multi_remove_handle(multi, handle);
        }
        if let Some(mut transfer) = active.remove(&(handle as usize)) {
            let code = unsafe { message.data.result };
            let outcome = if transfer.too_large {
                Err(NetworkError::ResponseTooLarge {
                    limit: transfer.config.max_response_bytes,
                })
            } else if code == CURLE_OK {
                transfer.response()
            } else {
                Err(NetworkError::Transport(error(code)))
            };
            let _ = transfer.complete(outcome);
            idle.push(transfer.take_handle());
        }
    }
}

fn wait_for_activity(
    multi: *mut CurlMulti,
    active: &mut HashMap<usize, Box<Transfer>>,
    idle: &mut Vec<*mut Curl>,
) {
    let mut ready = 0;
    let result = unsafe { curl_multi_poll(multi, ptr::null_mut(), 0, 20, &mut ready) };
    if result != CURLM_OK {
        cancel_all(
            multi,
            active,
            idle,
            NetworkError::Transport(format!("curl multi poll failed ({result})")),
        );
    }
}

fn cancel_dropped(
    multi: *mut CurlMulti,
    active: &mut HashMap<usize, Box<Transfer>>,
    idle: &mut Vec<*mut Curl>,
) {
    let cancelled = active
        .iter()
        .filter_map(|(key, transfer)| {
            transfer
                .completion
                .as_ref()
                .is_some_and(Completion::is_closed)
                .then_some(*key)
        })
        .collect::<Vec<_>>();
    for key in cancelled {
        if let Some(mut transfer) = active.remove(&key) {
            unsafe {
                let _ = curl_multi_remove_handle(multi, transfer.handle);
            }
            idle.push(transfer.take_handle());
        }
    }
}

fn resume_streams(active: &mut HashMap<usize, Box<Transfer>>) {
    for transfer in active.values_mut() {
        let should_resume = transfer
            .completion
            .as_ref()
            .and_then(Completion::stream_handle)
            .is_some_and(|handle| !handle.is_paused() && transfer.curl_paused);
        if should_resume {
            let code = unsafe { curl_easy_pause(transfer.handle, CURLPAUSE_CONT) };
            if code == CURLE_OK {
                transfer.curl_paused = false;
            }
        }
    }
}
fn cancel_all(
    multi: *mut CurlMulti,
    active: &mut HashMap<usize, Box<Transfer>>,
    idle: &mut Vec<*mut Curl>,
    error: NetworkError,
) {
    for (_, mut transfer) in active.drain() {
        unsafe {
            let _ = curl_multi_remove_handle(multi, transfer.handle);
        }
        let _ = transfer.complete(Err(error.clone()));
        idle.push(transfer.take_handle());
    }
}
fn reject_remaining(receiver: &Receiver<Command>, error: NetworkError) {
    while let Ok(Command::Submit(_, mut completion)) = receiver.try_recv() {
        completion.send(Err(error.clone()));
    }
}

struct Transfer {
    handle: *mut Curl,
    completion: Option<Completion>,
    config: CurlConfig,
    url: CString,
    url_string: String,
    ca_bundle: Option<CString>,
    proxy: Option<CString>,
    method: Method,
    method_string: Option<CString>,
    request_headers: *mut CurlSlist,
    body: Option<Vec<u8>>,
    status: Option<StatusCode>,
    headers: HeaderList,
    response_body: Vec<u8>,
    too_large: bool,
    received_body_bytes: usize,
    sent_stream_headers: bool,
    curl_paused: bool,
}
impl Transfer {
    fn new(
        handle: *mut Curl,
        request: ResourceRequest,
        completion: Completion,
        config: CurlConfig,
    ) -> Result<Box<Self>, (NetworkError, Completion)> {
        if url::Url::parse(&request.url).is_err() {
            return Err((
                NetworkError::InvalidRequest(format!("invalid URL `{}`", request.url)),
                completion,
            ));
        }
        let url = match CString::new(request.url.as_str()) {
            Ok(value) => value,
            Err(error) => {
                return Err((NetworkError::InvalidRequest(error.to_string()), completion));
            }
        };
        let mut transfer = Box::new(Self {
            handle,
            completion: Some(completion),
            config,
            url,
            url_string: request.url,
            ca_bundle: None,
            proxy: None,
            method: request.method,
            method_string: None,
            request_headers: ptr::null_mut(),
            body: request.body,
            status: None,
            headers: HeaderList::new(),
            response_body: Vec::new(),
            too_large: false,
            received_body_bytes: 0,
            sent_stream_headers: false,
            curl_paused: false,
        });
        for (name, value) in request.headers.iter() {
            let Ok(value) = value.to_str() else { continue };
            let line = match CString::new(format!("{}: {value}", name.as_str())) {
                Ok(line) => line,
                Err(error) => {
                    let completion = transfer.completion.take().unwrap();
                    return Err((NetworkError::InvalidRequest(error.to_string()), completion));
                }
            };
            let next = unsafe { curl_slist_append(transfer.request_headers, line.as_ptr()) };
            if next.is_null() {
                let completion = transfer.completion.take().unwrap();
                return Err((
                    NetworkError::Transport("failed to allocate request headers".into()),
                    completion,
                ));
            }
            transfer.request_headers = next;
        }
        Ok(transfer)
    }
    fn configure(&mut self) -> Result<(), NetworkError> {
        let profile = CString::new(self.config.impersonation_profile.as_str())
            .map_err(|error| NetworkError::InvalidRequest(error.to_string()))?;
        let code = unsafe {
            curl_easy_impersonate(
                self.handle,
                profile.as_ptr(),
                self.config.default_headers as c_int,
            )
        };
        if code != CURLE_OK {
            return Err(NetworkError::Transport(format!(
                "impersonation profile `{}` is unavailable: {}",
                self.config.impersonation_profile,
                error(code)
            )));
        }
        unsafe {
            setopt(self.handle, CURLOPT_URL, self.url.as_ptr())?;
            if self.config.prefer_http3 {
                setopt(self.handle, CURLOPT_HTTP_VERSION, CURL_HTTP_VERSION_3)?;
            }
            setopt(self.handle, CURLOPT_ACCEPT_ENCODING, c"".as_ptr())?;
            setopt(self.handle, CURLOPT_FOLLOWLOCATION, 0 as c_long)?;
            setopt(
                self.handle,
                CURLOPT_CONNECTTIMEOUT_MS,
                millis(self.config.connect_timeout)?,
            )?;
            setopt(
                self.handle,
                CURLOPT_TIMEOUT_MS,
                millis(self.config.request_timeout)?,
            )?;
            if let Some(path) = &self.config.ca_bundle {
                let path = path.to_str().ok_or_else(|| {
                    NetworkError::InvalidRequest("CA bundle path is not valid UTF-8".into())
                })?;
                let value = CString::new(path)
                    .map_err(|error| NetworkError::InvalidRequest(error.to_string()))?;
                setopt(self.handle, CURLOPT_CAINFO, value.as_ptr())?;
                self.ca_bundle = Some(value);
            }
            if let Some(proxy) = &self.config.proxy {
                let value = CString::new(proxy.url())
                    .map_err(|error| NetworkError::InvalidRequest(error.to_string()))?;
                setopt(self.handle, CURLOPT_PROXY, value.as_ptr())?;
                setopt(
                    self.handle,
                    CURLOPT_PROXYTYPE,
                    proxy.curl_proxy_type() as c_long,
                )?;
                self.proxy = Some(value);
            }
            if !self.request_headers.is_null() {
                setopt(self.handle, CURLOPT_HTTPHEADER, self.request_headers)?;
            }
            if self.method == Method::HEAD {
                setopt(self.handle, CURLOPT_NOBODY, 1 as c_long)?;
            } else if self.method != Method::GET || self.body.is_some() {
                let method = CString::new(self.method.as_str())
                    .map_err(|error| NetworkError::InvalidRequest(error.to_string()))?;
                setopt(self.handle, CURLOPT_CUSTOMREQUEST, method.as_ptr())?;
                self.method_string = Some(method);
            }
            if let Some(body) = &self.body {
                setopt(
                    self.handle,
                    CURLOPT_POSTFIELDS,
                    body.as_ptr().cast::<c_void>(),
                )?;
                setopt(self.handle, CURLOPT_POSTFIELDSIZE, body.len() as c_long)?;
            }
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
    fn response(&self) -> Result<ResourceResponse, NetworkError> {
        let mut status = 0 as c_long;
        let mut effective: *mut c_char = ptr::null_mut();
        let mut http_version = 0 as c_long;
        let mut downloaded_bytes = 0_i64;
        let mut uploaded_bytes = 0_i64;
        let mut header_bytes = 0 as c_long;
        unsafe {
            getinfo(self.handle, CURLINFO_RESPONSE_CODE, &mut status)?;
            getinfo(self.handle, CURLINFO_EFFECTIVE_URL, &mut effective)?;
            getinfo(self.handle, CURLINFO_HTTP_VERSION, &mut http_version)?;
            getinfo(self.handle, CURLINFO_SIZE_DOWNLOAD_T, &mut downloaded_bytes)?;
            getinfo(self.handle, CURLINFO_SIZE_UPLOAD_T, &mut uploaded_bytes)?;
            getinfo(self.handle, CURLINFO_HEADER_SIZE, &mut header_bytes)?;
        }
        let status = StatusCode::from_u16(status as u16)
            .map_err(|error| NetworkError::Transport(error.to_string()))?;
        let effective_url = if effective.is_null() {
            self.url_string.clone()
        } else {
            unsafe { CStr::from_ptr(effective).to_string_lossy().into_owned() }
        };
        let mut headers = self.headers.clone();
        headers.0.retain(|(name, _)| {
            name != http::header::CONTENT_ENCODING && name != http::header::CONTENT_LENGTH
        });
        Ok(ResourceResponse {
            status,
            headers,
            body: self.response_body.clone(),
            effective_url,
            metadata: crate::ResponseMetadata {
                http_version: match http_version {
                    CURL_HTTP_VERSION_1_0 => Some("HTTP/1.0".into()),
                    CURL_HTTP_VERSION_1_1 => Some("HTTP/1.1".into()),
                    CURL_HTTP_VERSION_2_0 => Some("HTTP/2".into()),
                    CURL_HTTP_VERSION_3 | CURL_HTTP_VERSION_3ONLY => Some("HTTP/3".into()),
                    _ => None,
                },
                downloaded_bytes: downloaded_bytes.max(0) as u64,
                uploaded_bytes: uploaded_bytes.max(0) as u64,
                header_bytes: header_bytes.max(0) as u64,
            },
        })
    }
    fn complete(&mut self, result: Result<ResourceResponse, NetworkError>) -> Result<(), ()> {
        self.completion.take().map_or(Err(()), |mut completion| {
            completion.send(result);
            Ok(())
        })
    }
    fn take_handle(&mut self) -> *mut Curl {
        std::mem::replace(&mut self.handle, ptr::null_mut())
    }
    fn header(&mut self, bytes: &[u8]) {
        let line = String::from_utf8_lossy(bytes)
            .trim_end_matches(['\r', '\n'])
            .to_string();
        if line.starts_with("HTTP/") {
            self.status = line
                .split_whitespace()
                .nth(1)
                .and_then(|value| value.parse::<u16>().ok())
                .and_then(|value| StatusCode::from_u16(value).ok());
            self.headers = HeaderList::new();
        } else if line.is_empty() {
            if !self.sent_stream_headers
                && let (Some(status), Some(completion)) = (self.status, self.completion.as_mut())
                && completion.is_stream()
            {
                let _ = completion.stream_event(ResourceStreamEvent::Headers {
                    status,
                    headers: self.headers.clone(),
                    url: self.url_string.clone(),
                });
                self.sent_stream_headers = true;
            }
        } else if let Some((name, value)) = line.split_once(':')
            && let (Ok(name), Ok(value)) = (
                HeaderName::from_bytes(name.trim().as_bytes()),
                HeaderValue::from_str(value.trim()),
            )
        {
            self.headers.append(name, value);
        }
    }
}
impl Drop for Transfer {
    fn drop(&mut self) {
        if !self.request_headers.is_null() {
            unsafe {
                curl_slist_free_all(self.request_headers);
            }
        }

        if !self.handle.is_null() {
            unsafe {
                curl_easy_cleanup(self.handle);
            }
        }
    }
}

extern "C" fn write_body(data: *mut c_char, size: usize, count: usize, user: *mut c_void) -> usize {
    let length = size.saturating_mul(count);
    if user.is_null() {
        return 0;
    }
    let transfer = unsafe { &mut *(user as *mut Transfer) };
    if transfer.received_body_bytes.saturating_add(length) > transfer.config.max_response_bytes {
        transfer.too_large = true;
        return 0;
    }
    let bytes = unsafe { std::slice::from_raw_parts(data.cast::<u8>(), length) };
    transfer.received_body_bytes = transfer.received_body_bytes.saturating_add(length);
    if let Some(completion) = transfer.completion.as_mut()
        && completion.is_stream()
    {
        match completion.stream_event(ResourceStreamEvent::Chunk(bytes.to_vec())) {
            ResourceStreamDirective::Continue => {}
            ResourceStreamDirective::Pause => {
                transfer.curl_paused = true;
                return CURL_WRITEFUNC_PAUSE;
            }
            ResourceStreamDirective::Cancel => return 0,
        }
    } else {
        transfer.response_body.extend_from_slice(bytes);
    }
    length
}
extern "C" fn write_header(
    data: *mut c_char,
    size: usize,
    count: usize,
    user: *mut c_void,
) -> usize {
    let length = size.saturating_mul(count);
    if user.is_null() {
        return 0;
    }
    unsafe { &mut *(user as *mut Transfer) }
        .header(unsafe { std::slice::from_raw_parts(data.cast::<u8>(), length) });
    length
}
fn millis(duration: std::time::Duration) -> Result<c_long, NetworkError> {
    c_long::try_from(duration.as_millis())
        .map_err(|_| NetworkError::InvalidRequest("timeout is too large".into()))
}
impl From<String> for NetworkError {
    fn from(value: String) -> Self {
        Self::Transport(value)
    }
}

unsafe impl Send for MultiExecutor {}
unsafe impl Sync for MultiExecutor {}
