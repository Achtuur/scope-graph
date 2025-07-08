use std::rc::Rc;

use crate::{
    pattern::{ChainScope, ChainScopeIter, MatchedPattern, PatternMatcher}, MatchableLabel, Scope, ScopeGraph
};

// const CHAIN_LABELS: &[MatchableLabel] = &[MatchableLabel::Parent, MatchableLabel::ExtendImpl];
const CHAIN_LABELS: &[MatchableLabel] = &[];

#[derive(Clone, Debug)]
pub struct CircleMatch {
    nodes: ChainScope,
}

impl CircleMatch {
    pub fn from_scope(scope: Scope) -> Self {
        CircleMatch {
            nodes: ChainScope {
                s: scope,
                parent: None,
            },
        }
    }

    pub fn tail(&self) -> Scope {
        self.nodes.s
    }

    pub fn contains(&self, c: &Scope) -> bool {
        self.scopes().any(|s| s == c)
    }

    pub fn step(self, scope: Scope) -> Self {
        Self {
            nodes: ChainScope {
                s: scope,
                parent: Some(Rc::new(self.nodes)),
            },
        }
    }

    pub fn to_vec(&self) -> Vec<Scope> {
        let mut s = self.scopes().copied().collect::<Vec<_>>();
        s.reverse();
        s
    }
}

impl MatchedPattern for CircleMatch {
    fn size(&self) -> usize {
        let mut count = 0;
        let mut current = &self.nodes;
        while let Some(parent) = &current.parent {
            count += 1;
            current = parent;
        }
        count + 1 // include the tail node
    }

    fn scopes(&self) -> impl Iterator<Item = &Scope> {
        ChainScopeIter {
            current: Some(&self.nodes),
        }
    }

    fn to_vec(&self) -> Vec<Scope> {
        let mut s = self.scopes().copied().collect::<Vec<_>>();
        s.reverse();
        s
    }
}

pub struct CircleMatcher;

impl PatternMatcher for CircleMatcher {
    type Match = CircleMatch;
    const EXCLUSIVE: bool = true;
    const NAME: &str = "Circle";

    /// Find all chains starting in `cur_scope`
    fn find_pattern_for_scope(graph: &ScopeGraph, cur_scope: Scope) -> Vec<Self::Match> {
        let mut cur_matches = vec![CircleMatch::from_scope(cur_scope)];
        let mut finished = Vec::new();

        while let Some(m) = cur_matches.pop() {
            let outgoing_edges = graph
                .get_outgoing_edges_with_labels(m.tail(), CHAIN_LABELS);
                // .peekable();

            // match outgoing_edges.peek() {
            //     // leaf node
            //     None => {
            //         if m.size() > MIN_SIZE {
            //             // only add matches with more than one node
            //             finished.push(m);
            //         }
            //     }
            //     _ => {
            //     }
            // }
            for edge in outgoing_edges {
                if !m.contains(&edge.to) {
                    cur_matches.push(m.clone().step(edge.to));
                } else {
                    finished.push(m.clone().step(edge.to));
                }
            }
        }
        finished
    }
}
