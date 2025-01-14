use std::cmp::Ordering;

use crate::{label::ScopeGraphLabel, order::LabelOrder, path::Path, regex::dfs::RegexAutomata, scope::Scope, scopegraph::{Edge, QueryResult, ScopeData, ScopeGraph}, Data, Label};

impl<Lbl, Data> ScopeGraph<Lbl, Data>
where
    Lbl: ScopeGraphLabel + Clone + std::fmt::Debug + PartialEq + Eq + std::hash::Hash,
    Data: std::fmt::Debug + Clone,
{
    pub fn query_statix(
        &self,
        scope: Scope,
        path_re: &RegexAutomata<Lbl>,
        order: &LabelOrder<Lbl>,
        data_wfd: impl Fn(&Data) -> bool
    ) -> Vec<QueryResult<Lbl, Data>>
    {
        todo!()
    }

    fn get_env(
        &self,
        scope: &ScopeData<Lbl, Data>,
        path: Path<Lbl>,
        path_re: &RegexAutomata<Lbl>,
        order: &LabelOrder<Lbl>,
        data_wfd: &impl Fn(&Data) -> bool
    ) -> Vec<QueryResult<Lbl, Data>>
    {
        // all edges where brzozowski derivative != 0
        let edges = scope.edges
        .iter()
        .filter(|edge| {
            let new_path = path.clone().step(edge.label.clone(), edge.to);
            path_re.partial_match(&new_path.as_lbl_vec())
        })
        .collect::<Vec<_>>();

        self.getEnvForLabels(&edges, path, path_re, order, data_wfd)
    }

    fn getEnvForLabels(
        &self,
        edges: &[&Edge<Lbl>],
        path: Path<Lbl>,
        path_re: &RegexAutomata<Lbl>,
        order: &LabelOrder<Lbl>,
        data_wfd: &impl Fn(&Data) -> bool
    ) -> Vec<QueryResult<Lbl, Data>>
    {
        let mut results = Vec::new();

        // 'max' label edges, ie all edges with highest priority label
        let max = edges
        .iter()
        .filter(|e| !edges.iter().any(|e2| order.cmp(&e.label, &e2.label).is_gt()));

        for max_edge in max {
            // all labels that are lower priority than `lbl`
            let lower_edges = edges.iter().filter_map(|e| {
                match order.cmp(&max_edge.label, &e.label).is_ge() {
                    true => Some(*e),
                    false => None,
                }
            })
            .collect::<Vec<_>>();
            let env = self.getShadowedEnv(max_edge, &lower_edges, path.clone(), path_re, order, data_wfd);
            results.extend(env.into_iter());
        }

        results
    }

    fn getShadowedEnv(
        &self,
        max_edge: &Edge<Lbl>,
        lower_edges: &[&Edge<Lbl>],
        path: Path<Lbl>,
        path_re: &RegexAutomata<Lbl>,
        order: &LabelOrder<Lbl>,
        data_wfd: &impl Fn(&Data) -> bool
    ) -> Vec<QueryResult<Lbl, Data>> {
        let lower_paths = self.getEnvForLabels(lower_edges, path.clone(), path_re, order, data_wfd);
        let max_path = self.getEnvForLabel(max_edge, path, path_re, order, data_wfd);
        max_path.shadow_vec(lower_paths)
    }

    fn getEnvForLabel(
        &self,
        edge: &Edge<Lbl>,
        path: Path<Lbl>,
        path_re: &RegexAutomata<Lbl>,
        order: &LabelOrder<Lbl>,
        data_wfd: &impl Fn(&Data) -> bool
    ) -> QueryResult<Lbl, Data> {
        if path_re.is_match(&path.as_lbl_vec()) {
            return QueryResult {
                path: path.step(edge.label.clone(), edge.to),
                data: self.scopes.get(&edge.to).unwrap().data.clone()
            }
        }

        todo!()
    }

}