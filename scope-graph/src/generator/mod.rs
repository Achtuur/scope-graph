use std::marker::PhantomData;

use crate::{data::ScopeGraphData, graph::ScopeGraph, label::ScopeGraphLabel, scope::Scope, SgData, SgLabel};

pub enum GraphPattern {
    /// Diamond pattern alongside with of the path
    ///
    ///    (0)
    ///   /   \
    /// (1) .. (N)
    ///   \   /
    ///   (N+1)
    Diamond(usize),
    /// Single path straight line. This will always have at least 1 element
    ///
    /// (0)
    ///  |
    /// (1)
    ///  |
    /// (N)
    ///  |
    /// (N+1)
    Linear(usize),
    /// Tree pattern with number of children,
    ///
    /// Each child of this tree will follow the rest of the path
    Tree(usize),
    Decl(SgData),
}

impl GraphPattern {
    pub fn add<G>(&self, graph: &mut G, mut child_scopes: Vec<Scope>) -> Vec<Scope>
    where
        G: ScopeGraph<SgLabel, SgData>,
    {
        let mut new_child_scopes = Vec::new();

        if child_scopes.is_empty() {
            let root = graph.add_scope(Scope::new(), SgData::NoData);
            child_scopes.push(root);
        }

        for child in child_scopes {
            match self {
                Self::Linear(length) => {
                    let mut cur_scope = child;
                    for _ in 0..*length {
                        let child_scope = graph.add_scope(Scope::new(), SgData::NoData);
                        graph.add_edge(child_scope, cur_scope, SgLabel::Parent);
                        cur_scope = child_scope
                    }
                    new_child_scopes.push(cur_scope);
                },
                Self::Diamond(diamond_size) => {
                    let top_scope = child;
                    let bottom_scope = graph.add_scope(Scope::new(), SgData::NoData);
                    for _ in 0..*diamond_size {
                        let diamond_scope = graph.add_scope(Scope::new(), SgData::NoData);
                        graph.add_edge(diamond_scope, top_scope, SgLabel::Parent);
                        graph.add_edge(bottom_scope, diamond_scope, SgLabel::Parent);
                    }
                    new_child_scopes.push(bottom_scope)
                },
                Self::Tree(n_child) => {
                    let root = child;
                    for _ in 0..*n_child {
                        let child_scope = graph.add_scope(Scope::new(), SgData::NoData);
                        graph.add_edge(child_scope, root, SgLabel::Parent);
                        new_child_scopes.push(child_scope);
                    }
                },
                Self::Decl(data) => {
                    let _ = graph.add_decl(child, SgLabel::Declaration, data.clone());
                    new_child_scopes.push(child);
                }
            }
        }
        new_child_scopes
    }
}

pub struct GraphGenerator<G>
where
    G: ScopeGraph<SgLabel, SgData>,
{
    patterns: Vec<GraphPattern>,
    graph: G,
}

impl<G> GraphGenerator<G>
where
    G: ScopeGraph<SgLabel, SgData>,
{
    pub fn new(graph: G) -> Self {
        Self {
            patterns: Vec::new(),
            graph,
        }
    }

    pub fn with_pattern(mut self, pattern: GraphPattern) -> Self {
        self.patterns.push(pattern);
        self
    }

    pub fn with_patterns(mut self, iter: impl IntoIterator<Item = GraphPattern>) -> Self {
        self.patterns.extend(iter);
        self
    }

    pub fn push(&mut self, pattern: GraphPattern) {
        self.patterns.push(pattern);
    }

    pub fn build(mut self) -> G {
        let mut child_scopes = Vec::new();
        for pattern in self.patterns {
            child_scopes = pattern.add(&mut self.graph, child_scopes);
        }
        self.graph
    }
}