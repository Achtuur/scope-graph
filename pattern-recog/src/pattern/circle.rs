use std::{collections::HashSet, rc::Rc};

use hashbrown::HashMap;

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
    const EXCLUSIVE: bool = false;
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



/// Returns (nodes_in_cycles, nodes_not_in_cycles)
/// Tarjan’s algorithm
/// I (with shame) asked chatgpt for this
pub fn find_cycle_nodes(graph: &ScopeGraph) -> (hashbrown::HashSet<Scope>, hashbrown::HashSet<Scope>) {
    let mut index = 0;
    let mut stack = Vec::new();
    let mut on_stack = hashbrown::HashSet::new();
    let mut indices = HashMap::new();
    let mut lowlink = HashMap::new();
    let mut cycles = hashbrown::HashSet::new();

    fn strongconnect(
        v: Scope,
        index: &mut i32,
        stack: &mut Vec<Scope>,
        on_stack: &mut hashbrown::HashSet<Scope>,
        indices: &mut HashMap<Scope, i32>,
        lowlink: &mut HashMap<Scope, i32>,
        graph: &ScopeGraph,
        cycles: &mut hashbrown::HashSet<Scope>,
    ) {
        indices.insert(v, *index);
        lowlink.insert(v, *index);
        *index += 1;
        stack.push(v);
        on_stack.insert(v);

        for edge in graph.get_outgoing_edges_with_labels(v, CHAIN_LABELS) {
            let w = edge.to;
            if !indices.contains_key(&w) {
                strongconnect(w, index, stack, on_stack, indices, lowlink, graph, cycles);
                let low_v = *lowlink.get(&v).unwrap();
                let low_w = *lowlink.get(&w).unwrap();
                lowlink.insert(v, low_v.min(low_w));
            } else if on_stack.contains(&w) {
                let low_v = *lowlink.get(&v).unwrap();
                let idx_w = *indices.get(&w).unwrap();
                lowlink.insert(v, low_v.min(idx_w));
            }
        }

        if indices[&v] == lowlink[&v] {
            // Start a new SCC
            let mut scc = Vec::new();
            loop {
                let w = stack.pop().unwrap();
                on_stack.remove(&w);
                scc.push(w);
                if w == v {
                    break;
                }
            }

            // If SCC has > 1 node, or a self-loop, it's a cycle
            if scc.len() > 1 {
                cycles.extend(scc);
            } else if graph.get_outgoing_edges_with_labels(&scc[0], CHAIN_LABELS).any(|e| e.to == scc[0]) {
                cycles.insert(scc[0]);
            }
        }
    }

    for &node in graph.keys() {
        if !indices.contains_key(&node) {
            strongconnect(
                node,
                &mut index,
                &mut stack,
                &mut on_stack,
                &mut indices,
                &mut lowlink,
                graph,
                &mut cycles,
            );
        }
    }

    let all_nodes: hashbrown::HashSet<Scope> = graph.keys().cloned().collect();
    let non_cycles = &all_nodes - &cycles;

    (cycles, non_cycles)
}