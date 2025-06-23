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
    data::ScopeGraphData, graph::{resolve::Resolver, Edge, ScopeData, ScopeMap}, label::ScopeGraphLabel, order::LabelOrder, path::Path, projection::ScopeGraphDataProjection, regex::dfs::RegexAutomaton, scope::Scope, BackgroundColor, ColorSet, ForeGroundColor
};

use super::{ScopeGraph, resolve::QueryResult};

mod resolve;

type ProjHash = u64;

// type ProjEnvs<Lbl, Data> = HashMap<ProjHash, Vec<QueryResult<Lbl, Data>>>;
type ProjEnvs<Lbl, Data> = HashMap<ProjHash, Vec<QueryResult<Lbl, Data>>>;

/// Key for the cache.
///
/// This is a tuple of index in regex automaton, the result from projecting the data and the target scope.
///
/// You can alternatively think of this as a (usize, DataProj) cache per scope.
type QueryCacheKey = (usize, Scope);

/// Cache for the results of a certain query
type QueryCache<Lbl, Data> = HashMap<QueryCacheKey, ProjEnvs<Lbl, Data>>;

/// Key for `ScopeGraphCache`
/// todo: add DP to key
type ParameterKey<Lbl> = (LabelOrder<Lbl>, RegexAutomaton<Lbl>, ProjHash);
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
    pub scopes: ScopeMap<Lbl, Data>,
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
        data_proj: Proj,
        proj_wfd: Proj::Output,
    ) -> Vec<QueryResult<Lbl, Data>>
    where
        Proj: ScopeGraphDataProjection<Data>,
    {

        let proj_hash = resolve::hash(&data_proj);
        let cache_entry = self
            .resolve_cache
            .entry((order.clone(), path_regex.clone(), proj_hash))
            .or_default();
        let mut resolver = CachedResolver::new(
           &self.scopes,
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
                let params_str = format!("({}, {}, {})", query_params.0, query_params.1, query_params.2 % 256);
                query_cache
                    .iter()
                    .filter(|(key, _)| !self.scope_holds_data(key.1))
                    // map with scope as key to not have duplicate notes
                    .fold(HashMap::new(), |mut acc, (keys, envs)| {
                        let key = keys.1; // scope
                        let entry: &mut QueryCache<Lbl, Data> = acc.entry(key).or_default();
                        entry.insert(*keys, envs.clone());
                        acc
                    })
                    .into_iter()
                    .filter_map(move |(key, envs)| {
                        if envs.is_empty() {
                            return None;
                        }

                        let vals = envs
                            .iter()
                            .map(|(keys, env)| {
                                let cache_str = env
                                    .values()
                                    .flat_map(|result| result.iter().map(|x| x.to_string()))
                                    .collect::<Vec<String>>()
                                    .join("\n");
                                format!("<b>(p{}, s{})</b>\n{}", keys.0, keys.1, cache_str)
                            })
                            .collect::<Vec<String>>()
                            .join("\n");

                        let cache_str = format!("<i>{}</i>\n<b>{:?}</b>\n{}", params_str.clone(), key, vals);
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
                let params_str = format!("({}, {})", query_params.0, query_params.1);
                query_cache
                    .iter()
                    .filter(|(key, _)| !self.scope_holds_data(key.1))
                    // map with scope as key to not have duplicate notes
                    .fold(HashMap::new(), |mut acc, (keys, envs)| {
                        let key = keys.1; // scope
                        let entry: &mut QueryCache<Lbl, Data> = acc.entry(key).or_default();
                        entry.insert(*keys, envs.clone());
                        acc
                    })
                    .into_iter()
                    .flat_map(move |(key, envs)| {
                        if envs.is_empty() {
                            return Vec::new();
                        }

                        let vals = envs
                            .iter()
                            .map(|(keys, env)| {
                                let cache_str = env
                                    .values()
                                    .flat_map(|result| result.iter().map(|x| x.to_string()))
                                    // .map(|result| result.to_string())
                                    .collect::<Vec<String>>()
                                    .join("\n");
                                // cache_str
                                // // uncomment this to show cache key
                                format!("<b>(reg{})</b><br>{}", keys.0, cache_str)
                            })
                            .collect::<Vec<String>>()
                            .join("<br>");

                        let id = format!("cache-{}", key.uml_id());
                        let cache_str = format!("{}<b>{:?}</b><br>{}", params_str, key, vals);
                        let note = MermaidItem::node(&id, cache_str, ItemShape::Card)
                            .add_class("cache-entry")
                            .add_class(BackgroundColor::get_class_name(key.0));
                        let edge = MermaidItem::edge(key.uml_id(), id, "", EdgeType::Dotted);
                        vec![note, edge]
                    })
            })
            .collect()
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
            scopes: ScopeMap::new(),
            resolve_cache: ScopeGraphCache::new(),
        }
    }

    pub fn scopes(&self) -> &ScopeMap<Lbl, Data> {
        &self.scopes
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
                        envs.values()
                            .flat_map(|envs| envs.iter().map(|q| &q.path))
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
                        envs.values()
                            .flat_map(|envs| envs.iter().map(|q| &q.path))
                            .flat_map(|path| path.as_mmd(ForeGroundColor::next_class(), true))
                    })
            })
            .map(|x| x.add_class("cache-edge"))
            .collect::<Vec<_>>()
    }
}
