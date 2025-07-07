use std::{
    io::{Write, stdin},
    sync::Arc,
};

use graphing::Renderer;
use scope_graph::{
    ColorSet, DRAW_CACHES, ForeGroundColor, SAVE_GRAPH, SgData, SgLabel, SgProjection,
    generator::{GraphGenerator, GraphPattern},
    graph::{CachedScopeGraph, ScopeGraph},
    order::LabelOrderBuilder,
    regex::{Regex, dfs::RegexAutomaton},
};

pub type UsedScopeGraph = CachedScopeGraph<SgLabel, SgData>;

fn graph_builder() -> UsedScopeGraph {
    let graph = UsedScopeGraph::new();
    let patterns = [
        GraphPattern::Linear(1),
        GraphPattern::Decl(SgData::var("x", "int")),
        GraphPattern::Tree(4),
        GraphPattern::ReverseTree(3),
        GraphPattern::Decl(SgData::var("x1", "int")),
        GraphPattern::Decl(SgData::var("x2", "int")),
        GraphPattern::Decl(SgData::var("x3", "int")),
        GraphPattern::Decl(SgData::var("x4", "int")),
        // GraphPattern::Linear(3),
        // GraphPattern::Linear(1),
        GraphPattern::Diamond(5),
        GraphPattern::Decl(SgData::var("y", "int")),
        GraphPattern::Decl(SgData::var("x", "int")),
        GraphPattern::Linear(10),
    ];
    let graph = GraphGenerator::new(graph).with_patterns(patterns).build();
    graph
        .as_uml_diagram("graph", DRAW_CACHES)
        .render_to_file("output/output0.puml")
        .unwrap();
    graph
        .as_mmd_diagram("graph", DRAW_CACHES)
        .render_to_file("output/output0.md")
        .unwrap();
    graph
}

fn query_test(graph: &mut UsedScopeGraph) {
    let order = LabelOrderBuilder::new()
        .push(SgLabel::Declaration, SgLabel::Parent)
        .build();

    // P*D;
    let label_reg = Regex::concat(Regex::kleene(SgLabel::Parent), SgLabel::Declaration);
    let matcher = RegexAutomaton::from_regex(label_reg.clone());
    matcher
        .to_uml()
        .render_to_file("output/regex.puml")
        .unwrap();
    matcher.to_mmd().render_to_file("output/regex.md").unwrap();

    let x_match: Arc<str> = Arc::from("y");
    let query_scope_set = [(x_match.clone(), vec![24]), (x_match.clone(), vec![30])];

    for (idx, set) in query_scope_set.into_iter().enumerate() {
        let title = format!(
            "Query sets {:?}, label_reg={}, label_order={}, proj={}",
            set,
            label_reg,
            order,
            SgProjection::VarName
        );

        let p = set.0;
        let start_scopes = set.1;
        let timer = std::time::Instant::now();
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

        println!("{:?}", timer.elapsed());
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
        mmd_diagram.render_to_file(&fname).unwrap();
        let fname = format!("output/output{}.puml", idx);
        uml_diagram.render_to_file(&fname).unwrap();
    }
}

fn circular_graph() -> UsedScopeGraph {
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
        .render_to_file("output/circular.md")
        .unwrap();
    graph
}

fn aron_example() {
    let mut graph = UsedScopeGraph::new();
    let s1 = graph.add_scope_default();
    let s2 = graph.add_scope_default();
    let s3 = graph.add_scope_default();
    let s4 = graph.add_scope_default();
    graph.add_edge(s1, s2, SgLabel::Parent);
    graph.add_edge(s2, s3, SgLabel::Parent);
    graph.add_edge(s3, s4, SgLabel::Parent);
    graph.add_edge(s4, s1, SgLabel::Parent);
    graph.add_decl(s1, SgLabel::Declaration, SgData::var("x", "int"));
    graph.add_decl(s3, SgLabel::Declaration, SgData::var("y", "int"));

    graph
        .as_uml_diagram("circle sg", true)
        .render_to_file("output/aron0.puml")
        .unwrap();

    let reg = Regex::concat(Regex::kleene(SgLabel::Parent), SgLabel::Declaration).compile();
    let label_order = LabelOrderBuilder::new()
        .push(SgLabel::Declaration, SgLabel::Parent)
        .build();
    let env = graph.query_proj(
        s2,
        &reg,
        &label_order,
        SgProjection::VarName,
        Arc::from("x"),
    );

    let mut diagram = graph.as_uml_diagram("circle sg 1st query", true);
    let q_uml = env
        .into_iter()
        .flat_map(|r| r.path.as_uml(ForeGroundColor::next_class(), true))
        .collect::<Vec<_>>();
    diagram.extend(q_uml);

    diagram.render_to_file("output/aron1.puml").unwrap();

    let env = graph.query_proj(
        s4,
        &reg,
        &label_order,
        SgProjection::VarName,
        Arc::from("y"),
    );

    let mut diagram = graph.as_uml_diagram("circle sg 2nd query", true);
    let q_uml = env
        .into_iter()
        .flat_map(|r| r.path.as_uml(ForeGroundColor::next_class(), true))
        .collect::<Vec<_>>();
    diagram.extend(q_uml);

    diagram.render_to_file("output/aron2.puml").unwrap();
}

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::WARN)
        .init();
    // aron_example();

    // return;

    // let mut graph = graph_builder();
    let mut graph = graph_builder();
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

fn save_graph(graph: &UsedScopeGraph, fname: &str) {
    let file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(fname)
        .unwrap();
    tracing::info!("Writing to file {}", fname);
    serde_json::to_writer(file, graph).unwrap();
}
