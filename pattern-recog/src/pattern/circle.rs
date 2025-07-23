use std::{collections::HashSet, rc::Rc};

use crate::{
    pattern::{ChainScope, ChainScopeIter, MatchedPattern, PatternMatcher}, MatchableLabel, Scope, ScopeGraph
};

// const CHAIN_LABELS: &[MatchableLabel] = &[MatchableLabel::Parent, MatchableLabel::ExtendImpl];
const CHAIN_LABELS: &[MatchableLabel] = &[];
const MIN_SIZE: usize = 2;

#[derive(Clone, Debug)]
pub struct CircleMatch {
    first: Scope,
    size: usize,
    nodes: ChainScope,
}

impl CircleMatch {
    pub fn from_scope(scope: Scope) -> Self {
        CircleMatch {
            first: scope,
            size: 1,
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
            first: self.first,
            size: self.size + 1,
            nodes: ChainScope {
                s: scope,
                parent: Some(Rc::new(self.nodes)),
            },
        }
    }

    pub fn is_circular(&self) -> bool {
        self.first == self.tail()
    }

    pub fn to_vec(&self) -> Vec<Scope> {
        let mut s = self.scopes().copied().collect::<Vec<_>>();
        s.reverse();
        s
    }
}

impl MatchedPattern for CircleMatch {
    fn size(&self) -> usize {
        self.size
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

            for edge in outgoing_edges {
                let step = m.clone().step(edge.to);
                if step.size() >= MIN_SIZE && step.is_circular() {
                    finished.push(step);
                } else if !m.contains(&edge.to) {
                    cur_matches.push(step);
                }

            }
        }
        finished
    }
}
