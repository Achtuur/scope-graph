use std::{collections::HashMap, sync::{Arc, Mutex, MutexGuard}};

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

impl<Lbl, Data> BaseScopeGraphHaver<Lbl, Data> for ForwardScopeGraph<'_, Lbl, Data> 
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
    pub fn query(
        &'s self,
        scope: Scope,
        path_regex: &'s RegexAutomata<Lbl>,
        order: &'s LabelOrder<Lbl>,
        data_equiv: impl Fn(&Data, &Data) -> bool,
        data_wellformedness: impl Fn(&Data) -> bool,
    ) -> Vec<QueryResult<Lbl, Data>> {
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
