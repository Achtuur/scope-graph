use std::{
    collections::{HashMap, HashSet},
    sync::Mutex,
};

use crate::{
    data::ScopeGraphData,
    label::{LabelOrEnd, ScopeGraphLabel},
    order::LabelOrder,
    path::Path,
    regex::{dfs::RegexAutomata, PartialRegex},
    scope::Scope,
};

use super::{BaseScopeGraph, QueryResult, ScopeData, ScopeGraph};

#[derive(Hash, PartialEq, Eq, Debug)]
pub struct CacheKey<Lbl>
where
    Lbl: ScopeGraphLabel,
{
    pub(super) scope: Scope,
    pub(super) lbl_order: LabelOrder<Lbl>,
    pub(super) path_re: RegexAutomata<Lbl>,
}

impl<Lbl> std::fmt::Display for CacheKey<Lbl>
where
    Lbl: ScopeGraphLabel,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{{{}, {}, {}}}",
            self.scope, self.lbl_order, self.path_re
        )
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

    pub fn resolve(&mut self, path: Path<Lbl>) -> Vec<QueryResult<Lbl, Data>> {
        tracing::info!("Resolving path: {}", path);
        let reg = PartialRegex::new(self.path_re);
        let mut envs = self.resolve_all(path.clone(), reg);
        // only keep envs that are well-formed
        envs.retain(|qr| self.data_wfd(&qr.data));
        envs
    }

    /// recursive call site for resolving
    fn resolve_all<'a: 'r>(
        &mut self,
        path: Path<Lbl>,
        reg: PartialRegex<'a, Lbl>,
    ) -> Vec<QueryResult<Lbl, Data>> {
        // println!("Resolving path: {}", path);
        self.considered_paths.lock().unwrap().push(path.clone());
        self.get_env(path, reg)
    }

    pub fn print_cache(&self) {
        // println!("Resolver cache:");
        // for (k, v) in self.scope_graph.resolve_cache.lock().unwrap().iter() {
        //     println!("{}: [", k);
        //     for qr in &v.envs {
        //         println!("\t{}", qr);
        //     }
        //     println!("]");
        // }
    }

    // todo: allow overload of data_wfd
    fn data_wfd(&self, data: &Data) -> bool {
        (self.data_wfd)(data)
    }

    fn get_env(
        &mut self,
        path: Path<Lbl>,
        reg: PartialRegex<'r, Lbl>,
    ) -> Vec<QueryResult<Lbl, Data>> {
        // all edges where brzozowski derivative != 0
        let scope = self.get_scope(path.target()).expect("Scope not found");
        let mut labels = scope
            .parents()
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

        if labels.is_empty() {
            // if no labels are found, we are at the end of the path
            labels.push(LabelOrEnd::End(reg));
        }
        // labels.push(LabelOrEnd::End);

        self.get_env_for_labels(&labels, path)
    }

    fn get_env_for_labels<'a>(
        &mut self,
        labels: &'a [LabelOrEnd<'r, Lbl>],
        // reg: PartialRegex<'r, Lbl>,
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

    fn get_shadowed_env<'a>(
        &mut self,
        max_lbl: &'a LabelOrEnd<'r, Lbl>,
        lower_lbls: &'a [LabelOrEnd<'r, Lbl>],
        path: Path<Lbl>,
    ) -> Vec<QueryResult<Lbl, Data>> {
        let lower_paths = self.get_env_for_labels(lower_lbls, path.clone());
        let max_path = self.get_env_for_label(max_lbl, path);
        // println!("lower_paths: {0:?}", lower_paths);
        // println!("max_path: {0:?}", max_path);
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
            LabelOrEnd::End(reg) => {
                // don't check wfd here
                match reg.is_accepting() {
                    true => vec![QueryResult {
                        path,
                        data: scope.data.clone(),
                    }],
                    false => Vec::new(),
                }
                // path check is not necessary since we check partial matches along the way
                // if self.path_re.is_match(path.as_lbl_vec())
                // {
                // return vec![QueryResult {
                //     path,
                //     data: scope.data.clone(),
                // }];
                // }
                // vec![]
            }
            // not yet at end
            LabelOrEnd::Label((label, partial_reg)) => {
                scope
                    .parents()
                    .iter()
                    .filter(|e| e.lbl() == label)
                    .map(|e| path.clone().step(e.lbl().clone(), e.target())) // create new paths
                    .flat_map(|p| self.resolve_all(p, partial_reg.clone())) // resolve new paths
                    .collect::<Vec<_>>()
            }
        }
    }

    fn shadow(
        &self,
        mut a1: Vec<QueryResult<Lbl, Data>>,
        mut a2: Vec<QueryResult<Lbl, Data>>,
    ) -> Vec<QueryResult<Lbl, Data>> {
        tracing::trace!("Shadowing...");
        a2.retain(|qr2| !a1.iter().any(|qr1| (self.data_eq)(&qr1.data, &qr2.data)));

        a1.append(&mut a2);
        a1
    }

    fn get_scope(&self, scope: Scope) -> Option<&ScopeData<Lbl, Data>> {
        self.scope_graph.scopes().get(&scope)
    }

    fn scope_data_wfd(&self, s: Scope) -> bool {
        let scope = self.get_scope(s).expect("Scope not found");
        self.data_wfd(&scope.data)
    }
}
