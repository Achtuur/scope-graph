pub trait ScopeGraphLabel {
    fn char(&self) -> char;
}

impl ScopeGraphLabel for char {
    fn char(&self) -> char {
        *self
    }
}