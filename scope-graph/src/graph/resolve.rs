use std::collections::HashSet;

use crate::{
    DRAW_MEM_ADDR,
    data::ScopeGraphData,
    label::{LabelOrEnd, ScopeGraphLabel},
    order::LabelOrder,
    path::{Path, ReversePath},
    regex::{RegexState, dfs::RegexAutomaton},
    scope::Scope,
};

use super::{BaseScopeGraph, ScopeData};
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueryResult<Lbl, Data>
where
    Lbl: ScopeGraphLabel + Clone,
    Data: ScopeGraphData,
{
    pub path: ReversePath<Lbl>,
    pub data: Data,
}

impl<Lbl, Data> std::fmt::Display for QueryResult<Lbl, Data>
where
    Lbl: ScopeGraphLabel,
    Data: ScopeGraphData,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match DRAW_MEM_ADDR {
            true => {
                write!(
                    f,
                    "{} > {}",
                    self.data.render_string(),
                    self.path.as_mem_addr()
                )
            }
            false => {
                write!(f, "{} > {}", self.data.render_string(), self.path)
            }
        }
    }
}

pub struct Resolver<'r, Lbl, Data, DEq, DWfd>
where
    Lbl: ScopeGraphLabel,
    Data: ScopeGraphData,
    DEq: for<'da, 'db> Fn(&'da Data, &'db Data) -> bool,
    DWfd: for<'da> Fn(&'da Data) -> bool,
{
    // scopegraph contains cache
    pub scope_graph: &'r BaseScopeGraph<Lbl, Data>,
    pub path_re: &'r RegexAutomaton<Lbl>,
    pub lbl_order: &'r LabelOrder<Lbl>,
    pub data_eq: DEq,
    pub data_wfd: DWfd,
}

impl<'r, Lbl, Data, DEq, DWfd> Resolver<'r, Lbl, Data, DEq, DWfd>
where
    Lbl: ScopeGraphLabel,
    Data: ScopeGraphData,
    DEq: for<'da, 'db> Fn(&'da Data, &'db Data) -> bool,
    DWfd: for<'da> Fn(&'da Data) -> bool,
{
    pub fn new(
        scope_graph: &'r BaseScopeGraph<Lbl, Data>,
        path_re: &'r RegexAutomaton<Lbl>,
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
        }
    }

    pub fn resolve(&mut self, path: Path<Lbl>) -> Vec<QueryResult<Lbl, Data>> {
        tracing::info!("Resolving path: {}", path);
        let reg = RegexState::new(self.path_re);
        self.resolve_all(path, reg)
    }

    /// recursive call site for resolving
    fn resolve_all<'a: 'r>(
        &mut self,
        path: Path<Lbl>,
        reg: RegexState<'a, Lbl>,
    ) -> Vec<QueryResult<Lbl, Data>> {
        self.get_env(path, reg)
    }

    fn data_wfd(&self, data: &Data) -> bool {
        (self.data_wfd)(data)
    }

    fn get_env(
        &mut self,
        path: Path<Lbl>,
        reg: RegexState<'r, Lbl>,
    ) -> Vec<QueryResult<Lbl, Data>> {
        let scope = self.get_scope(path.target()).expect("Scope not found");

        let mut labels = scope
            .outgoing()
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

        if reg.is_accepting() {
            labels.push(LabelOrEnd::End);
        }

        self.get_env_for_labels(&labels, path)
    }

    fn get_env_for_labels<'a>(
        &mut self,
        labels: &'a [LabelOrEnd<'r, Lbl>],
        path: Path<Lbl>,
    ) -> Vec<QueryResult<Lbl, Data>> {
        tracing::debug!("Resolving labels: {:?} for {:?}", labels, path.target());
        labels
            .iter()
            // 'max' labels ie all labels with lowest priority
            // max refers to the numerical worth, ie a < b, b would be max
            .filter(|l1| !labels.iter().any(|l2| self.lbl_order.is_less(l1, l2)))
            .flat_map(|max_lbl| {
                // all labels that are lower priority than `lbl`
                let lower_labels = labels
                    .iter()
                    .filter(|l| self.lbl_order.is_less(l, max_lbl))
                    .cloned()
                    .collect::<Vec<_>>();

                self.get_shadowed_env(max_lbl, &lower_labels, path.clone())
            })
            .collect::<Vec<_>>()
    }

    fn get_shadowed_env<'a>(
        &mut self,
        max_lbl: &'a LabelOrEnd<'r, Lbl>,
        lower_lbls: &'a [LabelOrEnd<'r, Lbl>],
        path: Path<Lbl>,
    ) -> Vec<QueryResult<Lbl, Data>> {
        let lower_paths = self.get_env_for_labels(lower_lbls, path.clone());
        let max_path = self.get_env_for_label(max_lbl, path);
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
            LabelOrEnd::End => match self.data_wfd(&scope.data) {
                true => vec![QueryResult {
                    path: ReversePath::from(path),
                    data: scope.data.clone(),
                }],
                false => Vec::new(),
            },
            // not yet at end
            LabelOrEnd::Label((label, partial_reg)) => {
                scope
                    .outgoing()
                    .iter()
                    .filter(|e| e.lbl() == label)
                    .map(|e| {
                        path.clone()
                            .step(e.lbl().clone(), e.target(), partial_reg.index())
                    })
                    .filter(|p| !p.is_circular(partial_reg.index()))
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
}
