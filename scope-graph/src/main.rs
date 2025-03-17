use std::{io::Write, os::unix::thread, sync::atomic::AtomicUsize};

use bottomup::BottomupScopeGraph;
use data::ScopeGraphData;
use forward::ForwardScopeGraph;
use graph::{BaseScopeGraph, BaseScopeGraphHaver};
use label::ScopeGraphLabel;
// use lbl_regex::*;
use order::LabelOrderBuilder;
use plantuml::{Color, PlantUmlDiagram};
use rand::Rng;
use regex::{dfs::RegexAutomata, Regex};
use scope::Scope;

mod label;
mod path;
mod scope;
mod forward;
mod data;
mod order;
mod regex;
pub mod resolve;
pub mod bottomup;
pub mod graph;


pub(crate) const COLORS: &[Color] = &[
    Color::Red,
    Color::Green,
    Color::Purple,
    Color::Blue,
    Color::Orange,
];

pub(crate) static COLOR_POINTER: AtomicUsize = AtomicUsize::new(0);

pub fn next_color() -> Color {
    let idx = COLOR_POINTER.load(std::sync::atomic::Ordering::Relaxed);
    let _ = COLOR_POINTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    COLORS[idx % COLORS.len()]
}

pub fn get_color(idx: usize) -> Color {
    COLORS[idx % COLORS.len()]
}

/// Enable caching when doing forward resolution
pub(crate) const FORWARD_ENABLE_CACHING: bool = true;

pub(crate) const DRAW_CACHES: bool = false;

pub(crate) type UsedScopeGraph<'s, Lbl, Data> = BottomupScopeGraph<'s, Lbl, Data>;

#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq, PartialOrd, Ord)]
enum Label {
    Parent,
    Declaration,
    /// Debug path that should never be taken
    NeverTake,
}

impl std::fmt::Display for Label {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Parent => write!(f, "P"),
            Self::Declaration => write!(f, "D"),
            Self::NeverTake => write!(f, "N"),
        }
    }
}

impl ScopeGraphLabel for Label {
    fn char(&self) -> char {
        match self {
            Self::Parent => 'P',
            Self::Declaration => 'D',
            Self::NeverTake => 'W',
        }
    }

    fn str(&self) -> &'static str {
        match self {
            Self::Parent => "Parent",
            Self::Declaration => "Declaration",
            Self::NeverTake => "NeverTake",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
enum Data {
    NoData,
    Variable(String, String),
}

impl Data {
    fn var(x: impl ToString, t: impl ToString) -> Self {
        Self::Variable(x.to_string(), t.to_string())
    }
}

impl std::fmt::Display for Data {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoData => write!(f, "no data"),
            Self::Variable(x, t) => write!(f, "{x}: {t}"),
        }
    }
}

impl ScopeGraphData for Data {
    fn variant_has_data(&self) -> bool {
        match self {
            Self::NoData => false,
            Self::Variable(_, _) => true,
        }
    }

    fn render_string(&self) -> String {
        match self {
            Self::NoData => "".to_string(),
            Self::Variable(x, t) => format!("{}: {}", x, t),
        }
    }
}

fn create_example_graph<'a>() -> UsedScopeGraph<'a, Label, Data> {
    let mut graph = UsedScopeGraph::new();
    let root = Scope::new();
    let scope1 = Scope::new();
    let scope2 = Scope::new();
    let scope3 = Scope::new();

    graph.add_scope(root, Data::NoData);
    graph.add_scope(scope1, Data::NoData);
    graph.add_scope(scope2, Data::NoData);
    graph.add_scope(scope3, Data::NoData);



    for _ in 0..1 {
        graph.add_decl(scope1, Label::Declaration, Data::var("x", "int"));
    }
    graph.add_decl(scope2, Label::Declaration, Data::var("x", "bool"));
    graph.add_decl(scope3, Label::Declaration, Data::var("x", "int"));

    graph.add_edge(scope1, root, Label::Parent);
    graph.add_edge(scope2, scope1, Label::Parent);
    graph.add_edge(scope2, scope3, Label::NeverTake);

    graph
}

fn recurse_add_scopes(graph: &mut UsedScopeGraph<Label, Data>, parent: Scope, depth: usize) {
    if depth == 0 {
        return;
    }

    let mut thread_rng = rand::rng();

    const MAX_CHILDREN: usize = 2;
    let r = thread_rng.random_range(1..=MAX_CHILDREN);

    for _ in 0..r {
        let scope = Scope::new();
        graph.add_scope(scope, Data::NoData);
        if thread_rng.random_bool(0.2) {
            graph.add_decl(scope, Label::Declaration, Data::var("x", "int"));
        }
        graph.add_edge(scope, parent, Label::Parent);
        recurse_add_scopes(graph, scope, depth - 1);
    }
}

// graph with 1 decl near the root and a lot of children
fn create_long_graph<'a>() -> UsedScopeGraph<'a, Label, Data> {
    let mut graph = UsedScopeGraph::new();
    let root = Scope::new();
    let scope1 = Scope::new();
    graph.add_scope(root, Data::NoData);
    graph.add_scope(scope1, Data::NoData);
    graph.add_decl(scope1, Label::Declaration, Data::var("x", "int"));
    graph.add_decl(scope1, Label::Declaration, Data::var("x", "bool"));
    graph.add_edge(scope1, root, Label::Parent);

    recurse_add_scopes(&mut graph, scope1, 3);
    graph
}

fn slides_example() {
    let mut graph = UsedScopeGraph::new();
    let root = Scope::new();
    let scope1 = Scope::new();
    let scope2 = Scope::new();
    let scope3 = Scope::new();
    let scope4 = Scope::new();
    let scope5 = Scope::new();
    graph.add_scope(root, Data::NoData);
    graph.add_scope(scope1, Data::NoData);
    graph.add_scope(scope2, Data::NoData);
    graph.add_scope(scope2, Data::NoData);
    graph.add_scope(scope3, Data::NoData);
    graph.add_scope(scope4, Data::NoData);
    graph.add_scope(scope5, Data::NoData);

    graph.add_decl(scope1, Label::Declaration, Data::var("x", "int"));
    graph.add_edge(scope1, root, Label::Parent);
    graph.add_edge(scope2, scope1, Label::Parent);
    graph.add_edge(scope3, scope1, Label::Parent);
    graph.add_edge(scope4, scope2, Label::Parent);
    graph.add_edge(scope5, scope4, Label::Parent);

    let order = LabelOrderBuilder::new()
    .push(Label::Declaration, Label::Parent)
    .build();

    // P*D;
    let label_reg = Regex::concat(
        Regex::kleene(Label::Parent),
        Label::Declaration,
    );
    let matcher = RegexAutomata::from_regex(label_reg.clone());

    let query_scope_set = [
        vec![scope2],
        vec![scope2, scope4],
        vec![scope2, scope4, scope5],
    ];

    for (idx, set) in query_scope_set.into_iter().enumerate() {
        let res = set.into_iter()
        .flat_map(|s| graph.query(
            s,
            &matcher,
            &order,
            |d1, d2| d1 == d2,
            |d| matches!(d, Data::Variable(x, t) if x == "x" && t == "int"),
        ));

        let title = format!(
            "Query1: {}, label_reg={}, label_order={}, data_eq=x:int",
            0, label_reg, order
        );

        let graph_uml = graph.as_uml(DRAW_CACHES);
        let res_uml = res
            .enumerate()
            .flat_map(|(i, r)| r.path.as_uml(get_color(i), false));

        let mut diagram = PlantUmlDiagram::new(title.as_str());
        diagram.extend(graph_uml);
        diagram.extend(res_uml);
        let uml = diagram.as_uml();

        let fname = format!("output/output{}.puml", idx);
        write_to_file(&fname, uml.as_bytes());
    }

}


fn main() {
    slides_example();
    return;

    let bu_graph = create_long_graph();
    let forward_graph = ForwardScopeGraph::from_base(bu_graph.sg().clone());

    let order = LabelOrderBuilder::new()
    .push(Label::Declaration, Label::Parent)
    .build();

    // P*D;
    let label_reg = Regex::concat(
        Regex::kleene(Label::Parent),
        Label::Declaration,
    );
    let matcher = RegexAutomata::from_regex(label_reg.clone());

    write_to_file("output/automata.mmd", matcher.to_mmd().as_bytes());

    let start_scope = bu_graph.first_scope_without_data(5).unwrap();
    let timer = std::time::Instant::now();
    let res_bu = bu_graph.query(
        start_scope,
        &matcher,
        &order,
        |d1, d2| d1 == d2,
        |d| matches!(d, Data::Variable(x, t) if x == "x" && t == "int"),
    );
    println!("run bu {:?}", timer.elapsed());

    let timer = std::time::Instant::now();
    let res_fw = forward_graph.query(
        start_scope,
        &matcher,
        &order,
        |d1, d2| d1 == d2,
        |d| matches!(d, Data::Variable(x, t) if x == "x" && t == "int"),
    );
    println!("run fw {:?}", timer.elapsed());

    if res_bu.is_empty() && res_fw.is_empty() {
        println!("No results found");
    } else {
        println!("bottomup: ");
        for r in &res_bu {
            println!("{}", r);
        }
        println!("fw: ");
        for r in &res_fw {
            println!("{}", r);
        }
    }

    let title = format!(
        "Query1: {}, label_reg={}, label_order={}, data_eq=x:int",
        start_scope, label_reg, order
    );
    let graph_uml = bu_graph.as_uml(DRAW_CACHES);

    let res_a_uml = res_bu
        .iter()
        .flat_map(|r| r.path.as_uml(Color::Red, false));

    let res_b_uml = res_fw
        .iter()
        .flat_map(|r| r.path.as_uml(Color::Blue, false));


    let mut diagram = PlantUmlDiagram::new(title.as_str());
    diagram.extend(graph_uml);
    diagram.extend(res_a_uml);
    diagram.extend(res_b_uml);
    let uml = diagram.as_uml();

    write_to_file("output/output.puml", uml.as_bytes());
}

fn write_to_file(fname: &str, content: &[u8]) {
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(fname)
        .unwrap();
    println!("Writing to file {}", fname);
    file.write_all(content).unwrap();
}