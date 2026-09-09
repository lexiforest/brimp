//! CDP request correlation, session events, cancellation, and deadlines.
use crate::transport::Transport;
use crate::{AutomationError as Error, CancellationToken};
use serde_json::Value;
use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex, mpsc};
use std::time::{Duration, Instant};

use brimp_protocol::MAX_FRAME_SIZE as MAX_FRAME;
#[derive(Default)]
struct Inbox {
    pending: HashMap<u64, mpsc::Sender<Value>>,
    events: VecDeque<Value>,
    failure: Option<String>,
}

pub struct Connection {
    transport: Arc<dyn Transport>,
    inbox: Arc<(Mutex<Inbox>, Condvar)>,
    reader: Mutex<Option<std::thread::JoinHandle<()>>>,
    next_id: AtomicU64,
    closed: Arc<AtomicBool>,
    deadline: Instant,
    timeout: Duration,
    cancellation: CancellationToken,
}

impl Connection {
    pub fn new(
        transport: Arc<dyn Transport>,
        timeout: Duration,
        cancellation: CancellationToken,
    ) -> Self {
        let deadline = Instant::now() + timeout;
        let closed = Arc::new(AtomicBool::new(false));
        let reader_closed = closed.clone();
        let reader_transport = transport.clone();
        let inbox = Arc::new((Mutex::new(Inbox::default()), Condvar::new()));
        let reader_inbox = inbox.clone();
        let reader = std::thread::spawn(move || {
            let result = (|| -> Result<(), String> {
                loop {
                    if reader_closed.load(Ordering::Acquire) {
                        return Err("connection closed".into());
                    }
                    let Some(bytes) = reader_transport
                        .receive()
                        .map_err(|error| error.to_string())?
                    else {
                        std::thread::sleep(Duration::from_millis(10));
                        continue;
                    };
                    let message: Value =
                        serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
                    if !message.is_object() {
                        return Err("CDP message must be an object".into());
                    }
                    let (lock, changed) = &*reader_inbox;
                    let mut state = lock.lock().unwrap();
                    if let Some(id) = message["id"].as_u64() {
                        if let Some(sender) = state.pending.remove(&id) {
                            let _ = sender.send(message);
                        }
                    } else {
                        if !message["method"].is_string() {
                            return Err("invalid CDP event".into());
                        }
                        // The direct CLI client consumes page-session events only.
                        // Browser target notifications otherwise accumulate for long crawls.
                        if message.get("sessionId").is_none() {
                            continue;
                        }
                        if state.events.len() >= 4096 {
                            return Err("worker event queue overflow".into());
                        }
                        state.events.push_back(message);
                        changed.notify_all();
                    }
                }
            })();
            let (lock, changed) = &*reader_inbox;
            let mut state = lock.lock().unwrap();
            state.failure = Some(result.unwrap_err());
            state.pending.clear();
            changed.notify_all();
        });
        Self {
            transport,
            inbox,
            reader: Mutex::new(Some(reader)),
            next_id: AtomicU64::new(1),
            closed,
            deadline,
            timeout,
            cancellation,
        }
    }

    pub fn check(&self) -> Result<(), Error> {
        if self.cancellation.is_cancelled() {
            self.close();
            return Err(Error::Cancellation);
        }
        if Instant::now() >= self.deadline {
            self.close();
            return Err(Error::Timeout(self.timeout));
        }
        if self.closed.load(Ordering::Acquire) {
            return Err(Error::Closed);
        }
        if let Some(failure) = &self.inbox.0.lock().unwrap().failure {
            return Err(Error::Transport(failure.clone()));
        }
        Ok(())
    }

    pub fn command(
        &self,
        method: &str,
        params: Value,
        session: Option<&str>,
    ) -> Result<Value, Error> {
        self.check()?;
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let request = brimp_protocol::Request {
            id,
            method: method.to_owned(),
            params,
            session_id: session.map(str::to_owned),
        };
        let bytes = serde_json::to_vec(&request).map_err(|e| Error::Internal(e.to_string()))?;
        if bytes.len() > MAX_FRAME {
            return Err(Error::InvalidInput("CDP request exceeds 64 MiB".into()));
        }
        let (tx, rx) = mpsc::channel();
        self.inbox.0.lock().unwrap().pending.insert(id, tx);
        let result = self.transport.send(&bytes);
        if let Err(error) = result {
            self.close();
            return Err(io_error(error));
        }
        loop {
            self.check()?;
            match rx.recv_timeout(Duration::from_millis(10)) {
                Ok(response) => {
                    if let Some(error) = response.get("error") {
                        if let Some(data) = error.get("data")
                            && let Ok(error) = serde_json::from_value::<Error>(data.clone())
                        {
                            return Err(error);
                        }
                        let message = error["message"]
                            .as_str()
                            .unwrap_or("worker command failed")
                            .to_owned();
                        return Err(if error["code"] == -32601 {
                            Error::Unsupported(format!("{method}: {message}"))
                        } else {
                            Error::Navigation(format!("{method}: {message}"))
                        });
                    }
                    return response
                        .get("result")
                        .cloned()
                        .ok_or_else(|| Error::Transport("CDP response has no result".into()));
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(_) => {
                    self.check()?;
                    return Err(Error::Transport("worker disconnected".into()));
                }
            }
        }
    }

    pub fn event(&self, session: &str, predicate: impl Fn(&Value) -> bool) -> Result<Value, Error> {
        loop {
            self.check()?;
            let (lock, changed) = &*self.inbox;
            let mut state = lock.lock().unwrap();
            if let Some(index) = state
                .events
                .iter()
                .rposition(|event| event["sessionId"] == session && predicate(event))
            {
                return Ok(state.events.remove(index).unwrap());
            }
            drop(
                changed
                    .wait_timeout(state, Duration::from_millis(10))
                    .unwrap(),
            );
        }
    }

    pub fn drain_events(&self, session: &str) -> Vec<Value> {
        let mut state = self.inbox.0.lock().unwrap();
        let mut matching = Vec::new();
        state.events.retain(|event| {
            if event["sessionId"] == session {
                matching.push(event.clone());
                false
            } else {
                true
            }
        });
        matching
    }

    pub fn close(&self) {
        if self.closed.swap(true, Ordering::AcqRel) {
            return;
        }
        self.transport.close();
        if let Some(reader) = self.reader.lock().unwrap().take() {
            let _ = reader.join();
        }
    }
}
impl Drop for Connection {
    fn drop(&mut self) {
        self.close();
    }
}
fn io_error(error: std::io::Error) -> Error {
    Error::Transport(error.to_string())
}
