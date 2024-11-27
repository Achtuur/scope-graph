use std::fmt::Display;

#[derive(PartialEq, Eq, Hash, Debug, Clone, scopegraphs::Label, Copy)]
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
