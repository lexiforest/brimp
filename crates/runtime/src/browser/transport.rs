//! Private, length-prefixed JSON transport shared by every host binding.
use crate::runtime::{AutomationError, CancellationToken};
use serde_json::{Value, json};
use std::{
    collections::HashMap,
    io::{Read, Write},
    path::Path,
    process::{Child, Command, Stdio},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU64, Ordering},
        mpsc,
    },
    time::{Duration, Instant},
};

const MAX_MESSAGE: usize = 64 * 1024 * 1024;
type Reply = Result<Value, AutomationError>;
type EventHandler = Arc<dyn Fn(Value) + Send + Sync>;

struct Inner {
    child: Mutex<Child>,
    outgoing: mpsc::SyncSender<(u64, Vec<u8>)>,
    pending: Mutex<HashMap<u64, mpsc::Sender<Reply>>>,
    handler: Mutex<Option<EventHandler>>,
    closed: AtomicBool,
    next: AtomicU64,
}

impl Inner {
    fn disconnect(&self) {
        self.closed.store(true, Ordering::Release);
        for (_, sender) in self.pending.lock().unwrap().drain() {
            let _ = sender.send(Err(AutomationError::Closed));
        }
    }
}

impl Drop for Inner {
    fn drop(&mut self) {
        self.disconnect();
        let child = self.child.get_mut().unwrap();
        let _ = child.kill();
        let _ = child.wait();
    }
}

#[derive(Clone)]
pub(super) struct Transport(Arc<Inner>);

fn read_message(reader: &mut impl Read) -> std::io::Result<Value> {
    let mut header = [0; 4];
    reader.read_exact(&mut header)?;
    let size = u32::from_be_bytes(header) as usize;
    if size == 0 || size > MAX_MESSAGE {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "invalid WebKit message length",
        ));
    }
    let mut data = vec![0; size];
    reader.read_exact(&mut data)?;
    serde_json::from_slice(&data).map_err(Into::into)
}

impl Transport {
    pub fn launch(executable: &Path) -> Result<Self, AutomationError> {
        let mut child = Command::new(executable)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(|e| {
                AutomationError::Transport(format!("launch {}: {e}", executable.display()))
            })?;
        let mut input = child.stdin.take().unwrap();
        let mut output = child.stdout.take().unwrap();
        let (outgoing, writes) = mpsc::sync_channel(64);
        let (events, incoming_events) = mpsc::channel();
        let inner = Arc::new(Inner {
            child: Mutex::new(child),
            outgoing,
            pending: Mutex::new(HashMap::new()),
            handler: Mutex::new(None),
            closed: AtomicBool::new(false),
            next: AtomicU64::new(1),
        });
        let weak = Arc::downgrade(&inner);
        std::thread::Builder::new()
            .name("brimp-webkit-write".into())
            .spawn(move || {
                while let Ok((id, data)) = writes.recv() {
                    let Some(inner) = weak.upgrade() else { break };
                    if !inner.pending.lock().unwrap().contains_key(&id) {
                        continue;
                    }
                    // Never keep the process owner alive while blocked on a full pipe.
                    drop(inner);
                    if input
                        .write_all(&(data.len() as u32).to_be_bytes())
                        .and_then(|_| input.write_all(&data))
                        .is_err()
                    {
                        if let Some(inner) = weak.upgrade() {
                            inner.disconnect();
                        }
                        break;
                    }
                }
            })
            .map_err(|e| AutomationError::Transport(e.to_string()))?;
        let weak = Arc::downgrade(&inner);
        std::thread::Builder::new()
            .name("brimp-webkit-read".into())
            .spawn(move || {
                while let Ok(message) = read_message(&mut output) {
                    let Some(inner) = weak.upgrade() else { break };
                    if let Some(id) = message["id"].as_u64() {
                        if let Some(sender) = inner.pending.lock().unwrap().remove(&id) {
                            let result = if message.get("error").is_some() {
                                Err(protocol_error(&message["error"]))
                            } else {
                                Ok(message["result"].clone())
                            };
                            let _ = sender.send(result);
                        }
                    } else {
                        let _ = events.send(message);
                    }
                }
                if let Some(inner) = weak.upgrade() {
                    inner.disconnect();
                }
                let _ = events.send(json!({"method":"Brimp.disconnected"}));
            })
            .map_err(|e| AutomationError::Transport(e.to_string()))?;
        let weak = Arc::downgrade(&inner);
        std::thread::Builder::new()
            .name("brimp-webkit-events".into())
            .spawn(move || {
                while let Ok(message) = incoming_events.recv() {
                    let handler = weak
                        .upgrade()
                        .and_then(|inner| inner.handler.lock().unwrap().clone());
                    if let Some(handler) = handler {
                        handler(message);
                    }
                }
            })
            .map_err(|e| AutomationError::Transport(e.to_string()))?;
        let transport = Self(inner);
        let hello = transport.call("Brimp.hello", json!({}), Duration::from_secs(30))?;
        if hello["protocol"] != 1 || hello["engine"] != "webkit" {
            return Err(AutomationError::Unsupported(
                "incompatible WebKit helper protocol".into(),
            ));
        }
        Ok(transport)
    }

    pub fn on_event(&self, callback: EventHandler) {
        *self.0.handler.lock().unwrap() = Some(callback);
    }
    pub fn is_closed(&self) -> bool {
        self.0.closed.load(Ordering::Acquire)
    }
    pub fn call(&self, method: &str, params: Value, timeout: Duration) -> Reply {
        let cancellation = CancellationToken::current();
        if self.is_closed() {
            return Err(AutomationError::Closed);
        }
        if cancellation.is_cancelled() {
            return Err(AutomationError::Cancellation);
        }
        let started = Instant::now();
        let id = self.0.next.fetch_add(1, Ordering::Relaxed);
        let data = serde_json::to_vec(&json!({"id":id,"method":method,"params":params}))
            .map_err(|e| AutomationError::InvalidInput(e.to_string()))?;
        if data.len() > MAX_MESSAGE {
            return Err(AutomationError::InvalidInput(
                "WebKit message exceeds 64 MiB".into(),
            ));
        }
        let (sender, receiver) = mpsc::channel();
        self.0.pending.lock().unwrap().insert(id, sender);
        let result = (|| {
            let mut outgoing = (id, data);
            loop {
                match self.0.outgoing.try_send(outgoing) {
                    Ok(()) => break,
                    Err(mpsc::TrySendError::Disconnected(_)) => {
                        return Err(AutomationError::Closed);
                    }
                    Err(mpsc::TrySendError::Full(value)) => outgoing = value,
                }
                if cancellation.is_cancelled() {
                    return Err(AutomationError::Cancellation);
                }
                if !timeout.is_zero() && started.elapsed() >= timeout {
                    return Err(AutomationError::Timeout(timeout));
                }
                std::thread::sleep(Duration::from_millis(5));
            }
            loop {
                if cancellation.is_cancelled() {
                    return Err(AutomationError::Cancellation);
                }
                if self.is_closed() {
                    return Err(AutomationError::Closed);
                }
                if !timeout.is_zero() && started.elapsed() >= timeout {
                    return Err(AutomationError::Timeout(timeout));
                }
                match receiver.recv_timeout(Duration::from_millis(5)) {
                    Ok(value) => return value,
                    Err(mpsc::RecvTimeoutError::Disconnected) => {
                        return Err(AutomationError::Closed);
                    }
                    Err(mpsc::RecvTimeoutError::Timeout) => (),
                }
            }
        })();
        self.0.pending.lock().unwrap().remove(&id);
        result
    }
    pub fn close(&self) {
        let _scope = CancellationToken::new().enter();
        if !self.is_closed() {
            let _ = self.call("Brimp.close", json!({}), Duration::from_secs(2));
        }
        self.0.disconnect();
        let mut child = self.0.child.lock().unwrap();
        let _ = child.kill();
        let _ = child.wait();
    }
}

fn protocol_error(error: &Value) -> AutomationError {
    let message = error["message"]
        .as_str()
        .unwrap_or("WebKit protocol error")
        .to_owned();
    match error["code"].as_str().unwrap_or(&message) {
        "closed" | "WindowNotFound" => AutomationError::Closed,
        "javascript" | "JavaScriptError" => AutomationError::JavaScript(message),
        "invalid_input" | "InvalidParameter" => AutomationError::InvalidInput(message),
        "unsupported" | "NotImplemented" => AutomationError::Unsupported(message),
        "navigation" => AutomationError::Navigation(message),
        _ => AutomationError::Internal(message),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn framing_handles_fragmented_reads_and_rejects_truncation() {
        struct Fragmented(std::io::Cursor<Vec<u8>>);
        impl Read for Fragmented {
            fn read(&mut self, out: &mut [u8]) -> std::io::Result<usize> {
                self.0.read(&mut out[..1])
            }
        }
        let data = br#"{"result":"hello"}"#;
        let wire = [(data.len() as u32).to_be_bytes().as_slice(), data].concat();
        assert_eq!(
            read_message(&mut Fragmented(std::io::Cursor::new(wire.clone()))).unwrap()["result"],
            "hello"
        );
        assert!(read_message(&mut &wire[..wire.len() - 1]).is_err());
        assert!(read_message(&mut &u32::MAX.to_be_bytes()[..]).is_err());
    }
}
