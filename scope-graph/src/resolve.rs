use std::cmp::Ordering;

use crate::{label::ScopeGraphLabel, order::LabelOrder, path::Path, regex::dfs::RegexAutomata, scope::Scope, scopegraph::{Edge, QueryResult, ScopeData, ScopeGraph}, Data, Label};

pub struct Resolver<'a, Lbl, Data, DEq, DWfd>
where
    Lbl: ScopeGraphLabel + Clone + std::fmt::Debug + PartialEq + Eq + std::hash::Hash,
    Data: std::fmt::Debug + Clone,
    DEq: for<'da, 'db> Fn(&'da Data, &'db Data) -> bool,
    DWfd: for<'da> Fn(&'da Data) -> bool,
{
    pub scope_graph: &'a ScopeGraph<Lbl, Data>,
    pub path_re: &'a RegexAutomata<Lbl>,
    pub lbl_order: &'a LabelOrder<Lbl>,
    pub data_eq: DEq,
    pub data_wfd: DWfd,
}


impl<'a, Lbl, Data, DEq, DWfd> Resolver<'a, Lbl, Data, DEq, DWfd>
where
    Lbl: ScopeGraphLabel + Clone + std::fmt::Debug + PartialEq + Eq + std::hash::Hash,
    Data: std::fmt::Debug + Clone,
    DEq: for<'da, 'db> Fn(&'da Data, &'db Data) -> bool,
    DWfd: for<'da> Fn(&'da Data) -> bool,
{
    pub fn resolve(
        &self,
        path: Path<Lbl>
    ) -> Vec<QueryResult<Lbl, Data>>
    {
        println!("Resolving path: {}", path);
        self.get_env(path)
    }

    fn get_env(
        &self,
        path: Path<Lbl>
    ) -> Vec<QueryResult<Lbl, Data>>
    {
        // all edges where brzozowski derivative != 0
        let scope = self.get_scope(path.target()).expect("Scope not found");
        let edges = scope.edges
        .iter()
        .filter(|edge| {
            let new_path = path.clone().step(edge.label.clone(), edge.to);
            self.path_re.partial_match(&new_path.as_lbl_vec())
        })
        .collect::<Vec<_>>();

        self.getEnvForLabels(&edges, path)
    }

    fn getEnvForLabels(
        &self,
        edges: &[&Edge<Lbl>],
        path: Path<Lbl>,
    ) -> Vec<QueryResult<Lbl, Data>>
    {
        let mut results = Vec::new();
        // println!("Resolving edges: {0:?}", edges);

        // 'max' label edges, ie all edges with highest priority label
        let max = edges
        .iter()
        .filter(|e| {
            !edges
            .iter()
            .any(|e2| self.lbl_order.cmp(&e.label, &e2.label).is_gt())
        })
        .collect::<Vec<_>>();

        // println!("Finding edges for max labels: {:?}", max);

        for max_edge in max {
            // all labels that are lower priority than `lbl`
            let lower_edges = edges.iter().filter_map(|e| {
                match self.lbl_order.cmp(&max_edge.label, &e.label).is_lt() {
                    true => Some(*e),
                    false => None,
                }
            })
            .collect::<Vec<_>>();
            let env = self.getShadowedEnv(max_edge, &lower_edges, path.clone());
            results.extend(env.into_iter());
        }

        results
    }

    fn getShadowedEnv(
        &self,
        max_edge: &Edge<Lbl>,
        lower_edges: &[&Edge<Lbl>],
        path: Path<Lbl>,
    ) -> Vec<QueryResult<Lbl, Data>> {
        let lower_paths = self.getEnvForLabels(lower_edges, path.clone());
        let max_path = self.getEnvForLabel(max_edge, path);
        println!("lower_paths: {0:?}", lower_paths);
        println!("max_path: {0:?}", max_path);
        self.shadow(max_path, lower_paths)
    }

    fn getEnvForLabel(
        &self,
        edge: &Edge<Lbl>,
        path: Path<Lbl>,
    ) -> Vec<QueryResult<Lbl, Data>> {
        // println!("get env for label: {:?} ({})", edge, path);
        let path_to_edge = path.clone().step(edge.label.clone(), edge.to);
        // println!("checking match: {:?} ({})", edge, path_to_edge);
        if self.path_re.is_match(&path_to_edge.as_lbl_vec()) && self.scope_data_wfd(path_to_edge.target())
        {
            println!("Matching path found: {}", path);
            return vec![QueryResult {
                path: path.step(edge.label.clone(), edge.to),
                data: self.get_scope(edge.to).unwrap().data.clone()
            }]
        }
        // if self.path_re.is_match(&path.as_lbl_vec()) && self.scope_data_wfd(path.target()) {
        // }

        let source_scope = self.get_scope(path.target()).unwrap();
        source_scope
        .edges
        .clone()
        .into_iter()
        .flat_map(|e| {
            let path_to_edge = path.clone().step(e.label, e.to);
            self.resolve(path_to_edge)
        })
        .collect()
    }

    fn shadow(
        &self,
        mut a1: Vec<QueryResult<Lbl, Data>>,
        a2: Vec<QueryResult<Lbl, Data>>,
    ) -> Vec<QueryResult<Lbl, Data>> {
        let mut keep_a2 = a2.into_iter()
        .filter(|qr2| {
            !a1.iter().any(|qr1| (self.data_eq)(&qr1.data, &qr2.data))
        })
        .collect::<Vec<_>>();

        a1.append(&mut keep_a2);
        println!("a1: {0:?}", a1);
        a1
    }

    fn get_scope(&self, scope: Scope) -> Option<&ScopeData<Lbl, Data>> {
        self.scope_graph.scopes.get(&scope)
    }

    fn scope_data_wfd(&self, s: Scope) -> bool {
        let scope = self.get_scope(s).expect("Scope not found");
        (self.data_wfd)(&scope.data)
    }

}