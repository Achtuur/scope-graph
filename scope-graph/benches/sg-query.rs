use std::{sync::Arc, time::Duration};

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use rand::Rng;
use scope_graph::{
    DRAW_CACHES, LibGraph, LibScope, SgData, SgLabel,
    generator::{GraphGenerator, GraphPattern},
    graph::{BaseScopeGraph, CachedScopeGraph, QueryResult, ScopeGraph},
    order::{LabelOrder, LabelOrderBuilder},
    regex::{Regex, dfs::RegexAutomaton},
    scope::Scope,
};
use scopegraphs::{
    completeness::{ImplicitClose, UncheckedCompleteness}, label_order, query_regex, render::RenderSettings, resolve::Resolve, Storage
};

fn query_libgraph(graph: &mut LibGraph, num_queries: usize) {
    let start_scope = scopegraphs::Scope(START_SCOPE);
    let query = graph
        .query()
        .with_path_wellformedness(query_regex!(SgLabel: Parent*Declaration))
        .with_label_order(label_order!(SgLabel:
            Declaration < Parent,
        ))
        .with_data_wellformedness(|data: &SgData| -> bool { data.name() == "x" })
        .with_data_equivalence(|d1: &SgData, d2: &SgData| -> bool { d1.name() == d2.name() });
    for _ in 0..num_queries {
        let envs = query.resolve(start_scope);
        let _data = envs.into_iter().next().expect("Query failed").data();
    }
}

fn query_graph<Sg>(
    graph: &mut Sg,
    num_queries: usize,
    order: &LabelOrder<SgLabel>,
    reg: &RegexAutomaton<SgLabel>,
) -> Vec<QueryResult<SgLabel, SgData>>
where
    Sg: ScopeGraph<SgLabel, SgData>,
{
    let mut thread_rng = rand::rng();
    let order = LabelOrderBuilder::new()
        .push(SgLabel::Declaration, SgLabel::Parent)
        .build();

    // P*D;
    let label_reg = Regex::concat(Regex::kleene(SgLabel::Parent), SgLabel::Declaration);
    let reg = RegexAutomaton::from_regex(label_reg.clone());
    let mut envs = Vec::new();
    for _ in 0..num_queries {
        // let start_scope = Scope(START_SCOPE);
        let start_scope = Scope(thread_rng.random_range(200..300));

        let m: Arc<str> = Arc::from("x");
        // let m = matches[thread_rng.random_range(0..matches.len())].clone();

        envs = graph.query_proj(
            start_scope,
           & reg,
            &order,
            |d| Arc::from(d.name()),
            m,
        );
    }
    graph.reset_cache(); // make next benchmark run from scratch
    envs
}

const START_SCOPE: usize = 280;

fn get_pattern() -> Vec<GraphPattern> {
    vec![
        GraphPattern::Decl(SgData::var("x", "int")),
        GraphPattern::Decl(SgData::var("x1", "int")),
        GraphPattern::Decl(SgData::var("x2", "int")),
        GraphPattern::Decl(SgData::var("x3", "int")),
        GraphPattern::Decl(SgData::var("x4", "int")),
        GraphPattern::Decl(SgData::var("x5", "int")),
        GraphPattern::Decl(SgData::var("x6", "int")),
        GraphPattern::Decl(SgData::var("x7", "int")),
        GraphPattern::Decl(SgData::var("x8", "int")),
        GraphPattern::Decl(SgData::var("x9", "int")),
        GraphPattern::Linear(30),
        GraphPattern::Linear(1),
        // GraphPattern::Tree(2),
        // GraphPattern::Diamond(50),
        GraphPattern::Decl(SgData::var("y", "int")),
        GraphPattern::Linear(300),
    ]
}


fn lib_graph_builder(graph: LibGraph) -> LibGraph
{
    GraphGenerator::new(graph).with_patterns(get_pattern()).build_sg()
}


fn graph_builder<Sg>(graph: Sg) -> Sg
where
    Sg: ScopeGraph<SgLabel, SgData>,
{
    GraphGenerator::new(graph).with_patterns(get_pattern()).build()
}

pub fn criterion_benchmark(c: &mut Criterion) {
    let storage = Storage::new();
    unsafe {
        let mut lib_graph: LibGraph = LibGraph::new(&storage, UncheckedCompleteness::new());
    
        lib_graph = lib_graph_builder(lib_graph);
        lib_graph
            .render_to("output/bench/libgraph.mmd", RenderSettings::default())
            .unwrap();

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
            let s2 = format!("cache bench {}", num_bench);
            let s3 = format!("lib {}", num_bench);
            group.bench_function(&s1, |b| {
                b.iter(|| query_graph(&mut graph, num_bench, &order, &matcher))
            });
            group.bench_function(&s2, |b| {
                b.iter(|| query_graph(&mut bu_graph, num_bench, &order, &matcher))
            });
            group.bench_function(&s3, |b| {
                b.iter(|| query_libgraph(&mut lib_graph, num_bench))
            });
        }
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

#[cfg(test)]
mod tests {
    use crate::{graph_builder, query_libgraph};

    #[test]
    fn test_query() {
        let mut graph = CachedScopeGraph::new();
        graph = graph_builder(graph);
        let num_bench = 1;
        let query = query_graph(&mut graph, num_bench);
        graph
            .as_uml_diagram("graph", true)
            .write_to_file("output/bench/graph.puml")
            .unwrap();
        graph
            .as_mmd_diagram("graph", DRAW_CACHES)
            .write_to_file("output/bench/graph.md")
            .unwrap();
    }

    #[test]
    fn test_libquery() {
        let storage = Storage::new();
        let mut lib_graph: LibGraph = LibGraph::new(&storage, ImplicitClose::default());
        lib_graph = lib_graph_builder(lib_graph);
        lib_graph.render_to("output/bench/libgraph.mmd", RenderSettings::default()).unwrap();
        query_libgraph(lib_graph, 2);
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
