use std::{io::Write, os::unix::thread};

use data::ScopeGraphData;
use forward::ForwardScopeGraph;
use graph::BaseScopeGraphHaver;
use label::ScopeGraphLabel;
// use lbl_regex::*;
use order::LabelOrderBuilder;
use rand::Rng;
use regex::{dfs::RegexAutomata, Regex};
use scope::Scope;

mod label;
mod path;
mod scope;
mod forward;
// mod lbl_regex;
mod data;
mod order;
mod regex;
pub mod resolve;
pub mod bottomup;
pub mod graph;


/// Enable caching when doing forward resolution
pub(crate) const FORWARD_ENABLE_CACHING: bool = true;

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

fn create_example_graph<'a>() -> ForwardScopeGraph<'a, Label, Data> {
    let mut graph = ForwardScopeGraph::new();
    let root = Scope::new();
    let scope1 = Scope::new();
    let scope2 = Scope::new();
    let scope3 = Scope::new();

    graph.add_scope(root, Data::NoData);
    graph.add_scope(scope1, Data::NoData);
    graph.add_scope(scope2, Data::NoData);
    graph.add_scope(scope3, Data::NoData);

    graph.add_edge(scope1, root, Label::Parent);
    graph.add_edge(scope2, scope1, Label::Parent);
    graph.add_edge(scope2, scope3, Label::NeverTake);

    graph.add_decl(scope1, Label::Declaration, Data::var("x", "int"));
    graph.add_decl(scope2, Label::Declaration, Data::var("x", "bool"));
    graph.add_decl(scope3, Label::Declaration, Data::var("x", "int"));
    graph
}

fn recurse_add_scopes(graph: &mut ForwardScopeGraph<Label, Data>, parent: Scope, depth: usize) {
    if depth == 0 {
        return;
    }

    let mut thread_rng = rand::rng();

    const MAX_CHILDREN: usize = 2;
    let r = thread_rng.random_range(1..=MAX_CHILDREN);

    for _ in 0..r {
        let scope = Scope::new();
        graph.add_scope(scope, Data::NoData);
        graph.add_edge(scope, parent, Label::Parent);
        recurse_add_scopes(graph, scope, depth - 1);
    }
}

// graph with 1 decl near the root and a lot of children
fn create_long_graph<'a>() -> ForwardScopeGraph<'a, Label, Data> {
    let mut graph = ForwardScopeGraph::new();
    let root = Scope::new();
    let scope1 = Scope::new();
    graph.add_scope(root, Data::NoData);
    graph.add_scope(scope1, Data::NoData);
    graph.add_edge(scope1, root, Label::Parent);
    graph.add_decl(scope1, Label::Declaration, Data::var("x", "int"));
    graph.add_decl(scope1, Label::Declaration, Data::var("x", "bool"));

    recurse_add_scopes(&mut graph, scope1, 6);
    graph
}

fn main() {
    let mut graph = create_long_graph();

    let order = LabelOrderBuilder::new().push(Label::Declaration, Label::Parent).build();

    // P*PD;
    let label_reg = Regex::concat(
        Regex::kleene(Label::Parent),
        // Regex::concat(Label::Parent, Label::Declaration),
        Label::Declaration,
    );
    let matcher = RegexAutomata::from_regex(label_reg.clone());

    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open("./automata.mmd")
        .unwrap();
    file.write_all(matcher.to_mmd().as_bytes()).unwrap();


    
    
    let start_scope = graph.find_scope(7).unwrap();
    let timer = std::time::Instant::now();
    let resA = graph.query(
        start_scope,
        &matcher,
        &order,
        |d1, d2| d1 == d2,
        |d| matches!(d, Data::Variable(x, t) if x == "x" && t == "int"),
    );
    println!("first run {:?}", timer.elapsed());
    let second_start = graph.find_scope(18).unwrap();
    let timer = std::time::Instant::now();
    let resB = graph.query(
        second_start,
        &matcher,
        &order,
        |d1, d2| d1 == d2,
        |d| matches!(d, Data::Variable(x, t) if x == "x" && t == "int"),
    );
    println!("second run {:?}", timer.elapsed());


    // println!("res: {0:?}", res);
    let title = format!(
        "Query1: {0:}, Query2: {1:}, label_reg={2:}, label_order={3:}, data_eq=x:int",
        start_scope, second_start, label_reg, order
    );
    let mut mmd = graph.as_mmd(&title);
    
    if resA.is_empty() {
        println!("No results found");
    } else {
        for r in resA {
            println!("r: {} ({:?})", r.path, r.data);
            mmd = r.path.as_mmd(mmd);
        }
    }

    if resB.is_empty() {
        println!("No results found");
    } else {
        for r in resB {
            println!("r: {} ({:?})", r.path, r.data);
            mmd = r.path.as_mmd(mmd);
        }
    }

    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open("./output.mmd")
        .unwrap();
    file.write_all(mmd.as_bytes()).unwrap();
}
