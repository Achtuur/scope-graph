use rand::Rng;

use crate::{LibGraph, LibScope, SgData, SgLabel, graph::ScopeGraph, scope::Scope};

#[derive(Debug, Clone)]
pub enum GraphPattern {
    /// Diamond pattern alongside width and height
    ///
    ///    (0)
    ///   /   \
    /// (1) .. (N)
    ///   \   /
    ///   (N+1)
    Diamond(usize, usize),
    /// Single path straight line. This will always have at least 1 element
    ///
    /// (0)
    ///  |
    /// (1)
    ///  |
    /// (N)
    Linear(usize),
    /// Linear with random chance for declarations
    ///
    /// Name of variable is `x_{i}`, with i between 0 and length
    LinearDecl(usize),
    LinearDeclLabel(usize, SgLabel),
    LinearLabel(usize, SgLabel),
    /// Tree pattern with number of children,
    ///
    /// Each child of this tree will follow the rest of the path
    Tree(usize),
    /// Reverse tree pattern with number of levels,
    ReverseTree(usize),
    /// Join all children into a single node
    Join,
    Decl(SgData),
    Circle(usize),
}

unsafe impl Send for GraphPattern {}
unsafe impl Sync for GraphPattern {}

impl std::fmt::Display for GraphPattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Diamond(n, m) => write!(f, "diamond-{n}-{m}"),
            Self::Linear(n) => write!(f, "linear-{n}"),
            Self::LinearDecl(n) => write!(f, "linear-decl-{n}"),
            Self::LinearLabel(n, label) => write!(f, "linear-label-{n}-{label}"),
            Self::LinearDeclLabel(n, label) => write!(f, "linear-decl-label-{n}-{label}"),
            Self::Tree(n_child) => write!(f, "tree-{n_child}"),
            Self::ReverseTree(levels) => write!(f, "reverse-tree-{levels}"),
            Self::Join => write!(f, "join"),
            Self::Decl(data) => write!(f, "decl-{data}"),
            Self::Circle(size) => write!(f, "circle-{size}"),
        }
    }
}

impl GraphPattern {
    pub fn size(&self) -> usize {
        match self {
            Self::Diamond(width, height) => width * height + 1,
            Self::Linear(length) => length + 1,
            Self::LinearDecl(length) => length + 1,
            Self::LinearDeclLabel(length, _) => length + 1,
            Self::LinearLabel(length, _) => length + 1,
            Self::Tree(n_child) => *n_child,
            Self::ReverseTree(levels) => *levels,
            Self::Join => 1,
            Self::Decl(_) => 1,
            Self::Circle(size) => *size,
        }
    }

    pub fn n_child(&self) -> usize {
        match self {
            Self::Tree(n_child) => *n_child,
            _ => 1,
        }
    }

    pub fn add<G>(&self, graph: &mut G, child_scopes: Vec<Scope>) -> Vec<Scope>
    where
        G: ScopeGraph<SgLabel, SgData>,
    {
        match self {
            Self::Linear(length) => {
                Self::LinearLabel(*length, SgLabel::Parent).add(graph, child_scopes)
            }
            Self::LinearLabel(length, label) => {
                let mut new_child_scopes = Vec::new();
                for child in child_scopes {
                    let mut cur_scope = child;
                    for _ in 0..*length {
                        let child_scope = graph.add_scope_default();
                        graph.add_edge(child_scope, cur_scope, *label);
                        cur_scope = child_scope
                    }
                    new_child_scopes.push(cur_scope);
                }
                new_child_scopes
            }
            Self::LinearDecl(length) => {
                Self::LinearDeclLabel(*length, SgLabel::Parent).add(graph, child_scopes)
            }
            Self::LinearDeclLabel(length, label) => {
                let mut new_child_scopes = Vec::new();
                for child in child_scopes {
                    let mut cur_scope = child;
                    for i in 0..*length {
                        let child_scope = graph.add_scope_default();
                        graph.add_edge(child_scope, cur_scope, *label);
                        cur_scope = child_scope;

                        // if rng.random_bool(0.1) {
                            let decl_data = SgData::var(format!("x_{i}"), "int");
                            let _ = graph.add_decl(cur_scope, SgLabel::Declaration, decl_data);
                        // }
                    }
                    new_child_scopes.push(cur_scope);
                }
                new_child_scopes
            }
            Self::Diamond(width, height) => child_scopes
                .into_iter()
                .flat_map(|child| {
                    let tree_children = Self::Tree(*width).add(graph, vec![child]);
                    // minus one since tree already creates first layer
                    let height_children = Self::Linear(*height - 1).add(graph, tree_children);
                    Self::Join.add(graph, height_children)
                })
                .collect::<Vec<_>>(),
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
                        let child_scope = graph.add_scope_default();
                        graph.add_edge(child_scope, root, SgLabel::Parent);
                        new_child_scopes.push(child_scope);
                    }
                }
                new_child_scopes
            }

            Self::Join => {
                let tail = graph.add_scope_default();
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
                }
                child_scopes
            }

            Self::Circle(size) => {
                let mut new_children = Vec::new();
                for child in &child_scopes {
                    let first = *child;
                    let mut last = *child;
                    for i in 0..*size {
                        let child_scope = graph.add_scope_default();
                        graph.add_edge(child_scope, last, SgLabel::Parent);
                        last = child_scope;
                        if i == 0 {
                            new_children.push(child_scope);
                        }
                    }
                    graph.add_edge(first, last, SgLabel::Parent);
                    // new_children.push(first);
                }
                new_children
            }
        }
    }

    // /// Add a pattern to a scopegraphs::ScopeGraph
    // pub fn add_sg(&self, graph: &mut LibGraph, child_scopes: Vec<LibScope>) -> Vec<LibScope> {
    //     match self {
    //         Self::Linear(length) => {
    //             Self::LinearLabel(*length, SgLabel::Parent).add_sg(graph, child_scopes)
    //         }
    //         Self::LinearLabel(length, label) => {
    //             let mut new_child_scopes = Vec::new();
    //             for child in child_scopes {
    //                 let mut cur_scope = child;
    //                 for _ in 0..*length {
    //                     let child_scope = graph.add_scope_default();
    //                     graph.add_edge(child_scope, *label, cur_scope);
    //                     cur_scope = child_scope
    //                 }
    //                 new_child_scopes.push(cur_scope);
    //             }
    //             new_child_scopes
    //         }
    //         Self::Diamond(diamond_size) => child_scopes
    //             .into_iter()
    //             .flat_map(|child| {
    //                 let new_child_scopes = Self::Tree(*diamond_size).add_sg(graph, vec![child]);
    //                 Self::Join.add_sg(graph, new_child_scopes)
    //             })
    //             .collect::<Vec<_>>(),
    //         Self::Decl(data) => {
    //             for child in &child_scopes {
    //                 graph.add_decl(*child, SgLabel::Declaration, data.clone());
    //             }
    //             child_scopes
    //         }
    //         Self::Tree(n_child) => {
    //             let mut new_child_scopes = Vec::new();
    //             for root in child_scopes {
    //                 for _ in 0..*n_child {
    //                     let child_scope = graph.add_scope_default();
    //                     graph.add_edge(child_scope, SgLabel::Parent, root);
    //                     new_child_scopes.push(child_scope);
    //                 }
    //             }
    //             new_child_scopes
    //         }

    //         Self::Join => {
    //             let tail = graph.add_scope_default();
    //             for child in child_scopes {
    //                 graph.add_edge(tail, SgLabel::Parent, child);
    //             }
    //             vec![tail]
    //         }

    //         Self::ReverseTree(levels) => {
    //             let mut child_scopes = child_scopes;
    //             while child_scopes.len() > 1 {
    //                 // let mut new_children = Vec::new();
    //                 let chunk_size = child_scopes.len() / levels;
    //                 child_scopes = child_scopes
    //                     .chunks(chunk_size.max(2)) // if chunk is 1 then nothing is reduced
    //                     .flat_map(|chunk| Self::Join.add_sg(graph, chunk.to_vec()))
    //                     .collect();
    //             }
    //             child_scopes
    //         }

    //         Self::Circle(size) => {
    //             for child in &child_scopes {
    //                 let first = *child;
    //                 let mut last = graph.add_scope_default();
    //                 for _ in 0..*size {
    //                     let child_scope = graph.add_scope_default();
    //                     graph.add_edge(child_scope, SgLabel::Parent, last);
    //                     last = child_scope;
    //                 }
    //                 graph.add_edge(first, SgLabel::Parent, last);
    //             }
    //             child_scopes
    //         }
    //         _ => todo!("implement pattern for libgraph"),
    //     }
    // }
}

pub struct GraphGenerator<G> {
    patterns: Vec<GraphPattern>,
    graph: G,
}

impl<G> GraphGenerator<G>
where
    G: Default,
{
    pub fn from_pattern(pattern: GraphPattern) -> Self {
        Self {
            patterns: vec![pattern],
            graph: G::default(),
        }
    }

    pub fn from_pattern_iter(iter: impl IntoIterator<Item = GraphPattern>) -> Self {
        Self {
            patterns: iter.into_iter().collect(),
            graph: G::default(),
        }
    }
}

impl<G: Default> std::default::Default for GraphGenerator<G> {
    fn default() -> Self {
        Self {
            patterns: Vec::new(),
            graph: G::default(),
        }
    }
}

impl<G> GraphGenerator<G> {
    pub fn with_graph(graph: G) -> Self {
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
}

impl<G> GraphGenerator<G>
where
    G: ScopeGraph<SgLabel, SgData>,
{
    pub fn build(mut self) -> G {
        let root = self.graph.add_scope(Scope::new(), SgData::NoData);
        let mut child_scopes = vec![root];
        for pattern in self.patterns {
            child_scopes = pattern.add(&mut self.graph, child_scopes);
        }
        self.graph
    }

    /// Build and append to existing graph
    pub fn build_with_graph(mut self, mut graph: G, start_scope: Scope) -> G  {
        let root = self.graph.add_scope(start_scope, SgData::NoData);
        let mut child_scopes = vec![root];
        for pattern in self.patterns {
            child_scopes = pattern.add(&mut graph, child_scopes);
        }
        graph.extend(self.graph);
        graph.add_edge(root, start_scope, SgLabel::Parent);
        graph
    }
}

// impl<'storage> GraphGenerator<LibGraph<'storage>> {
//     pub fn build_sg(mut self) -> LibGraph<'storage> {
//         let root = self.graph.add_scope_default();
//         let mut child_scopes = vec![root];
//         for pattern in self.patterns {
//             child_scopes = pattern.add_sg(&mut self.graph, child_scopes);
//         }
//         self.graph
//     }
// }
