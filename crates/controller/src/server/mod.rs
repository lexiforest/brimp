#[cfg(target_os = "macos")]
mod controller;
#[cfg(target_os = "macos")]
mod proxy;

use std::sync::{Arc, atomic::AtomicBool};

pub const SERVE_USAGE: &str = "usage: brimp serve --worker-path PATH [OPTIONS]\n\n  --cdp (launch a WebSocket CDP worker)\n  --worker-args PATH (JSON array of launch arguments)\n  --worker-arg ARG (repeatable)\n  --pool-size N\n  --max-pending N\n  --max-jobs N\n  --max-worker-memory-mb N\n  --operation-timeout SECONDS\n  --port PORT\n  --headless\n  --window-size WIDTH,HEIGHT\n\nBRIMP_WORKER_PATH supplies the default worker path. Controller support: macOS.";

pub fn run(arguments: impl Iterator<Item = String>, stop: Arc<AtomicBool>) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        controller::run(arguments, stop)
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = (arguments, stop);
        Err("serve process supervision is supported only on macOS".into())
    }
}
