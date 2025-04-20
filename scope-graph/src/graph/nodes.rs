use crate::{label::ScopeGraphLabel, scope::Scope};


/// Bi-directional edge between two scopes
#[derive(Clone, Copy, Debug)]
pub struct Edge<Lbl>
where
    Lbl: ScopeGraphLabel,
{
    pub to: (Scope, Lbl),
}

impl<Lbl: ScopeGraphLabel> Edge<Lbl> {
    pub fn new(scope: Scope, label: Lbl) -> Self {
        Self {
            to: (scope, label),
        }
    }

    pub fn target(&self) -> Scope {
        self.to.0
    }

    pub fn lbl(&self) -> &Lbl {
        &self.to.1
    }
}

#[derive(Clone, Debug)]
pub struct ScopeData<Lbl, Data>
where
    Lbl: ScopeGraphLabel + Clone,
{
    // pub edges: Vec<Edge<Lbl>>,
    pub children: Vec<Edge<Lbl>>,
    pub parents: Vec<Edge<Lbl>>,
    pub data: Data,
}

impl<Lbl, Data> ScopeData<Lbl, Data>
where
    Lbl: ScopeGraphLabel + Clone,
{
    pub fn new(data: Data) -> Self {
        Self {
            data,
            children: Vec::new(),
            parents: Vec::new(),
        }
    }

    pub fn children(&self) -> &[Edge<Lbl>] {
        &self.children
    }

    pub fn children_mut(&mut self) -> &mut Vec<Edge<Lbl>> {
        &mut self.children
    }

    pub fn parents(&self) -> &[Edge<Lbl>] {
        &self.parents
    }

    pub fn parents_mut(&mut self) -> &mut Vec<Edge<Lbl>> {
        &mut self.parents
    }
}