use std::collections::HashMap;

use plantuml::PlantUmlItem;
use resolve::CachedResolver;

use crate::{
    data::ScopeGraphData,
    graph::{BaseScopeGraph, ScopeData, ScopeMap},
    label::ScopeGraphLabel,
    order::LabelOrder,
    path::Path,
    regex::dfs::RegexAutomata,
    resolve::QueryResult,
    scope::Scope,
};

use super::ScopeGraph;

mod resolve;

type ProjHash = u64;

/// Key for the cache.
///
/// This is a tuple of index in regex automata, the result from projecting the data and the target scope.
///
/// You can alternatively think of this as a (usize, DataProj) cache per scope.
type QueryCacheKey = (usize, ProjHash, Scope);

/// Cache for the results of a certain query
type QueryCache<Lbl, Data> = HashMap<QueryCacheKey, Vec<QueryResult<Lbl, Data>>>;

/// Key for `ScopeGraphCache`
type ParameterKey<Lbl> = (LabelOrder<Lbl>, RegexAutomata<Lbl>);
/// Cache for the entire scope graph.
///
/// This contains a cache per set of query parameters
type ScopeGraphCache<Lbl, Data> = HashMap<ParameterKey<Lbl>, QueryCache<Lbl, Data>>;

#[derive(Debug)]
pub struct CachedScopeGraph<Lbl, Data>
where
    Lbl: ScopeGraphLabel,
    Data: ScopeGraphData,
{
    sg: BaseScopeGraph<Lbl, Data>,
    // pub scopes: HashMap<Scope, ScopeData<Lbl, Data>>,
    resolve_cache: ScopeGraphCache<Lbl, Data>,
}

impl<Lbl, Data> ScopeGraph<Lbl, Data> for CachedScopeGraph<Lbl, Data>
where
    Lbl: ScopeGraphLabel,
    Data: ScopeGraphData,
{
    fn add_edge(&mut self, source: Scope, target: Scope, label: Lbl) {
        self.sg.add_edge(source, target, label.clone());
    }

    fn scope_iter<'a>(&'a self) -> impl Iterator<Item = (&'a Scope, &'a ScopeData<Lbl, Data>)>
    where
        Lbl: 'a,
        Data: 'a,
    {
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
        &mut self,
        scope: Scope,
        path_regex: &RegexAutomata<Lbl>,
        order: &LabelOrder<Lbl>,
        data_equiv: DEq,
        data_wellformedness: DWfd,
    ) -> Vec<QueryResult<Lbl, Data>>
    where
        DEq: for<'da, 'db> Fn(&'da Data, &'db Data) -> bool,
        DWfd: for<'da> Fn(&'da Data) -> bool,
    {
        unreachable!("Use query_proj instead");
        // let cache = self.resolve_cache
        // .entry((order.clone(), path_regex.clone()))
        // .or_default();

        // let resolver = CachedResolver::new(
        //     &self.sg,
        //     cache,
        //     path_regex,
        //     order,
        //     &data_equiv,
        //     &data_wellformedness,
        // );
        // resolver.resolve(Path::start(scope))
    }

    fn query_proj<P, DProj, DEq>(
        &mut self,
        scope: Scope,
        path_regex: &RegexAutomata<Lbl>,
        order: &LabelOrder<Lbl>,
        data_proj: DProj,
        proj_wfd: P,
        data_equiv: DEq,
    ) -> Vec<QueryResult<Lbl, Data>>
    where
        P: std::hash::Hash + Eq,
        DProj: for<'da> Fn(&'da Data) -> P,
        DEq: for<'da, 'db> Fn(&'da Data, &'db Data) -> bool,
    {
        let mut resolver = CachedResolver::new(
            &self.sg,
            self.resolve_cache
                .entry((order.clone(), path_regex.clone()))
                .or_default(),
            path_regex,
            order,
            &data_equiv,
            // &data_wfd,
            data_proj,
            proj_wfd,
        );
        resolver.resolve(Path::start(scope))
    }

    fn generate_cache_uml<'a>(&'a self) -> Vec<PlantUmlItem>
    where
        Lbl: 'a,
        Data: 'a,
    {
        self.resolve_cache
            .iter()
            .flat_map(|(query_params, query_cache)| {
                query_cache
                    .iter()
                    .filter(|(key, _)| !self.scope_holds_data(key.2))
                    // map with scope as key to not have duplicate notes
                    .fold(HashMap::new(), |mut acc, (keys, envs)| {
                        let key = keys.2; // scope
                        let entry: &mut HashMap<QueryCacheKey, &Vec<QueryResult<Lbl, Data>>> =
                            acc.entry(key).or_default();
                        entry.insert(*keys, envs);
                        acc
                    })
                    .into_iter()
                    .filter_map(|(key, envs)| {
                        if envs.is_empty() {
                            return None;
                        }

                        let vals = envs
                            .iter()
                            .map(|(keys, env)| {
                                let cache_str = env
                                    .iter()
                                    .map(|result| result.to_string())
                                    .collect::<Vec<String>>()
                                    .join("\n");
                                // cache_str
                                format!("<b>{:?}</b>\n{}", keys, cache_str)
                            })
                            .collect::<Vec<String>>()
                            .join("\n");

                        let cache_str = format!("<b>{:?}</b>\n{}", key, vals);
                        Some(PlantUmlItem::note(key.uml_id(), cache_str))
                    })
            })
            .collect()
    }

    fn get_scope(&self, scope: Scope) -> Option<&ScopeData<Lbl, Data>> {
        self.sg.get_scope(scope)
    }
}

impl<'s, Lbl, Data> Default for CachedScopeGraph<Lbl, Data>
where
    Lbl: ScopeGraphLabel,
    Data: ScopeGraphData,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<'s, Lbl, Data> CachedScopeGraph<Lbl, Data>
where
    Lbl: ScopeGraphLabel,
    Data: ScopeGraphData,
{
    pub fn new() -> Self {
        Self {
            sg: BaseScopeGraph::new(),
            resolve_cache: ScopeGraphCache::new(),
        }
    }

    pub fn from_base(sg: BaseScopeGraph<Lbl, Data>) -> Self {
        Self {
            sg,
            resolve_cache: ScopeGraphCache::new(),
        }
    }

    pub fn scopes(&self) -> &ScopeMap<Lbl, Data> {
        self.sg.scopes()
    }
}
