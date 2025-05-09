use std::{sync::Arc, time::Duration};

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use rand::Rng;
use scope_graph::{
    generator::{GraphGenerator, GraphPattern}, graph::{BaseScopeGraph, CachedScopeGraph, QueryResult, ScopeGraph}, order::{LabelOrder, LabelOrderBuilder}, regex::{dfs::RegexAutomaton, Regex}, scope::Scope, SgData, SgLabel, DRAW_CACHES
};


fn query_graph<Sg>(graph: &mut Sg, num_queries: usize, order: &LabelOrder<SgLabel>, reg: &RegexAutomaton<SgLabel>) -> Vec<QueryResult<SgLabel, SgData>>
where
    Sg: ScopeGraph<SgLabel, SgData>,
{
    let mut thread_rng = rand::rng();

    // let matches: &[Arc<str>] = &[Arc::from("x"), Arc::from("y")];
    let mut envs = Vec::new();
    for _ in 0..num_queries {
        let start_scope = Scope(280);
        let start_scope = Scope(thread_rng.random_range(200..300));

        let m: Arc<str> = Arc::from("x");
        // let m = matches[thread_rng.random_range(0..matches.len())].clone();

        envs = graph.query_proj(
            start_scope,
            reg,
            order,
            |d| Arc::from(d.name()),
            m,
        );
    }
    graph.reset_cache(); // make next benchmark run from scratch
    envs
}

fn graph_builder<Sg>(graph: Sg) -> Sg
where Sg: ScopeGraph<SgLabel, SgData>
{
    let patterns = [
        GraphPattern::Decl(SgData::var("x", "int")),
        // GraphPattern::Decl(SgData::var("x1", "int")),
        // GraphPattern::Decl(SgData::var("x2", "int")),
        // GraphPattern::Decl(SgData::var("x3", "int")),
        // GraphPattern::Decl(SgData::var("x4", "int")),
        GraphPattern::Linear(30),
        // GraphPattern::Tree(2),
        GraphPattern::Tree(2),
        GraphPattern::Diamond(50),
        GraphPattern::Decl(SgData::var("y", "int")),
        GraphPattern::Linear(300),
    ];
    GraphGenerator::new(graph).with_patterns(patterns).build()
}

pub fn criterion_benchmark(c: &mut Criterion) {
    let mut graph = graph_builder(BaseScopeGraph::new());
    let mut bu_graph = CachedScopeGraph::from_base(graph.clone());
    graph
        .as_uml_diagram("title", false)
        .write_to_file("output/bench/graph.puml")
        .unwrap();
    graph
        .as_mmd_diagram("title", false)
        .write_to_file("output/bench/graph.md")
        .unwrap();

    let order = LabelOrderBuilder::new()
    .push(SgLabel::Declaration, SgLabel::Parent)
    .build();

    // P*D;
    let label_reg = Regex::concat(Regex::kleene(SgLabel::Parent), SgLabel::Declaration);
    let matcher = RegexAutomaton::from_regex(label_reg.clone());

    let mut group = c.benchmark_group("query");
    group.warm_up_time(Duration::from_secs(1));
    group.measurement_time(Duration::from_secs(1));

    for num_bench in [1, 2, 5] {
        let s1 = format!("bench {}", num_bench);
        let s2 = format!("map bench {}", num_bench);
        group.bench_function(&s1, |b| {
            b.iter(|| query_graph(&mut graph, num_bench, &order, &matcher))
        });
        group.bench_function(&s2, |b| {
            b.iter(|| query_graph(&mut bu_graph, num_bench, &order, &matcher))
        });
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

#[cfg(test)]
mod tests {
    use crate::graph_builder;


    #[test]
    fn test_query() {
        let mut graph = CachedScopeGraph::new();
        graph = graph_builder(graph);
        let num_bench = 1;
        let query = query_graph(&mut graph, num_bench);
        graph.as_uml_diagram("graph", true)
            .write_to_file("output/bench/graph.puml")
            .unwrap();
        graph.as_mmd_diagram("graph", DRAW_CACHES)
            .write_to_file("output/bench/graph.md")
            .unwrap();
    }

    #[test]
    fn test_build() {
        let mut graph = CachedScopeGraph::new();
        graph = graph_builder(graph);
        graph
            .as_uml_diagram("graph", DRAW_CACHES)
            .write_to_file("output/bench/graph.puml")
            .unwrap();
        graph
            .as_mmd_diagram("graph", DRAW_CACHES)
            .write_to_file("output/bench/graph.md")
            .unwrap();
    }
}
