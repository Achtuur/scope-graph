use crate::{SgData, SgLabel, graph::ScopeGraph, scope::Scope};

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
    /// Reverse tree pattern with number of levels,
    ReverseTree(usize),
    /// Join all children into a single node
    Join,
    Decl(SgData),
}

impl GraphPattern {
    pub fn add<G>(&self, graph: &mut G, child_scopes: Vec<Scope>) -> Vec<Scope>
    where
        G: ScopeGraph<SgLabel, SgData>,
    {
        match self {
            Self::Linear(length) => {
                let mut new_child_scopes = Vec::new();
                for child in child_scopes {
                    let mut cur_scope = child;
                    for _ in 0..*length {
                        let child_scope = graph.add_scope(Scope::new(), SgData::NoData);
                        graph.add_edge(child_scope, cur_scope, SgLabel::Parent);
                        cur_scope = child_scope
                    }
                    new_child_scopes.push(cur_scope);
                }
                new_child_scopes
            }
            Self::Diamond(diamond_size) => {
                let new_child_scopes = Self::Tree(*diamond_size).add(graph, child_scopes);
                Self::Join.add(graph, new_child_scopes)
            }
            Self::Decl(data) => {
                for child in &child_scopes {
                    let _ = graph.add_decl(*child, SgLabel::Declaration, data.clone());
                }
                child_scopes
            }
            Self::Tree(n_child) => {
                let mut new_child_scopes = Vec::new();
                for child in child_scopes {
                    let root = child;
                    for _ in 0..*n_child {
                        let child_scope = graph.add_scope(Scope::new(), SgData::NoData);
                        graph.add_edge(child_scope, root, SgLabel::Parent);
                        new_child_scopes.push(child_scope);
                    }
                }
                new_child_scopes
            }

            Self::Join => {
                let tail = graph.add_scope(Scope::new(), SgData::NoData);
                for child in child_scopes {
                    graph.add_edge(tail, child, SgLabel::Parent);
                }
                vec![tail]
            }
            Self::ReverseTree(levels) => {
                let mut child_scopes = child_scopes;
                while child_scopes.len() > 1 {
                    // let mut new_children = Vec::new();
                    let chunk_size = child_scopes.len() / levels;
                    child_scopes = child_scopes
                    .chunks(chunk_size.max(2)) // if chunk is 1 then nothing is reduced
                    .flat_map(|chunk| Self::Join.add(graph, chunk.to_vec()))
                    .collect();
                    // .for_each(|chunk| {
                    //     let new_tail = graph.add_scope_default();
                    //     for scope in chunk {
                    //         graph.add_edge(new_tail, *scope, SgLabel::Parent);
                    //     }
                    //     new_children.push(new_tail);
                    // });
                    // child_scopes = new_children;
                    // tree is reduced
                }
                child_scopes
            }
        }
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
        let root = self.graph.add_scope(Scope::new(), SgData::NoData);
        let mut child_scopes = vec![root];
        for pattern in self.patterns {
            child_scopes = pattern.add(&mut self.graph, child_scopes);
        }
        self.graph
    }
}
