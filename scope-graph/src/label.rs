use std::hash::Hash;

pub trait ScopeGraphLabel:
    PartialEq
    + Clone
    + std::fmt::Debug
    + std::fmt::Display
    + Eq
    + Ord
    + Hash
{
    fn char(&self) -> char;
    fn str(&self) -> &'static str;
}

impl ScopeGraphLabel for char {
    fn char(&self) -> char {
        *self
    }

    fn str(&self) -> &'static str {
        unimplemented!("char does not have a string representation")
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum LabelOrEnd<Lbl>
{
    Label(Lbl),
    End,
}
