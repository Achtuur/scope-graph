use std::sync::Arc;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rand::Rng;
use scope_graph::{
    data::ScopeGraphData,
    graph::{BaseScopeGraph, CachedScopeGraph, ScopeGraph},
    label::ScopeGraphLabel,
    order::LabelOrderBuilder,
    regex::{dfs::RegexAutomata, Regex},
    scope::Scope,
    SgLabel, SgData,
};
use serde::{Deserialize, Serialize};

const MAX_CHILDREN: usize = 2;
const GEN_DEPTH: usize = 10;

fn recurse_add_scopes<Sg: ScopeGraph<SgLabel, SgData>>(graph: &mut Sg, parent: Scope, depth: usize) {
    if depth == 0 {
        return;
    }
    let mut thread_rng = rand::rng();
    let r = thread_rng.random_range(1..=MAX_CHILDREN);
    for _ in 0..r {
        let scope = Scope::new();
        graph.add_scope(scope, SgData::NoData);
        if thread_rng.random_bool(0.2) {
            graph.add_decl(scope, SgLabel::Declaration, SgData::var("x", "int"));
        }
        graph.add_edge(scope, parent, SgLabel::Parent);
        recurse_add_scopes(graph, scope, depth - 1);
    }
}

// graph with 1 decl near the root and a lot of children
fn create_long_graph<Sg: ScopeGraph<SgLabel, SgData>>(graph: &mut Sg) {
    let root = Scope::new();
    let scope1 = Scope::new();
    graph.add_scope(root, SgData::NoData);
    graph.add_scope(scope1, SgData::NoData);
    graph.add_decl(scope1, SgLabel::Declaration, SgData::var("x", "int"));
    graph.add_decl(scope1, SgLabel::Declaration, SgData::var("x", "bool"));
    graph.add_edge(scope1, root, SgLabel::Parent);

    recurse_add_scopes(graph, scope1, GEN_DEPTH);
}

fn create_diamond_graph<Sg: ScopeGraph<SgLabel, SgData>>(graph: &mut Sg) {
    // diamond: (tailN -> tail0) -> (diamond0..diamondN) -> (root -> rootN)

    const ROOT_SIZE: usize = 10;
    const TAIL_SIZE: usize = ROOT_SIZE;
    const DIAMOND_SIZE: usize = 40;

    let mut root = Scope::new();
    graph.add_scope(root, SgData::NoData);
    graph.add_decl(root, SgLabel::Declaration, SgData::var("x", "int"));
    graph.add_decl(root, SgLabel::Declaration, SgData::var("y", "int"));
    let mut tail = Scope::new();
    graph.add_scope(tail, SgData::NoData);
    for _ in 0..ROOT_SIZE {
        let new_root = Scope::new();
        graph.add_scope(new_root, SgData::NoData);
        graph.add_edge(new_root, root, SgLabel::Parent);
        root = new_root;

        let new_tail = Scope::new();
        graph.add_scope(new_tail, SgData::NoData);
        graph.add_edge(tail, new_tail, SgLabel::Parent);
        tail = new_tail;
    }

    for _ in 0..DIAMOND_SIZE {
        let scope = Scope::new();
        graph.add_scope(scope, SgData::NoData);
        graph.add_edge(scope, root, SgLabel::Parent);
        graph.add_edge(tail, scope, SgLabel::Parent);
    }
}

fn query_graph<Sg>(mut graph: Sg, num_queries: usize) -> Sg
where
    Sg: ScopeGraph<SgLabel, SgData>,
{
    let order = LabelOrderBuilder::new()
        .push(SgLabel::Declaration, SgLabel::Parent)
        .build();

    // P*D;
    let label_reg = Regex::concat(Regex::kleene(SgLabel::Parent), SgLabel::Declaration);
    let matcher = RegexAutomata::from_regex(label_reg.clone());

    let mut thread_rng = rand::rng();

    let matches: &[Arc<str>] = &[Arc::from("x"), Arc::from("y")];

    for _ in 0..num_queries {
        // let r = thread_rng.random_range(1..=MAX_SCOPE_NUM);
        // let Some(start_scope) = graph.first_scope_without_data(3) else {
        //     continue;
        // };
        // let start_scope = graph.get_scope(Scope(3)).unwrap();
        let start_scope = Scope(3);

        let m = matches[thread_rng.random_range(0..matches.len())].clone();

        let _ = graph.query_proj(
            start_scope,
            &matcher,
            &order,
            |d| Arc::from(d.name()),
            m,
            |d1, d2| d1 == d2,
        );
    }
    graph
}

fn bench_graph<Sg>(mut graph: Sg, num_queries: usize) -> Sg
where
    Sg: ScopeGraph<SgLabel, SgData>,
{
    create_diamond_graph(&mut graph);
    Scope::reset_counter(); // so we can always select the same scope number
    query_graph(graph, num_queries)
}

pub fn criterion_benchmark(c: &mut Criterion) {
    let graph = bench_graph(BaseScopeGraph::new(), 0);
    graph
        .as_uml_diagram("title", false)
        .write_to_file("output/bench/graph.puml")
        .unwrap();

    let mut group = c.benchmark_group("diamonds");
    group.sample_size(100);

    for num_bench in [1, 2, 5] {
        let s1 = format!("non-cache bench {}", num_bench);
        let s2 = format!("cache bench {}", num_bench);
        group.bench_function(&s1, |b| {
            b.iter(|| bench_graph(black_box(BaseScopeGraph::new()), num_bench))
        });
        group.bench_function(&s2, |b| {
            b.iter(|| bench_graph(black_box(CachedScopeGraph::new()), num_bench))
        });
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

#[cfg(test)]
mod tests {

    #[test]
    fn test_query() {
        bench_graph(black_box(CachedScopeGraph::new()), num_bench)
    }
}
