use std::{collections::HashMap, marker::PhantomData};

use plantuml::{PlantUmlItem};

use crate::{data::ScopeGraphData, graph::{BaseScopeGraph, ScopeMap}, label::ScopeGraphLabel, order::LabelOrder, path::Path, regex::dfs::RegexAutomata, resolve::QueryResult, scope::Scope};

use super::ScopeGraph;


/// Cache for bottom-up resolution
///
/// Every scope holds a map of Data -> Path (to the data)
///
/// This completely caches every declaration, meaning that the
/// query resolution does not have to traverse the graph at all.
/// Every scope has complete information on all data visible data.
#[allow(type_alias_bounds)]
type BottomupCache<Lbl: ScopeGraphLabel, Data: ScopeGraphData>
    = HashMap<Scope, Vec<(Data, Path<Lbl>)>>;

// full caching
#[derive(Default)]
pub struct BottomupScopeGraph<Lbl, Data>
where
    Lbl: ScopeGraphLabel,
    Data: ScopeGraphData,
{
    sg: BaseScopeGraph<Lbl, Data>,
    data_cache: BottomupCache<Lbl, Data>,
}

impl<'s, Lbl, Data> ScopeGraph<'s, Lbl, Data> for BottomupScopeGraph<Lbl, Data>
where
    Lbl: ScopeGraphLabel,
    Data: ScopeGraphData,
{
    fn add_scope(&mut self, scope: Scope, data: Data) {
        self.sg.add_scope(scope, data);
    }

    fn add_edge(&mut self, source: Scope, target: Scope, label: Lbl) {
        self.sg.add_edge(source, target, label.clone());

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
        let data_scope = self.sg.add_decl(source, label.clone(), data.clone());
        let path = Path::start(data_scope)
        .step_back(label, source);

        let scope_entry = self.data_cache.entry(source).or_default();
        scope_entry.push((data, path));
        data_scope
    }

    fn query<DEq, DWfd>(
        & self,
        scope: Scope,
        path_regex: & RegexAutomata<Lbl>,
        order: & LabelOrder<Lbl>,
        data_equiv: DEq,
        data_wellformedness: DWfd,
    ) -> Vec<QueryResult<Lbl, Data>>
    where
        DEq: for<'da, 'db> Fn(&'da Data, &'db Data) -> bool,
        DWfd: for<'da> Fn(&'da Data) -> bool {
        // self.print_cache();
        // println!("cache size: {}", self.cache_size());
        let cache_entry = self.data_cache.get(&scope).expect("Scope not found in cache");

        // all matching data and path regex
        let query_results = cache_entry
        .iter()
        .filter(|(d, _)| data_wellformedness(d))
        .filter(|(_, p)| path_regex.is_match(p.as_lbl_vec()))
        .map(|(d, p)| {
            QueryResult {
                path: p.clone(),
                data: d.clone()
            }
        })
        .collect::<Vec<_>>();

        // an environment is shadowed if another env exists that
        // - has equivalent data
        // - path is less

        let shadows = |qr1: &QueryResult<Lbl, Data>, qr2: &QueryResult<Lbl, Data>| {
            qr1 != qr2
            && data_equiv(&qr1.data, &qr2.data)
            && order.path_is_less(&qr1.path, &qr2.path)
        };

        // shadowing
        query_results
        .iter()
        .filter(|qr| {
            !query_results
            .iter()
            .any(|qr2| shadows(qr2, qr))
        })
        .cloned()
        .collect::<Vec<_>>()
    }

    fn scope_iter<'a>(&'a self) -> impl Iterator<Item = (&'a Scope, &'a super::ScopeData<Lbl, Data>)> where Lbl: 'a, Data: 'a {
        self.sg.scope_iter()
    }

    fn scope_holds_data(&self, scope: Scope) -> bool {
        self.sg.scope_holds_data(scope)
    }
}

impl<Lbl, Data> BottomupScopeGraph<Lbl, Data>
where
    Lbl: ScopeGraphLabel,
    Data: ScopeGraphData,
{
    pub fn new() -> Self {
        Self {
            sg: BaseScopeGraph::new(),
            data_cache: HashMap::new(),
        }
    }

    pub fn base(&self) -> &BaseScopeGraph<Lbl, Data> {
        &self.sg
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

    /// Returns cache size in bytes
    pub fn cache_size(&self) -> usize {
        self.data_cache.values().map(|v| {
            let mem_size = v.iter().map(|(_, p)| {
                std::mem::size_of::<Data>() + p.mem_size()
            }).sum::<usize>();
            std::mem::size_of::<Scope>() + mem_size
        }).sum()
    }

    fn cache_uml<'a>(&self) -> Vec<PlantUmlItem>
    where Lbl: 'a, Data: 'a
    {
        self.data_cache
        .iter()
        .filter_map(|(scope, cache)| {
            if cache.is_empty() {
                return None;
            }

            let cache_str = cache.iter().map(|(d, p)| {
                format!("<b>{}</b>: {}", d, p)
            })
            .collect::<Vec<String>>()
            .join("\n");

            Some(
                PlantUmlItem::note(scope.0, cache_str)
            )
        })
        .collect::<Vec<_>>()
    }
}



#[cfg(test)]
mod tests {

    use crate::{order::LabelOrderBuilder, regex::Regex};

    use super::*;

    #[test]
    fn test_bug() {
        // let order = LabelOrderBuilder::new()
        // .push(Label::Declaration, Label::Parent)
        // .build();

        // // P*D;
        // let label_reg = Regex::concat(
        //     Regex::kleene(Label::Parent),
        //     Label::Declaration,
        // );
        // let matcher = RegexAutomata::from_regex(label_reg);

        // let data_equiv = |d1, d2| d1 == d2;
        // let data_wfd= |d| matches!(d, Data::Variable(x, t) if x == "x" && t == "int");

        // let query_results = vec![
        //     QueryResult {
        //         data: Data::var("x", "int"),
        //         path: Path::start(Scope(20))
        //         .step(Label::Parent, Scope(14))
        //         .step(Label::Declaration, Scope(15)),
        //     },
        //     QueryResult {
        //         data: Data::var("x", "int"),
        //         path: Path::start(Scope(20))
        //         .step(Label::Parent, Scope(14))
        //         .step(Label::Parent, Scope(6))
        //         .step(Label::Parent, Scope(4))
        //         .step(Label::Declaration, Scope(5)),
        //     },
        //     QueryResult {
        //         data: Data::var("x", "int"),
        //         path: Path::start(Scope(20))
        //         .step(Label::Parent, Scope(14))
        //         .step(Label::Parent, Scope(4))
        //         .step(Label::Parent, Scope(1))
        //         .step(Label::Declaration, Scope(2)),
        //     },
        // ];

        // let shadowed = query_results
        // .iter()
        // .filter(|qr| {
        //     !query_results
        //     .iter()
        //     .any(|qr2| {
        //         let is_different = *qr != qr2;
        //         let data_eq = data_equiv(&qr.data, &qr2.data);
        //         let order_less = order.path_is_less(&qr2.path, &qr.path);
        //         println!("{} {}", qr, qr2);
        //         println!("{} {} {}\n", is_different, data_eq, order_less);
        //         is_different && data_eq && order_less
        //     })
        // })
        // .cloned()
        // .collect::<Vec<_>>();

        // println!("shadowed:");
        // for s in shadowed {
        //     println!("{s}")
        // }
    }
}