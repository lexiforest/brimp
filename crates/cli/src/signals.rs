use crate::{Result, internal};
use brimp_runtime::runtime::CancellationToken;

#[cfg(unix)]
pub struct Signals(Vec<signal_hook_registry::SigId>);
#[cfg(unix)]
impl Signals {
    pub fn install(token: CancellationToken) -> Result<Self> {
        let mut guard = Self(Vec::new());
        for signal in [libc::SIGINT, libc::SIGTERM] {
            let token = token.clone();
            // Only an atomic store occurs in the signal handler.
            guard.0.push(
                unsafe { signal_hook_registry::register(signal, move || token.cancel()) }
                    .map_err(internal)?,
            );
        }
        Ok(guard)
    }
}
#[cfg(unix)]
impl Drop for Signals {
    fn drop(&mut self) {
        for id in &self.0 {
            signal_hook_registry::unregister(*id);
        }
    }
}
#[cfg(windows)]
pub struct Signals;
#[cfg(windows)]
static TOKEN: std::sync::OnceLock<CancellationToken> = std::sync::OnceLock::new();
#[cfg(windows)]
unsafe extern "system" fn handler(kind: u32) -> i32 {
    if kind <= 2 {
        if let Some(token) = TOKEN.get() {
            token.cancel();
        }
        1
    } else {
        0
    }
}
#[cfg(windows)]
impl Signals {
    pub fn install(token: CancellationToken) -> Result<Self> {
        TOKEN
            .set(token)
            .map_err(|_| internal("signal handler already installed"))?;
        if unsafe { windows_sys::Win32::System::Console::SetConsoleCtrlHandler(Some(handler), 1) }
            == 0
        {
            return Err(internal(std::io::Error::last_os_error()));
        }
        Ok(Self)
    }
}
#[cfg(windows)]
impl Drop for Signals {
    fn drop(&mut self) {
        unsafe {
            windows_sys::Win32::System::Console::SetConsoleCtrlHandler(Some(handler), 0);
        }
    }
}
