use std::collections::HashSet;

use crate::{MatchableLabel, Scope, ScopeGraph};

const FANOUT_LABELS: &[MatchableLabel] = &[MatchableLabel::ClassMember];

#[derive(Debug)]
pub struct FanoutMatch {
    root: Scope,
    leaves: Vec<Scope>,
}

impl FanoutMatch {
    pub fn from_scope(scope: Scope) -> Self {
        Self {
            root: scope,
            leaves: Vec::new(),
        }
    }

    pub fn push_leaf(&mut self, leaf: Scope) {
        self.leaves.push(leaf);
    }

    pub fn scopes(&self) -> impl Iterator<Item = &Scope> {
        std::iter::once(&self.root).chain(self.leaves.iter())
    }

    pub fn to_vec(&self) -> Vec<Scope> {
        let mut v = vec![self.root];
        v.extend(self.leaves.iter().copied());
        v
    }

    pub fn len(&self) -> usize {
        self.leaves.len()
    }
}

pub fn find_fanout(graph: &ScopeGraph) -> Vec<FanoutMatch> {
    let mut matches = Vec::<FanoutMatch>::new();

    let scopes = &graph.scopes;
    let mut available_scopes  = scopes.iter().cloned().collect::<HashSet<_>>();

    for s in &graph.scopes {
        // if !available_scopes.contains(s) {
        //     // already found a match for this scope, skip it
        //     continue;
        // }

        let new_matches = find_fanout_recursive(graph, *s);

        for m in new_matches {
            // for s in m.scopes() {
            //     available_scopes.remove(s);
            // }
            matches.push(m);
        }
    }

    matches
}

fn find_fanout_recursive(graph: &ScopeGraph, cur_scope: Scope) -> Vec<FanoutMatch> {
    let mut t = FanoutMatch::from_scope(cur_scope);
    let outgoing_edges = graph.get_outgoing_edges_with_labels(cur_scope, FANOUT_LABELS);
    for edge in outgoing_edges {
        t.push_leaf(edge.to);
    }

    match t.leaves.len() {
        0..=1 => Vec::new(),
        2.. => vec![t],
    }
}