use std::sync::atomic::{AtomicUsize, Ordering};

static SCOPE_COUNTER: AtomicUsize = AtomicUsize::new(0);

/// A single scope in the scope graph. Each scope is assigned an incrementing id.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Scope(pub usize);

impl Scope {
    /// Create a new scope with the given id.
    pub fn new() -> Self {
        Scope(SCOPE_COUNTER.fetch_add(1, Ordering::Relaxed))
    }

    pub fn id(&self) -> usize {
        self.0
    }
}
