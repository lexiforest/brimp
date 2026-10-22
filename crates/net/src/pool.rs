use std::sync::Mutex;

use crate::error::Error;
use crate::ffi::{Curl, curl_easy_cleanup, curl_easy_init, curl_easy_reset, global_init};

#[derive(Debug)]
struct IdleHandle(*mut Curl);

// An easy handle is used by only one transfer thread at a time. Returning it to
// the mutex-protected pool transfers ownership to the next borrower.
unsafe impl Send for IdleHandle {}

/// Retains easy handles so sequential requests can reuse their TLS and HTTP
/// connections. The pool belongs to one client/configuration, preventing a
/// connection established under one impersonation profile or proxy from being
/// reused by another.
#[derive(Debug, Default)]
pub(crate) struct CurlHandlePool {
    idle: Mutex<Vec<IdleHandle>>,
}

impl CurlHandlePool {
    pub(crate) fn acquire(&self) -> Result<*mut Curl, Error> {
        global_init();
        let mut idle = self.idle.lock().expect("curl handle pool poisoned");
        let reused = idle.pop();
        let was_reused = reused.is_some();
        let handle = reused
            .map(|handle| handle.0)
            .unwrap_or_else(|| unsafe { curl_easy_init() });
        let idle_remaining = idle.len();
        drop(idle);
        if handle.is_null() {
            return Err(Error::Initialization(
                "failed to initialize libcurl".to_string(),
            ));
        }
        unsafe {
            curl_easy_reset(handle);
        }
        if trace_enabled() {
            eprintln!(
                "brimp-net pool={:p} acquire handle={handle:p} reused={} idle_remaining={idle_remaining}",
                self, was_reused
            );
        }
        Ok(handle)
    }

    pub(crate) fn release(&self, handle: *mut Curl) {
        let mut idle = self.idle.lock().expect("curl handle pool poisoned");
        // Clear pointers into the completed transfer before retaining a handle.
        unsafe {
            curl_easy_reset(handle);
        }
        if idle.len() >= 64 {
            unsafe {
                curl_easy_cleanup(handle);
            }
            return;
        }
        idle.push(IdleHandle(handle));
        if trace_enabled() {
            eprintln!(
                "brimp-net pool={:p} release handle={handle:p} idle={}",
                self,
                idle.len()
            );
        }
    }
}

pub(crate) fn trace_enabled() -> bool {
    std::env::var_os("BRIMP_NET_TRACE").is_some()
}

impl Drop for CurlHandlePool {
    fn drop(&mut self) {
        let handles = self.idle.get_mut().expect("curl handle pool poisoned");
        for handle in handles.drain(..) {
            unsafe {
                curl_easy_cleanup(handle.0);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::CurlHandlePool;

    #[test]
    fn reuses_released_easy_handle() {
        let pool = CurlHandlePool::default();
        let first = pool.acquire().unwrap();
        pool.release(first);

        let second = pool.acquire().unwrap();

        assert_eq!(second, first);
        pool.release(second);
    }
}
