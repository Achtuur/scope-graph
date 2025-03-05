use crate::{label::ScopeGraphLabel, path::Path};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueryResult<Lbl, Data>
where
    Lbl: ScopeGraphLabel + Clone,
    Data: Clone,
{
    pub path: Path<Lbl>,
    pub data: Data,
}

impl<Lbl, Data> std::fmt::Display for QueryResult<Lbl, Data>
where
    Lbl: ScopeGraphLabel + Clone,
    Data: Clone + std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} -[] {:?}", self.path, self.data)
    }
}
