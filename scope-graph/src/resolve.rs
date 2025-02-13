use std::{collections::{HashMap, HashSet}, path::is_separator, sync::Mutex};

use crate::{
    label::{LabelOrEnd, ScopeGraphLabel},
    order::{LabelOrderBuilder, LabelOrder},
    path::Path,
    regex::dfs::RegexAutomata,
    scope::Scope,
    scopegraph::{QueryResult, ScopeData, ScopeGraph},
};

#[derive(Hash, PartialEq, Eq, Debug)]
pub(crate) struct CacheKey<'c, Lbl>
where Lbl: ScopeGraphLabel + Clone + std::fmt::Debug + Eq + std::hash::Hash,
{
    scope: Scope,
    lbl_order: &'c LabelOrder<Lbl>,
    path_re: &'c RegexAutomata<Lbl>,
}

impl<Lbl> std::fmt::Display for CacheKey<'_, Lbl>
where Lbl: ScopeGraphLabel + Clone + std::fmt::Display + Eq + std::hash::Hash + Ord
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{{}, {}, {}}}", self.scope, self.lbl_order, self.path_re)
    }
}

#[derive(Debug)]
pub(crate) struct CacheValue<Lbl, Data>
where
    Lbl: ScopeGraphLabel + Clone + std::fmt::Debug + Eq + std::hash::Hash + Ord,
    Data: std::fmt::Debug + Clone,
{
    envs: Vec<QueryResult<Lbl, Data>>,
}

pub(crate) type ResolveCache<'c, Lbl, Data> = HashMap<CacheKey<'c, Lbl>, CacheValue<Lbl, Data>>;

pub struct Resolver<'r, Lbl, Data, DEq, DWfd>
where
    Lbl: ScopeGraphLabel + Clone + std::fmt::Debug + Eq + std::hash::Hash + Ord,
    Data: std::fmt::Debug + Clone,
    DEq: for<'da, 'db> Fn(&'da Data, &'db Data) -> bool,
    DWfd: for<'da> Fn(&'da Data) -> bool,

{
    // scopegraph contains cache
    pub scope_graph: &'r ScopeGraph<'r, Lbl, Data>,
    pub path_re: &'r RegexAutomata<Lbl>,
    pub lbl_order: &'r LabelOrder<Lbl>,
    pub data_eq: DEq,
    pub data_wfd: DWfd,
    pub considered_paths: Mutex<Vec<Path<Lbl>>>,
}

impl<'r, Lbl, Data, DEq, DWfd> Resolver<'r, Lbl, Data, DEq, DWfd>
where
    Lbl: ScopeGraphLabel + Clone + std::fmt::Debug + std::fmt::Display + Eq + std::hash::Hash + Ord,
    Data: std::fmt::Debug + Clone,
    DEq: for<'da, 'db> Fn(&'da Data, &'db Data) -> bool,
    DWfd: for<'da> Fn(&'da Data) -> bool,
{
    pub fn new(
        scope_graph: &'r ScopeGraph<'r, Lbl, Data>,
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
        println!("Resolving path: {}", path);
        self.considered_paths.lock().unwrap().push(path.clone());
        let envs = self.get_env(path.clone());
        self.cache_env(&path, envs.clone());
        envs
    }

    fn cache_env(&self, path: &Path<Lbl>, mut envs: Vec<QueryResult<Lbl, Data>>) {
        let key = CacheKey {
            scope: path.target(),
            lbl_order: self.lbl_order,
            path_re: self.path_re,
        };
        // 'path' is path from start of query to current scope
        // envs contains the path from starting scope of query to target scope
        // the env should NOT contain part of the path that is in `path`

        envs.iter_mut().for_each(|env| {
            env.path = env.path.clone().trim_matching_start(path);
        });


        let val = CacheValue {
            envs,
        };
        println!("path: {0:?}", path);
        println!("Caching: {}: {:?}", key, val);

        self.scope_graph.resolve_cache.lock().unwrap().insert(key, val);
    }

    fn get_cached_env(&self, path: &Path<Lbl>) -> Option<Vec<QueryResult<Lbl, Data>>> {
        let key = CacheKey {
            scope: path.target(),
            lbl_order: self.lbl_order,
            path_re: self.path_re,
        };
        let cache = self.scope_graph.resolve_cache.lock().unwrap();
        let matching_envs = cache
        .get(&key)?
        .envs
        .iter()
        .filter(|qr| {
            // data EQ here? how?
            (self.data_wfd)(&qr.data) // data WFD should match whatever is in the cache
        })
        .map(|qr| {
            // append 'path' to 'qr.env'
            qr
        })
        .cloned()
        .collect::<Vec<_>>();
        match matching_envs.is_empty() {
            true => None,
            false => Some(matching_envs),
        }
    }

    fn get_env(&self, path: Path<Lbl>) -> Vec<QueryResult<Lbl, Data>> {
        // all edges where brzozowski derivative != 0
        let scope = self.get_scope(path.target()).expect("Scope not found");

        if let Some(env) = self.get_cached_env(&path) {
            println!("Cache hit for {}", path);
            return env;
        }

        let mut labels = scope
            .edges
            .iter()
            .map(|e| e.label.clone())
            // get unique labels by using hashset
            .fold(HashSet::new(), |mut set, lbl| {
                let mut label_vec = path.as_lbl_vec();
                label_vec.push(&lbl);
                if self.path_re.partial_match(&label_vec) {
                    set.insert(LabelOrEnd::Label(lbl));
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
                if self.path_re.is_match(&path.as_lbl_vec()) && (self.data_wfd)(&scope.data) {
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
                    .edges
                    .iter()
                    .filter(|e| &e.label == label)
                    .map(|e| path.clone().step(e.label.clone(), e.to)) // create new paths
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
        self.scope_graph.scopes.get(&scope)
    }

    fn scope_data_wfd(&self, s: Scope) -> bool {
        let scope = self.get_scope(s).expect("Scope not found");
        (self.data_wfd)(&scope.data)
    }
}

impl<'r, Lbl, Data, DEq, DWfd> Resolver<'r, Lbl, Data, DEq, DWfd>
where
    Lbl: ScopeGraphLabel + Clone + std::fmt::Display + Eq + std::hash::Hash + Ord,
    Data: std::fmt::Debug + Clone,
    DEq: for<'da, 'db> Fn(&'da Data, &'db Data) -> bool,
    DWfd: for<'da> Fn(&'da Data) -> bool,
{
    pub fn print_cache(&self) {
        println!("Resolver cache:");
        for (k, v) in self.scope_graph.resolve_cache.lock().unwrap().iter() {
            println!("{}: [", k);
            for qr in &v.envs {
                println!("\t{}", qr);
            }
            println!("]");
        }
    }
}