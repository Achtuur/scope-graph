use std::collections::HashMap;

use crate::{
    data::ScopeGraphData, label::ScopeGraphLabel, order::LabelOrder, path::Path,
    regex::dfs::RegexAutomata, scope::Scope,
};

#[derive(Clone, Copy, Debug)]
struct Edge<Lbl>
where
    Lbl: ScopeGraphLabel + Clone,
{
    to: Scope,
    label: Lbl,
}

#[derive(Clone, Debug)]
struct ScopeData<Lbl, Data>
where
    Lbl: ScopeGraphLabel + Clone,
{
    edges: Vec<Edge<Lbl>>,
    data: Data,
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
    scopes: HashMap<Scope, ScopeData<Lbl, Data>>,
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
    Lbl: ScopeGraphLabel + Clone + std::fmt::Debug + PartialEq + Eq + std::hash::Hash,
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
    pub fn query<DF>(
        &self,
        scope: Scope,
        path_regex: &RegexAutomata<Lbl>,
        order: &LabelOrder<Lbl>,
        data_wellformedness: DF,
    ) -> Vec<QueryResult<Lbl, Data>>
    where
        DF: Fn(&Data) -> bool,
    {
        let scope_data = self
            .scopes
            .get(&scope)
            .expect("Attempting to query non-existant scope");
        self.query_internal(
            scope_data,
            Path::start(scope),
            path_regex,
            order,
            &data_wellformedness,
        )
    }

    fn query_internal<DF>(
        &self,
        current_scope: &ScopeData<Lbl, Data>,
        path: Path<Lbl>,
        path_regex: &RegexAutomata<Lbl>,
        order: &LabelOrder<Lbl>,
        data_equiv: &DF,
    ) -> Vec<QueryResult<Lbl, Data>>
    where
        DF: Fn(&Data) -> bool,
    {
        let mut results: Vec<QueryResult<Lbl, Data>> = Vec::new();

        let (mut ordered, unordered): (Vec<_>, Vec<_>) = current_scope
            .edges
            .iter()
            .partition(|edge| order.contains(&edge.label));

        ordered.sort_by(|a, b| order.cmp(&a.label, &b.label));

        // should break after all paths with current label are found
        // or not? ambigious reference
        for edge in ordered {
            let t_scope = self.scopes.get(&edge.to).unwrap();
            let new_path = path.clone().step(edge.label.clone(), edge.to);

            if path_regex.is_match(&new_path.as_lbl_vec()) && data_equiv(&t_scope.data) {
                results.push(QueryResult {
                    path: new_path,
                    data: t_scope.data.clone(),
                });
            } else if path_regex.partial_match(&new_path.as_lbl_vec()) {
                let mut child_query =
                    self.query_internal(t_scope, new_path, path_regex, order, data_equiv);
                results.append(&mut child_query);
            }

            // break if something is found
            if !results.is_empty() {
                break;
            }
        }

        // dont break
        for edge in unordered {
            let t_scope = self.scopes.get(&edge.to).unwrap();
            let new_path = path.clone().step(edge.label.clone(), edge.to);

            if path_regex.is_match(&new_path.as_lbl_vec()) && data_equiv(&t_scope.data) {
                results.push(QueryResult {
                    path: new_path,
                    data: t_scope.data.clone(),
                })
            } else if path_regex.partial_match(&new_path.as_lbl_vec()) {
                let mut child_query =
                    self.query_internal(t_scope, new_path, path_regex, order, data_equiv);
                results.append(&mut child_query);
            }
        }
        results
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
