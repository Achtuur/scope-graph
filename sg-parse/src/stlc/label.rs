use std::fmt::Display;

use scope_graph::label::ScopeGraphLabel;

#[derive(PartialEq, Eq, Hash, Debug, Clone, scopegraphs::Label, Copy, PartialOrd, Ord)]
pub enum StlcLabel {
    Parent,
    Declaration,
    Record,
    Extension,
}

impl Display for StlcLabel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StlcLabel::Parent => write!(f, "Parent"),
            StlcLabel::Declaration => write!(f, "Declaration"),
            StlcLabel::Extension => write!(f, "Extension"),
            StlcLabel::Record => write!(f, "Record"),
        }
    }
}

impl ScopeGraphLabel for StlcLabel {
    fn char(&self) -> char {
        match self {
            StlcLabel::Parent => 'P',
            StlcLabel::Declaration => 'D',
            StlcLabel::Extension => 'E',
            StlcLabel::Record => 'R',
        }
    }

    fn str(&self) -> &'static str {
        match self {
            StlcLabel::Parent => "Parent",
            StlcLabel::Declaration => "Declaration",
            StlcLabel::Extension => "Extension",
            StlcLabel::Record => "Record",
        }
    }
}
