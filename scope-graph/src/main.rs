use std::{
    io::{Write, stdin},
    sync::Arc,
};

use scope_graph::{
    ColorSet, DRAW_CACHES, ForeGroundColor, SAVE_GRAPH, SgData, SgLabel, SgProjection,
    generator::{GraphGenerator, GraphPattern},
    graph::{CachedScopeGraph, ScopeGraph},
    order::LabelOrderBuilder,
    regex::{Regex, dfs::RegexAutomaton},
};

pub type UsedScopeGraph<Lbl, Data> = CachedScopeGraph<Lbl, Data>;

fn graph_builder() -> UsedScopeGraph<SgLabel, SgData> {
    let graph = UsedScopeGraph::<SgLabel, SgData>::new();
    let patterns = [
        GraphPattern::Linear(1),
        GraphPattern::Decl(SgData::var("x", "int")),
        GraphPattern::Tree(7),
        GraphPattern::ReverseTree(3),
        // GraphPattern::Decl(SgData::var("x1", "int")),
        // GraphPattern::Decl(SgData::var("x2", "int")),
        // GraphPattern::Decl(SgData::var("x3", "int")),
        // GraphPattern::Decl(SgData::var("x4", "int")),
        // GraphPattern::Linear(3),
        // GraphPattern::Linear(1),
        // GraphPattern::Diamond(5),
        // GraphPattern::Decl(SgData::var("y", "int")),
        // GraphPattern::Linear(10),
    ];
    let graph = GraphGenerator::new(graph).with_patterns(patterns).build();
    graph
        .as_uml_diagram("graph", DRAW_CACHES)
        .write_to_file("output/output0.puml")
        .unwrap();
    graph
        .as_mmd_diagram("graph", DRAW_CACHES)
        .write_to_file("output/output0.md")
        .unwrap();
    graph
}

fn query_test(graph: &mut UsedScopeGraph<SgLabel, SgData>) {
    let order = LabelOrderBuilder::new()
        .push(SgLabel::Declaration, SgLabel::Parent)
        .build();

    // P*D;
    let label_reg = Regex::concat(
        Regex::concat(SgLabel::Parent, SgLabel::Parent),
        SgLabel::Declaration,
    );
    let matcher = RegexAutomaton::from_regex(label_reg.clone());
    matcher.to_uml().write_to_file("output/regex.puml").unwrap();

    let x_match: Arc<str> = Arc::from("y");
    let query_scope_set = [(x_match.clone(), vec![0, 1]), (x_match.clone(), vec![2])];

    for (idx, set) in query_scope_set.into_iter().enumerate() {
        let title = format!(
            "Query1: {}, label_reg={}, label_order={}, proj={}",
            0, label_reg, order, SgProjection::VarName
        );

        let p = set.0;
        let start_scopes = set.1;

        let (res_uml, res_mmd) = start_scopes
            .into_iter()
            .flat_map(|s| {
                let scope = graph.first_scope_without_data(s).unwrap();
                graph.query_proj(scope, &matcher, &order, SgProjection::VarName, p.clone())
            })
            .fold((Vec::new(), Vec::new()), |(mut uml_acc, mut mmd_acc), r| {
                let fg_class = ForeGroundColor::next_class();
                let uml = r.path.as_uml(fg_class.clone(), true);
                let mmd = r.path.as_mmd(fg_class, true);
                uml_acc.extend(uml);
                mmd_acc.extend(mmd);
                (uml_acc, mmd_acc)
            });

        // mmd
        // let cache_mmd = graph.cache_path_mmd(11);
        let mut mmd_diagram = graph.as_mmd_diagram(&title, DRAW_CACHES);
        // mmd_diagram.extend(cache_mmd);
        mmd_diagram.extend(res_mmd);

        // uml
        // let cache_uml = graph.cache_path_uml(11);
        let mut uml_diagram = graph.as_uml_diagram(&title, DRAW_CACHES);
        // uml_diagram.extend(cache_uml);
        uml_diagram.extend(res_uml);

        let fname = format!("output/output{}.md", idx);
        mmd_diagram.write_to_file(&fname).unwrap();
        uml_diagram.write_to_file(&fname).unwrap();
    }
}

fn circular_graph() -> UsedScopeGraph<SgLabel, SgData> {
    let mut graph = CachedScopeGraph::new();
    let s1 = graph.add_scope_default();
    let s2 = graph.add_scope_default();
    let s3 = graph.add_scope_default();
    graph.add_edge(s1, s2, SgLabel::Parent);
    graph.add_edge(s2, s1, SgLabel::Parent);
    graph.add_edge(s3, s1, SgLabel::Parent);
    let s4 = graph.add_decl(s1, SgLabel::Declaration, SgData::var("x", "int"));
    let s5 = graph.add_decl(s2, SgLabel::Declaration, SgData::var("y", "int"));
    graph
        .as_mmd_diagram("circular", DRAW_CACHES)
        .write_to_file("output/circular.md")
        .unwrap();
    graph
}

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    // slides_example();

    // let mut graph = graph_builder();
    let mut graph = circular_graph();
    query_test(&mut graph);

    if SAVE_GRAPH {
        println!("Type s or save to save the graph...");
        let mut input = String::new();
        stdin().read_line(&mut input).unwrap();
        if input.trim() == "s" || input.trim() == "save" {
            save_graph(&graph, "output/graph.json");
            println!("saved!");
        }
    }
}

fn save_graph(graph: &UsedScopeGraph<SgLabel, SgData>, fname: &str) {
    let file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(fname)
        .unwrap();
    tracing::info!("Writing to file {}", fname);
    serde_json::to_writer(file, graph).unwrap();
}
