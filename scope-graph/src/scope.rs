use std::sync::atomic::{AtomicUsize, Ordering};

use deepsize::DeepSizeOf;
use serde::{Deserialize, Serialize};

static SCOPE_COUNTER: AtomicUsize = AtomicUsize::new(0);

/// A single scope in the scope graph. Each scope is assigned an incrementing id.
#[derive(Default, Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, DeepSizeOf)]
pub struct Scope(pub usize);

impl From<usize> for Scope {
    fn from(id: usize) -> Self {
        Scope(id)
    }
}

impl Scope {
    /// Create a new scope with the given id.
    pub fn new() -> Self {
        Scope(SCOPE_COUNTER.fetch_add(1, Ordering::Relaxed))
    }

    pub fn id(&self) -> usize {
        self.0
    }

    pub fn uml_id(&self) -> String {
        format!("scope_{}", self.0)
    }

    pub fn reset_counter() {
        SCOPE_COUNTER.fetch_and(0, Ordering::Relaxed);
    }
}

impl std::fmt::Display for Scope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
