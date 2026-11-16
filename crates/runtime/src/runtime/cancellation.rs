use std::cell::RefCell;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

thread_local! {
    static CURRENT: RefCell<CancellationToken> = RefCell::new(CancellationToken::new());
}

/// Restores the caller's cancellation scope, including when an operation fails.
pub struct CancellationScope(CancellationToken, std::marker::PhantomData<std::rc::Rc<()>>);

impl Drop for CancellationScope {
    fn drop(&mut self) {
        CURRENT.with(|current| current.replace(self.0.clone()));
    }
}

#[derive(Clone, Debug, Default)]
pub struct CancellationToken(Arc<AtomicBool>);
impl CancellationToken {
    /// Cancellation attached to the current host call or owner-thread command.
    pub fn current() -> Self {
        CURRENT.with(|current| current.borrow().clone())
    }
    pub fn enter(&self) -> CancellationScope {
        CancellationScope(
            CURRENT.with(|current| current.replace(self.clone())),
            std::marker::PhantomData,
        )
    }
    pub fn new() -> Self {
        Self::default()
    }
    pub fn flag(&self) -> Arc<AtomicBool> {
        self.0.clone()
    }
    pub fn cancel(&self) {
        self.0.store(true, Ordering::Release);
    }
    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::Acquire)
    }
}
