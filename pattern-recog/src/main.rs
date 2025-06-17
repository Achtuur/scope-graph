use std::collections::HashMap;

use graphing::{plantuml::{EdgeDirection, NodeType, PlantUmlDiagram, PlantUmlItem}, Renderer};


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Node(usize);

struct Graph {
    nodes: Vec<Node>,
    edges: Vec<(Node, Node)>,
}

impl Graph {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }

    pub fn add_node(&mut self, node: usize) {
        self.nodes.push(Node(node));
    }

    pub fn add_edge(&mut self, from: usize, to: usize) {
        self.edges.push((Node(from), Node(to)));
    }

    pub fn from_edges(edges: impl IntoIterator<Item = (usize, usize)>) -> Self {
        let mut graph = Self::new();
        for (from, to) in edges {
            if !graph.nodes.contains(&Node(from)) {
                graph.add_node(from);
            }

            if !graph.nodes.contains(&Node(to)) {
                graph.add_node(to);
            }
            graph.add_edge(from, to);
        }
        graph
    }

    pub fn match_subgraph(&self, pattern: Pattern) -> Vec<Vec<vf2::NodeIndex>> {
        let pattern_graph = pattern.subgraph();
        let vf2 = vf2::subgraph_isomorphisms(&pattern_graph, self);

        pattern.prune_matches(vf2.iter())
        // let set = vf2.iter().fold(HashSet::new(), |mut acc, iso| {
        //     let mut iso = iso;
        //     iso.sort();
        //     acc.insert(iso);
        //     acc
        // });

        // set
    }

    pub fn diagram(&self) -> PlantUmlDiagram {
        let mut diagram = PlantUmlDiagram::new("graph");
        for node in &self.nodes {
            diagram.push(PlantUmlItem::node(node.0, node.0, NodeType::Node));
        }
        for (from, to) in &self.edges {
            diagram.push(PlantUmlItem::edge(from.0, to.0, "", EdgeDirection::Up));
        }
        diagram
    }
}

impl vf2::Graph for Graph {
    type NodeLabel = usize;

    type EdgeLabel = usize;

    fn is_directed(&self) -> bool {
        true
    }

    fn node_count(&self) -> usize {
        self.nodes.len()
    }

    fn node_label(&self, idx: vf2::NodeIndex) -> Option<&Self::NodeLabel> {
        self.nodes.get(idx).map(|n| &n.0)
    }

    fn neighbors(&self, node: vf2::NodeIndex, direction: vf2::Direction) -> impl Iterator<Item = vf2::NodeIndex> {
        self.edges.iter()
        .filter(move |(from, to)| match direction {
            vf2::Direction::Outgoing => from.0 == node,
            vf2::Direction::Incoming => to.0 == node,
        })
        .map(move |(from, to)| match direction {
            vf2::Direction::Outgoing => to.0,
            vf2::Direction::Incoming => from.0,
        })
    }

    fn contains_edge(&self, source: vf2::NodeIndex, target: vf2::NodeIndex) -> bool {
        self.edges.iter().any(|(from, to)| from.0 == source && to.0 == target)
    }

    fn edge_label(&self, source: vf2::NodeIndex, target: vf2::NodeIndex) -> Option<&Self::EdgeLabel> {
        self.edges.iter()
            .find(|(from, to)| from.0 == source && to.0 == target)
            .map(|(from, to)| &to.0)
    }
}

enum Pattern {
    Cycle(usize),
    Diamond(usize),
    Tree(usize),
    Chain(usize),
}

impl Pattern {
    pub fn subgraph(&self) -> Graph {
        match self {
            Self::Cycle(n) => {
                let mut graph = Graph::new();
                for i in 0..*n {
                    graph.add_node(i);
                    graph.add_edge(i, (i + 1) % n);
                }
                graph
            },
            Self::Diamond(n) => {
                let mut graph = Graph::new();
                graph.add_node(0);
                graph.add_node(n + 1);
                for i in 1..=*n {
                    graph.add_node(i);
                    graph.add_edge(i, 0);
                    graph.add_edge(n + 1, i);
                }
                graph
            },
            Self::Tree(n) => {
                let mut graph = Graph::new();
                graph.add_node(0);
                for i in 1..=*n {
                    graph.add_node(i);
                    graph.add_edge(i, 0);
                }
                graph
            },
            Self::Chain(n) => {
                let mut graph = Graph::new();
                graph.add_node(0);
                for i in 1..*n {
                    graph.add_node(i);
                    graph.add_edge(i - 1, i);
                }
                graph
            },
        }
    }

    /// Prunes matches to remove matches that are actually the same match.
    ///
    /// Depends on the pattern on how exactly to do this.
    pub fn prune_matches(&self, matches: impl IntoIterator<Item = Vec<vf2::NodeIndex>>) -> Vec<Vec<vf2::NodeIndex>> {
        matches.into_iter().fold(HashMap::new(), |mut acc, iso| {
            // matches are all same length, so if sum is the same, then they contain the same nodes
            // this preserves the order of the nodes in the match as well
            let sum = iso.iter().sum::<usize>();
            acc.insert(sum, iso);
            acc
        })
        .into_values()
        .collect::<Vec<_>>()
    }
}


fn main() {
    let graph = Graph::from_edges([
        (0, 1),
        (1, 2),
        (2, 3),
        (3, 4),
        (4, 0),
        // (0, 2),
        (4, 5),
        (5, 0),
        // diamond 1
        (0, 6),
        (0, 7),
        (6, 2),
        (7, 2),
        // diamond 2
        (0, 8),
        (0, 9),
        (8, 3),
        (9, 3),
    ]);
    graph.diagram().render_to_file("output/graph.puml").unwrap();

    let pattern = Pattern::Tree(2);
    pattern.subgraph().diagram().render_to_file("output/pattern.puml").unwrap();
    let matches = graph.match_subgraph(pattern);

    println!("set: {0:?}", matches);
}

