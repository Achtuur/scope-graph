use std::hash::Hash;

use crate::regex::RegexState;

pub trait ScopeGraphLabel:
    PartialEq + Clone + std::fmt::Debug + std::fmt::Display + Eq + Ord + Hash
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
pub enum LabelOrEnd<'a, Lbl>
where
    Lbl: ScopeGraphLabel,
{
    Label((Lbl, RegexState<'a, Lbl>)),
    /// $
    End,
}

impl<Lbl: ScopeGraphLabel> std::fmt::Display for LabelOrEnd<'_, Lbl> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LabelOrEnd::Label((lbl, _)) => write!(f, "{}", lbl),
            LabelOrEnd::End => write!(f, "$"),
        }
    }
}
