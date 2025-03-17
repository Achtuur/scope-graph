use std::collections::HashMap;

use plantuml::{Color, EdgeDirection, NodeType, PlantUmlItem};

use crate::{data::ScopeGraphData, label::ScopeGraphLabel, scope::{self, Scope}};


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
            (s.0 == scope_num).then_some(*s)
        })
    }

    fn first_scope_without_data(&self, scope_num: usize) -> Option<Scope> {
        self.scopes.iter().find_map(|(s, d)| {
            if d.data.variant_has_data() {
                return None;
            }
            (s.0 >= scope_num).then_some(*s)
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

    fn scope_holds_data(&self, scope: Scope) -> bool {
        self.scopes.get(&scope).map(|d| d.data.variant_has_data()).unwrap_or_default()
    }

    pub fn as_uml(&self) -> Vec<PlantUmlItem> {
        let scope_nodes = self.scopes
        .iter()
        .map(|(s, d)| {
            let node_type = match d.data.variant_has_data() {
                true => NodeType::Card,
                false => NodeType::Node,
            };
            let contents = match d.data.variant_has_data() {
                true => d.data.render_string(),
                false => s.0.to_string(),
            };
            PlantUmlItem::node(s.uml_id(), contents, node_type)
        });

        let edges = self
        .scopes
        .iter()
        .flat_map(move |(s, d)| {
            d.edges.iter().map(move |edge| {
                let dir = match self.scope_holds_data(edge.to) {
                    true => EdgeDirection::Right,
                    false => EdgeDirection::Up,
                };

                PlantUmlItem::edge(s.uml_id(), edge.to.uml_id(), edge.label.str(), dir)
                .with_line_color(Color::Black)
            })
        });

        scope_nodes.chain(edges).collect()
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

    fn cache_uml<'a>(&'a self) -> Vec<PlantUmlItem>
    where Lbl: 'a, Data: 'a {
        Vec::new()
    }

    fn find_scope(&self, scope_num: usize) -> Option<Scope> {
        self.sg().find_scope(scope_num)
    }

    fn first_scope_without_data(&self, scope_num: usize) -> Option<Scope> {
        self.sg().first_scope_without_data(scope_num)
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

    fn as_uml<'a>(&'a self, display_cache: bool) -> Vec<PlantUmlItem>
    where Lbl: 'a, Data: 'a
    {
        let mut items = self.sg().as_uml();
        match display_cache {
            true => {
                items.extend(self.cache_uml());
                items
            },
            false => items,
        }
    }
}