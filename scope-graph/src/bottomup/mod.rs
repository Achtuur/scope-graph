use std::{collections::HashMap, marker::PhantomData};

use crate::{data::ScopeGraphData, graph::{BaseScopeGraph, BaseScopeGraphHaver, ScopeMap}, label::ScopeGraphLabel, order::LabelOrder, path::Path, regex::dfs::RegexAutomata, resolve::QueryResult, scope::Scope};


/// Cache for bottom-up resolution
///
/// Every scope holds a map of Data -> Path (to the data)
///
/// This completely caches every declaration, meaning that the
/// query resolution does not have to traverse the graph at all.
/// Every scope has complete information on all data visible data.
type BottomupCache<Lbl: ScopeGraphLabel, Data: ScopeGraphData>
    = HashMap<Scope, Vec<(Data, Path<Lbl>)>>;

// full caching
pub struct BottomupScopeGraph<'s, Lbl, Data>
where
    Lbl: ScopeGraphLabel,
    Data: ScopeGraphData,
{
    sg: BaseScopeGraph<Lbl, Data>,
    data_cache: BottomupCache<Lbl, Data>,
    // just make sure the lifetime and generics are always used
    _pd: &'s PhantomData<(Lbl, Data)>,
}


impl<Lbl, Data> BaseScopeGraphHaver<Lbl, Data> for BottomupScopeGraph<'_, Lbl, Data>
where
    Lbl: ScopeGraphLabel,
    Data: ScopeGraphData,
{
    fn sg(&self) -> &BaseScopeGraph<Lbl, Data> {
        &self.sg
    }

    fn sg_mut(&mut self) -> &mut BaseScopeGraph<Lbl, Data> {
        &mut self.sg
    }

    fn find_scope(&self, scope_num: usize) -> Option<Scope> {
        self.sg().find_scope(scope_num)
    }

    fn add_scope(&mut self, scope: Scope, data: Data) {
        self.sg_mut().add_scope(scope, data);
    }

    fn add_edge(&mut self, source: Scope, target: Scope, label: Lbl) {
        self.sg_mut().add_edge(source, target, label.clone());

        // child scope should inherit cache and extend path
        let target_cache = self.data_cache.get(&target).cloned().unwrap_or_default();
        let mut new_cache = target_cache
        .into_iter()
        .map(|(d, p)| {
            (d, p.step_back(label.clone(), source))
        })
        .collect::<Vec<_>>();

        match self.data_cache.entry(source) {
            std::collections::hash_map::Entry::Occupied(mut occupied_entry) => {
                occupied_entry.get_mut().append(&mut new_cache);
            },
            std::collections::hash_map::Entry::Vacant(vacant_entry) => {
                vacant_entry.insert(new_cache);
            },
        }
    }

    fn add_decl(&mut self, source: Scope, label: Lbl, data: Data) -> Scope {
        let data_scope = self.sg_mut().add_decl(source, label.clone(), data.clone());
        let path = Path::start(data_scope)
        .step_back(label, source);

        let scope_entry = self.data_cache.entry(source).or_default();
        scope_entry.push((data, path));
        data_scope
    }

    fn as_mmd(&self, title: &str) -> String {
        self.sg().as_mmd(title)
    }


}


impl<'s, Lbl, Data> BottomupScopeGraph<'s, Lbl, Data>
where
    Lbl: ScopeGraphLabel,
    Data: ScopeGraphData,
{
    pub fn new() -> Self {
        Self {
            sg: BaseScopeGraph::new(),
            data_cache: HashMap::new(),
            _pd: &PhantomData,
        }
    }

    pub fn scopes(&self) -> &ScopeMap<Lbl, Data> {
        &self.sg.scopes
    }

    pub fn print_cache(&self) {
        for (scope, data) in &self.data_cache {
            println!("Scope: {}", scope);
            for (d, p) in data {
                println!("  Data: {} Path: {}", d, p);
            }
        }
    }

    pub(crate) fn query(
        &'s self,
        scope: Scope,
        path_regex: &'s RegexAutomata<Lbl>,
        order: &'s LabelOrder<Lbl>,
        data_equiv: impl Fn(&Data, &Data) -> bool,
        data_wellformedness: impl Fn(&Data) -> bool,
    ) -> Vec<QueryResult<Lbl, Data>> {
        self.print_cache();
        let cache_entry = self.data_cache.get(&scope).expect("Scope not found in cache");

        // all matching data and path regex
        let query_results = cache_entry
        .iter()
        .filter(|(d, _)| data_wellformedness(d))
        .filter(|(_, p)| path_regex.is_match(&p.as_lbl_vec()))
        .map(|(d, p)| {
            QueryResult {
                path: p.clone(),
                data: d.clone()
            }
        })
        .collect::<Vec<_>>();

        for (idx, qr) in query_results.iter().enumerate() {
            println!("{}: {}", idx, qr);
        }

        // an environment is shadowed if another env exists that
        // - has equivalent data
        // - path is less

        // shadowing
        query_results
        .iter()
        .filter(|qr| {
            !query_results
            .iter()
            .any(|qr2| {
                *qr != qr2
                && data_equiv(&qr.data, &qr2.data)
                && order.path_is_less(&qr2.path, &qr.path)
            })
        })
        .cloned()
        .collect::<Vec<_>>()
    }
}
