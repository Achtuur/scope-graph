use std::{
    hash::{DefaultHasher, Hash, Hasher}, time::Instant,
};

use hashbrown::hash_set::HashSet;
use smallvec::SmallVec;

use crate::{
    data::ScopeGraphData, debugonly_debug, debugonly_trace, graph::{
        resolve::{DisplayMap, DisplayVec, QueryProfiler, QueryStats}, ScopeMap
    }, label::{LabelOrEnd, ScopeGraphLabel}, order::LabelOrder, path::{Path, ReversePath}, projection::ScopeGraphDataProjection, regex::{dfs::RegexAutomaton, RegexState}, scope::Scope, ENABLE_CACHING
};

use super::{ProjEnvs, QueryCache, QueryResult, ScopeData};

#[inline(always)]
pub(super) fn hash<T: Hash>(t: &T) -> u64 {
    let mut hasher = DefaultHasher::new();
    t.hash(&mut hasher);
    hasher.finish()
}

// type ProjEnvs<Lbl, Data> = HashMap<ProjHash, SmallVec<[QueryResult<Lbl, Data>; 16]>>;

// todo: reuse code from Resolver
pub struct CachedResolver<'r, Lbl, Data, Proj>
where
    Lbl: ScopeGraphLabel,
    Data: ScopeGraphData,
    Proj: ScopeGraphDataProjection<Data>,
{
    // scopegraph contains cache
    scope_graph: &'r ScopeMap<Lbl, Data>,

    cache: &'r mut QueryCache<Lbl, Data>,

    path_re: &'r RegexAutomaton<Lbl>,
    lbl_order: &'r LabelOrder<Lbl>,
    /// Data projection function
    data_proj: Proj,
    /// DProj output that results in well-formed data
    ///
    /// `DWfd := |data: &Data| data_proj(data) == proj_wfd`
    proj_wfd: Proj::Output,
    proj_wfd_hash: u64,
    pub profiler: QueryProfiler,
}

impl<'r, Lbl, Data, Proj> CachedResolver<'r, Lbl, Data, Proj>
where
    Lbl: ScopeGraphLabel,
    Data: ScopeGraphData,
    Proj: ScopeGraphDataProjection<Data>,
{
    pub fn new(
        scope_graph: &'r ScopeMap<Lbl, Data>,
        cache: &'r mut QueryCache<Lbl, Data>,
        path_re: &'r RegexAutomaton<Lbl>,
        lbl_order: &'r LabelOrder<Lbl>,
        data_proj: Proj,
        proj_wfd: Proj::Output,
    ) -> CachedResolver<'r, Lbl, Data, Proj> {
        Self {
            scope_graph,
            cache,
            path_re,
            lbl_order,
            data_proj,
            proj_wfd_hash: hash(&proj_wfd),
            proj_wfd,
            profiler: QueryProfiler::new(),
        }
    }

    /// Helper function to avoid the ugly field accessor syntax
    fn data_proj(&self, data: &Data) -> Proj::Output {
        self.data_proj.project(data)
    }

    pub fn resolve(&mut self, path: Path<Lbl>) -> (Vec<QueryResult<Lbl, Data>>, QueryStats) {
        tracing::info!(
            "Resolving query: {}, {}, {}",
            path,
            self.path_re,
            self.lbl_order
        );
        self.profiler.start_time = Instant::now();
        let reg = RegexState::new(self.path_re);
        let mut all_envs = self.resolve_all(path.clone(), reg);
        let envs = all_envs.remove(&self.proj_wfd_hash).unwrap_or_default().to_vec();
        (envs, (&self.profiler).into())
    }

    /// recursive call site for resolving
    fn resolve_all<'a: 'r>(
        &mut self,
        path: Path<Lbl>,
        reg: RegexState<'a, Lbl>,
    ) -> ProjEnvs<Lbl, Data> {
        debugonly_trace!("Resolving path: {}", path);
        self.get_env(path, reg)
    }

    fn get_env(&mut self, path: Path<Lbl>, reg: RegexState<'r, Lbl>) -> ProjEnvs<Lbl, Data> {
        // all edges where brzozowski derivative != 0
        let scope = self.get_scope(path.target()).expect("Scope not found");
        self.profiler.inc_nodes_visited();

        debugonly_debug!("Checking cache for path {}", path);
        // let timer = std::time::Instant::now();
        let cached_env = self.get_cached_env(&path, &reg);
        // println!("cache read: {:?}", timer.elapsed());
        if !cached_env.is_empty() {
            debugonly_debug!("Cache hit for {}", path);
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
        // let timer = std::time::Instant::now();
        self.cache_env(&path, reg, &envs);
        // println!("cache write: {:?}", timer.elapsed());
        envs
    }

    fn get_env_for_labels<'a>(
        &mut self,
        labels: &'a [LabelOrEnd<'r, Lbl>],
        path: Path<Lbl>,
    ) -> ProjEnvs<Lbl, Data> {
        if labels.is_empty() {
            return ProjEnvs::default();
        }
        debugonly_trace!("Resolving labels: {} for {}", DisplayVec(labels), path);
        labels
            .iter()
            .filter(|l1| !labels.iter().any(|l2| self.lbl_order.is_less(l1, l2)))
            // 'max' labels ie all labels with lowest priority
            // max refers to the numerical worth, ie a < b, b would be max
            .map(|max_lbl| {
                // all labels that are lower priority than `lbl`
                let lower_labels = labels
                    .iter()
                    .filter(|l| self.lbl_order.is_less(l, max_lbl))
                    .cloned()
                    .collect::<SmallVec<[_; 8]>>();

                debugonly_trace!("Resolving envs {} < {}", max_lbl, DisplayVec(&lower_labels));
                self.get_shadowed_env(max_lbl, &lower_labels, path.clone())
            })
            .flatten()
            .collect()
            // .fold(ProjEnvs::default(), |mut acc, envs| {
            //     // merge all envs into one
            //     for (proj, mut new_envs) in envs {
            //         let e = acc.entry(proj).or_default();
            //         e.append(&mut new_envs);
            //     }
            //     acc
            // })
    }

    fn get_shadowed_env<'a>(
        &mut self,
        max_lbl: &'a LabelOrEnd<'r, Lbl>,
        lower_lbls: &'a [LabelOrEnd<'r, Lbl>],
        path: Path<Lbl>,
    ) -> ProjEnvs<Lbl, Data> {
        let lower_paths = self.get_env_for_labels(lower_lbls, path.clone());
        let max_path = self.get_env_for_label(max_lbl, path);
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
            LabelOrEnd::End => ProjEnvs::from([(
                hash(&self.data_proj(&scope.data)),
                vec![QueryResult {
                    path: ReversePath::start(path.target()),
                    data: scope.data.clone(),
                }],
            )]),
            // not yet at end
            LabelOrEnd::Label((label, partial_reg)) => {
                scope
                    .outgoing()
                    .iter()
                    .filter(|e| e.lbl() == label)
                    .map(|e| path.step(e.lbl().clone(), e.target(), partial_reg.index())) // create new paths
                    .filter(|p| !p.is_circular(partial_reg.index())) // create new paths
                    .flat_map(|p| {
                        self.profiler.inc_edges_traversed();
                        self.resolve_all(p, partial_reg.clone())
                    }) // resolve new paths
                    .fold(ProjEnvs::default(), |mut acc, (p, mut envs)| {
                        // path is a path from the starting scope to the current one.
                        // in the cache, we want to store the path from the _data_ to the current scope.
                        // hence, every step we add the traversed label to the query result.
                        for qr in &mut envs {
                            qr.path = qr
                            .path
                            .step(
                                label.clone(),
                                path.target(),
                                partial_reg.index()
                            );
                        }

                        // we have to fold here, since there can be multiple entries for each projection (`p`)
                        let e = acc.entry(p).or_default();
                        e.append(&mut envs);
                        acc
                    })
            }
        }
    }

    #[allow(clippy::map_entry)] // makes code more reabable imo
    fn shadow(
        &self,
        mut envs1: ProjEnvs<Lbl, Data>,
        envs2: ProjEnvs<Lbl, Data>,
    ) -> ProjEnvs<Lbl, Data> {
        // debugonly_trace!("Shadowing {} < {}", DisplayMap(&envs1), DisplayMap(&envs2));
        for (proj, e2) in envs2 {
            // env1 shadows env2 always, so if env1 has a P, we know a2 is shadowed
            if !envs1.contains_key(&proj) {
                // we checked whether envs1 contains proj
                unsafe { envs1.insert_unique_unchecked(proj, e2); }
            }
        }
        envs1
    }

    fn get_scope(&self, scope: Scope) -> Option<&ScopeData<Lbl, Data>> {
        self.scope_graph.get(&scope)
    }

    fn cache_env(
        &mut self,
        path: &Path<Lbl>,
        reg: RegexState<'_, Lbl>,
        env_map: &ProjEnvs<Lbl, Data>,
    ) {
        if !ENABLE_CACHING {
            return;
        }

        debugonly_debug!("Caching envs...");
        for (proj, envs) in env_map {
            // debugonly_trace!(
            //     "Cache env for path {}: {} envs",
            //     path.target(),
            //     DisplayVec(envs)
            // );
            let key = (reg.index(), path.target());
            // this is the entry for the path
            // its a map of proj -> [envs]
            let entry = self.cache.entry(key).or_default();
            // this replaces any existing cache
            // but we will only ever have one entry for the given key (I assume)
            self.profiler.inc_cache_writes();
            // let cached_envs = entry.entry(*proj).or_default();
            // for e in envs {
            //     if !cached_envs.contains(e) {
            //         debugonly_trace!("Adding env to cache: {e}");
            //         cached_envs.push(e.clone());
            //     }
            // }

            entry.insert(*proj, envs.clone());
        }
    }

    fn get_cached_env(&self, path: &Path<Lbl>, reg: &RegexState<'r, Lbl>) -> ProjEnvs<Lbl, Data> {
        if !ENABLE_CACHING {
            return ProjEnvs::default();
        }
        self.profiler.inc_cache_reads();

        let key = (reg.index(), path.target());
        // let proj_wfd_hash = hash(&self.proj_wfd);
        match self.cache.get(&key) {
            // this makes it about 2x slower??
            // Some(entry) if entry.contains_key(&self.proj_wfd_hash) => {
            //     self.profiler.inc_cache_hits();
            //     entry.clone()
            // }
            Some(entry) => {
                self.profiler.inc_cache_hits();
                entry.clone()
            }
            _ => ProjEnvs::default(),
        }
    }
}
