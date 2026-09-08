#[cfg(target_os = "macos")]
mod client;
#[cfg(target_os = "macos")]
mod controller;
#[cfg(target_os = "macos")]
#[path = "mac/platform.rs"]
mod platform;
#[cfg(target_os = "macos")]
mod proxy;
#[cfg(target_os = "macos")]
pub use client::WorkerConnection;
#[cfg(not(target_os = "macos"))]
#[path = "unsupported.rs"]
mod client;
#[cfg(not(target_os = "macos"))]
pub use client::WorkerConnection;
pub mod browser;

use std::sync::{Arc, atomic::AtomicBool};

pub const SERVE_USAGE: &str = "usage: brimp serve --worker-path PATH [OPTIONS]\n\n  --worker-protocol framed-cdp|cdp\n  --worker-arg ARG (repeatable)\n  --pool-size N\n  --max-pending N\n  --max-jobs N\n  --max-worker-memory-mb N\n  --operation-timeout SECONDS\n  --port PORT\n  --headless\n  --window-size WIDTH,HEIGHT\n\nBRIMP_WORKER_PATH supplies the default worker path. Controller support: macOS.";

pub fn run(arguments: impl Iterator<Item = String>, stop: Arc<AtomicBool>) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        controller::run(arguments, stop)
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = (arguments, stop);
        Err("worker-backed commands are supported only on macOS".into())
    }
}
