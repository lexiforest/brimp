use std::sync::{
    Arc, Mutex,
    atomic::{AtomicU8, Ordering},
};

use http::StatusCode;

use super::{HeaderList, NetworkError};

#[derive(Debug, Clone)]
pub enum ResourceStreamEvent {
    Headers {
        status: StatusCode,
        headers: HeaderList,
        url: String,
    },
    Chunk(Vec<u8>),
    Complete,
    Error(NetworkError),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResourceStreamDirective {
    Continue,
    Pause,
    Cancel,
    /// Headers handed off to a redirect. Stop this transfer without reporting
    /// a failed request; the consumer starts the next hop separately.
    Redirect,
}

const STREAM_RUNNING: u8 = 0;
const STREAM_PAUSED: u8 = 1;
const STREAM_CANCELLED: u8 = 2;

/// Controls one in-flight streaming request. Clones refer to the same curl
/// transfer and may safely be used by the page task that consumes a chunk.
#[derive(Debug)]
pub struct ResourceStreamHandle {
    state: Arc<AtomicU8>,
    notify: Arc<tokio::sync::Notify>,
    delegate: Option<Arc<Mutex<Option<ResourceStreamHandle>>>>,
    owner: bool,
}

impl ResourceStreamHandle {
    pub fn new() -> Self {
        Self {
            state: Arc::new(AtomicU8::new(STREAM_RUNNING)),
            notify: Arc::new(tokio::sync::Notify::new()),
            delegate: None,
            owner: true,
        }
    }

    pub(crate) fn proxy() -> Self {
        Self {
            state: Arc::new(AtomicU8::new(STREAM_RUNNING)),
            notify: Arc::new(tokio::sync::Notify::new()),
            delegate: Some(Arc::new(Mutex::new(None))),
            owner: true,
        }
    }

    pub(crate) fn attach(&self, handle: ResourceStreamHandle) {
        if self.is_cancelled() {
            handle.cancel();
        }
        *self.delegate.as_ref().unwrap().lock().unwrap() = Some(handle);
    }

    pub fn resume(&self) {
        let _ = self.state.compare_exchange(
            STREAM_PAUSED,
            STREAM_RUNNING,
            Ordering::AcqRel,
            Ordering::Acquire,
        );
        self.notify.notify_waiters();
        self.notify.notify_one();
        if let Some(delegate) = &self.delegate
            && let Some(handle) = delegate.lock().unwrap().as_ref()
        {
            handle.resume();
        }
    }

    pub fn cancel(&self) {
        self.state.store(STREAM_CANCELLED, Ordering::Release);
        self.notify.notify_waiters();
        self.notify.notify_one();
        if let Some(delegate) = &self.delegate
            && let Some(handle) = delegate.lock().unwrap().as_ref()
        {
            handle.cancel();
        }
    }

    pub(crate) async fn cancelled(&self) {
        loop {
            let wait = self.notify.notified();
            tokio::pin!(wait);
            wait.as_mut().enable();
            if self.is_cancelled() {
                return;
            }
            wait.await;
        }
    }
    pub(crate) async fn wait_resumed(&self) {
        loop {
            let wait = self.notify.notified();
            tokio::pin!(wait);
            wait.as_mut().enable();
            if !self.is_paused() || self.is_cancelled() {
                return;
            }
            wait.await;
        }
    }
    pub fn is_cancelled(&self) -> bool {
        self.state.load(Ordering::Acquire) == STREAM_CANCELLED
    }

    pub(crate) fn pause(&self) {
        let _ = self.state.compare_exchange(
            STREAM_RUNNING,
            STREAM_PAUSED,
            Ordering::AcqRel,
            Ordering::Acquire,
        );
    }

    pub(crate) fn is_paused(&self) -> bool {
        self.state.load(Ordering::Acquire) == STREAM_PAUSED
    }
}

impl Default for ResourceStreamHandle {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for ResourceStreamHandle {
    fn clone(&self) -> Self {
        Self {
            state: Arc::clone(&self.state),
            notify: self.notify.clone(),
            delegate: self.delegate.clone(),
            owner: false,
        }
    }
}

impl Drop for ResourceStreamHandle {
    fn drop(&mut self) {
        if self.owner {
            self.cancel();
        }
    }
}

pub type ResourceStreamCallback =
    Box<dyn FnMut(ResourceStreamEvent, &ResourceStreamHandle) -> ResourceStreamDirective + Send>;

/// Deliver an already buffered response with the same replay-on-Pause contract as curl.
/// The delivery thread never blocks the transport coordinator and owns no native state.
pub(crate) fn deliver_buffered(
    response: super::ResourceResponse,
    mut callback: ResourceStreamCallback,
    handle: ResourceStreamHandle,
) {
    std::thread::spawn(move || {
        let mut deliver = |event: ResourceStreamEvent| {
            loop {
                if handle.is_cancelled() {
                    return false;
                }
                // Arm before calling the consumer so a concurrent resume is not lost.
                handle.pause();
                match callback(event.clone(), &handle) {
                    ResourceStreamDirective::Continue => {
                        handle.resume();
                        return true;
                    }
                    ResourceStreamDirective::Cancel | ResourceStreamDirective::Redirect => {
                        handle.cancel();
                        return false;
                    }
                    ResourceStreamDirective::Pause => {
                        while handle.is_paused() && !handle.is_cancelled() {
                            std::thread::sleep(std::time::Duration::from_millis(1));
                        }
                    }
                }
            }
        };
        if !deliver(ResourceStreamEvent::Headers {
            status: response.status,
            headers: response.headers,
            url: response.effective_url,
        }) {
            return;
        }
        for bytes in response.body.chunks(16 * 1024) {
            if !deliver(ResourceStreamEvent::Chunk(bytes.to_vec())) {
                return;
            }
        }
        deliver(ResourceStreamEvent::Complete);
    });
}
