use std::collections::HashSet;

use crate::{MatchableLabel, Scope, ScopeGraph};

const TREE_LABELS: &[MatchableLabel] = &[MatchableLabel::Parent, MatchableLabel::ExtendImpl];

#[derive(Debug)]
pub struct TreeMatch {
    root: Scope,
    leaves: Vec<Scope>,
}

impl TreeMatch {
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

pub fn find_tree(graph: &ScopeGraph) -> Vec<TreeMatch> {
    let mut matches = Vec::<TreeMatch>::new();

    let scopes = &graph.scopes;
    let mut available_scopes  = scopes.iter().cloned().collect::<HashSet<_>>();

    for s in &graph.scopes {
        // if !available_scopes.contains(s) {
        //     // already found a match for this scope, skip it
        //     continue;
        // }

        let new_matches = find_trees_recursive(graph, *s);

        for m in new_matches {
            // for s in m.scopes() {
            //     available_scopes.remove(s);
            // }
            matches.push(m);
        }
    }

    matches
}

fn find_trees_recursive(graph: &ScopeGraph, cur_scope: Scope) -> Vec<TreeMatch> {
    let mut t = TreeMatch::from_scope(cur_scope);
    let incoming_edges = graph.get_incoming_edges_with_labels(cur_scope, TREE_LABELS);
    for edge in incoming_edges {
        t.push_leaf(edge.from);
    }

    match t.leaves.len() {
        // if length is below 2, then it's just a chain
        0..=1 => Vec::new(),
        2.. => vec![t],
    }
}