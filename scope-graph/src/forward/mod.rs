use std::{collections::HashMap, sync::{Arc, Mutex, MutexGuard}};

use plantuml::{PlantUmlItem};
use resolve::{ResolveCache, Resolver};

use crate::{
    data::ScopeGraphData, graph::{BaseScopeGraph, BaseScopeGraphHaver, ScopeData, ScopeMap}, label::ScopeGraphLabel, order::{LabelOrder, LabelOrderBuilder}, path::Path, regex::dfs::RegexAutomata, resolve::QueryResult, scope::Scope
};

mod resolve;



#[derive(Debug)]
pub struct ForwardScopeGraph<'s, Lbl, Data>
where
    Lbl: ScopeGraphLabel,
    Data: ScopeGraphData,
{
    sg: BaseScopeGraph<Lbl, Data>,
    // pub scopes: HashMap<Scope, ScopeData<Lbl, Data>>,
    pub(crate) resolve_cache: Mutex<ResolveCache<'s, Lbl, Data>>,
}

impl<'s, Lbl, Data> BaseScopeGraphHaver<Lbl, Data> for ForwardScopeGraph<'s, Lbl, Data>
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

    fn cache_uml<'a>(&'a self) -> Vec<PlantUmlItem>
    where Lbl: 'a, Data: 'a {
        self.resolve_cache
        .lock()
        .unwrap()
        .iter()
        .filter_map(|(key, value)| {
            if value.envs.is_empty() {
                return None;
            }

            let vals = value.envs.iter().map(|env| {
                env.to_string()
            })
            .collect::<Vec<String>>()
            .join("\n");

            let cache_str = format!("<b>{}</b>\n{}", key, vals);
            Some(
                PlantUmlItem::note(key.scope.0, cache_str)
            )
        })
        .collect::<Vec<_>>()
    }
}


impl<'s, Lbl, Data> ForwardScopeGraph<'s, Lbl, Data>
where
    Lbl: ScopeGraphLabel,
    Data: ScopeGraphData,
{
    pub fn new() -> Self {
        Self {
            sg: BaseScopeGraph::new(),
            resolve_cache: Mutex::new(HashMap::new()),
        }
    }

    pub fn from_base(sg: BaseScopeGraph<Lbl, Data>) -> Self {
        Self {
            sg,
            resolve_cache: Mutex::new(HashMap::new()),
        }
    }

    pub fn scopes(&self) -> &ScopeMap<Lbl, Data> {
        &self.sg.scopes
    }

    /// Returns the data associated with the scope baesd on path_regex and name of the data
    ///
    /// Data is now just a string, so the query returns `data_name` if succesful
    ///
    /// # Arguments
    ///
    /// * Scope: Starting scope
    /// * path_regex: Regular expression to match the path
    /// * data_name: Name of the data to return
    pub fn query<DEq, DWfd>(
        &'s self,
        scope: Scope,
        path_regex: &'s RegexAutomata<Lbl>,
        order: &'s LabelOrder<Lbl>,
        data_equiv: DEq,
        data_wellformedness: DWfd,
    ) -> Vec<QueryResult<Lbl, Data>>
    where
        DEq: for<'da, 'db> Fn(&'da Data, &'db Data) -> bool,
        DWfd: for<'da> Fn(&'da Data) -> bool,
    {
        let resolver = Resolver::new(
            self,
            path_regex,
            order,
            &data_equiv,
            &data_wellformedness,
        );
        let res = resolver.resolve(Path::start(scope));
        // resolver.print_cache();
        res
    }
}
