//! Typed observation independent of any transport. Adapters schedule callbacks promptly.
use super::AutomationError;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, Ordering},
    },
};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestRecord {
    pub id: u64,
    pub method: String,
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: Option<Vec<u8>>,
    pub resource_type: Option<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResponseRecord {
    pub request_id: u64,
    pub url: String,
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub from_cache: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MutationRecord {
    pub kind: String,
    pub target: u64,
    pub added: Vec<u64>,
    pub removed: Vec<u64>,
    pub attribute: Option<String>,
    pub old_value: Option<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum PageEvent {
    Console {
        level: String,
        message: String,
    },
    Error {
        message: String,
    },
    Lifecycle {
        name: String,
        url: String,
    },
    Mutation {
        record: MutationRecord,
    },
    Request {
        request: RequestRecord,
    },
    Response {
        response: ResponseRecord,
    },
    RequestFinished {
        request_id: u64,
        bytes: usize,
    },
    RequestFailed {
        request_id: u64,
        message: String,
        cancelled: bool,
    },
    WebSocket {
        id: u64,
        url: String,
        kind: String,
        data: Vec<u8>,
        message: Option<String>,
    },
    EventSource {
        request_id: u64,
        event_name: String,
        event_id: String,
        data: String,
    },
}
impl PageEvent {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Console { .. } => "console",
            Self::Error { .. } => "error",
            Self::Lifecycle { .. } => "lifecycle",
            Self::Mutation { .. } => "mutation",
            Self::Request { .. } => "request",
            Self::Response { .. } => "response",
            Self::RequestFinished { .. } => "request_finished",
            Self::RequestFailed { .. } => "request_failed",
            Self::WebSocket { .. } => "websocket",
            Self::EventSource { .. } => "eventsource",
        }
    }
}
pub type EventHandler = Arc<dyn Fn(PageEvent) + Send + Sync>;
#[derive(Clone, Default)]
pub(crate) struct EventHub(Arc<Hub>);
#[derive(Default)]
struct Hub {
    next: AtomicU64,
    listeners: Mutex<BTreeMap<u64, (String, EventHandler)>>,
}
impl std::fmt::Debug for EventHub {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EventHub").finish_non_exhaustive()
    }
}
impl EventHub {
    pub fn on(&self, event: &str, callback: EventHandler) -> Result<Subscription, AutomationError> {
        if ![
            "console",
            "error",
            "lifecycle",
            "mutation",
            "request",
            "response",
            "request_finished",
            "request_failed",
            "websocket",
            "eventsource",
        ]
        .contains(&event)
        {
            return Err(AutomationError::InvalidInput(format!(
                "unknown page event: {event}"
            )));
        }
        let id = self.0.next.fetch_add(1, Ordering::Relaxed);
        self.0
            .listeners
            .lock()
            .unwrap()
            .insert(id, (event.into(), callback));
        Ok(Subscription {
            hub: self.clone(),
            id,
        })
    }
    pub fn has_listener(&self, event: &str) -> bool {
        self.0
            .listeners
            .lock()
            .unwrap()
            .values()
            .any(|(name, _)| name == event)
    }
    pub fn emit(&self, event: PageEvent) {
        let listeners = self
            .0
            .listeners
            .lock()
            .unwrap()
            .values()
            .filter(|(name, _)| name == event.name())
            .map(|(_, callback)| callback.clone())
            .collect::<Vec<_>>();
        for callback in listeners {
            let _ =
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| callback(event.clone())));
        }
    }
    pub fn clear(&self) {
        self.0.listeners.lock().unwrap().clear();
    }
}
/// Dropping or explicitly unsubscribing releases the listener.
pub struct Subscription {
    hub: EventHub,
    id: u64,
}
impl Subscription {
    pub fn unsubscribe(&self) {
        self.hub.0.listeners.lock().unwrap().remove(&self.id);
    }
}
impl Drop for Subscription {
    fn drop(&mut self) {
        self.unsubscribe();
    }
}

pub(crate) fn headers(headers: &crate::network::HeaderList) -> Vec<(String, String)> {
    headers
        .iter()
        .map(|(name, value)| {
            (
                name.to_string(),
                String::from_utf8_lossy(value.as_bytes()).into_owned(),
            )
        })
        .collect()
}
