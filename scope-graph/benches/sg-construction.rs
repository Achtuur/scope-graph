use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rand::Rng;
use scope_graph::{
    data::ScopeGraphData,
    graph::{BaseScopeGraph, CachedScopeGraph, ScopeGraph},
    label::ScopeGraphLabel,
    order::LabelOrderBuilder,
    regex::{dfs::RegexAutomata, Regex},
    scope::Scope,
};

const MAX_CHILDREN: usize = 2;
const GEN_DEPTH: usize = 10;

#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq, PartialOrd, Ord)]
pub enum Label {
    Parent,
    Declaration,
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
        }
    }

    fn str(&self) -> &'static str {
        match self {
            Self::Parent => "Parent",
            Self::Declaration => "Declaration",
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

fn recurse_add_scopes<'a, Sg: ScopeGraph<Label, Data>>(
    graph: &mut Sg,
    parent: Scope,
    depth: usize,
) {
    if depth == 0 {
        return;
    }
    let mut thread_rng = rand::rng();
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
fn create_long_graph<'a, Sg: ScopeGraph<Label, Data>>(graph: &mut Sg) {
    let root = Scope::new();
    let scope1 = Scope::new();
    graph.add_scope(root, Data::NoData);
    graph.add_scope(scope1, Data::NoData);
    graph.add_decl(scope1, Label::Declaration, Data::var("x", "int"));
    graph.add_decl(scope1, Label::Declaration, Data::var("x", "bool"));
    graph.add_edge(scope1, root, Label::Parent);

    recurse_add_scopes(graph, scope1, GEN_DEPTH);
}

fn query_graph<'s, Sg>(mut graph: Sg, num_queries: usize)
where
    Sg: ScopeGraph<Label, Data>,
{
    const MAX_SCOPE_NUM: usize = 50;
    let order = LabelOrderBuilder::new()
        .push(Label::Declaration, Label::Parent)
        .build();

    // P*D;
    let label_reg = Regex::concat(Regex::kleene(Label::Parent), Label::Declaration);
    let matcher = RegexAutomata::from_regex(label_reg.clone());

    let mut thread_rng = rand::rng();
    for _ in 0..num_queries {
        let r = thread_rng.random_range(1..=MAX_SCOPE_NUM);
        let Some(start_scope) = graph.first_scope_without_data(r) else {
            continue;
        };
        let _ = graph.query(
            start_scope,
            &matcher,
            &order,
            |d1, d2| d1 == d2,
            |d| matches!(d, Data::Variable(x, t) if x == "x" && t == "int"),
        );
    }
}

fn bench_graph<Sg>(mut graph: Sg, num_queries: usize)
where
    Sg: ScopeGraph<Label, Data>,
{
    create_long_graph(&mut graph);
    query_graph(graph, num_queries);
}

fn build_graph<Sg: ScopeGraph<Label, Data>>(mut graph: Sg) {
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
    graph.add_decl(scope4, Label::Declaration, Data::var("x", "int"));
    graph.add_edge(scope1, root, Label::Parent);
    graph.add_edge(scope2, scope1, Label::Parent);
    graph.add_edge(scope3, scope1, Label::Parent);
    graph.add_edge(scope4, scope2, Label::Parent);
    graph.add_edge(scope5, scope4, Label::Parent);

    graph.add_edge(scope6, scope2, Label::Parent);
    graph.add_edge(scope6, scope3, Label::Parent);
}

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("non-cache bench 50", |b| {
        b.iter(|| bench_graph(black_box(BaseScopeGraph::new()), 50))
    });
    c.bench_function("cache bench 50", |b| {
        b.iter(|| bench_graph(black_box(CachedScopeGraph::new()), 50))
    });
    c.bench_function("non-cache bench 250", |b| {
        b.iter(|| bench_graph(black_box(BaseScopeGraph::new()), 250))
    });
    c.bench_function("cache bench 250", |b| {
        b.iter(|| bench_graph(black_box(CachedScopeGraph::new()), 250))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
