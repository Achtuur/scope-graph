mod reduced_path;

use std::{cell::RefCell, collections::HashMap, marker::PhantomData};

use plantuml::PlantUmlItem;
use reduced_path::ReducedPath;

use crate::{
    data::{self, ScopeGraphData},
    graph::{BaseScopeGraph, ScopeMap},
    label::ScopeGraphLabel,
    order::LabelOrder,
    path::Path,
    regex::dfs::RegexAutomata,
    resolve::QueryResult,
    scope::Scope,
};

use super::{Edge, ScopeData, ScopeGraph};

/// Cache for bottom-up resolution
///
/// Every scope holds a map of Data -> Path (to the data)
///
/// This completely caches every declaration, meaning that the
/// query resolution does not have to traverse the graph at all.
/// Every scope has complete information on all data visible data.
type BottomupCache2<Lbl, Data> = HashMap<Scope, CacheValue<Lbl, Data>>;

type CacheValue<Lbl, Data> = RefCell<Vec<(CachedData<Data>, ReducedPath<Lbl>)>>;

#[derive(Debug, Clone, PartialEq, Eq)]
struct CachedData<Data: ScopeGraphData> {
    pub data: Data,
    pub decl_scope: Scope,
}

impl<Data: ScopeGraphData> std::fmt::Display for CachedData<Data> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}) {}", self.decl_scope, self.data)
    }
}

impl<Data: ScopeGraphData> CachedData<Data> {
    pub fn new(data: Data, decl_scope: Scope) -> Self {
        Self { data, decl_scope }
    }
}

// full caching
pub struct BottomupScopeGraph2<Lbl, Data>
where
    Lbl: ScopeGraphLabel,
    Data: ScopeGraphData,
{
    sg: BaseScopeGraph<Lbl, Data>,
    data_cache: BottomupCache2<Lbl, Data>,
}

impl<'s, Lbl, Data> ScopeGraph<'s, Lbl, Data> for BottomupScopeGraph2<Lbl, Data>
where
    Lbl: ScopeGraphLabel,
    Data: ScopeGraphData,
{
    fn add_scope(&mut self, scope: Scope, data: Data) {
        self.sg.add_scope(scope, data);
        self.data_cache.insert(scope, RefCell::new(Vec::new()));
    }

    fn add_edge(&mut self, source: Scope, target: Scope, label: Lbl) {
        self.sg.add_edge(source, target, label);
        self.propogate_cache(target);
    }

    fn add_decl(&mut self, source: Scope, label: Lbl, data: Data) -> Scope {
        let data_scope = self.sg.add_decl(source, label.clone(), data.clone());
        self.insert_cache(source, data, label, data_scope);
        self.propogate_cache(source);
        data_scope
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
        tracing::debug!("Querying from scope {}", scope);
        // self.print_cache();
        let cache_entry = self
            .data_cache
            .get(&scope)
            .expect("Scope not found in cache");

        // all matching data and path regex
        let query_results = cache_entry
            .borrow()
            .iter()
            .filter(|(d, _)| data_wellformedness(&d.data))
            .inspect(|_| tracing::trace!("data is wellformed"))
            .filter(|(_, p)| path_regex.is_match(p.as_lbl_vec()))
            .inspect(|_| tracing::trace!("path is match"))
            .map(|(d, p)| QueryResult {
                path: Path::start(scope).step(p.peek().unwrap().clone(), d.decl_scope),
                data: d.data.clone(),
            })
            .collect::<Vec<_>>();

        tracing::trace!("query_results: {0:?}", query_results);

        // an environment is shadowed if another env exists that
        // - has equivalent data
        // - path is less

        let shadows = |qr1: &QueryResult<Lbl, Data>, qr2: &QueryResult<Lbl, Data>| {
            qr1 != qr2
                && data_equiv(&qr1.data, &qr2.data)
                && order.path_is_less(&qr1.path, &qr2.path)
        };

        // shadowing
        query_results
            .iter()
            .filter(|qr| !query_results.iter().any(|qr2| shadows(qr2, qr)))
            .cloned()
            .collect::<Vec<_>>()
    }

    fn scope_iter<'a>(
        &'a self,
    ) -> impl Iterator<Item = (&'a Scope, &'a super::ScopeData<Lbl, Data>)>
    where
        Lbl: 'a,
        Data: 'a,
    {
        self.sg.scope_iter()
    }

    fn scope_holds_data(&self, scope: Scope) -> bool {
        self.sg.scope_holds_data(scope)
    }

    fn generate_cache_uml<'a>(&'a self) -> Vec<PlantUmlItem>
    where
        Lbl: 'a,
        Data: 'a,
    {
        self.data_cache
            .iter()
            .filter_map(|(scope, cache)| {
                let cache = cache.borrow();
                if cache.is_empty() {
                    return None;
                }

                let cache_str = cache
                    .iter()
                    .map(|(d, p)| format!("<b>{}</b>: {}", d, p))
                    .collect::<Vec<String>>()
                    .join("\n");

                Some(PlantUmlItem::note(scope.0, cache_str))
            })
            .collect::<Vec<_>>()
    }
}

impl<'s, Lbl, Data> BottomupScopeGraph2<Lbl, Data>
where
    Lbl: ScopeGraphLabel,
    Data: ScopeGraphData,
{
    pub fn new() -> Self {
        Self {
            sg: BaseScopeGraph::new(),
            data_cache: HashMap::new(),
        }
    }

    pub fn base(&self) -> &BaseScopeGraph<Lbl, Data> {
        &self.sg
    }

    pub fn scopes(&self) -> &ScopeMap<Lbl, Data> {
        &self.sg.scopes
    }

    pub fn print_cache(&self) {
        println!("{:?}", self.data_cache);
    }

    /// Inserts new cache entry from a declaration
    ///
    /// # Arguments
    ///
    /// * `scope` - the scope that has the declaration scope as a child
    /// * `data` - the data that was declared
    /// * `label` - the label to the declaration scope
    /// * `decl_scope` - the scope that holds the data of the declaration
    pub fn insert_cache(&mut self, scope: Scope, data: Data, label: Lbl, decl_scope: Scope) {
        tracing::trace!("Inserting cache for {}: {}", scope, data);
        let cache_entry = self.data_cache.entry(scope).or_default();
        cache_entry
            .borrow_mut()
            .push((CachedData::new(data, decl_scope), ReducedPath::start(label)));
    }

    pub fn propogate_cache(&mut self, source: Scope) {
        tracing::trace!("Propogating cache from {}", source);
        // pass own cache to each child
        // in each child, only add new entry if it does not exist
        let Some(source_cache) = self.data_cache.get(&source) else {
            // nothing to propogate
            tracing::trace!("Nothing to propogate from {}", source);
            return;
        };

        let sd: &ScopeData<Lbl, Data> = self
            .sg
            .scopes
            .get(&source)
            .expect("Attempted to take edges of non-existant scope");
        for child_edge in sd.children() {
            if source == child_edge.target() {
                panic!("Cyclical dependency in scope graph");
            }
            let mut child_cache = self
                .data_cache
                .get(&child_edge.target())
                .expect("Child scope not found in cache")
                .borrow_mut();

            // each item in source cache with label added on that does not exist in child cache should be inserted
            for (data, path) in source_cache.borrow().iter() {
                tracing::trace!("Propogating {} to {}", data, child_edge.target());

                let already_exists = child_cache
                    .iter()
                    .filter(|(d, _)| d == data)
                    .any(|(_, p)| p.is_same_path_with_start(child_edge.lbl(), path));
                if already_exists {
                    continue;
                }
                // add new cache entry
                // todo: use rc or something to reuse path if path is identical after pushing
                let mut new_path = path.clone();
                new_path.push(child_edge.lbl().clone());
                child_cache.push((data.clone(), new_path));
            }
        }

        // recursively propogate cache to children
        let child_scopes = sd.children().iter().map(|s| s.target()).collect::<Vec<_>>();
        for child_scope in child_scopes {
            self.propogate_cache(child_scope);
        }
    }
}
