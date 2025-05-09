use std::collections::HashMap;

use graphing::{
    mermaid::{
        item::{ItemShape, MermaidItem},
        theme::EdgeType,
    },
    plantuml::PlantUmlItem,
};
use resolve::CachedResolver;
use serde::{Deserialize, Serialize};

use crate::{
    BackgroundColor, ColorSet, ForeGroundColor,
    data::ScopeGraphData,
    graph::{BaseScopeGraph, ScopeData, ScopeMap},
    label::ScopeGraphLabel,
    order::LabelOrder,
    path::Path,
    regex::dfs::RegexAutomaton,
    scope::Scope,
};

use super::{ScopeGraph, resolve::QueryResult};

mod resolve;

type ProjHash = u64;

type ProjEnvs<Lbl, Data> = HashMap<ProjHash, Vec<QueryResult<Lbl, Data>>>;

/// Key for the cache.
///
/// This is a tuple of index in regex automata, the result from projecting the data and the target scope.
///
/// You can alternatively think of this as a (usize, DataProj) cache per scope.
type QueryCacheKey = (usize, Scope);

/// Cache for the results of a certain query
type QueryCache<Lbl, Data> = HashMap<QueryCacheKey, ProjEnvs<Lbl, Data>>;

/// Key for `ScopeGraphCache`
type ParameterKey<Lbl> = (LabelOrder<Lbl>, RegexAutomaton<Lbl>);
/// Cache for the entire scope graph.
///
/// This contains a cache per set of query parameters
type ScopeGraphCache<Lbl, Data> = HashMap<ParameterKey<Lbl>, QueryCache<Lbl, Data>>;

#[derive(Debug, Serialize, Deserialize)]
pub struct CachedScopeGraph<Lbl, Data>
where
    Lbl: ScopeGraphLabel,
    Data: ScopeGraphData,
{
    sg: BaseScopeGraph<Lbl, Data>,
    // pub scopes: HashMap<Scope, ScopeData<Lbl, Data>>,
    #[serde(skip)]
    resolve_cache: ScopeGraphCache<Lbl, Data>,
}

impl<Lbl, Data> ScopeGraph<Lbl, Data> for CachedScopeGraph<Lbl, Data>
where
    Lbl: ScopeGraphLabel,
    Data: ScopeGraphData,
{
    fn reset_cache(&mut self) {
        self.resolve_cache.clear();
    }


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

    fn add_scope(&mut self, scope: Scope, data: Data) -> Scope {
        self.sg.add_scope(scope, data)
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
        unreachable!("Use query_proj instead");
    }

    fn query_proj<P, DProj>(
        &mut self,
        scope: Scope,
        path_regex: &RegexAutomaton<Lbl>,
        order: &LabelOrder<Lbl>,
        data_proj: DProj,
        proj_wfd: P,
    ) -> Vec<QueryResult<Lbl, Data>>
    where
        P: std::hash::Hash + Eq,
        DProj: for<'da> Fn(&'da Data) -> P,
    {
        // todo: fix key to not have to clone
        let cache_entry = self.resolve_cache.entry((order.clone(), path_regex.clone())).or_default();
        let mut resolver = CachedResolver::new(
            &self.sg,
            cache_entry,
            path_regex,
            order,
            data_proj,
            proj_wfd,
        );
        let envs = resolver.resolve(Path::start(scope));
        tracing::info!(
            "Resolved query: {}, {}, {}, found:",
            scope,
            path_regex,
            order,
        );
        for qr in &envs {
            tracing::info!("\t{}", qr);
        }
        envs
    }

    fn generate_cache_uml(&self) -> Vec<PlantUmlItem> {
        self.resolve_cache
            .iter()
            .flat_map(|(query_params, query_cache)| {
                query_cache
                    .iter()
                    .filter(|(key, _)| !self.scope_holds_data(key.1))
                    // map with scope as key to not have duplicate notes
                    .fold(HashMap::new(), |mut acc, (keys, envs)| {
                        let key = keys.1; // scope
                        let entry: &mut QueryCache<Lbl, Data> =
                            acc.entry(key).or_default();
                        entry.insert(*keys, envs.clone());
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
                                    .values()
                                    .flat_map(|result| result.into_iter().map(|x| x.to_string()))
                                    .collect::<Vec<String>>()
                                    .join("\n");
                                format!(
                                    "<b>(p{}, s{})</b>\n{}",
                                    keys.0, keys.1, cache_str
                                )
                            })
                            .collect::<Vec<String>>()
                            .join("\n");

                        let cache_str = format!("<b>{:?}</b>\n{}", key, vals);
                        let item = PlantUmlItem::note(key.uml_id(), cache_str)
                            .add_class("cache-entry")
                            .add_class(BackgroundColor::get_class_name(key.0));
                        Some(item)
                    })
            })
            .collect()
    }

    fn generate_cache_mmd(&self) -> Vec<MermaidItem> {
        self.resolve_cache
            .iter()
            .flat_map(|(query_params, query_cache)| {
                query_cache
                    .iter()
                    .filter(|(key, _)| !self.scope_holds_data(key.1))
                    // map with scope as key to not have duplicate notes
                    .fold(HashMap::new(), |mut acc, (keys, envs)| {
                        let key = keys.1; // scope
                        let entry: &mut QueryCache<Lbl, Data> =
                            acc.entry(key).or_default();
                        entry.insert(*keys, envs.clone());
                        acc
                    })
                    .into_iter()
                    .flat_map(|(key, envs)| {
                        if envs.is_empty() {
                            return Vec::new();
                        }

                        let vals = envs
                            .iter()
                            .map(|(keys, env)| {
                                let cache_str = env
                                    .values()
                                    .flat_map(|result| result.into_iter().map(|x| x.to_string()))
                                    // .map(|result| result.to_string())
                                    .collect::<Vec<String>>()
                                    .join("\n");
                                cache_str
                                // // uncomment this to show cache key
                                // format!(
                                //     "<b>(p{}, {:08x}, s{})</b><br>{}",
                                //     keys.0, keys.1, keys.2, cache_str
                                // )
                            })
                            .collect::<Vec<String>>()
                            .join("<br>");

                        let id = format!("cache-{}", key.uml_id());

                        let cache_str = format!("<b>{:?}</b><br>{}", key, vals);
                        let note = MermaidItem::node(&id, cache_str, ItemShape::Card)
                            .add_class("cache-entry")
                            .add_class(BackgroundColor::get_class_name(key.0));
                        let edge = MermaidItem::edge(key.uml_id(), id, "", EdgeType::Dotted);
                        vec![note, edge]
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

    /// draw the path to the data in the cache for a specific scope
    pub fn cache_path_uml(&self, scope_num: usize) -> Vec<PlantUmlItem> {
        self.resolve_cache
            .iter()
            .flat_map(|(_, query_cache)| {
                query_cache
                    .iter()
                    .filter(|(k, _)| k.1 == Scope(scope_num))
                    .flat_map(|(_, envs)| {
                        envs
                        .values()
                        .flat_map(|envs| envs.into_iter().map(|q| &q.path))
                        .flat_map(|path| path.as_uml(ForeGroundColor::next_class(), true))
                    })
            })
            .map(|x| x.add_class("cache-edge"))
            .collect::<Vec<_>>()
    }

    pub fn cache_path_mmd(&self, scope_num: usize) -> Vec<MermaidItem> {
        self.resolve_cache
            .iter()
            .flat_map(|(_, query_cache)| {
                query_cache
                    .iter()
                    .filter(|(k, _)| k.1 == Scope(scope_num))
                    .flat_map(|(_, envs)| {
                        envs
                        .values()
                        .flat_map(|envs| envs.into_iter().map(|q| &q.path))
                        .flat_map(|path| path.as_mmd(ForeGroundColor::next_class(), true))
                    })
            })
            .map(|x| x.add_class("cache-edge"))
            .collect::<Vec<_>>()
    }
}
