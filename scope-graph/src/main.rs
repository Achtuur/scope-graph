use label::ScopeGraphLabel;
use regex::Regex;
use scope::Scope;
use scopegraph::ScopeGraph;

mod scopegraph;
mod scope;
mod label;

#[derive(Debug, Clone, Copy)]
enum Label {
    Parent,
    Declaration,
}

impl ScopeGraphLabel for Label {
    fn char(&self) -> char {
        match self {
            Self::Parent => 'P',
            Self::Declaration => 'D',
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

fn main() {
    let mut graph = ScopeGraph::new();
    let root = Scope::new();
    let scope1 = Scope::new();
    let scope2 = Scope::new();

    graph.add_scope(root, Data::NoData);
    graph.add_scope(scope1, Data::NoData);
    graph.add_scope(scope2,  Data::NoData);

    graph.add_edge(scope1, root, Label::Parent);
    graph.add_edge(scope2, scope1, Label::Parent);

    graph.add_decl(scope1, Label::Declaration, Data::var("x", "bool"));
    graph.add_decl(scope2, Label::Declaration, Data::var("x", "bool"));

    println!("graph: {0:?}", graph);

    let label_reg = Regex::new("P*D").unwrap();
    let res = graph.query(scope2,
        &label_reg,
        |d| matches!(d, Data::Variable(x, t) if x == "x" && t == "bool")
    );
    println!("res: {0:?}", res);

}
