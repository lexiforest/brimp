use brimp_worker_api::{AutomationError as Error, CancellationToken};
use serde_json::Value;
use std::time::Duration;
pub struct WorkerConnection;
fn unsupported() -> Error {
    Error::Unsupported("worker-backed commands are supported only on macOS".into())
}
impl WorkerConnection {
    pub fn spawn(_: &str, _: Duration, _: CancellationToken) -> Result<Self, Error> {
        Err(unsupported())
    }
    pub fn command(&self, _: &str, _: Value, _: Option<&str>) -> Result<Value, Error> {
        Err(unsupported())
    }
    pub fn event(&self, _: &str, _: impl Fn(&Value) -> bool) -> Result<Value, Error> {
        Err(unsupported())
    }
    pub fn drain_events(&self, _: &str) -> Vec<Value> {
        Vec::new()
    }
    pub fn check(&self) -> Result<(), Error> {
        Err(unsupported())
    }
    pub fn close(&self) {}
}
