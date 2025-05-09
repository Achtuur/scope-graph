use std::{
    collections::{HashMap, HashSet},
    hash::{DefaultHasher, Hash, Hasher},
};

use crate::{
    ENABLE_CACHING,
    data::ScopeGraphData,
    graph::BaseScopeGraph,
    label::{LabelOrEnd, ScopeGraphLabel},
    order::LabelOrder,
    path::{Path, ReversePath},
    regex::{PartialRegex, dfs::RegexAutomaton},
    scope::Scope,
};

use super::{ProjHash, QueryCache, QueryResult, ScopeData};

fn hash<T: Hash>(t: &T) -> u64 {
    let mut hasher = DefaultHasher::new();
    t.hash(&mut hasher);
    hasher.finish()
}

type ProjEnvs<Lbl, Data> = HashMap<ProjHash, Vec<QueryResult<Lbl, Data>>>;

// todo: reuse code from Resolver
pub struct CachedResolver<'r, Lbl, Data, P, DProj>
where
    Lbl: ScopeGraphLabel,
    Data: ScopeGraphData,
    P: std::hash::Hash + Eq,
    DProj: for<'da> Fn(&'da Data) -> P,
{
    // scopegraph contains cache
    scope_graph: &'r BaseScopeGraph<Lbl, Data>,

    cache: &'r mut QueryCache<Lbl, Data>,

    path_re: &'r RegexAutomaton<Lbl>,
    lbl_order: &'r LabelOrder<Lbl>,
    /// Data projection function
    data_proj: DProj,
    /// DProj output that results in well-formed data
    ///
    /// `DWfd := |data: &Data| data_proj(data) == proj_wfd`
    proj_wfd: P,
}

impl<'r, Lbl, Data, P, DProj> CachedResolver<'r, Lbl, Data, P, DProj>
where
    Lbl: ScopeGraphLabel + Clone + std::fmt::Debug + std::fmt::Display + Eq + std::hash::Hash + Ord,
    Data: ScopeGraphData,
    P: std::hash::Hash + Eq,
    DProj: for<'da> Fn(&'da Data) -> P,
{
    pub fn new(
        scope_graph: &'r BaseScopeGraph<Lbl, Data>,
        cache: &'r mut QueryCache<Lbl, Data>,
        path_re: &'r RegexAutomaton<Lbl>,
        lbl_order: &'r LabelOrder<Lbl>,
        data_proj: DProj,
        proj_wfd: P,
    ) -> CachedResolver<'r, Lbl, Data, P, DProj> {
        Self {
            scope_graph,
            cache,
            path_re,
            lbl_order,
            data_proj,
            proj_wfd,
        }
    }

    /// Helper function to avoid the ugly field accessor syntax
    fn data_wfd(&self, data: &Data) -> bool {
        (self.data_proj)(data) == self.proj_wfd
    }

    /// Helper function to avoid the ugly field accessor syntax
    fn data_proj(&self, data: &Data) -> P {
        (self.data_proj)(data)
    }

    pub fn resolve(&mut self, path: Path<Lbl>) -> Vec<QueryResult<Lbl, Data>> {
        tracing::info!(
            "Resolving query: {}, {}, {}",
            path,
            self.path_re,
            self.lbl_order
        );
        let reg = PartialRegex::new(self.path_re);
        let mut all_envs = self.resolve_all(path.clone(), reg);
        let h = hash(&self.proj_wfd);
        all_envs.remove(&h).unwrap_or_default()
    }

    /// recursive call site for resolving
    fn resolve_all<'a: 'r>(
        &mut self,
        path: Path<Lbl>,
        reg: PartialRegex<'a, Lbl>,
    ) -> ProjEnvs<Lbl, Data> {
        tracing::trace!("Resolving path: {}", path);
        self.get_env(path, reg)
    }

    fn get_env(
        &mut self,
        path: Path<Lbl>,
        reg: PartialRegex<'r, Lbl>,
    ) -> ProjEnvs<Lbl, Data> {
        // all edges where brzozowski derivative != 0
        let scope = self.get_scope(path.target()).expect("Scope not found");

        tracing::debug!("Checking cache for path {}", path);
        let cached_env = self.get_cached_env(&path, &reg);
        if !cached_env.is_empty() {
            tracing::debug!("Cache hit for {}", path);
            return cached_env;
        }

        let mut labels = scope
        .outgoing()
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

        if reg.is_accepting() {
            labels.push(LabelOrEnd::End);
        }

        let envs = self.get_env_for_labels(&labels, path.clone());
        self.cache_env(&path, reg, &envs);
        envs
    }

    fn get_env_for_labels<'a>(
        &mut self,
        labels: &'a [LabelOrEnd<'r, Lbl>],
        path: Path<Lbl>,
    ) -> ProjEnvs<Lbl, Data> {
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

                self.get_shadowed_env(max_lbl, &lower_labels, path.clone())
            })
            .collect()
    }

    fn get_shadowed_env<'a>(
        &mut self,
        max_lbl: &'a LabelOrEnd<'r, Lbl>,
        lower_lbls: &'a [LabelOrEnd<'r, Lbl>],
        path: Path<Lbl>,
    ) -> ProjEnvs<Lbl, Data> {
        let lower_paths = self.get_env_for_labels(lower_lbls, path.clone());
        let max_path = self.get_env_for_label(max_lbl, path.clone());
        self.shadow(lower_paths, max_path)
    }

    fn get_env_for_label<'a>(
        &mut self,
        label: &'a LabelOrEnd<'r, Lbl>,
        path: Path<Lbl>,
    ) -> ProjEnvs<Lbl, Data> {
        let scope = self.get_scope(path.target()).unwrap().clone();
        match label {
            // reached end of a path
            LabelOrEnd::End => {
                ProjEnvs::from([(
                    hash(&self.data_proj(&scope.data)),
                    vec![QueryResult {
                        path: ReversePath::start(path.target()),
                        data: scope.data.clone(),
                    }]
                )])
            }
            // not yet at end
            LabelOrEnd::Label((label, partial_reg)) => {
                scope
                    .outgoing()
                    .iter()
                    .filter(|e| e.lbl() == label)
                    .map(|e| path.step(e.lbl().clone(), e.target())) // create new paths
                    .flat_map(|p| self.resolve_all(p, partial_reg.clone())) // resolve new paths
                    .map(|(p, mut envs)| {
                        // path is a path from the starting scope to the current one.
                        // in the cache, we want to store the path from the _data_ to the current scope.
                        // hence, every step we add the traversed label to the query result.
                        envs.iter_mut().for_each(|qr| {
                            qr.path = qr.path.step(label.clone(), path.target());
                        });
                        (p, envs)
                    })
                    .collect()
            }
        }
    }

    fn shadow(
        &self,
        mut envs1: ProjEnvs<Lbl, Data>,
        envs2: ProjEnvs<Lbl, Data>,
    ) -> ProjEnvs<Lbl, Data> {
        tracing::trace!("Shadowing...");
        for (proj, e2) in envs2 {
            // env1 shadows env2 always, so if env1 has a P, we know a2 is shadowed
            if envs1.contains_key(&proj) {
                continue;
            }
            envs1.insert(proj, e2);
        }
        envs1
    }

    fn get_scope(&self, scope: Scope) -> Option<&ScopeData<Lbl, Data>> {
        self.scope_graph.scopes().get(&scope)
    }

    fn cache_env(
        &mut self,
        path: &Path<Lbl>,
        reg: PartialRegex<'_, Lbl>,
        env_map: &ProjEnvs<Lbl, Data>,
    ) {
        if !ENABLE_CACHING {
            return;
        }

        tracing::debug!("Caching envs...");
        for (proj, envs) in env_map {
            tracing::trace!("Cache env for path {}: {} envs", path.target(), envs.len());
            let key = (reg.index(), path.target());
            let entry = self.cache.entry(key).or_default();
            // this replaces any existing cache
            // but we will only ever have one entry for the given key (I assume)
            entry.insert(*proj, envs.clone());
        }
    }

    fn get_cached_env(
        &self,
        path: &Path<Lbl>,
        reg: &PartialRegex<'r, Lbl>,
    ) -> ProjEnvs<Lbl, Data> {
        if !ENABLE_CACHING {
            return ProjEnvs::default();
        }

        let key = (reg.index(), path.target());
        self.cache.get(&key).cloned().unwrap_or_default()
    }
}
