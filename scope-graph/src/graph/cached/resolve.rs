use std::{collections::{HashMap, HashSet}, hash::{DefaultHasher, Hash, Hasher}, path::is_separator, sync::Mutex};

use tracing_subscriber::fmt::layer;

use crate::{
    data::ScopeGraphData, graph::BaseScopeGraph, label::{LabelOrEnd, ScopeGraphLabel}, order::{LabelOrder, LabelOrderBuilder}, path::Path, regex::dfs::RegexAutomata, scope::Scope, FORWARD_ENABLE_CACHING
};

use super::{CachedScopeGraph, QueryCache, QueryResult, ScopeData};

// todo: reuse code from Resolver
pub struct CachedResolver<'r, Lbl, Data, DEq, P, DProj>
where
    Lbl: ScopeGraphLabel,
    Data: ScopeGraphData,
    P: std::hash::Hash + Eq,
    DProj: for<'da> Fn(&'da Data) -> P,
    DEq: for<'da, 'db> Fn(&'da Data, &'db Data) -> bool,
    // DWfd: for<'da> Fn(&'da Data) -> bool,

{
    // scopegraph contains cache
    scope_graph: &'r BaseScopeGraph<Lbl, Data>,

    cache: &'r mut QueryCache<Lbl, Data>,

    path_re: &'r RegexAutomata<Lbl>,
    lbl_order: &'r LabelOrder<Lbl>,
    data_eq: DEq,
    // data_wfd: DWfd,
    considered_paths: Mutex<Vec<Path<Lbl>>>,

    data_proj: DProj,
    proj_wfd: P,
}

impl<'r, Lbl, Data, DEq, P, DProj> CachedResolver<'r, Lbl, Data, DEq, P, DProj>
where
    Lbl: ScopeGraphLabel + Clone + std::fmt::Debug + std::fmt::Display + Eq + std::hash::Hash + Ord,
    Data: ScopeGraphData,
    DEq: for<'da, 'db> Fn(&'da Data, &'db Data) -> bool,
    // DWfd: for<'da> Fn(&'da Data) -> bool,
    P: std::hash::Hash + Eq,
    DProj: for<'da> Fn(&'da Data) -> P,
{
    pub fn new(
        scope_graph: &'r BaseScopeGraph<Lbl, Data>,
        cache: &'r mut QueryCache<Lbl, Data>,
        path_re: &'r RegexAutomata<Lbl>,
        lbl_order: &'r LabelOrder<Lbl>,
        data_eq: DEq,
        // data_wfd: DWfd,
        data_proj: DProj,
        proj_wfd: P,
    ) -> CachedResolver<'r, Lbl, Data, DEq, P, DProj> {
        Self {
            scope_graph,
            cache,
            path_re,
            lbl_order,
            data_eq,
            // data_wfd,
            considered_paths: Mutex::new(Vec::new()),
            data_proj,
            proj_wfd,
        }
    }

    pub fn resolve(&mut self, path: Path<Lbl>) -> Vec<QueryResult<Lbl, Data>> {
        // println!("Resolving path: {}", path);
        // self.considered_paths.lock().unwrap().push(path.clone());
        // let envs = self.get_env(path.clone());
        // self.cache_env(&path, envs.clone());
        // envs

        let mut envs = self.resolve_all(path.clone());
        // only keep envs that are well-formed
        envs.retain(|qr| {
            self.data_wfd(&qr.data)
        });
        envs
    }

    /// recursive call site for resolving
    fn resolve_all(&mut self, path: Path<Lbl>) -> Vec<QueryResult<Lbl, Data>> {
        // println!("Resolving path: {}", path);
        self.considered_paths.lock().unwrap().push(path.clone());
        let envs = self.get_env(path.clone());
        self.cache_env(&path, envs.clone());
        envs
    }


    pub fn print_cache(&self) {
        // println!("Resolver cache:");
        // for (k, v) in self.scope_graph.resolve_cache.lock().unwrap().iter() {
        //     println!("{}: [", k);
        //     for qr in &v.envs {
        //         println!("\t{}", qr);
        //     }
        //     println!("]");
        // }
    }

    // todo: allow overload of data_wfd
    fn data_wfd(&self, data: &Data) -> bool {
        (self.data_proj)(data) == self.proj_wfd
    }

    fn cache_key_with_data(&self, path: &Path<Lbl>, data: &Data) -> (usize, u64, Scope) {
        let mut hasher = DefaultHasher::new();
        (self.data_proj)(data).hash(&mut hasher);
        let hash = hasher.finish();
        let scope = path.target();
        let automata_idx = self.path_re.index_of(path.as_lbl_vec()).expect("Path regex not applied previously");
        (automata_idx, hash, scope)
    }

    fn cache_key_with_proj(&self, path: &Path<Lbl>, proj: &P) -> (usize, u64, Scope) {
        let mut hasher = DefaultHasher::new();
        proj.hash(&mut hasher);
        let hash = hasher.finish();
        let scope = path.target();
        let automata_idx = self.path_re.index_of(path.as_lbl_vec()).expect("Path regex not applied previously");
        (automata_idx, hash, scope)
    }

    fn cache_env(&mut self, path: &Path<Lbl>, envs: Vec<QueryResult<Lbl, Data>>) {
        if !FORWARD_ENABLE_CACHING {
            return;
        }

        tracing::debug!("Caching envs...");
        for qr in envs {
            tracing::debug!("Cache env for path {}: {}", path.target(), qr);
            let key = self.cache_key_with_data(path, &qr.data);
            self.cache.entry(key).or_default().push(qr);
        }
    }

    fn get_cached_env(&self, path: &Path<Lbl>) -> Option<Vec<QueryResult<Lbl, Data>>> {
        if !FORWARD_ENABLE_CACHING {
            return None;
        }

        // todo: return full cache of that scope instead of only matched data
        let key = self.cache_key_with_proj(path, &self.proj_wfd);
        let entry = self.cache.get(&key)?;

        Some(entry.to_owned())
    }

    fn get_env(&mut self, path: Path<Lbl>) -> Vec<QueryResult<Lbl, Data>> {
        // all edges where brzozowski derivative != 0
        let scope = self.get_scope(path.target()).expect("Scope not found");

        if let Some(env) = self.get_cached_env(&path) {
            tracing::debug!("Cache hit for {}", path);
            return env;
        }

        let mut labels = scope
            .parents()
            .iter()
            .map(|e| e.lbl())
            // get unique labels by using hashset
            .fold(HashSet::new(), |mut set, lbl| {
                let mut label_vec = path.as_lbl_vec();
                label_vec.push(lbl);
                if self.path_re.partial_match(label_vec) {
                    set.insert(LabelOrEnd::Label(lbl.clone()));
                }
                set
            })
            .into_iter()
            .collect::<Vec<_>>();
        labels.push(LabelOrEnd::End);

        self.get_env_for_labels(&labels, path)
    }

    fn get_env_for_labels(
        &mut self,
        labels: &[LabelOrEnd<Lbl>],
        // edges: &[&Edge<Lbl>],
        path: Path<Lbl>,
    ) -> Vec<QueryResult<Lbl, Data>> {
        let mut results = Vec::new();
        // println!("Resolving edges: {0:?}", labels);

        // 'max' labels ie all labels with lowest priority
        // max refers to the numerical worth, ie a < b, b would be max
        let max = labels
            .iter()
            .filter(|l1| !labels.iter().any(|l2| self.lbl_order.is_less(l1, l2)))
            .collect::<Vec<_>>();

        // println!("max: {0:?}", max);

        for max_lbl in max {
            // all labels that are lower priority than `lbl`
            let lower_labels = labels
                .iter()
                .filter(|l| self.lbl_order.is_less(l, max_lbl))
                .cloned()
                .collect::<Vec<_>>();

            // println!("lower: {0:?}", lower_labels);

            let env = self.get_shadowed_env(max_lbl, &lower_labels, path.clone());
            results.extend(env.into_iter());
        }

        results
    }

    fn get_shadowed_env(
        &mut self,
        max_lbl: &LabelOrEnd<Lbl>,
        lower_lbls: &[LabelOrEnd<Lbl>],
        path: Path<Lbl>,
    ) -> Vec<QueryResult<Lbl, Data>> {
        let lower_paths = self.get_env_for_labels(lower_lbls, path.clone());
        let max_path = self.get_env_for_label(max_lbl, path);
        // println!("lower_paths: {0:?}", lower_paths);
        // println!("max_path: {0:?}", max_path);
        self.shadow(lower_paths, max_path)
    }

    fn get_env_for_label(
        &mut self,
        label: &LabelOrEnd<Lbl>,
        path: Path<Lbl>,
    ) -> Vec<QueryResult<Lbl, Data>> {
        let scope = self.get_scope(path.target()).unwrap().clone();
        match label {
            // reached end of a path
            LabelOrEnd::End => {
                if self.path_re.is_match(path.as_lbl_vec())
                // don't check wfd here
                // && self.data_wfd(&scope.data)
                {
                    return vec![QueryResult {
                        path,
                        data: scope.data.clone(),
                    }];
                }
                vec![]
            }
            // not yet at end
            LabelOrEnd::Label(label) => {
                scope
                    .parents()
                    .iter()
                    .filter(|e| e.lbl() == label)
                    .map(|e| path.clone().step(e.lbl().clone(), e.target())) // create new paths
                    .flat_map(|p| self.resolve_all(p)) // resolve new paths
                    .collect::<Vec<_>>()
            }
        }
    }

    fn shadow(
        &self,
        mut a1: Vec<QueryResult<Lbl, Data>>,
        mut a2: Vec<QueryResult<Lbl, Data>>,
    ) -> Vec<QueryResult<Lbl, Data>> {
        tracing::trace!("Shadowing...");
        a2.retain(|qr2| {
            !a1
                .iter()
                .any(|qr1| (self.data_eq)(&qr1.data, &qr2.data))
        });

        a1.append(&mut a2);
        a1
    }

    fn get_scope(&self, scope: Scope) -> Option<&ScopeData<Lbl, Data>> {
        self.scope_graph.scopes().get(&scope)
    }

    fn scope_data_wfd(&self, s: Scope) -> bool {
        let scope = self.get_scope(s).expect("Scope not found");
        self.data_wfd(&scope.data)
    }
}
