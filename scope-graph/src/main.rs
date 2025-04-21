use std::{io::Write, sync::Arc};

use plantuml::PlantUmlDiagram;
use rand::Rng;
use scope_graph::{
    data::ScopeGraphData,
    get_color,
    graph::{BaseScopeGraph, CachedScopeGraph, ScopeGraph},
    label::ScopeGraphLabel,
    order::LabelOrderBuilder,
    regex::{dfs::RegexAutomata, Regex},
    scope::Scope,
    DRAW_CACHES,
};

pub type UsedScopeGraph<'s, Lbl, Data> = BaseScopeGraph<Lbl, Data>;

#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq, PartialOrd, Ord)]
pub enum Label {
    Parent,
    Declaration,
    A,
    B,
    C,
    /// Debug path that should never be taken
    NeverTake,
}

impl std::fmt::Display for Label {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.char())
    }
}

impl ScopeGraphLabel for Label {
    fn char(&self) -> char {
        match self {
            Self::Parent => 'P',
            Self::Declaration => 'D',
            Self::NeverTake => 'W',
            Self::A => 'A',
            Self::B => 'B',
            Self::C => 'C',
        }
    }

    fn str(&self) -> &'static str {
        match self {
            Self::Parent => "Parent",
            Self::Declaration => "Declaration",
            Self::NeverTake => "NeverTake",
            Self::A => "A",
            Self::B => "B",
            Self::C => "C",
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

    fn name(&self) -> String {
        match self {
            Self::NoData => "no data".to_string(),
            Self::Variable(x, _) => x.to_string(),
        }
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
    let scope6 = Scope::new();
    graph.add_scope(root, Data::NoData);
    graph.add_scope(scope1, Data::NoData);
    graph.add_scope(scope2, Data::NoData);
    graph.add_scope(scope2, Data::NoData);
    graph.add_scope(scope3, Data::NoData);
    graph.add_scope(scope4, Data::NoData);
    graph.add_scope(scope5, Data::NoData);
    graph.add_scope(scope6, Data::NoData);

    graph.add_decl(scope1, Label::Declaration, Data::var("x", "int"));
    graph.add_decl(scope1, Label::Declaration, Data::var("y", "int"));
    // graph.add_decl(scope2, Label::Declaration, Data::var("x", "int"));
    graph.add_edge(scope1, root, Label::Parent);
    graph.add_edge(scope2, scope1, Label::Parent);
    graph.add_edge(scope3, scope1, Label::Parent);
    graph.add_edge(scope4, scope2, Label::Parent);
    graph.add_edge(scope5, scope4, Label::Parent);

    graph.add_edge(scope6, scope2, Label::Parent);
    graph.add_edge(scope6, scope3, Label::Parent);

    let order = LabelOrderBuilder::new()
        .push(Label::Declaration, Label::Parent)
        .build();

    // P*D;
    let label_reg = Regex::concat(Regex::kleene(Label::Parent), Label::Declaration);
    let matcher = RegexAutomata::from_regex(label_reg.clone());

    let y_match: Arc<str> = Arc::from("y");
    let x_match: Arc<str> = Arc::from("x");
    let query_scope_set = [
        (y_match, vec![scope6]),
        (x_match, vec![scope6]),
        // vec![scope2, scope6],
        // vec![scope2, scope6, scope5],
    ];

    for (idx, set) in query_scope_set.into_iter().enumerate() {
        let title = format!(
            "Query1: {}, label_reg={}, label_order={}, data_eq=x:int",
            0, label_reg, order
        );
        let mut diagram = PlantUmlDiagram::new(title.as_str());

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
            .flat_map(|(i, r)| r.path.as_uml(get_color(i), false));
        diagram.extend(res_uml);

        let graph_uml = graph.as_uml(DRAW_CACHES);
        diagram.extend(graph_uml);
        let uml = diagram.as_uml();

        let fname = format!("output/output{}.puml", idx);
        write_to_file(&fname, uml.as_bytes());
    }
}

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::TRACE)
        .init();

    slides_example();

    // let mut bu_graph = create_long_graph();
    // let mut forward_graph = CachedScopeGraph::from_base(bu_graph.clone());

    // let order = LabelOrderBuilder::new()
    // .push(Label::Declaration, Label::Parent)
    // .build();

    // // P*D;
    // // let label_reg = Regex::or(
    // //     Regex::concat(Label::A, Label::B),
    // //     Regex::or(Label::Parent, Label::Declaration)
    // // );
    // let label_reg = Regex::or(
    //     Regex::concat(
    //         Regex::or(Label::A, Label::B),
    //         Regex::or(Regex::kleene(Label::Parent), Label::Declaration)
    //     ),
    //     Regex::concat(
    //         Regex::kleene(Label::Parent),
    //         Label::Declaration,
    //     ));
    // let matcher = RegexAutomata::from_regex(label_reg.clone());

    // write_to_file("output/automata.mmd", matcher.to_mmd().as_bytes());
    // write_to_file("output/automata.puml", matcher.uml_diagram().as_uml().as_bytes());

    // let p: Arc<str> = Arc::from("x");

    // let start_scope = bu_graph.first_scope_without_data(5).unwrap();
    // let timer = std::time::Instant::now();
    // let res_bu = bu_graph.query_proj(
    //     start_scope,
    //     &matcher,
    //     &order,
    //     |d| Arc::from(d.name()),
    //     p.clone(),
    //     |d1, d2| d1 == d2,
    // );
    // println!("run bu {:?}", timer.elapsed());

    // let timer = std::time::Instant::now();

    // let res_fw = forward_graph.query_proj(
    //     start_scope,
    //     &matcher,
    //     &order,
    //     |d| Arc::from(d.name()),
    //     p,
    //     |d1, d2| d1 == d2,
    // );
    // println!("run fw {:?}", timer.elapsed());

    // if res_bu.is_empty() && res_fw.is_empty() {
    //     println!("No results found");
    // } else {
    //     println!("bottomup: ");
    //     for r in &res_bu {
    //         println!("{}", r);
    //     }
    //     println!("fw: ");
    //     for r in &res_fw {
    //         println!("{}", r);
    //     }
    // }

    // let title = format!(
    //     "Query1: {}, label_reg={}, label_order={}, data_eq=x:int",
    //     start_scope, label_reg, order
    // );
    // let graph_uml = bu_graph.as_uml(DRAW_CACHES);

    // let res_a_uml = res_bu
    //     .iter()
    //     .flat_map(|r| r.path.as_uml(Color::Red, false));

    // let res_b_uml = res_fw
    //     .iter()
    //     .flat_map(|r| r.path.as_uml(Color::Blue, false));

    // let mut diagram = PlantUmlDiagram::new(title.as_str());
    // diagram.extend(graph_uml);
    // diagram.extend(res_a_uml);
    // diagram.extend(res_b_uml);
    // let uml = diagram.as_uml();

    // write_to_file("output/output.puml", uml.as_bytes());
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
