use std::collections::HashMap;

use plantuml::{Color, EdgeDirection, NodeType, PlantUmlItem};

use crate::{data::ScopeGraphData, label::ScopeGraphLabel, order::LabelOrder, regex::dfs::RegexAutomata, resolve::QueryResult, scope::Scope};

use super::{Edge, ScopeData, ScopeGraph, ScopeMap};

/// Base scope graph behaviour
///
/// Creation of scopes, does not implement query/caching logic
///
/// saves some duplication, to test faster
#[derive(Debug, Clone, Default)]
pub struct BaseScopeGraph<Lbl, Data>
where
    Lbl: ScopeGraphLabel + Clone,
    Data: ScopeGraphData + Clone,
{
    pub scopes: ScopeMap<Lbl, Data>,
}

impl<Lbl, Data> BaseScopeGraph<Lbl, Data>
where
    Lbl: ScopeGraphLabel + Clone,
    Data: ScopeGraphData + Clone,
{
    pub fn new() -> Self {
        Self {
            scopes: HashMap::new(),
        }
    }
}

impl<'s, Lbl, Data> ScopeGraph<'s, Lbl, Data> for BaseScopeGraph<Lbl, Data>
where Lbl: ScopeGraphLabel + Clone,
      Data: ScopeGraphData + Clone,
{
    fn add_scope(&mut self, scope: Scope, data: Data) {
        tracing::trace!("Adding scope: {} with data: {}", scope, data);
        self.scopes.insert(scope, ScopeData::new(data));
    }

    fn add_edge(&mut self, source: Scope, target: Scope, label: Lbl) {
        tracing::debug!("Adding edge: {} <-> {} with label: {}", source, target, label);

        let edge_to_parent = Edge::new(target, label.clone());
        self
            .scopes
            .get_mut(&source)
            .expect("Attempting to add edge to non-existant scope")
            .parents_mut()
            .push(edge_to_parent);

        let edge_to_child = Edge::new(source, label);
        self
            .scopes
            .get_mut(&target)
            .expect("Attempting to add edge to non-existant scope")
            .children_mut()
            .push(edge_to_child);
    }

    fn query<DEq, DWfd>(
        & self,
        scope: Scope,
        path_regex: & RegexAutomata<Lbl>,
        order: & LabelOrder<Lbl>,
        data_equiv: DEq,
        data_wellformedness: DWfd,
    ) -> Vec<QueryResult<Lbl, Data>>
    where
        DEq: for<'da, 'db> Fn(&'da Data, &'db Data) -> bool,
        DWfd: for<'da> Fn(&'da Data) -> bool {
        unreachable!("Can't query base graph")
    }

    fn scope_iter<'a>(&'a self) -> impl Iterator<Item = (&'a Scope, &'a ScopeData<Lbl, Data>)> where Lbl: 'a, Data: 'a {
        self.scopes.iter()
    }

    fn scope_holds_data(&self, scope: Scope) -> bool {
        self.scopes.get(&scope).map(|d| d.data.variant_has_data()).unwrap_or_default()
    }

}