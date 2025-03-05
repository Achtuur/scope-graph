use std::collections::HashMap;

use crate::{data::ScopeGraphData, label::ScopeGraphLabel, scope::Scope};


#[derive(Clone, Copy, Debug)]
pub struct Edge<Lbl>
where
    Lbl: ScopeGraphLabel + Clone,
{
    pub to: Scope,
    pub label: Lbl,
}

#[derive(Clone, Debug)]
pub struct ScopeData<Lbl, Data>
where
    Lbl: ScopeGraphLabel + Clone,
{
    pub edges: Vec<Edge<Lbl>>,
    pub data: Data,
}

impl<Lbl, Data> ScopeData<Lbl, Data>
where
    Lbl: ScopeGraphLabel + Clone,
{
    pub fn new(data: Data) -> Self {
        Self {
            edges: Vec::new(),
            data,
        }
    }
}

pub type ScopeMap<Lbl, Data> = HashMap<Scope, ScopeData<Lbl, Data>>;

/// Base scope graph behaviour
///
/// Creation of scopes, does not implement query/caching logic
///
/// saves some duplication, to test faster
#[derive(Debug, Clone)]
pub struct BaseScopeGraph<Lbl, Data>
where
    Lbl: ScopeGraphLabel + Clone,
    Data: ScopeGraphData + Clone,
{
    pub scopes: HashMap<Scope, ScopeData<Lbl, Data>>,
}

impl<Lbl, Data> BaseScopeGraph<Lbl, Data>
where
    Lbl: ScopeGraphLabel + Clone,
    Data: ScopeGraphData + Clone,
{
    pub fn new() -> Self {
        Self {
            scopes: HashMap::new(),
        }
    }

    pub fn find_scope(&self, scope_num: usize) -> Option<Scope> {
        self.scopes.keys().find_map(|s| {
            if s.0 == scope_num {
                Some(*s)
            } else {
                None
            }
        })
    }

    pub fn add_scope(&mut self, scope: Scope, data: Data) {
        self.scopes.insert(scope, ScopeData::new(data));
    }

    pub fn add_edge(&mut self, source: Scope, target: Scope, label: Lbl) {
        let new_edge = Edge { to: target, label };

        self.scopes
            .get_mut(&source)
            .expect("Attempting to add edge to non-existant scope")
            .edges
            .push(new_edge);
    }

    /// Adds a declaration by adding a scope containing `data` and an edge.
    ///
    /// Returns the newly created scope
    pub fn add_decl(&mut self, source: Scope, label: Lbl, data: Data) -> Scope {
        let decl_scope = Scope::new();
        self.add_scope(decl_scope, data);
        self.add_edge(source, decl_scope, label);
        decl_scope
    }

    pub fn as_mmd(&self, title: &str) -> String {
        let mut mmd = format!(
            "---\n\
            title: \"{}\"\n\
            ---\n\
            flowchart LR\n\
            ",
            title
        );

        // for s in self.scopes.keys() {
        //     // todo, different node based on data
        //     mmd += &format!("\tscope_{}((\"{}\"))\n", s.0, s.0);
        // }

        for (s, d) in self.scopes.iter() {
            if d.data.variant_has_data() {
                mmd += &format!("\tscope_{}[\"{}\"]\n", s.0, d.data.render_string());
            } else {
                mmd += &format!("\tscope_{}((\"{}\"))\n", s.0, s.0);
            }
        }

        for (s, d) in self.scopes.iter() {
            for edge in d.edges.iter() {
                mmd += &format!(
                    "scope_{} ==>|\"{}\"| scope_{}\n",
                    s.0,
                    edge.label.str(),
                    edge.to.0
                )
            }
        }

        mmd
    }

    pub fn as_uml(&self) -> String {
        let scope_decls = self
        .scopes
        .iter()
        .map(|(s, d)| {
            match d.data.variant_has_data() {
                true => format!("card \"{1:}\" as scope_{0:}", s.0, d.data.render_string()),
                false => format!("usecase \"{0:}\" as scope_{0:}", s.0),
            }
        })
        .collect::<Vec<String>>()
        .join("\n");

        let edges = self
        .scopes
        .iter()
        .flat_map(|(s, d)| {
            d.edges.iter().map(|edge| {
                let target_has_data = self.scopes.get(&edge.to).unwrap().data.variant_has_data();
                let dir = match target_has_data {
                    true => "",
                    false => "u",
                };
                format!(
                    "scope_{0:} -{3:}-> scope_{1:} : {2:}",
                    s.0, edge.to.0, edge.label.str(), dir
                )
            })
        })
        .collect::<Vec<String>>()
        .join("\n");

        format!("{}\n{}\n", scope_decls, edges)
    }
}

/// trait to do auto implementations
///
/// this code is not good.
pub trait BaseScopeGraphHaver<Lbl, Data>
where
    Lbl: ScopeGraphLabel,
    Data: ScopeGraphData,
{
    fn sg(&self) -> &BaseScopeGraph<Lbl, Data>;
    fn sg_mut(&mut self) -> &mut BaseScopeGraph<Lbl, Data>;

    fn find_scope(&self, scope_num: usize) -> Option<Scope> {
        self.sg().find_scope(scope_num)
    }

    fn add_scope(&mut self, scope: Scope, data: Data) {
        self.sg_mut().add_scope(scope, data);
    }

    fn add_edge(&mut self, source: Scope, target: Scope, label: Lbl) {
        self.sg_mut().add_edge(source, target, label)
    }

    fn add_decl(&mut self, source: Scope, label: Lbl, data: Data) -> Scope {
        self.sg_mut().add_decl(source, label, data)
    }

    fn as_mmd(&self, title: &str) -> String {
        self.sg().as_mmd(title)
    }

    fn as_uml(&self) -> String {
        self.sg().as_uml()
    }
}