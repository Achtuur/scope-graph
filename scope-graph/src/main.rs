use std::io::Write;

use data::ScopeGraphData;
use label::ScopeGraphLabel;
// use lbl_regex::*;
use order::LabelOrder;
use regex::{dfs::RegexAutomata, Regex};
use scope::Scope;
use scopegraph::ScopeGraph;

mod label;
mod path;
mod scope;
mod scopegraph;
// mod lbl_regex;
mod data;
mod order;
mod regex;
mod resolve;

#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
enum Data {
    NoData,
    Variable(String, String),
}

impl Data {
    fn var(x: impl ToString, t: impl ToString) -> Self {
        Self::Variable(x.to_string(), t.to_string())
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

fn main() {
    let mut graph = ScopeGraph::new();
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

    for _ in 0..2 {
        graph.add_decl(scope1, Label::Declaration, Data::var("x", "int"));
    }
    // graph.add_decl(scope2, Label::Declaration, Data::var("x", "int"));
    graph.add_decl(scope2, Label::Declaration, Data::var("x", "bool"));
    graph.add_decl(scope3, Label::Declaration, Data::var("x", "int"));

    println!("graph: {0:?}", graph);

    // let label_reg = Regex::new("P*D").unwrap();
    // let label_reg = vec![
    //     LabelRegex::ZeroOrMore(Label::Parent),
    //     LabelRegex::Single(Label::Declaration)
    // ];
    // let matcher = LabelRegexMatcher::new(label_reg);

    let order = LabelOrder::new().push(Label::Declaration, Label::Parent);

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

    let (res, considered_paths) = graph.query(
        scope2,
        &matcher,
        &order,
        |d1, d2| d1 == d2,
        |d| matches!(d, Data::Variable(x, t) if x == "x" && t == "int"),
    );

    // println!("res: {0:?}", res);
    let title = format!(
        "Query: label_reg={}, label_order={}, data_eq=x:int",
        label_reg, order
    );
    let mut mmd = graph.as_mmd(&title);
    if res.is_empty() {
        println!("No results found");
    } else {
        for r in res {
            println!("r: {} ({:?})", r.path, r.data);
            mmd = r.path.as_mmd(mmd);
        }
    }

    for p in considered_paths {
        println!("Considered path: {}", p);
        mmd = p.as_mmd_debug(mmd);
    }

    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open("./output.mmd")
        .unwrap();
    file.write_all(mmd.as_bytes()).unwrap();
}
