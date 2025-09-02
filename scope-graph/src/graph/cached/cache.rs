use std::{cell::RefCell, fmt::Write, hash::RandomState, rc::Rc, sync::Arc, time::Instant};

use graphing::plantuml::{EdgeDirection, PlantUmlItem};
use serde::{Deserialize, Serialize};

use crate::{data::ScopeGraphData, debug_tracing, graph::{resolve::QueryProfiler, QueryResult, ScopeGraph, ScopeMap}, label::ScopeGraphLabel, order::LabelOrder, path::Path, regex::{dfs::RegexAutomaton, RegexState}, scope::Scope, util::DisplayVec, BackgroundColor, ColorSet, DO_CIRCLE_CHECK};

pub type ProjHash = u64;

/// (label order, automaton, hash of the projection function)
pub type ResolveCacheKey<Lbl> = (LabelOrder<Lbl>, RegexAutomaton<Lbl>, ProjHash);

/// Cache for entire scope graph, across multiple queries.
#[derive(Debug, Default)]
pub struct ResolveCache<Lbl, Data>
where Lbl: ScopeGraphLabel, Data: ScopeGraphData
{
    pub(crate) cache: hashbrown::HashMap<ResolveCacheKey<Lbl>, QueryCache<Lbl, Data>>,
}

impl<Lbl, Data> ResolveCache<Lbl, Data>
where Lbl: ScopeGraphLabel, Data: ScopeGraphData
{
    pub fn new() -> Self {
        Self {
            cache: hashbrown::HashMap::new(),
        }
    }

    pub fn get_mut(&mut self, key: ResolveCacheKey<Lbl>) -> &mut QueryCache<Lbl, Data> {
        self.cache.entry(key).or_default()
    }

    pub fn clear(&mut self) {
        self.cache.clear();
    }

    pub fn generate_uml<S: ScopeGraph<Lbl, Data>>(&self, graph: &S) -> impl Iterator<Item = PlantUmlItem> {
        self.cache
        .iter()
        .flat_map(|(key, query_cache)| {
            let mut s = String::new();
            writeln!(&mut s, "<b>({}, {})</b>", key.0, key.1).unwrap();
            query_cache.generate_uml(graph, s)
        })
    }
}

pub type QueryCacheKey = (usize, Scope);
pub type QueryCacheMap<Lbl, Data> = hashbrown::HashMap<QueryCacheKey, EnvCache<Lbl, Data>>;

/// Cache for a single query
#[derive(Debug)]
#[repr(transparent)]
pub struct QueryCache<Lbl, Data>
where Lbl: ScopeGraphLabel, Data: ScopeGraphData
{
    pub(crate) cache: Rc<RefCell<QueryCacheMap<Lbl, Data>>>,
}

impl<Lbl, Data> std::default::Default for QueryCache<Lbl, Data>
where Lbl: ScopeGraphLabel, Data: ScopeGraphData
{
    fn default() -> Self {
        Self {
            cache: Rc::new(RefCell::new(hashbrown::HashMap::default())),
        }
    }
}

impl<Lbl, Data> QueryCache<Lbl, Data>
where Lbl: ScopeGraphLabel, Data: ScopeGraphData
{
    pub fn get_envs(&self, reg: &RegexState<'_, Lbl>, path: &Path<Lbl>, profiler: &QueryProfiler) -> Option<ProjEnvs<Lbl, Data>> {
        let key = (reg.index(), path.target());
        self.cache.borrow().get(&key).and_then(|entry| entry.get_env(path, profiler))
    }

    pub fn clear_envs(&self, reg: &RegexState<'_, Lbl>, path: &Path<Lbl>) {
        let key = (reg.index(), path.target());
        self.cache.borrow_mut().remove(&key);
    }

    pub fn insert(&self, reg: &RegexState<'_, Lbl>, path: &Path<Lbl>, envs: ProjEnvs<Lbl, Data>) {
        let key = (reg.index(), path.target());
        let mut cache = self.cache.borrow_mut();
        let entry = cache.entry(key).or_insert(EnvCache::new(path.clone()));
        entry.insert(path.clone(), envs);
    }

    fn generate_uml(&self, scopes: &impl ScopeGraph<Lbl, Data>, header: String) -> impl IntoIterator<Item = PlantUmlItem> {
        let c = self.cache.borrow();
        c
        .iter()
        .filter_map(move |((_, scope), env_cache)| {
            if scopes.scope_holds_data(*scope) {
                return None;
            }

            let entries = env_cache
            .cache
            .group_by_hash()
            .values()
            .map(|envs| {
                let mut s = format!("<i>{}</i>:\n", env_cache.path);
                for e in envs {
                    writeln!(&mut s, "  {e}").unwrap();
                }
                s
            })
            .collect::<Vec<String>>()
            .join("\n");

            let contents = format!("{header}\n{entries}");

            Some(PlantUmlItem::note(scope.uml_id(), contents, EdgeDirection::Right)
                .add_class("cache-entry")
                .add_class(BackgroundColor::get_class_name(scope.id())))
        })
        .collect::<Vec<_>>()
    }
}

#[derive(Debug)]
pub struct EnvCache<Lbl, Data>
where Lbl: ScopeGraphLabel, Data: ScopeGraphData
{
    cache: ProjEnvs<Lbl, Data>,
    /// Paths that were traversed to generate this entry
    ///
    /// This is to deal with circular paths mainly
    path: Path<Lbl>,
}

impl<Lbl, Data> EnvCache<Lbl, Data>
where Lbl: ScopeGraphLabel, Data: ScopeGraphData
{
    pub fn new(path: Path<Lbl>) -> Self {
        Self {
            path,
            cache: ProjEnvs::with_capacity(4),
        }
    }

    pub fn get_env(&self, path: &Path<Lbl>, profiler: &QueryProfiler) -> Option<ProjEnvs<Lbl, Data>> {
        debug_tracing!(trace, "Checking cache ({}) for path: {}", self.path, path);
        return Some(self.cache.clone());
        // if !DO_CIRCLE_CHECK {
        // }

        // let timer = Instant::now();
        // let is_circular = &self.path != path && self.path.partially_contains(path.without_head_unless_start());
        // profiler.inc_circ_check_timer(timer.elapsed());

        // if is_circular {
        //     debug_tracing!(trace, "Cache invalid; path is contained");
        //     return None;
        // }
        // Some(self.cache.clone())
    }

    // pub fn insert(&mut self, hash: ProjHash, path: Path<Lbl>, env: Vec<QueryResult<Lbl, Data>>) {
    //     self.path = path;
    //     self.cache.insert(hash, env);
    //     // let entry = self.cache.entry(hash).or_default();
    //     // entry.append(&mut env);
    //     // entry.reserve(env.len());
    //     // for e in env {
    //     //     // if !e.path.is_circular() {
    //     //     // }
    //     //     entry.push(e);
    //     // }
    // }

    pub fn insert(&mut self, path: Path<Lbl>, env: ProjEnvs<Lbl, Data>) {
        debug_tracing!(trace, "Inserting envs into cache for path: {}", path);
        self.path = path;
        self.cache.extend(env);
    }
}


#[derive(Debug, Clone)]
#[repr(transparent)]
pub(crate) struct ProjEnvs<Lbl: ScopeGraphLabel, Data: ScopeGraphData> {
    inner: Vec<(ProjHash, QueryResult<Lbl, Data>)>,
}

impl<Lbl, Data> std::fmt::Display for ProjEnvs<Lbl, Data>
where Lbl: ScopeGraphLabel, Data: ScopeGraphData
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.inner.is_empty() {
            write!(f, "[]")
        } else {
            write!(
                f,
                "[{}]",
                self.inner
                    .iter()
                    .map(|(p, qr)| format!("{p}: {qr}",))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        }
    }
}

impl<Lbl, Data> std::default::Default for ProjEnvs<Lbl, Data>
where Lbl: ScopeGraphLabel, Data: ScopeGraphData
{
    fn default() -> Self {
        Self::new()
    }
}

impl<Lbl: ScopeGraphLabel, Data: ScopeGraphData> ProjEnvs<Lbl, Data> {
    #[inline]
    pub fn new() -> Self {
        Self {
            inner: Vec::new(),
        }
    }

    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: Vec::with_capacity(capacity),
        }
    }

    #[inline]
    pub fn new_with_env(hash: ProjHash, env: QueryResult<Lbl, Data>) -> Self {
        Self {
            inner: vec![(hash, env)],
        }
    }

    pub fn shadow(&mut self, mut other: Self) {
        other.inner.retain(|(proj, _)| {
            !self.inner.iter().any(|(p, _)| *p == *proj)
        });
        self.extend(other);
    }

    pub fn step_paths(&mut self, label: &Lbl, scope: Scope, reg_idx: usize) {
        for (_, env) in self.inner.iter_mut() {
            env.path = env.path.step(label.clone(), scope, reg_idx);
        }
    }

    #[inline(always)]
    pub fn push(&mut self, hash: ProjHash, env: QueryResult<Lbl, Data>) {
        self.inner.push((hash, env));
    }

    #[inline]
    pub fn insert(&mut self, hash: ProjHash, env: Vec<QueryResult<Lbl, Data>>) {
        for e in env {
            self.inner.push((hash, e));
        }
    }

    #[inline]
    pub fn contains_key(&self, hash: ProjHash) -> bool {
        self.inner.iter().any(|(h, _)| *h == hash)
    }

    #[inline]
    pub fn extend(&mut self, other: Self) {
        self.inner.extend(other.inner);
    }

    // pub fn iter(&self) -> impl Iterator<Item = &(ProjHash, QueryResult<Lbl, Data>)> {
    //     self.inner.borrow().iter()
    // }

    pub(crate) fn group_by_hash(&self) -> hashbrown::HashMap<ProjHash, Vec<QueryResult<Lbl, Data>>> {
        let mut map = hashbrown::HashMap::new();
        for (hash, env) in self.inner.iter() {
            map.entry(*hash).or_insert_with(Vec::new).push(env.clone());
        }
        map
    }

    pub fn clone_envs_by_hash(&self, hash: &ProjHash) -> Vec<QueryResult<Lbl, Data>>
    {
        self.inner
        .iter()
        .filter(move |(h, _)| h == hash)
        .map(|(_, e)| e)
        .cloned()
        .collect::<Vec<_>>()
    }

    // pub fn is_empty(&self) -> bool {
    //     self.inner.is_empty()
    // }
}

impl<Lbl, Data> IntoIterator for ProjEnvs<Lbl, Data>
where Lbl: ScopeGraphLabel, Data: ScopeGraphData
{
    type Item = (ProjHash, QueryResult<Lbl, Data>);
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

impl<Lbl, Data> FromIterator<(ProjHash, QueryResult<Lbl, Data>)> for ProjEnvs<Lbl, Data>
where Lbl: ScopeGraphLabel, Data: ScopeGraphData
{
    fn from_iter<T: IntoIterator<Item = (ProjHash, QueryResult<Lbl, Data>)>>(iter: T) -> Self {
        let iter = iter.into_iter().collect::<Vec<_>>();
        Self {
            inner: iter,
        }
        // println!("iter.size_hint().: {0:?}", iter.size_hint());
        // let mut envs = ProjEnvs::new();
        // for item in iter {
        //     envs.push(item.0, item.1);
        // }
        // envs
    }
}



// #[derive(Debug, Clone)]
// pub(crate) struct ProjEnvsCell<Lbl: ScopeGraphLabel, Data: ScopeGraphData> {
//     // inner: Vec<(ProjHash, Vec<QueryResult<Lbl, Data>>)>,
//     inner: Arc<RefCell<Vec<(ProjHash, QueryResult<Lbl, Data>)>>>,
// }

// impl<Lbl, Data> std::default::Default for ProjEnvsCell<Lbl, Data>
// where Lbl: ScopeGraphLabel, Data: ScopeGraphData
// {
//     fn default() -> Self {
//         Self::new()
//     }
// }

// impl<Lbl: ScopeGraphLabel, Data: ScopeGraphData> ProjEnvsCell<Lbl, Data> {
//     pub fn new() -> Self {
//         Self {
//             inner: Arc::new(RefCell::new(Vec::new()))
//         }
//     }

//     pub fn with_capacity(capacity: usize) -> Self {
//         Self {
//             inner: Arc::new(RefCell::new(Vec::with_capacity(capacity))),
//         }
//     }

//     pub fn new_with_env(hash: ProjHash, env: QueryResult<Lbl, Data>) -> Self {
//         Self {
//             inner: Arc::new(RefCell::new(vec![(hash, env)])),
//         }
//     }

//     pub fn shadow(&mut self, other: Self) {
//         let mut s_b = self.inner.borrow_mut();
//         let o = other.inner.replace(Vec::new());
//         for (proj, e) in o {
//             if !s_b.iter().any(|(p, _)| *p == proj) {
//                 s_b.push((proj, e));
//             }
//             // if !self.inner.iter().any(|(p, _)| *p == proj) {
//             //     self.inner.push((proj, e));
//             // }
//         }
//     }

//     pub fn step_paths(&mut self, label: &Lbl, scope: Scope, reg_idx: usize) {
//         for (_, env) in self.inner.borrow_mut().iter_mut() {
//             env.path = env.path.step(label.clone(), scope, reg_idx);
//         }
//     }

//     pub fn push(&mut self, hash: ProjHash, env: QueryResult<Lbl, Data>) {
//         self.inner.borrow_mut().push((hash, env));
//     }

//     pub fn insert(&mut self, hash: ProjHash, env: Vec<QueryResult<Lbl, Data>>) {
//         for e in env {
//             self.inner.borrow_mut().push((hash, e));
//         }
//     }

//     pub fn contains_key(&self, hash: ProjHash) -> bool {
//         self.inner.borrow().iter().any(|(h, _)| *h == hash)
//     }

//     pub fn extend(&mut self, other: Self) {
//         let o = other.inner.replace(Vec::new());
//         self.inner.borrow_mut().extend(o);
//     }

//     // pub fn iter(&self) -> impl Iterator<Item = &(ProjHash, QueryResult<Lbl, Data>)> {
//     //     self.inner.borrow().iter()
//     // }

//     pub(crate) fn group_by_hash(&self) -> hashbrown::HashMap<ProjHash, Vec<QueryResult<Lbl, Data>>> {
//         let mut map = hashbrown::HashMap::new();
//         for (hash, env) in self.inner.borrow().iter() {
//             map.entry(*hash).or_insert_with(Vec::new).push(env.clone());
//         }
//         map
//     }

//     pub fn clone_envs_by_hash(&self, hash: &ProjHash) -> Vec<QueryResult<Lbl, Data>>
//     {
//         self.inner
//         .borrow()
//         .iter()
//         .filter(move |(h, _)| h == hash)
//         .map(|(_, e)| e)
//         .cloned()
//         .collect::<Vec<_>>()
//     }

//     // pub fn is_empty(&self) -> bool {
//     //     self.inner.is_empty()
//     // }
// }

// // impl<Lbl, Data> IntoIterator for ProjEnvs<Lbl, Data>
// // where Lbl: ScopeGraphLabel, Data: ScopeGraphData
// // {
// //     type Item = (ProjHash, QueryResult<Lbl, Data>);
// //     type IntoIter = std::vec::IntoIter<Self::Item>;

// //     fn into_iter(self) -> Self::IntoIter {
// //         let v = self.inner.replace(Vec::new());
// //         v.into_iter()
// //     }
// // }

// impl<Lbl, Data> FromIterator<(ProjHash, QueryResult<Lbl, Data>)> for ProjEnvsCell<Lbl, Data>
// where Lbl: ScopeGraphLabel, Data: ScopeGraphData
// {
//     fn from_iter<T: IntoIterator<Item = (ProjHash, QueryResult<Lbl, Data>)>>(iter: T) -> Self {
//         let iter = iter.into_iter();
//         let mut envs = ProjEnvsCell::with_capacity(iter.size_hint().1.unwrap_or_default());
//         for item in iter {
//             envs.push(item.0, item.1);
//         }
//         envs
//     }
// }