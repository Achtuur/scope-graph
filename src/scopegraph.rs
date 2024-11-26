use std::collections::HashMap;

use regex::Regex;

use crate::{label::ScopeGraphLabel, scope::Scope};



#[derive(Clone, Copy, Debug)]
struct Edge<Lbl>
where Lbl: ScopeGraphLabel + Clone
{
    to: Scope,
    label: Lbl,
}

#[derive(Clone, Debug)]
struct ScopeData<Lbl, Data>
where Lbl: ScopeGraphLabel + Clone
{
    edges: Vec<Edge<Lbl>>,
    data: Data,
}

impl<Lbl, Data> ScopeData<Lbl, Data>
where Lbl: ScopeGraphLabel + Clone
{

    pub fn new(data: Data) -> Self {
        Self {
            edges: Vec::new(),
            data,
        }
    }

    fn is_final(&self) -> bool {
        self.edges.is_empty()
    }
}

#[derive(Debug)]
pub struct ScopeGraph<Lbl, Data>
where
    Lbl: ScopeGraphLabel + Clone
{
    scopes: HashMap<Scope, ScopeData<Lbl, Data>>,
}

impl<Lbl, Data> ScopeGraph<Lbl, Data>
where
    Lbl: ScopeGraphLabel + Clone
{
    pub fn new() -> Self {
        Self {
            scopes: HashMap::new(),
        }
    }

    pub fn add_scope(&mut self, scope: Scope, data: Data) {
        self.scopes.insert(scope, ScopeData::new(data));
    }

    pub fn add_edge(&mut self, source: Scope, target: Scope, label: Lbl) {
        let new_edge = Edge {
            to: target,
            label,
        };

        self.scopes.get_mut(&source)
        .expect("Attempting to add edge to non-existant scope")
        .edges.push(new_edge);
    }

    pub fn add_decl(&mut self, source: Scope, label: Lbl, data: Data) {
        let decl_scope = Scope::new();
        self.add_scope(decl_scope, data);
        self.add_edge(source, decl_scope, label);
    }
}

// queries

#[derive(Debug)]
pub struct QueryResult<Data> {
    path: String,
    data: Data,
}

impl<Lbl, Data> ScopeGraph<Lbl, Data>
where
    Lbl: ScopeGraphLabel + Clone + std::fmt::Debug,
    Data: std::fmt::Debug + Clone
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
    pub fn query<DF>(&self, scope: Scope, path_regex: &Regex, data_wellformedness: DF) -> Vec<QueryResult<Data>>
    where
        DF: Fn(&Data) -> bool
    {
        let scope = self.scopes.get(&scope)
        .expect("Attempting to query non-existant scope");
        self.query_internal(scope, String::new(), path_regex, &data_wellformedness)
    }

    fn query_internal<DF>(&self, current_scope: &ScopeData<Lbl, Data>, path: String, path_regex: &Regex, data_equiv: &DF) -> Vec<QueryResult<Data>>
    where
        DF: Fn(&Data) -> bool
    {
        let mut results: Vec<QueryResult<Data>> = Vec::new();
        for edge in &current_scope.edges {
            let t_scope = self.scopes.get(&edge.to).unwrap();
            let new_path = format!("{}{}", path, edge.label.char());
            if t_scope.is_final() {
                if path_regex.is_match(&new_path) && data_equiv(&t_scope.data) {
                    results.push(QueryResult {
                        path: new_path,
                        data: t_scope.data.clone(),
                    })
                }
            } else {
                let mut child_query = self.query_internal(t_scope, new_path, path_regex, data_equiv);
                results.append(&mut child_query);
            }
        }
        results
    }
}