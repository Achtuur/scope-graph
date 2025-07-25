use std::{fmt::Write, hash::RandomState};

use graphing::plantuml::{EdgeDirection, PlantUmlItem};
use serde::{Deserialize, Serialize};

use crate::{data::ScopeGraphData, debugonly_trace, graph::{QueryResult, ScopeGraph, ScopeMap}, label::ScopeGraphLabel, order::LabelOrder, path::Path, regex::{dfs::RegexAutomaton, RegexState}, scope::Scope, BackgroundColor, ColorSet, DO_CIRCLE_CHECK};

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

/// Cache for a single query
#[derive(Debug)]
#[repr(transparent)]
pub struct QueryCache<Lbl, Data>
where Lbl: ScopeGraphLabel, Data: ScopeGraphData
{
    pub(crate) cache: hashbrown::HashMap<QueryCacheKey, EnvCache<Lbl, Data>>,
}

impl<Lbl, Data> std::default::Default for QueryCache<Lbl, Data>
where Lbl: ScopeGraphLabel, Data: ScopeGraphData
{
    fn default() -> Self {
        Self {
            cache: hashbrown::HashMap::default(),
        }
    }
}

impl<Lbl, Data> QueryCache<Lbl, Data>
where Lbl: ScopeGraphLabel, Data: ScopeGraphData
{
    pub fn get_envs(&self, reg: &RegexState<'_, Lbl>, path: &Path<Lbl>) -> &ProjEnvs<Lbl, Data> {
        let key = (reg.index(), path.target());
        self.cache.get(&key)
            .and_then(|entry| entry.get_env(path))
            .unwrap_or_default()
    }

    pub fn clear_envs(&mut self, reg: &RegexState<'_, Lbl>, path: &Path<Lbl>) {
        let key = (reg.index(), path.target());
        self.cache.remove(&key);
    }

    pub fn insert(&mut self, reg: &RegexState<'_, Lbl>, path: &Path<Lbl>, envs: ProjEnvs<Lbl, Data>) {
        let key = (reg.index(), path.target());
        let entry = self.cache.entry(key).or_insert(EnvCache::new(path.clone()));

        for (hash, env) in envs {
            entry.insert(hash, path.clone(), env);
        }
    }

    fn generate_uml(&self, scopes: &impl ScopeGraph<Lbl, Data>, header: String) -> impl Iterator<Item = PlantUmlItem> {
        self.cache
        .iter()
        .filter_map(move |((_, scope), env_cache)| {
            if scopes.scope_holds_data(*scope) {
                return None;
            }

            let entries = env_cache
            .cache
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

    }
}


pub type ProjEnvs<Lbl, Data> = hashbrown::HashMap<ProjHash, Vec<QueryResult<Lbl, Data>>>;

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

    pub fn get_env(&self, path: &Path<Lbl>) -> Option<&ProjEnvs<Lbl, Data>> {
        debugonly_trace!("Checking cache ({}) for path: {}", self.path, path);

        if !DO_CIRCLE_CHECK {
            return Some(&self.cache);
        }

        if &self.path != path && self.path.partially_contains(path) {
            debugonly_trace!("Cache invalid; path is contained");
            return None;
        }
        Some(&self.cache)
    }

    pub fn insert(&mut self, hash: ProjHash, path: Path<Lbl>, env: Vec<QueryResult<Lbl, Data>>) {
        self.path = path;
        self.cache.insert(hash, env);
        // let entry = self.cache.entry(hash).or_default();
        // entry.append(&mut env);
        // entry.reserve(env.len());
        // for e in env {
        //     // if !e.path.is_circular() {
        //     // }
        //     entry.push(e);
        // }
    }
}

