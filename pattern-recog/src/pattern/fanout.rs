use crate::{
    MatchableLabel, Scope, ScopeGraph,
    pattern::{MatchedPattern, PatternMatcher},
};

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
}

impl MatchedPattern for FanoutMatch {
    fn size(&self) -> usize {
        self.leaves.len()
    }

    fn scopes(&self) -> impl Iterator<Item = &Scope> {
        std::iter::once(&self.root).chain(self.leaves.iter())
    }
}

pub struct FanoutMatcher;

impl PatternMatcher for FanoutMatcher {
    type Match = FanoutMatch;
    const EXCLUSIVE: bool = false;
    const NAME: &str = "Fanout";

    fn find_pattern_for_scope(graph: &ScopeGraph, scope: Scope) -> Vec<Self::Match> {
        let mut t = FanoutMatch::from_scope(scope);
        let outgoing_edges = graph.get_outgoing_edges_with_labels(scope, FANOUT_LABELS);
        for edge in outgoing_edges {
            t.push_leaf(edge.to);
        }

        match t.leaves.len() {
            0..=1 => Vec::new(),
            2.. => vec![t],
        }
    }
}

// pub fn find_fanout(graph: &ScopeGraph) -> Vec<FanoutMatch> {
//     let mut matches = Vec::<FanoutMatch>::new();

//     let scopes = &graph.scopes;
//     let mut available_scopes  = scopes.iter().cloned().collect::<HashSet<_>>();

//     for s in &graph.scopes {
//         // if !available_scopes.contains(s) {
//         //     // already found a match for this scope, skip it
//         //     continue;
//         // }

//         let new_matches = find_fanout_recursive(graph, *s);

//         for m in new_matches {
//             // for s in m.scopes() {
//             //     available_scopes.remove(s);
//             // }
//             matches.push(m);
//         }
//     }

//     matches
// }

// fn find_fanout_recursive(graph: &ScopeGraph, cur_scope: Scope) -> Vec<FanoutMatch> {
//     let mut t = FanoutMatch::from_scope(cur_scope);
//     let outgoing_edges = graph.get_outgoing_edges_with_labels(cur_scope, FANOUT_LABELS);
//     for edge in outgoing_edges {
//         t.push_leaf(edge.to);
//     }

//     match t.leaves.len() {
//         0..=1 => Vec::new(),
//         2.. => vec![t],
//     }
// }
