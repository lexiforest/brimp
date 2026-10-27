//! Session-owned origin storage and lock arbitration. No JS values cross threads.
use brimp_jsc::{JsException, JsRuntime, ProtectedJsObject};
use serde_json::{Value, json};
use std::{
    cell::RefCell,
    collections::{BTreeMap, HashMap},
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, Ordering},
        mpsc,
    },
    thread::Thread,
};

pub(crate) type Partitions = BTreeMap<String, BTreeMap<String, indexmap::IndexMap<String, String>>>;
#[derive(Debug, Default)]
pub(crate) struct Storage {
    pub data: Mutex<Partitions>,
    locks: Mutex<Vec<Lock>>,
    subscribers: Mutex<Vec<Subscriber>>,
    path: Option<std::path::PathBuf>,
    _file_lock: Mutex<Option<std::fs::File>>,
}
impl Storage {
    pub fn open(path: Option<std::path::PathBuf>) -> Result<Self, String> {
        let Some(path) = path else {
            return Ok(Self::default());
        };
        let mut lock_path = path.as_os_str().to_os_string();
        lock_path.push(".lock");
        let lock = std::fs::OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(lock_path)
            .map_err(|e| e.to_string())?;
        lock.try_lock()
            .map_err(|e| format!("Storage file is already in use: {e}"))?;
        let data = match std::fs::read(&path) {
            Ok(bytes) => {
                serde_json::from_slice(&bytes).map_err(|e| format!("Invalid storage file: {e}"))?
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Partitions::default(),
            Err(error) => return Err(error.to_string()),
        };
        Ok(Self {
            data: Mutex::new(data),
            locks: Mutex::default(),
            subscribers: Mutex::default(),
            path: Some(path),
            _file_lock: Mutex::new(Some(lock)),
        })
    }
    pub fn persist(&self, data: &Partitions) -> Result<(), String> {
        let Some(path) = &self.path else {
            return Ok(());
        };
        let lock = self._file_lock.lock().unwrap();
        if lock.is_none() {
            return Err("Storage session is closed".into());
        }
        let mut temporary = path.as_os_str().to_os_string();
        temporary.push(".tmp");
        let bytes = serde_json::to_vec(data).map_err(|e| e.to_string())?;
        std::fs::write(&temporary, bytes).map_err(|e| e.to_string())?;
        std::fs::rename(&temporary, path).map_err(|e| e.to_string())
    }
    pub fn close(&self) {
        self._file_lock.lock().unwrap().take();
    }
    pub fn persistent(&self) -> bool {
        self.path.is_some()
    }
}
#[derive(Debug)]
struct Subscriber {
    owner: u64,
    context: std::sync::Weak<super::BrowsingContext>,
    sender: mpsc::Sender<(u64, String)>,
    wake: Thread,
}
#[derive(Debug)]
struct Lock {
    id: u64,
    owner: u64,
    origin: String,
    name: String,
    shared: bool,
    held: bool,
    sender: mpsc::Sender<(u64, String)>,
    wake: Thread,
}
impl Lock {
    fn notify(&self, status: &'static str) {
        let _ = self.sender.send((self.id, status.to_owned()));
        self.wake.unpark();
    }
}
fn grant(locks: &mut [Lock]) {
    for i in 0..locks.len() {
        if locks[i].held {
            continue;
        }
        let blocked = locks.iter().enumerate().any(|(j, other)| {
            j != i
                && (other.held || j < i)
                && other.origin == locks[i].origin
                && other.name == locks[i].name
                && !(other.shared && locks[i].shared)
        });
        if !blocked {
            locks[i].held = true;
            locks[i].notify("granted");
        }
    }
}
pub(crate) struct Realm {
    // Opaque-origin state belongs only to this realm and never enters the storage file.
    pub ephemeral: Mutex<Partitions>,
    id: u64,
    storage: Arc<Storage>,
    sender: mpsc::Sender<(u64, String)>,
    receiver: mpsc::Receiver<(u64, String)>,
    callbacks: RefCell<HashMap<u64, ProtectedJsObject>>,
}
static NEXT_ID: AtomicU64 = AtomicU64::new(1);
impl Realm {
    pub fn new(storage: Arc<Storage>) -> Self {
        let (sender, receiver) = mpsc::channel();
        Self {
            ephemeral: Mutex::default(),
            id: NEXT_ID.fetch_add(1, Ordering::Relaxed),
            storage,
            sender,
            receiver,
            callbacks: RefCell::default(),
        }
    }
    pub fn opaque_origin(&self) -> String {
        format!("opaque:{}", self.id)
    }
    pub fn subscribe(&self, context: &Arc<super::BrowsingContext>, callback: ProtectedJsObject) {
        self.callbacks.borrow_mut().insert(0, callback);
        self.storage.subscribers.lock().unwrap().push(Subscriber {
            owner: self.id,
            context: Arc::downgrade(context),
            sender: self.sender.clone(),
            wake: std::thread::current(),
        });
    }
    pub fn broadcast(&self, origin: &str, event: Value) {
        for subscriber in self.storage.subscribers.lock().unwrap().iter() {
            if subscriber.owner == self.id {
                continue;
            }
            let same_origin = subscriber
                .context
                .upgrade()
                .and_then(|context| context.current_url())
                .and_then(|url| url::Url::parse(&url).ok())
                .is_some_and(|url| url.origin().ascii_serialization() == origin);
            if same_origin {
                let _ = subscriber.sender.send((0, event.to_string()));
                subscriber.wake.unpark();
            }
        }
    }
    pub fn request(&self, origin: String, request: &Value, callback: ProtectedJsObject) -> u64 {
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        self.callbacks.borrow_mut().insert(id, callback);
        let mut locks = self.storage.locks.lock().unwrap();
        let name = request["name"].as_str().unwrap_or_default().to_owned();
        let shared = request["mode"] == "shared";
        let steal = request["steal"] == true;
        if steal {
            locks.retain(|lock| {
                if lock.origin == origin && lock.name == name && lock.held {
                    lock.notify("stolen");
                    false
                } else {
                    true
                }
            });
        }
        let mut entry = Lock {
            id,
            owner: self.id,
            origin,
            name,
            shared,
            held: false,
            sender: self.sender.clone(),
            wake: std::thread::current(),
        };
        let unavailable = locks.iter().any(|lock| {
            lock.origin == entry.origin && lock.name == entry.name && !(lock.shared && shared)
        });
        if request["ifAvailable"] == true && unavailable {
            entry.notify("unavailable");
        } else if steal {
            entry.held = true;
            entry.notify("granted");
            locks.insert(0, entry);
        } else {
            locks.push(entry);
            grant(&mut locks);
        }
        id
    }
    pub fn release(&self, id: u64) {
        self.callbacks.borrow_mut().remove(&id);
        let mut locks = self.storage.locks.lock().unwrap();
        locks.retain(|lock| lock.id != id || lock.owner != self.id);
        grant(&mut locks);
    }
    pub fn query(&self, origin: &str) -> Value {
        let locks = self.storage.locks.lock().unwrap();
        let snapshot = |held| {
            locks.iter().filter(|lock| lock.origin == origin && lock.held == held && !lock.name.starts_with('-'))
            .map(|lock| json!({"name":lock.name,"mode":if lock.shared {"shared"} else {"exclusive"},"clientId":lock.owner.to_string()})).collect::<Vec<_>>()
        };
        json!({"held":snapshot(true),"pending":snapshot(false)})
    }
    pub fn pump(&self, js: &JsRuntime) -> Result<bool, JsException> {
        let mut progress = false;
        while let Ok((id, status)) = self.receiver.try_recv() {
            let callback = self.callbacks.borrow().get(&id).cloned();
            if let Some(callback) = callback {
                js.call_function_with_string(&callback, &status)?;
                progress = true;
            }
        }
        Ok(progress)
    }
}
impl Drop for Realm {
    fn drop(&mut self) {
        self.storage
            .subscribers
            .lock()
            .unwrap()
            .retain(|subscriber| subscriber.owner != self.id);
        let mut locks = self.storage.locks.lock().unwrap();
        locks.retain(|lock| lock.owner != self.id);
        grant(&mut locks);
    }
}
