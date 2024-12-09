use std::io::Write;

use data::ScopeGraphData;
use label::ScopeGraphLabel;
use lbl_regex::*;
use scope::Scope;
use scopegraph::ScopeGraph;

mod scopegraph;
mod scope;
mod label;
mod path;
mod lbl_regex;
mod data;

#[derive(Debug, Clone, Copy, PartialEq)]
enum Label {
    Parent,
    Declaration,
    /// Debug path that should never be taken
    NeverTake,
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

#[derive(Debug, Clone)]
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
    graph.add_scope(scope2,  Data::NoData);
    graph.add_scope(scope3, Data::NoData);

    graph.add_edge(scope1, root, Label::Parent);
    graph.add_edge(scope2, scope1, Label::Parent);
    graph.add_edge(scope2, scope3, Label::NeverTake);

    for _ in 0..10 {
        graph.add_decl(scope1, Label::Declaration, Data::var("x", "int"));
    }
    graph.add_decl(scope2, Label::Declaration, Data::var("x", "int"));
    graph.add_decl(scope2, Label::Declaration, Data::var("x", "bool"));
    graph.add_decl(scope3, Label::Declaration, Data::var("x", "int"));

    println!("graph: {0:?}", graph);

    // let label_reg = Regex::new("P*D").unwrap();
    let label_reg = vec![
        LabelRegex::ZeroOrMore(Label::Parent),
        LabelRegex::Single(Label::Declaration)
    ];
    let matcher = LabelRegexMatcher::new(label_reg);
    let res = graph.query(scope2,
        &matcher,
        |d| matches!(d, Data::Variable(x, t) if x == "x" && t == "int")
    );

    // println!("res: {0:?}", res);
    let mut mmd = graph.as_mmd("Query: label_reg=P*D, data_eq=x:int");
    if res.is_empty() {
        println!("No results found");
    } else {
        for r in res {
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
