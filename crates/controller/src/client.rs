use crate::platform::{self, WorkerProcess, WorkerStream};
use brimp_worker_api::{AutomationError as Error, CancellationToken};
use serde_json::{Value, json};
use std::collections::{HashMap, VecDeque};
use std::io::{Read, Write};
use std::net::Shutdown;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex, mpsc};
use std::time::{Duration, Instant};

const MAX_FRAME: usize = 64 * 1024 * 1024;
#[derive(Default)]
struct Inbox {
    pending: HashMap<u64, mpsc::Sender<Value>>,
    events: VecDeque<Value>,
    failure: Option<String>,
}

pub struct WorkerConnection {
    process: WorkerProcess,
    writer: Mutex<WorkerStream>,
    inbox: Arc<(Mutex<Inbox>, Condvar)>,
    reader: Mutex<Option<std::thread::JoinHandle<()>>>,
    next_id: AtomicU64,
    closed: AtomicBool,
    deadline: Instant,
    timeout: Duration,
    cancellation: CancellationToken,
}

impl WorkerConnection {
    pub fn spawn(
        path: &str,
        timeout: Duration,
        cancellation: CancellationToken,
    ) -> Result<Self, Error> {
        let deadline = Instant::now() + timeout;
        let (process, stream) = platform::spawn_framed_worker(path, &["--headless".into()])
            .map_err(|e| Error::Transport(format!("could not launch worker `{path}`: {e}")))?;
        stream
            .set_write_timeout(Some(Duration::from_millis(100)))
            .map_err(io_error)?;
        let mut reader_stream = stream.try_clone().map_err(io_error)?;
        let inbox = Arc::new((Mutex::new(Inbox::default()), Condvar::new()));
        let reader_inbox = inbox.clone();
        let reader = std::thread::spawn(move || {
            let result = (|| -> Result<(), String> {
                loop {
                    let message = read_frame(&mut reader_stream)?;
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
        Ok(Self {
            process,
            writer: Mutex::new(stream),
            inbox,
            reader: Mutex::new(Some(reader)),
            next_id: AtomicU64::new(1),
            closed: AtomicBool::new(false),
            deadline,
            timeout,
            cancellation,
        })
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
        let mut request = json!({"id":id,"method":method,"params":params});
        if let Some(session) = session {
            request["sessionId"] = session.into();
        }
        let bytes = serde_json::to_vec(&request).map_err(|e| Error::Internal(e.to_string()))?;
        if bytes.len() > MAX_FRAME {
            return Err(Error::InvalidInput("CDP request exceeds 64 MiB".into()));
        }
        let (tx, rx) = mpsc::channel();
        self.inbox.0.lock().unwrap().pending.insert(id, tx);
        let result = {
            let mut writer = self.writer.lock().unwrap();
            writer
                .write_all(&(bytes.len() as u32).to_be_bytes())
                .and_then(|()| writer.write_all(&bytes))
        };
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
        let _ = self.writer.lock().unwrap().shutdown(Shutdown::Both);
        self.process.terminate();
        if let Some(reader) = self.reader.lock().unwrap().take() {
            let _ = reader.join();
        }
    }
}
impl Drop for WorkerConnection {
    fn drop(&mut self) {
        self.close();
    }
}
fn io_error(error: std::io::Error) -> Error {
    Error::Transport(error.to_string())
}
fn read_frame(reader: &mut impl Read) -> Result<Value, String> {
    let mut length = [0; 4];
    reader
        .read_exact(&mut length)
        .map_err(|e| format!("worker disconnected or truncated frame: {e}"))?;
    let length = u32::from_be_bytes(length) as usize;
    if length > MAX_FRAME {
        return Err("worker frame exceeds 64 MiB".into());
    }
    let mut bytes = vec![0; length];
    reader
        .read_exact(&mut bytes)
        .map_err(|e| format!("truncated worker frame: {e}"))?;
    let value: Value =
        serde_json::from_slice(&bytes).map_err(|e| format!("invalid worker JSON: {e}"))?;
    if !value.is_object() {
        return Err("worker frame must be an object".into());
    }
    Ok(value)
}
