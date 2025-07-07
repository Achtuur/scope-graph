
use std::{
    cell::LazyCell, collections::{HashMap, HashSet}, fs::OpenOptions, io::{BufWriter, Write}, sync::{Arc, LazyLock}
};

use data_parse::{JavaLabel, ParsedScopeGraph};
use graphing::{
    plantuml::{EdgeDirection, NodeType, PlantUmlDiagram, PlantUmlItem}, Color, Renderer
};
use serde::{Deserialize, Serialize};

use crate::{stat::Stats, MatchableLabel, ScopeGraph};

mod chain;
mod tree;
mod fanout;
pub use chain::*;
pub use fanout::*;
pub use tree::*;


#[derive(Debug)]
pub struct PatternMatches {
    chain_matches: Vec<ChainMatch>,
    fanout_matches: Vec<FanoutMatch>,
    tree_matches: Vec<TreeMatch>,
}

impl PatternMatches {
    pub fn from_graph(graph: &ScopeGraph) -> Self {
        let timer = std::time::Instant::now();
        let chain_matches = find_chain(graph);
        println!("chain: {:?}", timer.elapsed());
        let timer = std::time::Instant::now();
        let fanout_matches = find_fanout(graph);
        println!("fanout: {:?}", timer.elapsed());
        let timer = std::time::Instant::now();
        let tree_matches = find_tree(graph);
        println!("tree: {:?}", timer.elapsed());

        Self {
            chain_matches,
            fanout_matches,
            tree_matches,
        }
    }
}

impl std::fmt::Display for PatternMatches {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let chain_stats = self.chain_matches.iter().map(|m| m.len()).collect::<Stats>();
        let fanout_stats = self.fanout_matches.iter().map(|m| m.len()).collect::<Stats>();
        let tree_stats = self.tree_matches.iter().map(|m| m.len()).collect::<Stats>();

        writeln!(f, "Chain: {}", chain_stats)?;
        writeln!(f, "Fanout: {}", fanout_stats)?;
        writeln!(f, "Tree: {}", tree_stats)?;

        Ok(())
    }
}


#[derive(Clone, Debug)]
pub enum Pattern {
    Cycle(usize),
    Diamond(usize),
    Tree(usize),
    Chain(usize),
    FanOut(usize),
}

impl Pattern {
    pub fn subgraph(&self) -> ScopeGraph {
        let mut graph = ScopeGraph::new();
        match self {
            Self::Cycle(n) => {
                for i in 0..*n {
                    graph.add_node(i);
                    graph.add_edge(i, (i + 1) % n);
                }
            }
            Self::Diamond(n) => {
                graph.add_node(0);
                graph.add_node(n + 1);
                for i in 1..=*n {
                    graph.add_node(i);
                    graph.add_edge_labeled(0, i, MatchableLabel::ExtendImpl); // classes implement interface
                    graph.add_edge_labeled(i, n + 1, MatchableLabel::ExtendImpl); // interface extends another class (usually object)
                }
            }
            Self::Tree(n) => {
                graph.add_node(0);
                for i in 1..=*n {
                    graph.add_node(i);
                    graph.add_edge_labeled(i, 0, MatchableLabel::Parent);
                }
            }
            Self::Chain(n) => {
                graph.add_node(0);
                for i in 1..*n {
                    graph.add_node(i);
                    graph.add_edge_labeled(i - 1, i, MatchableLabel::Parent);
                }
            }
            Self::FanOut(n) => {
                graph.add_node(0);
                for i in 1..=*n {
                    graph.add_node(i);
                    graph.add_edge_labeled(0, i, MatchableLabel::ClassMember);
                }
            }
        }
        graph
    }

    pub fn file_name(&self) -> String {
        match self {
            Self::Cycle(n) => format!("cycle_{n}.puml"),
            Self::Diamond(n) => format!("diamond_{n}.puml"),
            Self::Tree(n) => format!("tree_{n}.puml"),
            Self::Chain(n) => format!("chain_{n}.puml"),
            Self::FanOut(n) => format!("fanout_{n}.puml"),
        }
    }

    /// Prunes matches to remove matches that are actually the same match.
    ///
    /// Depends on the pattern on how exactly to do this.
    pub fn prune_matches(
        &self,
        matches: impl IntoIterator<Item = Vec<vf2::NodeIndex>>,
    ) -> impl Iterator<Item = Vec<vf2::NodeIndex>> {
        matches
            .into_iter()
            .fold(HashMap::new(), |mut acc, iso| {
                // matches are all same length, so if sum is the same, then they contain the same nodes
                // this preserves the order of the nodes in the match as well
                let sum = iso.iter().sum::<usize>();
                acc.insert(sum, iso);
                acc
            })
            .into_values()
    }
}
