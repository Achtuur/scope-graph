use std::collections::HashMap;

use crate::{
    MatchableLabel, Scope, ScopeGraph,
    pattern::{MatchedPattern, PatternMatcher},
};

// const DIAMOND_LABELS: &[MatchableLabel] = &[];
const DIAMOND_LABELS: &[MatchableLabel] = &[MatchableLabel::ExtendImpl];

#[derive(Debug)]
pub struct DiamondMatch {
    bot: Scope,
    top: Scope,
    middle: Vec<Scope>,
    // root: Scope,
    // leaves: Vec<Scope>,
}

impl DiamondMatch {
    pub fn new(bot: Scope, top: Scope, middle: Vec<Scope>) -> Self {
        Self { bot, top, middle }
    }

    pub fn push_leaf(&mut self, leaf: Scope) {
        self.middle.push(leaf);
    }
}

impl MatchedPattern for DiamondMatch {
    fn size(&self) -> usize {
        self.middle.len()
    }

    fn scopes(&self) -> impl Iterator<Item = &Scope> {
        std::iter::once(&self.bot)
            .chain(std::iter::once(&self.top))
            .chain(self.middle.iter())
    }
}

pub struct DiamondMatcher;

impl PatternMatcher for DiamondMatcher {
    type Match = DiamondMatch;
    const EXCLUSIVE: bool = false;
    const NAME: &str = "Diamond";

    fn find_pattern_for_scope(graph: &ScopeGraph, scope: Scope) -> Vec<Self::Match> {
        let outgoing_edges = graph.get_outgoing_edges_with_labels(scope, DIAMOND_LABELS);
        let middle_scopes = outgoing_edges.map(|edge| edge.to);
        // map of top scope -> [middle scopes]
        let top_scopes: HashMap<Scope, Vec<Scope>> =
            middle_scopes.fold(HashMap::new(), |mut acc, scope| {
                let outgoing_edges = graph.get_outgoing_edges_with_labels(scope, DIAMOND_LABELS);
                for e in outgoing_edges {
                    acc.entry(e.to).or_default().push(scope);
                }
                acc
            });

        top_scopes
            .into_iter()
            .filter(|(_, middle_scopes)| middle_scopes.len() > 1)
            .map(|(top, middle_scopes)| DiamondMatch::new(scope, top, middle_scopes))
            .collect()
    }
}

// pub fn find_diamond(graph: &ScopeGraph) -> Vec<DiamondMatch> {
//     let mut matches = Vec::<DiamondMatch>::new();

//     let scopes = &graph.scopes;
//     let mut available_scopes  = scopes.iter().cloned().collect::<HashSet<_>>();

//     for s in &graph.scopes {
//         // if !available_scopes.contains(s) {
//         //     // already found a match for this scope, skip it
//         //     continue;
//         // }

//         let new_matches = find_diamond_recursive(graph, *s);

//         for m in new_matches {
//             // for s in m.scopes() {
//             //     available_scopes.remove(s);
//             // }
//             matches.push(m);
//         }
//     }

//     matches
// }

// fn find_diamond_recursive(graph: &ScopeGraph, cur_scope: Scope) -> Vec<DiamondMatch> {

//     // assume cur_scope = bottom
//     // get all outgoing edges with labels
//     // get all outgoing edges of those scopes
//     // any scope that appears in at least 2 of them is a top

//     let outgoing_edges = graph.get_outgoing_edges_with_labels(cur_scope, DIAMOND_LABELS);
//     let middle_scopes = outgoing_edges.map(|edge| edge.to);
//     // map of top scope -> [middle scopes]
//     let top_scopes: HashMap<Scope, Vec<Scope>> = middle_scopes.fold(HashMap::new(), |mut acc, scope| {
//         let outgoing_edges = graph.get_outgoing_edges_with_labels(scope, DIAMOND_LABELS);
//         for e in outgoing_edges {
//             acc.entry(e.to).or_default().push(scope);
//         }
//         acc
//     });

//     top_scopes
//     .into_iter()
//     .filter(|(_, middle_scopes)| middle_scopes.len() > 1)
//     .map(|(top, middle_scopes)| {
//         DiamondMatch::new(cur_scope, top, middle_scopes)
//     })
//     .collect()

//     // let mut t = DiamondMatch::from_scope(cur_scope);
//     // let outgoing_edges = graph.get_outgoing_edges_with_labels(cur_scope, DIAMOND_LABELS);
//     // for edge in outgoing_edges {
//     //     t.push_leaf(edge.to);
//     // }

//     // match t.middle.len() {
//     //     0..=1 => Vec::new(),
//     //     2.. => vec![t],
//     // }
// }
