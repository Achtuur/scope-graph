use std::{
    collections::HashSet,
    hash::{DefaultHasher, Hash, Hasher},
    sync::Mutex,
};

use crate::{
    data::ScopeGraphData,
    graph::BaseScopeGraph,
    label::{LabelOrEnd, ScopeGraphLabel},
    order::LabelOrder,
    path::Path,
    regex::{dfs::RegexAutomata, PartialRegex},
    scope::Scope,
    FORWARD_ENABLE_CACHING,
};

use super::{QueryCache, QueryResult, ScopeData};

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
    /// Data projection function
    data_proj: DProj,
    /// DProj output that results in well-formed data
    ///
    /// ie DWfd := |data: &Data| data_proj(data) == proj_wfd
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
            data_proj,
            proj_wfd,
        }
    }

    pub fn resolve(&mut self, path: Path<Lbl>) -> Vec<QueryResult<Lbl, Data>> {
        tracing::info!(
            "Resolving query: {}, {}, {}",
            path,
            self.path_re,
            self.lbl_order
        );
        let reg = PartialRegex::new(self.path_re);
        let mut envs = self.resolve_all(path.clone(), reg);
        // only keep envs that are well-formed
        envs.retain(|qr| self.data_wfd(&qr.data));
        tracing::info!(
            "Resolved query: {}, {}, {}, found:",
            path,
            self.path_re,
            self.lbl_order
        );
        for qr in &envs {
            tracing::info!("\t{}", qr);
        }
        envs
    }

    /// recursive call site for resolving
    fn resolve_all<'a: 'r>(
        &mut self,
        path: Path<Lbl>,
        reg: PartialRegex<'a, Lbl>,
    ) -> Vec<QueryResult<Lbl, Data>> {
        tracing::trace!("Resolving path: {}", path);
        self.get_env(path, reg)
    }

    fn get_env(
        &mut self,
        path: Path<Lbl>,
        reg: PartialRegex<'r, Lbl>,
    ) -> Vec<QueryResult<Lbl, Data>> {
        // all edges where brzozowski derivative != 0
        let scope = self.get_scope(path.target()).expect("Scope not found");

        tracing::debug!("Checking cache for path {}", path);
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
                let mut this_reg = reg.clone();
                if this_reg.step(lbl).is_some() {
                    set.insert(LabelOrEnd::Label((lbl.clone(), this_reg)));
                }
                set
            })
            .into_iter()
            .collect::<Vec<_>>();
        labels.push(LabelOrEnd::End(reg.clone()));

        let envs = self.get_env_for_labels(&labels, path.clone());
        // don't cache on the scope that holds the data itself as that is useless
        self.cache_env(&path, reg, envs.clone());
        envs
    }

    fn get_env_for_labels<'a>(
        &mut self,
        labels: &'a [LabelOrEnd<'r, Lbl>],
        path: Path<Lbl>,
    ) -> Vec<QueryResult<Lbl, Data>> {
        labels
            .iter()
            .filter(|l1| !labels.iter().any(|l2| self.lbl_order.is_less(l1, l2)))
            // 'max' labels ie all labels with lowest priority
            // max refers to the numerical worth, ie a < b, b would be max
            .flat_map(|max_lbl| {
                // all labels that are lower priority than `lbl`
                let lower_labels = labels
                    .iter()
                    .filter(|l| self.lbl_order.is_less(l, max_lbl))
                    .cloned()
                    .collect::<Vec<_>>();

                self.get_shadowed_env(max_lbl, &lower_labels, &path)
            })
            .collect::<Vec<_>>()
    }

    fn get_shadowed_env<'a>(
        &mut self,
        max_lbl: &'a LabelOrEnd<'r, Lbl>,
        lower_lbls: &'a [LabelOrEnd<'r, Lbl>],
        path: &Path<Lbl>,
    ) -> Vec<QueryResult<Lbl, Data>> {
        let lower_paths = self.get_env_for_labels(lower_lbls, path.clone());
        let max_path = self.get_env_for_label(max_lbl, path);
        // println!("lower_paths: {0:?}", lower_paths);
        // println!("max_path: {0:?}", max_path);
        self.shadow(lower_paths, max_path)
    }

    fn get_env_for_label<'a>(
        &mut self,
        label: &'a LabelOrEnd<'r, Lbl>,
        path: &Path<Lbl>,
    ) -> Vec<QueryResult<Lbl, Data>> {
        let scope = self.get_scope(path.target()).unwrap().clone();
        match label {
            // reached end of a path
            LabelOrEnd::End(reg) => {
                // don't check wfd here
                match reg.is_accepting() {
                    true => vec![QueryResult {
                        path: path.clone(),
                        data: scope.data.clone(),
                    }],
                    false => Vec::new(),
                }
            }
            // not yet at end
            LabelOrEnd::Label((label, partial_reg)) => {
                scope
                    .parents()
                    .iter()
                    .filter(|e| e.lbl() == label)
                    .map(|e| path.clone().step(e.lbl().clone(), e.target())) // create new paths
                    .flat_map(|p| self.resolve_all(p, partial_reg.clone())) // resolve new paths
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
            !a1.iter()
                .inspect(|qr1| {
                    let res = (self.data_eq)(&qr1.data, &qr2.data);
                    tracing::trace!("Comparing {} with {}, shadowed?{}", qr1, qr2, res);
                })
                .any(|qr1| (self.data_eq)(&qr1.data, &qr2.data))
        });

        a1.append(&mut a2);
        a1
    }

    fn get_scope(&self, scope: Scope) -> Option<&ScopeData<Lbl, Data>> {
        self.scope_graph.scopes().get(&scope)
    }

    
    // todo: allow overload of data_wfd
    fn data_wfd(&self, data: &Data) -> bool {
        (self.data_proj)(data) == self.proj_wfd
    }

    fn cache_key_with_data(
        &self,
        path: &Path<Lbl>,
        reg: &PartialRegex<'_, Lbl>,
        data: &Data,
    ) -> (usize, u64, Scope) {
        let mut hasher = DefaultHasher::new();
        (self.data_proj)(data).hash(&mut hasher);
        let hash = hasher.finish();
        let scope = path.target();
        let automata_idx = reg.index();
        (automata_idx, hash, scope)
    }

    fn cache_env(
        &mut self,
        path: &Path<Lbl>,
        reg: PartialRegex<'_, Lbl>,
        envs: Vec<QueryResult<Lbl, Data>>,
    ) {
        if !FORWARD_ENABLE_CACHING {
            return;
        }

        tracing::debug!("Caching envs...");
        for mut qr in envs {
            tracing::debug!("Cache env for path {}: {}", path.target(), qr);
            // todo: consider shadowing here?
            let key = self.cache_key_with_data(path, &reg, &qr.data);
            let entry = self.cache.entry(key).or_default();
            // todo: use different representation for path
            // with arcs and starting from the target
            qr.path = qr.path.trim_matching_start(path);
            // todo: remove this linear search by not adding duplicates to cache in the first place
            if !entry.contains(&qr) {
                entry.push(qr);
            }
        }
    }

    fn get_cached_env(&self, path: &Path<Lbl>) -> Option<Vec<QueryResult<Lbl, Data>>> {
        if !FORWARD_ENABLE_CACHING {
            return None;
        }

        let envs = self
            .cache
            .iter()
            .filter(|(k, _)| k.2 == path.target()) // get all envs for the scope
            .flat_map(|(_, v)| {
                v.iter()
                    .cloned() // todo: remove or soften this clone
                    .map(|mut qr| {
                        // prepend 'path' to qr so we connect the path to the env in the cache entry
                        // ie if path is 1 -> 2 -> 3 and qr is 3 -> 4, we want to return 1 -> 2 -> 3 -> 4
                        tracing::trace!("Prepending path {} to cached env {}", path, qr);
                        qr.path = qr.path.prepend(path);
                        qr
                    })
            })
            .collect::<Vec<_>>();

        match envs.len() {
            0 => None,
            _ => Some(envs),
        }
    }
}
