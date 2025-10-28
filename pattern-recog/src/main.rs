use data_parse::ParsedScopeGraph;
use graphing::Renderer;
use pattern_recog::{pattern::*, *};

fn main() {
    real_graph();
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

    // let graph = ScopeGraph::from_edges([
    //     (0, MatchableLabel::ExtendImpl, 1),
    //     (0, MatchableLabel::ExtendImpl, 2),
    //     (0, MatchableLabel::ExtendImpl, 3),
    //     (1, MatchableLabel::ExtendImpl, 4),
    //     (2, MatchableLabel::ExtendImpl, 3),
    //     (3, MatchableLabel::ExtendImpl, 4),
    // ]);

    let graph = ScopeGraph::from_edges([
        (1, MatchableLabel::ExtendImpl, 0),
        (2, MatchableLabel::ExtendImpl, 0),
        (3, MatchableLabel::ExtendImpl, 0),
        (4, MatchableLabel::ExtendImpl, 1),
        (4, MatchableLabel::ExtendImpl, 2),
        (4, MatchableLabel::ExtendImpl, 3),
        (4, MatchableLabel::ExtendImpl, 5),
        (5, MatchableLabel::ExtendImpl, 6),
        (6, MatchableLabel::ExtendImpl, 4),
        (7, MatchableLabel::ExtendImpl, 4),
        (8, MatchableLabel::ExtendImpl, 4),
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
    for m in c_matches {
        println!("m: {0:?}", m)
    }
}

fn real_graph() {
    fn inner(path: &str, std_only: bool) -> PatternMatches {
        println!("Parsing graph from file...");
        let mut graph = ParsedScopeGraph::from_file(path).unwrap();

        if std_only {
            graph.filter_scopes(|s| !s.resource.contains("commons"));
        }

        graph.scopes = graph.scopes.into_iter().collect();
        let searchable_graph = ScopeGraph::from(graph);
        PatternMatches::from_graph(&searchable_graph)
    }
    let m_csv = inner("data-parse/raw/commons-csv-scopegraph.json", false);
    let m_io = inner("data-parse/raw/commons-io-scopegraph.json", false);
    let m_lang3 = inner("data-parse/raw/commons-lang-scopegraph.json", false);
    // let m_std = inner("data-parse/raw/commons-csv-scopegraph.json", true);

    let tab = [
        // m_std.to_latex_table("Java Standard Library"),
        m_csv.to_latex_table("Commons CSV"),
        m_io.to_latex_table("Commons IO"),
        m_lang3.to_latex_table("Commons Lang3"),
    ]
    .join("\n");
    println!("{}", tab);
}
