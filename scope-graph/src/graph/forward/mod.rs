use std::{collections::HashMap, sync::{Arc, Mutex, MutexGuard}};

use plantuml::{PlantUmlItem};
use resolve::{ResolveCache, Resolver};

use crate::{
    data::ScopeGraphData, graph::{BaseScopeGraph, ScopeData, ScopeMap}, label::ScopeGraphLabel, order::{LabelOrder, LabelOrderBuilder}, path::Path, regex::dfs::RegexAutomata, resolve::QueryResult, scope::Scope
};

use super::ScopeGraph;

mod resolve;


#[derive(Debug)]
pub struct ForwardScopeGraph<Lbl, Data>
where
    Lbl: ScopeGraphLabel,
    Data: ScopeGraphData,
{
    sg: BaseScopeGraph<Lbl, Data>,
    // pub scopes: HashMap<Scope, ScopeData<Lbl, Data>>,
    pub(crate) resolve_cache: Mutex<ResolveCache<Lbl, Data>>,
}

impl<'s, Lbl, Data> ScopeGraph<'s, Lbl, Data> for ForwardScopeGraph<Lbl, Data>
where
    Lbl: ScopeGraphLabel,
    Data: ScopeGraphData,
{
    fn add_edge(&mut self, source: Scope, target: Scope, label: Lbl) {
        self.sg.add_edge(source, target, label.clone());
    }

    fn scope_iter<'a>(&'a self) -> impl Iterator<Item = (&'a Scope, &'a ScopeData<Lbl, Data>)> where Lbl: 'a, Data: 'a {
        self.sg.scope_iter()
    }

    fn scope_holds_data(&self, scope: Scope) -> bool {
        self.sg.scope_holds_data(scope)
    }

    fn find_scope(&self, scope_num: usize) -> Option<Scope> {
        self.sg.find_scope(scope_num)
    }

    fn first_scope_without_data(&self, scope_num: usize) -> Option<Scope> {
        self.sg.first_scope_without_data(scope_num)
    }

    fn add_scope(&mut self, scope: Scope, data: Data) {
        self.sg.add_scope(scope, data);
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
        DWfd: for<'da> Fn(&'da Data) -> bool,
    {
        let resolver = Resolver::new(
            self,
            path_regex,
            order,
            &data_equiv,
            &data_wellformedness,
        );
        resolver.resolve(Path::start(scope))
    }

    fn generate_cache_uml<'a>(&'a self) -> Vec<PlantUmlItem>
    where Lbl: 'a, Data: 'a {
        self.resolve_cache
            .lock()
            .unwrap()
            .iter()
            .filter_map(|(key, value)| {
                if value.envs.is_empty() {
                    return None;
                }

                let vals = value.envs.iter().map(|env| {
                    env.to_string()
                })
                .collect::<Vec<String>>()
                .join("\n");

                let cache_str = format!("<b>{}</b>\n{}", key, vals);
                Some(
                    PlantUmlItem::note(key.scope.0, cache_str)
                )
            })
            .collect::<Vec<_>>()
    }
}

impl<'s, Lbl, Data> ForwardScopeGraph<Lbl, Data>
where
    Lbl: ScopeGraphLabel,
    Data: ScopeGraphData,
{
    pub fn new() -> Self {
        Self {
            sg: BaseScopeGraph::new(),
            resolve_cache: Mutex::new(HashMap::new()),
        }
    }

    pub fn from_base(sg: BaseScopeGraph<Lbl, Data>) -> Self {
        Self {
            sg,
            resolve_cache: Mutex::new(HashMap::new()),
        }
    }

    pub fn scopes(&self) -> &ScopeMap<Lbl, Data> {
        &self.sg.scopes
    }
}
