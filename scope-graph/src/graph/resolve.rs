use std::{collections::HashSet, sync::atomic::AtomicUsize, time::{Duration, Instant}};

use crate::{
    data::ScopeGraphData, debugonly_debug, debugonly_trace, graph::ScopeMap, label::{LabelOrEnd, ScopeGraphLabel}, order::LabelOrder, path::{Path, ReversePath}, regex::{dfs::RegexAutomaton, RegexState}, scope::Scope, DRAW_MEM_ADDR
};

use super::ScopeData;

#[derive(Debug)]
pub(crate) struct QueryProfiler {
    pub start_time: Instant,
    pub edges_traversed: AtomicUsize,
    pub nodes_visited: AtomicUsize,
    pub cache_reads: AtomicUsize,
    pub cache_writes: AtomicUsize,
    pub cache_hits: AtomicUsize,
    /// size estimate in bytes
    /// assuming that hashmap is simply a list of [(K, V)] for simplicity
    pub cache_size_estimate: AtomicUsize,
}

impl QueryProfiler {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            edges_traversed: AtomicUsize::new(0),
            nodes_visited: AtomicUsize::new(0),
            cache_reads: AtomicUsize::new(0),
            cache_writes: AtomicUsize::new(0),
            cache_hits: AtomicUsize::new(0),
            cache_size_estimate: AtomicUsize::new(0),
        }
    }
}

impl QueryProfiler {
    #[inline(always)]
    pub fn inc_edges_traversed(&self) {
        self.edges_traversed
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    #[inline(always)]
    pub fn inc_nodes_visited(&self) {
        self.nodes_visited
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    #[inline(always)]
    pub fn inc_cache_reads(&self) {
        self.cache_reads
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    #[inline(always)]
    pub fn inc_cache_writes(&self) {
        self.cache_writes
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    #[inline(always)]
    pub fn inc_cache_hits(&self) {
        self.cache_hits
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }
}

#[derive(Debug, Default, serde::Serialize)]
pub struct QueryStats {
    pub time: Duration,
    pub edges_traversed: usize,
    pub nodes_visited: usize,
    pub cache_reads: usize,
    pub cache_writes: usize,
    pub cache_hits: usize,
    /// size estimate in bytes
    /// assuming that hashmap is simply a list of [(K, V)] for simplicity
    pub cache_size_estimate: usize,
}

impl std::ops::Add for QueryStats {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            time: self.time + other.time,
            edges_traversed: self.edges_traversed + other.edges_traversed,
            nodes_visited: self.nodes_visited + other.nodes_visited,
            cache_reads: self.cache_reads + other.cache_reads,
            cache_writes: self.cache_writes + other.cache_writes,
            cache_hits: self.cache_hits + other.cache_hits,
            cache_size_estimate: self.cache_size_estimate + other.cache_size_estimate,
        }
    }
}

impl std::ops::Div<usize> for QueryStats {
    type Output = Self;

    fn div(self, rhs: usize) -> Self {
        Self {
            time: self.time / rhs as u32,
            edges_traversed: self.edges_traversed / rhs,
            nodes_visited: self.nodes_visited / rhs,
            cache_reads: self.cache_reads / rhs,
            cache_writes: self.cache_writes / rhs,
            cache_hits: self.cache_hits / rhs,
            cache_size_estimate: self.cache_size_estimate / rhs,
        }
    }
}

impl std::fmt::Display for QueryStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Time: {:?}, Edges traversed: {}, Nodes visited: {}, Cache reads: {}, Cache writes: {}, Cache hits: {}, Cache size estimate: {} bytes",
            self.time,
            self.edges_traversed,
            self.nodes_visited,
            self.cache_reads,
            self.cache_writes,
            self.cache_hits,
            self.cache_size_estimate
        )
    }
}

impl From<&QueryProfiler> for QueryStats {
    fn from(profiler: &QueryProfiler) -> Self {
        Self {
            time: profiler.start_time.elapsed(),
            edges_traversed: profiler.edges_traversed.load(std::sync::atomic::Ordering::Relaxed),
            nodes_visited: profiler.nodes_visited.load(std::sync::atomic::Ordering::Relaxed),
            cache_reads: profiler.cache_reads.load(std::sync::atomic::Ordering::Relaxed),
            cache_writes: profiler.cache_writes.load(std::sync::atomic::Ordering::Relaxed),
            cache_hits: profiler.cache_hits.load(std::sync::atomic::Ordering::Relaxed),
            cache_size_estimate: profiler.cache_size_estimate
                .load(std::sync::atomic::Ordering::Relaxed),
        }
    }
}

pub struct DisplayVec<'a, T: std::fmt::Display>(pub &'a [T]);

impl<T: std::fmt::Display> std::fmt::Display for DisplayVec<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0.is_empty() {
            write!(f, "[]")
        } else {
            write!(
                f,
                "[{}]",
                self.0
                    .iter()
                    .map(|e| e.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        }
    }
}

pub struct DisplayMap<'a, K: std::fmt::Display, V>(pub &'a std::collections::HashMap<K, V>);

// impl <K: std::fmt::Display, V: std::fmt::Display> std::fmt::Display for DisplayMap<'_, K, V> {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         if self.0.is_empty() {
//             write!(f, "{{}}")
//         } else {
//             write!(f, "{{{}}}", self.0.iter().map(|(k, v)| format!("{}: {}", k, v)).collect::<Vec<_>>().join(", "))
//         }
//     }
// }

impl<K: std::fmt::Display, T: std::fmt::Display> std::fmt::Display for DisplayMap<'_, K, Vec<T>> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0.is_empty() {
            write!(f, "{{}}")
        } else {
            write!(
                f,
                "{{{}}}",
                self.0
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, DisplayVec(v)))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueryResult<Lbl, Data>
where
    Lbl: ScopeGraphLabel + Clone,
    Data: ScopeGraphData,
{
    pub path: ReversePath<Lbl>,
    pub data: Data,
}

impl<Lbl, Data> std::fmt::Display for QueryResult<Lbl, Data>
where
    Lbl: ScopeGraphLabel,
    Data: ScopeGraphData,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match DRAW_MEM_ADDR {
            true => {
                write!(
                    f,
                    "{} ⊢ {}",
                    self.data.render_string(),
                    self.path.as_mem_addr()
                )
            }
            false => {
                write!(f, "{} ⊢ {}", self.data.render_string(), self.path)
            }
        }
    }
}

pub struct Resolver<'r, Lbl, Data, DEq, DWfd>
where
    Lbl: ScopeGraphLabel,
    Data: ScopeGraphData,
    DEq: for<'da, 'db> Fn(&'da Data, &'db Data) -> bool,
    DWfd: for<'da> Fn(&'da Data) -> bool,
{
    // scopegraph contains cache
    pub scope_map: &'r ScopeMap<Lbl, Data>,
    pub path_re: &'r RegexAutomaton<Lbl>,
    pub lbl_order: &'r LabelOrder<Lbl>,
    pub data_eq: DEq,
    pub data_wfd: DWfd,
    pub profiler: QueryProfiler,
}

impl<'r, Lbl, Data, DEq, DWfd> Resolver<'r, Lbl, Data, DEq, DWfd>
where
    Lbl: ScopeGraphLabel,
    Data: ScopeGraphData,
    DEq: for<'da, 'db> Fn(&'da Data, &'db Data) -> bool,
    DWfd: for<'da> Fn(&'da Data) -> bool,
{
    pub fn new(
        scope_map: &'r ScopeMap<Lbl, Data>,
        path_re: &'r RegexAutomaton<Lbl>,
        lbl_order: &'r LabelOrder<Lbl>,
        data_eq: DEq,
        data_wfd: DWfd,
    ) -> Resolver<'r, Lbl, Data, DEq, DWfd> {
        Self {
            scope_map,
            path_re,
            lbl_order,
            data_eq,
            data_wfd,
            profiler: QueryProfiler::new(),
        }
    }

    pub fn resolve(&mut self, path: Path<Lbl>) -> (Vec<QueryResult<Lbl, Data>>, QueryStats) {
        self.profiler.start_time = Instant::now();
        tracing::info!("Resolving path: {}", path);
        let reg = RegexState::new(self.path_re);
        let envs = self.resolve_all(path, reg);
        (envs, (&self.profiler).into())
    }

    /// recursive call site for resolving
    fn resolve_all<'a: 'r>(
        &mut self,
        path: Path<Lbl>,
        reg: RegexState<'a, Lbl>,
    ) -> Vec<QueryResult<Lbl, Data>> {
        self.get_env(path, reg)
    }

    fn data_wfd(&self, data: &Data) -> bool {
        (self.data_wfd)(data)
    }

    fn get_env(
        &mut self,
        path: Path<Lbl>,
        reg: RegexState<'r, Lbl>,
    ) -> Vec<QueryResult<Lbl, Data>> {
        let Some(scope) = self.get_scope(path.target()) else {
            panic!("Scope {} not found in scope graph (len = {})", path.target(), self.scope_map.len());
        };
        self.profiler.inc_nodes_visited();

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

        self.get_env_for_labels(&labels, path)
    }

    fn get_env_for_labels<'a>(
        &mut self,
        labels: &'a [LabelOrEnd<'r, Lbl>],
        path: Path<Lbl>,
    ) -> Vec<QueryResult<Lbl, Data>> {
        debugonly_debug!("Resolving labels: {:?} for {:?}", labels, path.target());
        labels
            .iter()
            // 'max' labels ie all labels with lowest priority
            // max refers to the numerical worth, ie a < b, b would be max
            .filter(|l1| !labels.iter().any(|l2| self.lbl_order.is_less(l1, l2)))
            .flat_map(|max_lbl| {
                // all labels that are lower priority than `lbl`
                let lower_labels = labels
                    .iter()
                    .filter(|l| self.lbl_order.is_less(l, max_lbl))
                    .cloned()
                    .collect::<Vec<_>>();

                self.get_shadowed_env(max_lbl, &lower_labels, path.clone())
            })
            .collect::<Vec<_>>()
    }

    fn get_shadowed_env<'a>(
        &mut self,
        max_lbl: &'a LabelOrEnd<'r, Lbl>,
        lower_lbls: &'a [LabelOrEnd<'r, Lbl>],
        path: Path<Lbl>,
    ) -> Vec<QueryResult<Lbl, Data>> {
        let lower_paths = self.get_env_for_labels(lower_lbls, path.clone());
        let max_path = self.get_env_for_label(max_lbl, path);
        self.shadow(lower_paths, max_path)
    }

    fn get_env_for_label<'a>(
        &mut self,
        label: &'a LabelOrEnd<'r, Lbl>,
        path: Path<Lbl>,
    ) -> Vec<QueryResult<Lbl, Data>> {
        let scope = self.get_scope(path.target()).unwrap().clone();
        match label {
            // reached end of a path
            LabelOrEnd::End => match self.data_wfd(&scope.data) {
                true => vec![QueryResult {
                    path: ReversePath::from(path),
                    data: scope.data.clone(),
                }],
                false => Vec::new(),
            },
            // not yet at end
            LabelOrEnd::Label((label, partial_reg)) => {
                scope
                    .outgoing()
                    .iter()
                    .filter(|e| e.lbl() == label)
                    .map(|e| {
                        path.clone()
                            .step(e.lbl().clone(), e.target(), partial_reg.index())
                    })
                    .filter(|p| !p.is_circular(partial_reg.index()))
                    .flat_map(|p| {
                        self.profiler.inc_edges_traversed();
                        self.resolve_all(p, partial_reg.clone())
                    }) // resolve new paths
                    .collect::<Vec<_>>()
            }
        }
    }

    fn shadow(
        &self,
        mut a1: Vec<QueryResult<Lbl, Data>>,
        mut a2: Vec<QueryResult<Lbl, Data>>,
    ) -> Vec<QueryResult<Lbl, Data>> {
        debugonly_trace!("Shadowing...");
        a2.retain(|qr2| !a1.iter().any(|qr1| (self.data_eq)(&qr1.data, &qr2.data)));

        a1.append(&mut a2);
        a1
    }

    fn get_scope(&self, scope: Scope) -> Option<&ScopeData<Lbl, Data>> {
        self.scope_map.get(&scope)
    }
}
