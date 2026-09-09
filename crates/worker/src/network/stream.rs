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
}

const STREAM_RUNNING: u8 = 0;
const STREAM_PAUSED: u8 = 1;
const STREAM_CANCELLED: u8 = 2;

/// Controls one in-flight streaming request. Clones refer to the same curl
/// transfer and may safely be used by the page task that consumes a chunk.
#[derive(Debug)]
pub struct ResourceStreamHandle {
    state: Arc<AtomicU8>,
    delegate: Option<Arc<Mutex<Option<ResourceStreamHandle>>>>,
    owner: bool,
}

impl ResourceStreamHandle {
    pub fn new() -> Self {
        Self {
            state: Arc::new(AtomicU8::new(STREAM_RUNNING)),
            delegate: None,
            owner: true,
        }
    }

    pub(super) fn proxy() -> Self {
        Self {
            state: Arc::new(AtomicU8::new(STREAM_RUNNING)),
            delegate: Some(Arc::new(Mutex::new(None))),
            owner: true,
        }
    }

    pub(super) fn attach(&self, handle: ResourceStreamHandle) {
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
        if let Some(delegate) = &self.delegate
            && let Some(handle) = delegate.lock().unwrap().as_ref()
        {
            handle.resume();
        }
    }

    pub fn cancel(&self) {
        self.state.store(STREAM_CANCELLED, Ordering::Release);
        if let Some(delegate) = &self.delegate
            && let Some(handle) = delegate.lock().unwrap().as_ref()
        {
            handle.cancel();
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
