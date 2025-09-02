
use std::cell::RefCell;

use graphing::{
    mermaid::{
        item::{ItemShape, MermaidItem},
        theme::EdgeType,
    },
    plantuml::{EdgeDirection, PlantUmlItem},
};
use resolve::CachedResolver;
use serde::{Deserialize, Serialize};
use deepsize::DeepSizeOf;

use crate::{
    data::ScopeGraphData, debug_tracing, graph::{circle::CircleMatcher, resolve::{QueryStats, Resolver}, Edge, ScopeData, ScopeMap}, label::ScopeGraphLabel, order::LabelOrder, path::Path, projection::ScopeGraphDataProjection, regex::dfs::RegexAutomaton, scope::Scope, BackgroundColor, ColorSet, ForeGroundColor
};

use super::{ScopeGraph, resolve::QueryResult};

mod resolve;
mod cache;

pub(crate) use cache::*;

// type ProjHash = u64;

// /// Map of projected data -> environment
// type ProjEnvs<Lbl, Data> = hashbrown::HashMap<ProjHash, Vec<QueryResult<Lbl, Data>>>;

// /// Key for the cache.
// ///
// /// This is a tuple of index in regex automaton, the result from projecting the data and the target scope.
// ///
// /// You can alternatively think of this as a (usize, DataProj) cache per scope.
// type QueryCacheKey = (usize, Scope);

// /// Cache for the results of a certain query
// type QueryCache<Lbl, Data> = hashbrown::HashMap<QueryCacheKey, ProjEnvs<Lbl, Data>>;

// /// Key for `ScopeGraphCache` (label order, label regex, projection FUNCTION hash).
// type ParameterKey<Lbl> = (LabelOrder<Lbl>, RegexAutomaton<Lbl>, ProjHash);
// /// Cache for the entire scope graph.
// ///
// /// This contains a cache per set of query parameters
// type ScopeGraphCache<Lbl, Data> = hashbrown::HashMap<ParameterKey<Lbl>, QueryCache<Lbl, Data>>;


// type StdProjEnvs<Lbl, Data> = std::collections::HashMap<ProjHash, Vec<QueryResult<Lbl, Data>>>;
// type StdQueryCache<Lbl, Data> = std::collections::HashMap<QueryCacheKey, StdProjEnvs<Lbl, Data>>;
// type StdCache<Lbl, Data> = std::collections::HashMap<ParameterKey<Lbl>, StdQueryCache<Lbl, Data>>;

#[derive(Debug, Serialize, Deserialize)]
pub struct CachedScopeGraph<Lbl, Data>
where
    Lbl: ScopeGraphLabel,
    Data: ScopeGraphData,
{
    pub scopes: ScopeMap<Lbl, Data>,
    #[serde(skip)]
    resolve_cache: ResolveCache<Lbl, Data>,
    #[serde(skip)]
    scopes_in_cycle: RefCell<hashbrown::HashSet<Scope>>,
    #[serde(skip)]
    cycle_scope_cache: RefCell<hashbrown::HashMap<Scope, bool>>,
}

impl<Lbl, Data> CachedScopeGraph<Lbl, Data>
where
    Lbl: ScopeGraphLabel,
    Data: ScopeGraphData,
{

    /// Returns number of scopes
    pub fn size(&self) -> usize {
        self.scopes.len()
    }

    pub fn query_stats<DEq, DWfd>(
        &mut self,
        scope: Scope,
        path_regex: &RegexAutomaton<Lbl>,
        order: &LabelOrder<Lbl>,
        data_equiv: DEq,
        data_wellformedness: DWfd,
    ) -> (Vec<QueryResult<Lbl, Data>>, QueryStats)
    where
        DEq: for<'da, 'db> Fn(&'da Data, &'db Data) -> bool,
        DWfd: for<'da> Fn(&'da Data) -> bool,
    {
        let mut resolver = Resolver::new(
            &self.scopes,
            path_regex,
            order,
            &data_equiv,
            &data_wellformedness,
        );
        resolver.resolve(Path::start(scope))
    }

    pub fn query_proj_stats<Proj>(
        &mut self,
        scope: Scope,
        path_regex: &RegexAutomaton<Lbl>,
        order: &LabelOrder<Lbl>,
        data_proj: Proj,
        proj_wfd: Proj::Output,
        caching_enabled: bool,
    ) -> (Vec<QueryResult<Lbl, Data>>, QueryStats)
    where
        Proj: ScopeGraphDataProjection<Data>,
    {
        let proj_hash = resolve::hash(&data_proj);
        let cache_entry = self
            .resolve_cache
            .get_mut((order.clone(), path_regex.clone(), proj_hash));
        let mut resolver = CachedResolver::new(
            &self.scopes,
            cache_entry,
            self.scopes_in_cycle.borrow(),
            path_regex,
            order,
            data_proj,
            proj_wfd,
            caching_enabled,
        );
        let (envs, mut stats) = resolver.resolve(Path::start(scope));

        // let std_cache: StdCache<Lbl, Data> = self.resolve_cache
        // .iter()
        // .fold(std::collections::HashMap::new(), |mut acc, (key, cache)| {
        //     let entry = acc.entry(key.clone()).or_default();
        //     cache.into_iter().for_each(|(k, v)| {
        //         let entry2= entry.entry(*k).or_default();
        //         entry2.extend(v.iter().map(|(k, v)| (*k, v.clone())));
        //     });
        //     acc
        // });

        // stats.cache_size_estimate = std_cache.deep_size_of() as f32 / self.scopes.deep_size_of() as f32;

        // for qr in &envs {
        //     tracing::info!("\t{}", qr);
        // }
        (envs, stats)
    }

    pub(crate) fn map(&self) -> &ScopeMap<Lbl, Data> {
        &self.scopes
    }
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
        debug_tracing!(trace, "Adding scope: {} with data: {}", scope, data);
        self.scopes.insert(scope, ScopeData::new(data));
        scope
    }

    fn add_edge(&mut self, source: Scope, target: Scope, label: Lbl) {
        tracing::debug!(
            "Adding edge: {} -> {} with label: {}",
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

        self.determine_cycles(source);
        self.determine_cycles(target);
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

    fn extend(&mut self, other: Self) {
        self.scopes.extend(other.scopes);
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
        let mut resolver = Resolver::new(
            &self.scopes,
            path_regex,
            order,
            &data_equiv,
            &data_wellformedness,
        );
        resolver.resolve(Path::start(scope)).0
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
            .get_mut((order.clone(), path_regex.clone(), proj_hash));
        let mut resolver = CachedResolver::new(
            &self.scopes,
            cache_entry,
            self.scopes_in_cycle.borrow(),
            path_regex,
            order,
            data_proj,
            proj_wfd,
            true,
        );
        let envs = resolver.resolve(Path::start(scope)).0;
        tracing::info!("{:?}", resolver.profiler);
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
        self.resolve_cache.generate_uml(self).collect()
    }

    fn generate_cache_mmd(&self) -> Vec<MermaidItem> {
        todo!()
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
            resolve_cache: ResolveCache::new(),
            scopes_in_cycle:RefCell::new(hashbrown::HashSet::new()),
            cycle_scope_cache: RefCell::new(hashbrown::HashMap::new()),
        }
    }

    pub fn scope_is_in_cycle(&self, scope: Scope) -> bool {
        if let Some(b) = self.cycle_scope_cache.borrow().get(&scope) {
            return *b;
        }

        // add scopes
        let scopes = CircleMatcher::scopes_in_cycle(self.scopes(), scope);
        let contains = scopes.contains(&scope);

        self.cycle_scope_cache.borrow_mut().extend(scopes.into_iter().map(|s| (s, true)));
        self.cycle_scope_cache.borrow_mut().insert(scope, contains);
        contains
    }

    fn determine_cycles(&mut self, scope: Scope) {
        let scopes = CircleMatcher::scopes_in_cycle(self.scopes(), scope);
        self.scopes_in_cycle.borrow_mut().extend(scopes);
    }

    pub fn scopes(&self) -> &ScopeMap<Lbl, Data> {
        &self.scopes
    }

    /// draw the path to the data in the cache for a specific scope
    pub fn cache_path_uml(&self, scope_num: usize) -> Vec<PlantUmlItem> {
        todo!()
        // self.resolve_cache
        //     .cache.iter()
        //     .flat_map(|(_, query_cache)| {
        //         query_cache
        //             .cache.iter()
        //             .filter(|(k, _)| k.1 == Scope(scope_num))
        //             .flat_map(|(_, envs)| {
        //                 envs.values()
        //                     .flat_map(|envs| envs.iter().map(|q| &q.path))
        //                     .flat_map(|path| path.as_uml(ForeGroundColor::next_class(), true))
        //             })
        //     })
        //     .map(|x| x.add_class("cache-edge"))
        //     .collect::<Vec<_>>()
    }

    pub fn cache_path_mmd(&self, scope_num: usize) -> Vec<MermaidItem> {
        todo!()
        // self.resolve_cache
        //     .cache.iter()
        //     .flat_map(|(_, query_cache)| {
        //         query_cache
        //             .cache.iter()
        //             .filter(|(k, _)| k.1 == Scope(scope_num))
        //             .flat_map(|(_, envs)| {
        //                 envs.values()
        //                     .flat_map(|envs| envs.iter().map(|q| &q.path))
        //                     .flat_map(|path| path.as_mmd(ForeGroundColor::next_class(), true))
        //             })
        //     })
        //     .map(|x| x.add_class("cache-edge"))
        //     .collect::<Vec<_>>()
    }
}
