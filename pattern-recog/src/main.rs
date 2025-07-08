use data_parse::ParsedScopeGraph;
use graphing::Renderer;
use pattern_recog::{pattern::*, *};

fn main() {
    real_graph2();
    // test();
}

fn test() {
    // let graph = ScopeGraph::from_edges([
    //     (1, MatchableLabel::Parent, 0),
    //     (2, MatchableLabel::Parent, 0),
    //     (100, MatchableLabel::Parent, 1),
    //     (101, MatchableLabel::Parent, 1),
    //     (200, MatchableLabel::Parent, 2),
    //     (201, MatchableLabel::Parent, 2),
    //     (3, MatchableLabel::Parent, 2),
    //     (4, MatchableLabel::ClassMember, 3),
    //     (300, MatchableLabel::Parent, 3),
    //     (301, MatchableLabel::Parent, 3),
    //     (302, MatchableLabel::Parent, 3),
    // ]);

    let graph = ScopeGraph::from_edges([
        (1, MatchableLabel::ExtendImpl, 0),
        (2, MatchableLabel::ExtendImpl, 0),
        (3, MatchableLabel::ExtendImpl, 0),
        (4, MatchableLabel::ExtendImpl, 1),
        (4, MatchableLabel::ExtendImpl, 2),
        (4, MatchableLabel::ExtendImpl, 3),
        (4, MatchableLabel::ExtendImpl, 5),
        // (5, MatchableLabel::ExtendImpl, 4),
    ]);

    // let mut graph = ScopeGraph::new();
    // graph.add_node(0);
    // (1..250000).for_each(|i| {
    //     graph.add_node(i);
    //     graph.add_edge_labeled(i - 1, i, MatchableLabel::Parent);
    // });

    graph.diagram().render_to_file("output/graph.puml").unwrap();

    // let matches = graph.match_subgraph(&pattern, "test");
    // println!("found {0:?} matches", matches.len());
    let timer = std::time::Instant::now();
    let c_matches = CircleMatcher::search(&graph)
        .into_iter()
        .collect::<Vec<_>>();
    println!("{:?}", timer.elapsed());
    println!("c_matches: {0:?}", c_matches);
}

fn real_graph2() {
    println!("Parsing graph from file...");
    let mut graph =
        ParsedScopeGraph::from_file("data-parse/raw/commons-csv.scopegraph.json").unwrap();

    // graph.filter_scopes(|s| s.resource.contains("commons"));

    graph.scopes = graph.scopes.into_iter().collect();
    let searchable_graph = ScopeGraph::from(graph);

    let matches = PatternMatches::from_graph(&searchable_graph);
    println!("Matches: {}", matches);
}