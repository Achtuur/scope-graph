use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{
    data::ScopeGraphData, label::ScopeGraphLabel, order::LabelOrder, path::Path,
    projection::ScopeGraphDataProjection, regex::dfs::RegexAutomaton, scope::Scope,
};

use super::{
    Edge, ScopeData, ScopeGraph, ScopeMap,
    resolve::{QueryResult, Resolver},
};

/// Base scope graph behaviour
///
/// Creation of scopes, does not implement query/caching logic
///
/// saves some duplication, to test faster
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
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

    pub fn scopes(&self) -> &ScopeMap<Lbl, Data> {
        &self.scopes
    }
}

impl<Lbl, Data> ScopeGraph<Lbl, Data> for BaseScopeGraph<Lbl, Data>
where
    Lbl: ScopeGraphLabel + Clone,
    Data: ScopeGraphData + Clone,
{
    fn reset_cache(&mut self) {}

    fn add_scope(&mut self, scope: Scope, data: Data) -> Scope {
        tracing::trace!("Adding scope: {} with data: {}", scope, data);
        self.scopes.insert(scope, ScopeData::new(data));
        scope
    }

    fn add_edge(&mut self, source: Scope, target: Scope, label: Lbl) {
        tracing::debug!(
            "Adding edge: {} <-> {} with label: {}",
            source,
            target,
            label
        );

        let edge_to_parent = Edge::new(target, label.clone());
        self.scopes
            .get_mut(&source)
            .expect("Attempting to add edge to non-existant scope")
            .outgoing_mut()
            .push(edge_to_parent);

        let edge_to_child = Edge::new(source, label);
        self.scopes
            .get_mut(&target)
            .expect("Attempting to add edge to non-existant scope")
            .incoming_mut()
            .push(edge_to_child);
    }

    fn get_scope(&self, scope: Scope) -> Option<&ScopeData<Lbl, Data>> {
        self.scopes.get(&scope)
    }

    fn query<DEq, DWfd>(
        &mut self,
        scope: Scope,
        path_regex: &RegexAutomaton<Lbl>,
        order: &LabelOrder<Lbl>,
        data_equiv: DEq,
        data_wellformedness: DWfd,
    ) -> Vec<QueryResult<Lbl, Data>>
    where
        DEq: for<'da, 'db> Fn(&'da Data, &'db Data) -> bool,
        DWfd: for<'da> Fn(&'da Data) -> bool,
    {
        let mut resolver =
            Resolver::new(self, path_regex, order, &data_equiv, &data_wellformedness);
        resolver.resolve(Path::start(scope))
    }

    fn query_proj<Proj>(
        &mut self,
        scope: Scope,
        path_regex: &RegexAutomaton<Lbl>,
        order: &LabelOrder<Lbl>,
        proj: Proj,
        proj_wfd: Proj::Output,
    ) -> Vec<QueryResult<Lbl, Data>>
    where
        Proj: ScopeGraphDataProjection<Data>,
    {
        let data_wfd = |data: &Data| proj.project(data) == proj_wfd;
        let data_equiv = |a: &Data, b: &Data| proj.project(a) == proj.project(b);
        self.query(scope, path_regex, order, data_equiv, data_wfd)
    }

    fn scope_iter<'a>(&'a self) -> impl Iterator<Item = (&'a Scope, &'a ScopeData<Lbl, Data>)>
    where
        Lbl: 'a,
        Data: 'a,
    {
        self.scopes.iter()
    }

    fn scope_holds_data(&self, scope: Scope) -> bool {
        self.scopes
            .get(&scope)
            .map(|d| d.data.variant_has_data())
            .unwrap_or_default()
    }
}
