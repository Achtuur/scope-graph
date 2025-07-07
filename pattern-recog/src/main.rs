use std::sync::Arc;

use data_parse::ParsedScopeGraph;
use graphing::Renderer;
use pattern_recog::{pattern::*, stat::Stats, *};


fn main() {
    real_graph2();
    // test();


    // let colors = [
    //     Color::RED,
    //     Color::GREEN,
    //     Color::BLUE,
    //     Color::YELLOW,
    //     Color::ORANGE,
    //     Color::PURPLE,
    //     Color::CYAN,
    // ];

    // let mut diagram = graph.diagram();
    // for (m_i, m) in matches.into_iter().enumerate() {
    //     let color = colors[m_i % colors.len()];
    //     for i in m.windows(2) {
    //         let item = PlantUmlItem::edge(Scope(i[0]), Scope(i[1]), m_i, EdgeDirection::Norank)
    //         .with_line_color(color);
    //         diagram.push(item);
    //     }
    // }
    // diagram.render_to_file("output/matches.puml").unwrap();

}

fn test() {
    let graph = ScopeGraph::from_edges([
        (1, MatchableLabel::Parent, 0),
        (2, MatchableLabel::Parent, 0),
        (100, MatchableLabel::Parent, 1),
        (101, MatchableLabel::Parent, 1),
        (200, MatchableLabel::Parent, 2),
        (201, MatchableLabel::Parent, 2),
        (3, MatchableLabel::Parent, 2),
        (4, MatchableLabel::ClassMember, 3),
        (300, MatchableLabel::Parent, 3),
        (301, MatchableLabel::Parent, 3),
        (302, MatchableLabel::Parent, 3),
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
    let c_matches = find_fanout(&graph).into_iter().collect::<Vec<_>>();
    println!("{:?}", timer.elapsed());
    println!("c_matches: {0:?}", c_matches);
}

fn real_graph2() {
    println!("Parsing graph from file...");
    let mut graph =
        ParsedScopeGraph::from_file("data-parse/raw/commons-csv.scopegraph.json").unwrap();

    graph.filter_scopes(|s| s.resource.contains("commons"));

    graph.scopes = graph.scopes.into_iter().collect();
    let searchable_graph = ScopeGraph::from(graph);

    let matches = PatternMatches::from_graph(&searchable_graph);
    println!("Matches: {}", matches);

}

fn real_graph() {
    println!("Parsing graph from file...");
    let mut graph =
        ParsedScopeGraph::from_file("data-parse/raw/commons-csv.scopegraph.json").unwrap();

    graph.filter_scopes(|s| s.resource.contains("commons"));

    graph.scopes = graph.scopes.into_iter().collect();
    let searchable_graph = Arc::from(ScopeGraph::from(graph));

    let patterns = [
        // cycles dont appear
        // Pattern::Cycle(2),
        // Pattern::Cycle(4),
        // Pattern::Cycle(8),
        // Pattern::Cycle(12),
        Pattern::Chain(4),
        Pattern::Chain(6),
        Pattern::Chain(8),
        Pattern::Chain(10),
        Pattern::Diamond(2),
        Pattern::Diamond(4),
        Pattern::Diamond(8),
        Pattern::FanOut(2),
        Pattern::FanOut(4),
        Pattern::FanOut(8),
        Pattern::Tree(2),
        Pattern::Tree(4),
        Pattern::Tree(8),
    ];

    println!("Matching subgraph...");
    let mut handles = Vec::new();
    for p in patterns {
        let s_graph_clone = searchable_graph.clone();
        let h = std::thread::spawn(move || {
            let timer = std::time::Instant::now();
            let matches = s_graph_clone.match_subgraph(&p, "commons-csv");
            println!("{:?}: {:?}", p, timer.elapsed());
            println!("{:?} matches: {:?}", p, matches.len());
        });
        handles.push(h);
    }

    for h in handles {
        h.join().unwrap();
    }

    // println!("Matching subgraph...");
    // let timer = std::time::Instant::now();
    // let matches = searchable_graph.match_subgraph(&pattern);
    // println!("{:?}", timer.elapsed());
    // println!("matches: {0:?}", matches.len());
}
