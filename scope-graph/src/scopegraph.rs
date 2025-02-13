use std::{collections::HashMap, sync::{Arc, Mutex, MutexGuard}};

use crate::{
    data::ScopeGraphData, label::ScopeGraphLabel, order::{LabelOrder, LabelOrderBuilder}, path::Path,
    regex::dfs::RegexAutomata, resolve::{CacheKey, CacheValue, ResolveCache, Resolver}, scope::Scope,
};

#[derive(Clone, Copy, Debug)]
pub struct Edge<Lbl>
where
    Lbl: ScopeGraphLabel + Clone,
{
    pub to: Scope,
    pub label: Lbl,
}

#[derive(Clone, Debug)]
pub struct ScopeData<Lbl, Data>
where
    Lbl: ScopeGraphLabel + Clone,
{
    pub edges: Vec<Edge<Lbl>>,
    pub data: Data,
}

impl<Lbl, Data> ScopeData<Lbl, Data>
where
    Lbl: ScopeGraphLabel + Clone,
{
    pub fn new(data: Data) -> Self {
        Self {
            edges: Vec::new(),
            data,
        }
    }
}

#[derive(Debug)]
pub struct ScopeGraph<'s, Lbl, Data>
where
    Lbl: ScopeGraphLabel + Clone + std::fmt::Debug + Eq + std::hash::Hash + Ord,
    Data: std::fmt::Debug + Clone,
{
    pub scopes: HashMap<Scope, ScopeData<Lbl, Data>>,
    pub(crate) resolve_cache: Mutex<ResolveCache<'s, Lbl, Data>>,
}

impl<'s, Lbl, Data> ScopeGraph<'s, Lbl, Data>
where
    Lbl: ScopeGraphLabel + Clone + std::fmt::Debug + Eq + std::hash::Hash + Ord,
    Data: std::fmt::Debug + Clone,
{
    pub fn new() -> Self {
        Self {
            scopes: HashMap::new(),
            resolve_cache: Mutex::new(ResolveCache::new()),
        }
    }

    pub fn find_scope(&self, scope_num: usize) -> Option<Scope> {
        self.scopes.keys().find_map(|s| {
            if s.0 == scope_num {
                Some(*s)
            } else {
                None
            }
        })
    }

    pub fn add_scope(&mut self, scope: Scope, data: Data) {
        self.scopes.insert(scope, ScopeData::new(data));
    }

    pub fn add_edge(&mut self, source: Scope, target: Scope, label: Lbl) {
        let new_edge = Edge { to: target, label };

        self.scopes
            .get_mut(&source)
            .expect("Attempting to add edge to non-existant scope")
            .edges
            .push(new_edge);
    }

    pub fn add_decl(&mut self, source: Scope, label: Lbl, data: Data) {
        let decl_scope = Scope::new();
        self.add_scope(decl_scope, data);
        self.add_edge(source, decl_scope, label);
    }
}

// queries

#[derive(Debug, Clone)]
pub struct QueryResult<Lbl, Data>
where
    Lbl: ScopeGraphLabel + Clone,
    Data: Clone,
{
    pub path: Path<Lbl>,
    pub data: Data,
}

impl<Lbl, Data> std::fmt::Display for QueryResult<Lbl, Data>
where
    Lbl: ScopeGraphLabel + Clone,
    Data: Clone + std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} -[] {:?}", self.path, self.data)
    }
}

impl<'s, Lbl, Data> ScopeGraph<'s, Lbl, Data>
where
    Lbl: ScopeGraphLabel + Clone + std::fmt::Debug + std::fmt::Display + Eq + std::hash::Hash + Ord,
    Data: std::fmt::Debug + Clone,
{
    /// Returns the data associated with the scope baesd on path_regex and name of the data
    ///
    /// Data is now just a string, so the query returns `data_name` if succesful
    ///
    /// # Arguments
    ///
    /// * Scope: Starting scope
    /// * path_regex: Regular expression to match the path
    /// * data_name: Name of the data to return
    pub fn query(
        &'s self,
        scope: Scope,
        path_regex: &'s RegexAutomata<Lbl>,
        order: &'s LabelOrder<Lbl>,
        data_equiv: impl Fn(&Data, &Data) -> bool,
        data_wellformedness: impl Fn(&Data) -> bool,
    ) -> Vec<QueryResult<Lbl, Data>> {
        let resolver = Resolver::new(
            self,
            path_regex,
            order,
            &data_equiv,
            &data_wellformedness,
        );
        let res = resolver.resolve(Path::start(scope));
        resolver.print_cache();
        res
    }
}

// rendering
impl<Lbl, Data> ScopeGraph<'_, Lbl, Data>
where
    Lbl: ScopeGraphLabel + Clone + std::fmt::Debug + Eq + std::hash::Hash + Ord,
    Data: std::fmt::Debug + Clone + ScopeGraphData,
{
    pub fn as_mmd(&self, title: &str) -> String {
        let mut mmd = format!(
            "---\n\
            title: \"{}\"\n\
            ---\n\
            flowchart LR\n\
            ",
            title
        );

        for s in self.scopes.keys() {
            // todo, different node based on data
            mmd += &format!("\tscope_{}((\"{}\"))\n", s.0, s.0);
        }

        for (s, d) in self.scopes.iter() {
            if d.data.variant_has_data() {
                mmd += &format!("\tscope_{}[\"{}\"]\n", s.0, d.data.render_string());
            } else {
                mmd += &format!("\tscope_{}((\"{}\"))\n", s.0, s.0);
            }
        }

        for (s, d) in self.scopes.iter() {
            for edge in d.edges.iter() {
                mmd += &format!(
                    "scope_{} ==>|\"{}\"| scope_{}\n",
                    s.0,
                    edge.label.str(),
                    edge.to.0
                )
            }
        }

        mmd
    }
}
