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

        let top_scopes: HashMap<Scope, Vec<Scope>> =
            middle_scopes.fold(HashMap::new(), |mut acc, middle_scope| {
                let outgoing_edges =
                    graph.get_outgoing_edges_with_labels(middle_scope, DIAMOND_LABELS);
                // level 1 diamond
                for top_edge in outgoing_edges {
                    acc.entry(top_edge.to).or_default().push(middle_scope);

                    // lvl 2 scopes, pretend that an edge from middle -> top2 exists
                    graph
                        .get_outgoing_edges_with_labels(top_edge.to, DIAMOND_LABELS)
                        .for_each(|next_top_edge| {
                            acc.entry(next_top_edge.to).or_default().push(middle_scope);

                            // // lvl 3, pretend edge from middle -> top3
                            // graph.get_outgoing_edges_with_labels(next_top_edge.to, DIAMOND_LABELS).for_each(|next2_top_edge| {
                            //     acc.entry(next2_top_edge.to).or_default().push(middle_scope);
                            // });
                        });
                }
                acc
            });

        // let outgoing_edges = graph.get_outgoing_edges_with_labels(scope, DIAMOND_LABELS);
        // let middle_scopes = outgoing_edges.map(|edge| edge.to);

        // // map of top scope -> [middle scopes]
        // let top_scopes: HashMap<Scope, Vec<Scope>> =
        //     middle_scopes.fold(HashMap::new(), |mut acc, scope| {
        //         let outgoing_edges = graph.get_outgoing_edges_with_labels(scope, DIAMOND_LABELS);
        //         for e in outgoing_edges {
        //             acc.entry(e.to).or_default().push(scope);
        //         }
        //         acc
        //     });

        top_scopes
            .into_iter()
            .filter(|(_, middle_scopes)| middle_scopes.len() > 1)
            .map(|(top, middle_scopes)| DiamondMatch::new(scope, top, middle_scopes))
            .collect()
    }
}
