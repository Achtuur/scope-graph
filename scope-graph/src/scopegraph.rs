use std::{collections::HashMap, sync::Mutex};

use crate::{
    data::ScopeGraphData, label::ScopeGraphLabel, order::{LabelOrder, LabelOrderBuilder}, path::Path,
    regex::dfs::RegexAutomata, resolve::Resolver, scope::Scope,
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
pub struct ScopeGraph<Lbl, Data>
where
    Lbl: ScopeGraphLabel + Clone,
{
    pub scopes: HashMap<Scope, ScopeData<Lbl, Data>>,
}

impl<Lbl, Data> ScopeGraph<Lbl, Data>
where
    Lbl: ScopeGraphLabel + Clone,
{
    pub fn new() -> Self {
        Self {
            scopes: HashMap::new(),
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

#[derive(Debug)]
pub struct QueryResult<Lbl: ScopeGraphLabel, Data> {
    pub path: Path<Lbl>,
    pub data: Data,
}

impl<Lbl, Data> ScopeGraph<Lbl, Data>
where
    Lbl: ScopeGraphLabel + Clone + std::fmt::Debug + Eq + std::hash::Hash + Ord,
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
        &self,
        scope: Scope,
        path_regex: &RegexAutomata<Lbl>,
        order: &LabelOrder<Lbl>,
        data_equiv: impl Fn(&Data, &Data) -> bool,
        data_wellformedness: impl Fn(&Data) -> bool,
    ) -> (Vec<QueryResult<Lbl, Data>>, Vec<Path<Lbl>>) {
        let resolver = Resolver {
            scope_graph: self,
            path_re: path_regex,
            lbl_order: order,
            data_eq: &data_equiv,
            data_wfd: &data_wellformedness,
            considered_paths: Mutex::new(Vec::new()),
            cache: HashMap::new(),
        };
        let res = resolver.resolve(Path::start(scope));
        let considered_paths = resolver.considered_paths.into_inner().unwrap();
        (res, considered_paths)
    }
}

// rendering
impl<Lbl, Data> ScopeGraph<Lbl, Data>
where
    Lbl: ScopeGraphLabel + Clone,
    Data: ScopeGraphData,
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
