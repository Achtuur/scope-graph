use std::{collections::{HashMap, HashSet}, path::is_separator, sync::Mutex};

use crate::{
    data::ScopeGraphData, label::{LabelOrEnd, ScopeGraphLabel}, order::{LabelOrder, LabelOrderBuilder}, path::Path, regex::dfs::RegexAutomata, scope::Scope, FORWARD_ENABLE_CACHING
};

use super::{BaseScopeGraph, QueryResult, ScopeData};

#[derive(Hash, PartialEq, Eq, Debug)]
pub struct CacheKey<Lbl>
where Lbl: ScopeGraphLabel,
{
    pub(super) scope: Scope,
    pub(super) lbl_order: LabelOrder<Lbl>,
    pub(super) path_re: RegexAutomata<Lbl>,
}

impl<Lbl> std::fmt::Display for CacheKey<Lbl>
where Lbl: ScopeGraphLabel
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{{}, {}, {}}}", self.scope, self.lbl_order, self.path_re)
    }
}

#[derive(Debug)]
pub struct CacheValue<Lbl, Data>
where
    Lbl: ScopeGraphLabel,
    Data: ScopeGraphData,
{
    pub(super) envs: Vec<QueryResult<Lbl, Data>>,
}

pub type ResolveCache<Lbl, Data> = HashMap<CacheKey<Lbl>, CacheValue<Lbl, Data>>;

pub struct Resolver<'r, Lbl, Data, DEq, DWfd>
where
    Lbl: ScopeGraphLabel,
    Data: ScopeGraphData,
    DEq: for<'da, 'db> Fn(&'da Data, &'db Data) -> bool,
    DWfd: for<'da> Fn(&'da Data) -> bool,

{
    // scopegraph contains cache
    pub scope_graph: &'r BaseScopeGraph<Lbl, Data>,
    pub path_re: &'r RegexAutomata<Lbl>,
    pub lbl_order: &'r LabelOrder<Lbl>,
    pub data_eq: DEq,
    pub data_wfd: DWfd,
    pub considered_paths: Mutex<Vec<Path<Lbl>>>,
}

impl<'r, Lbl, Data, DEq, DWfd> Resolver<'r, Lbl, Data, DEq, DWfd>
where
    Lbl: ScopeGraphLabel + Clone + std::fmt::Debug + std::fmt::Display + Eq + std::hash::Hash + Ord,
    Data: ScopeGraphData,
    DEq: for<'da, 'db> Fn(&'da Data, &'db Data) -> bool,
    DWfd: for<'da> Fn(&'da Data) -> bool,
{
    pub fn new(
        scope_graph: &'r BaseScopeGraph<Lbl, Data>,
        path_re: &'r RegexAutomata<Lbl>,
        lbl_order: &'r LabelOrder<Lbl>,
        data_eq: DEq,
        data_wfd: DWfd,
    ) -> Resolver<'r, Lbl, Data, DEq, DWfd> {
        Self {
            scope_graph,
            path_re,
            lbl_order,
            data_eq,
            data_wfd,
            considered_paths: Mutex::new(Vec::new()),
        }
    }

    pub fn resolve(&self, path: Path<Lbl>) -> Vec<QueryResult<Lbl, Data>> {
        // println!("Resolving path: {}", path);
        self.considered_paths.lock().unwrap().push(path.clone());
        self.get_env(path)
    }

    fn get_env(&self, path: Path<Lbl>) -> Vec<QueryResult<Lbl, Data>> {
        // all edges where brzozowski derivative != 0
        let scope = self.get_scope(path.target()).expect("Scope not found");

        let mut labels = scope
            .parents()
            .iter()
            .map(|e| e.lbl())
            // get unique labels by using hashset
            .fold(HashSet::new(), |mut set, lbl| {
                let mut label_vec = path.as_lbl_vec();
                label_vec.push(&lbl);
                if self.path_re.partial_match(label_vec) {
                    set.insert(LabelOrEnd::Label(lbl.clone()));
                }
                set
            })
            .into_iter()
            .collect::<Vec<_>>();
        labels.push(LabelOrEnd::End);

        self.get_env_for_labels(&labels, path)
    }

    fn get_env_for_labels(
        &self,
        labels: &[LabelOrEnd<Lbl>],
        // edges: &[&Edge<Lbl>],
        path: Path<Lbl>,
    ) -> Vec<QueryResult<Lbl, Data>> {
        let mut results = Vec::new();
        // println!("Resolving edges: {0:?}", labels);

        // 'max' labels ie all labels with lowest priority
        // max refers to the numerical worth, ie a < b, b would be max
        let max = labels
            .iter()
            .filter(|l1| !labels.iter().any(|l2| self.lbl_order.is_less(l1, l2)))
            .collect::<Vec<_>>();

        // println!("max: {0:?}", max);

        for max_lbl in max {
            // all labels that are lower priority than `lbl`
            let lower_labels = labels
                .iter()
                .filter(|l| self.lbl_order.is_less(l, max_lbl))
                .cloned()
                .collect::<Vec<_>>();

            // println!("lower: {0:?}", lower_labels);

            let env = self.get_shadowed_env(max_lbl, &lower_labels, path.clone());
            results.extend(env.into_iter());
        }

        results
    }

    fn get_shadowed_env(
        &self,
        max_lbl: &LabelOrEnd<Lbl>,
        lower_lbls: &[LabelOrEnd<Lbl>],
        path: Path<Lbl>,
    ) -> Vec<QueryResult<Lbl, Data>> {
        let lower_paths = self.get_env_for_labels(lower_lbls, path.clone());
        let max_path = self.get_env_for_label(max_lbl, path);
        // println!("lower_paths: {0:?}", lower_paths);
        // println!("max_path: {0:?}", max_path);
        self.shadow(lower_paths, max_path)
    }

    fn get_env_for_label(
        &self,
        label: &LabelOrEnd<Lbl>,
        path: Path<Lbl>,
    ) -> Vec<QueryResult<Lbl, Data>> {
        let scope = self.get_scope(path.target()).unwrap().clone();
        match label {
            // reached end of a path
            LabelOrEnd::End => {
                if self.path_re.is_match(path.as_lbl_vec()) && (self.data_wfd)(&scope.data) {
                    return vec![QueryResult {
                        path,
                        data: scope.data.clone(),
                    }];
                }
                vec![]
            }
            // not yet at end
            LabelOrEnd::Label(label) => {
                scope
                    .parents()
                    .iter()
                    .filter(|e| e.lbl() == label)
                    .map(|e| path.clone().step(e.lbl().clone(), e.target())) // create new paths
                    .flat_map(|p| self.resolve(p)) // resolve new paths
                    .collect::<Vec<_>>()
            }
        }
    }

    fn shadow(
        &self,
        mut a1: Vec<QueryResult<Lbl, Data>>,
        a2: Vec<QueryResult<Lbl, Data>>,
    ) -> Vec<QueryResult<Lbl, Data>> {
        let mut keep_a2 = a2
            .into_iter()
            .filter(|qr2| {
                !a1
                .iter()
                .any(|qr1| (self.data_eq)(&qr1.data, &qr2.data))
            })
            .collect::<Vec<_>>();

        a1.append(&mut keep_a2);
        a1
    }

    fn get_scope(&self, scope: Scope) -> Option<&ScopeData<Lbl, Data>> {
        self.scope_graph.scopes().get(&scope)
    }

    fn scope_data_wfd(&self, s: Scope) -> bool {
        let scope = self.get_scope(s).expect("Scope not found");
        (self.data_wfd)(&scope.data)
    }
}
