use std::{
    collections::{HashMap, HashSet},
    io::Write,
};

use indicatif::{ProgressBar, ProgressStyle};

use crate::{MatchableLabel, Scope, ScopeGraph, stat::Stats};

mod chain;
mod diamond;
mod fanout;
mod tree;
mod circle;
pub use chain::*;
pub use diamond::*;
pub use fanout::*;
pub use tree::*;
pub use circle::*;

#[derive(Debug)]
pub struct PatternMatches {
    chain_matches: Vec<ChainMatch>,
    fanout_matches: Vec<FanoutMatch>,
    tree_matches: Vec<TreeMatch>,
    diamond_matches: Vec<DiamondMatch>,
    circle_matches: Vec<CircleMatch>,
}

impl PatternMatches {
    pub fn from_graph(graph: &ScopeGraph) -> Self {
        let timer = std::time::Instant::now();
        let chain_matches = ChainMatcher::search(graph);
        // let chain_matches = Vec::new();
        println!("chain: {:?}", timer.elapsed());
        let timer = std::time::Instant::now();
        let fanout_matches = FanoutMatcher::search(graph);
        println!("fanout: {:?}", timer.elapsed());
        let timer = std::time::Instant::now();
        let tree_matches = TreeMatcher::search(graph);
        println!("tree: {:?}", timer.elapsed());
        let timer = std::time::Instant::now();
        let diamond_matches = DiamondMatcher::search(graph);
        println!("diamond: {:?}", timer.elapsed());
        let timer = std::time::Instant::now();
        let circle_matches = CircleMatcher::search(graph);
        println!("diamond: {:?}", timer.elapsed());

        Self {
            chain_matches,
            fanout_matches,
            tree_matches,
            diamond_matches,
            circle_matches,
        }
    }
}

impl std::fmt::Display for PatternMatches {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        macro_rules! size_stats {
            ($matches:expr) => {
                $matches
                    .iter()
                    .map(|m| m.size())
                    .collect::<Stats>()
            };
        }

        let chain_stats = size_stats!(self.chain_matches);
        let fanout_stats = size_stats!(self.fanout_matches);
        let tree_stats = size_stats!(self.tree_matches);
        let diamond_stats = size_stats!(self.diamond_matches);
        let circle_stats = size_stats!(self.circle_matches);

        writeln!(f, "Chain: {:?}", chain_stats)?;
        writeln!(f, "Fanout: {:?}", fanout_stats)?;
        writeln!(f, "Tree: {:?}", tree_stats)?;
        writeln!(f, "Diamond: {:?}", diamond_stats)?;
        writeln!(f, "Circle: {:?}", circle_stats)?;

        Ok(())
    }
}

pub trait MatchedPattern {
    /// Size of this pattern, depends on pattern what this size means.
    fn size(&self) -> usize;
    fn scopes(&self) -> impl Iterator<Item = &Scope>;

    fn to_vec(&self) -> Vec<Scope> {
        self.scopes().copied().collect()
    }
}

pub trait PatternMatcher {
    type Match: MatchedPattern;
    /// If true, then a scope can only be part of one match.
    const EXCLUSIVE: bool = false;
    const NAME: &str;

    fn find_pattern_for_scope(graph: &ScopeGraph, scope: Scope) -> Vec<Self::Match>;

    fn search(graph: &ScopeGraph) -> Vec<Self::Match> {
        let mut matches = Vec::<Self::Match>::new();

        let scopes = &graph.scopes;
        let bar = ProgressBar::new(scopes.len() as u64).with_message(Self::NAME);

        bar.set_style(
            ProgressStyle::with_template(
                "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
            )
            .unwrap()
            .progress_chars("##-"),
        );

        let mut available_scopes = scopes.iter().cloned().collect::<HashSet<_>>();

        for s in scopes {
            bar.inc(1);
            bar.set_message(format!("{} ({} matches)", Self::NAME, matches.len()));
            if Self::EXCLUSIVE && !available_scopes.contains(s) {
                // already found a match for this scope, skip it
                continue;
            }

            let new_matches = Self::find_pattern_for_scope(graph, *s);
            for m in new_matches {
                if Self::EXCLUSIVE {
                    for s in m.scopes() {
                        available_scopes.remove(s);
                    }
                }
                matches.push(m);
            }
        }

        bar.finish();
        matches
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
