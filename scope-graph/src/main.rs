use std::{
    io::{Write, stdin},
    sync::Arc,
};

use scope_graph::{
    ColorSet, DRAW_CACHES, ForeGroundColor, SAVE_GRAPH, SgData, SgLabel,
    generator::{GraphGenerator, GraphPattern},
    graph::{CachedScopeGraph, ScopeGraph},
    order::LabelOrderBuilder,
    regex::{Regex, dfs::RegexAutomata},
    scope::Scope,
};

pub type UsedScopeGraph<'s, Lbl, Data> = CachedScopeGraph<Lbl, Data>;

fn slides_example() {
    let mut graph = UsedScopeGraph::new();
    let root = Scope::new();
    let scope1 = Scope::new();
    let scope2 = Scope::new();
    let scope3 = Scope::new();
    let scope4 = Scope::new();
    let scope5 = Scope::new();
    let scope6 = Scope::new();
    graph.add_scope(root, SgData::NoData);
    graph.add_scope(scope1, SgData::NoData);
    graph.add_scope(scope2, SgData::NoData);
    graph.add_scope(scope2, SgData::NoData);
    graph.add_scope(scope3, SgData::NoData);
    graph.add_scope(scope4, SgData::NoData);
    graph.add_scope(scope5, SgData::NoData);
    graph.add_scope(scope6, SgData::NoData);

    graph.add_decl(scope1, SgLabel::Declaration, SgData::var("x", "int"));
    // graph.add_decl(scope1, Label::Declaration, Data::var("x", "bool"));
    graph.add_decl(scope1, SgLabel::Declaration, SgData::var("y", "int"));
    graph.add_decl(scope2, SgLabel::Declaration, SgData::var("x", "int"));
    graph.add_edge(scope1, root, SgLabel::Parent);
    graph.add_edge(scope2, scope1, SgLabel::Parent);
    graph.add_edge(scope3, scope1, SgLabel::Parent);
    graph.add_edge(scope4, scope2, SgLabel::Parent);
    graph.add_edge(scope5, scope4, SgLabel::Parent);

    graph.add_edge(scope6, scope2, SgLabel::Parent);
    graph.add_edge(scope6, scope3, SgLabel::Parent);

    let order = LabelOrderBuilder::new()
        .push(SgLabel::Declaration, SgLabel::Parent)
        .build();

    // P*D;
    let label_reg = Regex::concat(Regex::kleene(SgLabel::Parent), SgLabel::Declaration);
    let matcher = RegexAutomata::from_regex(label_reg.clone());

    let y_match: Arc<str> = Arc::from("y");
    let x_match: Arc<str> = Arc::from("x");
    let query_scope_set = [
        (y_match, vec![scope6]),
        // (x_match.clone(), vec![scope5]),
        // (x_match, vec![scope5]),
    ];

    for (idx, set) in query_scope_set.into_iter().enumerate() {
        let title = format!(
            "Query1: {}, label_reg={}, label_order={}, data_eq=x:int",
            0, label_reg, order
        );
        // let mut diagram = graph.as_uml_diagram(DRAW_CACHES);
        // println!("diagram: {0:?}", diagram);

        let p = set.0;
        let start_scopes = set.1;

        let res_uml = start_scopes
            .into_iter()
            .flat_map(|s| {
                graph.query_proj(
                    s,
                    &matcher,
                    &order,
                    |d| Arc::from(d.name()),
                    p.clone(),
                    |d1, d2| d1 == d2,
                )
            })
            .enumerate()
            .flat_map(|(i, r)| r.path.as_mmd(ForeGroundColor::get_class_name(i), true))
            .collect::<Vec<_>>();

        let mut diagram = graph.as_mmd_diagram("graph", DRAW_CACHES);
        diagram.extend(res_uml);

        let fname = format!("output/output{}.md", idx);
        diagram.write_to_file(&fname).unwrap();
    }
}

fn graph_builder<'a>() -> UsedScopeGraph<'a, SgLabel, SgData> {
    let graph = UsedScopeGraph::<SgLabel, SgData>::new();
    let patterns = [
        GraphPattern::Linear(1),
        GraphPattern::Decl(SgData::var("x", "int")),
        // GraphPattern::Decl(SgData::var("z", "int")),
        GraphPattern::Linear(1),
        GraphPattern::Diamond(2),
        GraphPattern::Decl(SgData::var("y", "int")),
        GraphPattern::Linear(2),
        // GraphPattern::Diamond(5),
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
    let label_reg = Regex::concat(Regex::kleene(SgLabel::Parent), SgLabel::Declaration);
    let matcher = RegexAutomata::from_regex(label_reg.clone());
    matcher
        .uml_diagram()
        .write_to_file("output/regex.puml")
        .unwrap();

    let y_match: Arc<str> = Arc::from("x");
    let x_match: Arc<str> = Arc::from("x");
    let query_scope_set = [(y_match.clone(), vec![4, 9])];

    for (idx, set) in query_scope_set.into_iter().enumerate() {
        let title = format!(
            "Query1: {}, label_reg={}, label_order={}, data_eq=x",
            0, label_reg, order
        );

        let p = set.0;
        let start_scopes = set.1;

        let (res_uml, res_mmd) = start_scopes
            .into_iter()
            .flat_map(|s| {
                let scope = graph.first_scope_without_data(s).unwrap();
                graph.query_proj(
                    scope,
                    &matcher,
                    &order,
                    |d| Arc::from(d.name()),
                    p.clone(),
                    |d1, d2| d1.name() == d2.name(),
                )
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

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::WARN)
        .init();

    // slides_example();

    // let mut graph = create_long_graph();
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

fn write_to_file(fname: &str, content: &[u8]) {
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(fname)
        .unwrap();
    tracing::info!("Writing to file {}", fname);
    file.write_all(content).unwrap();
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
