use std::{collections::{HashSet, LinkedList}, rc::Rc};

use crate::{MatchableLabel, Scope, ScopeGraph};

const CHAIN_LABELS: &[MatchableLabel] = &[MatchableLabel::Parent, MatchableLabel::ExtendImpl];

#[derive(Clone, Debug)]
struct ChainScope {
    s: Scope,
    parent: Option<Rc<ChainScope>>,
}

pub struct ChainScopeIter<'a> {
    current: Option<&'a ChainScope>,
}

impl<'a> Iterator for ChainScopeIter<'a> {
    type Item = &'a Scope;

    fn next(&mut self) -> Option<Self::Item> {
        let cur = self.current?;
        self.current = cur.parent.as_deref();
        Some(&cur.s)
    }
}


#[derive(Clone, Debug)]
pub struct ChainMatch {
    nodes: ChainScope,
}

impl ChainMatch {
    pub fn from_scope(scope: Scope) -> Self {
        ChainMatch {
            nodes: ChainScope {s: scope, parent: None},
        }
    }

    pub fn tail(&self) -> Scope {
        self.nodes.s
    }

    pub fn step(self, scope: Scope) -> Self {
        Self {
            nodes: ChainScope {
                s: scope,
                parent: Some(Rc::new(self.nodes)),
            },
        }
    }

    pub fn len(&self) -> usize {
        let mut count = 0;
        let mut current = &self.nodes;
        while let Some(parent) = &current.parent {
            count += 1;
            current = parent;
        }
        count + 1 // include the tail node
    }

    pub fn scopes(&self) -> impl Iterator<Item = &Scope> {
        ChainScopeIter {
            current: Some(&self.nodes),
        }
    }

    pub fn to_vec(&self) -> Vec<Scope> {
        let mut s = self.scopes().copied().collect::<Vec<_>>();
        s.reverse();
        s
    }

    // pub fn contains(&self, node: &Scope) -> bool {
    //     self.nodes.contains(node)
    // }

    // pub fn extend(&mut self, other: ChainMatch) {
    //     self.nodes.extend(other.nodes);
    // }
}

/// Find all trees in the graph
pub fn find_chain(graph: &ScopeGraph) -> Vec<ChainMatch> {
    let mut matches = Vec::<ChainMatch>::new();

    let scopes = &graph.scopes;
    let mut available_scopes  = scopes.iter().cloned().collect::<HashSet<_>>();

    for s in scopes {
        if !available_scopes.contains(s) {
            // already found a match for this scope, skip it
            continue;
        }

        let new_matches = find_chains_recursive(graph, *s);

        for m in new_matches {
            for s in m.scopes() {
                available_scopes.remove(s);
            }
            matches.push(m);
        }
    }

    matches
}

/// Find a tree starting in `cur_scope`
fn find_chains_recursive(graph: &ScopeGraph, cur_scope: Scope) -> Vec<ChainMatch> {
    let mut cur_matches = vec![ChainMatch::from_scope(cur_scope)];
    let mut finished = Vec::new();

    while let Some(m) = cur_matches.pop() {
        let outgoing_edges = graph.get_outgoing_edges_with_labels(m.tail(), CHAIN_LABELS).collect::<Vec<_>>();

        match outgoing_edges.len() {
            // leaf node
            0 => {
                if m.len() > 1 {
                    // only add matches with more than one node
                    finished.push(m);
                }
            }
            _ => {
                for edge in outgoing_edges {
                    cur_matches.push(m.clone().step(edge.to));
                }
            }
        }
    }
    finished
}