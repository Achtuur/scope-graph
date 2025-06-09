use std::{sync::Arc, time::Duration};

use criterion::{Criterion, criterion_group, criterion_main};
use graphing::Renderer;
use rand::Rng;
use scope_graph::{
    LibGraph, SgData, SgLabel, SgProjection,
    generator::{GraphGenerator, GraphPattern},
    graph::{BaseScopeGraph, CachedScopeGraph, QueryResult, ScopeGraph},
    order::{LabelOrder, LabelOrderBuilder},
    regex::{Regex, dfs::RegexAutomaton},
    scope::Scope,
};
use scopegraphs::{
    Storage, completeness::UncheckedCompleteness, label_order, query_regex, render::RenderSettings,
    resolve::Resolve,
};

use crate::common::{construct_base_graph, construct_cached_graph, construct_libgraph, query_graph, query_libgraph};

mod common;

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
        GraphPattern::Tree(2),
        GraphPattern::Circle(15),
        GraphPattern::Linear(30),
        // GraphPattern::Tree(2),
        // GraphPattern::Diamond(50),
        GraphPattern::Decl(SgData::var("y", "int")),
        GraphPattern::Linear(25),
        GraphPattern::ReverseTree(2),
        GraphPattern::Linear(250),
    ]
}

pub fn criterion_benchmark(c: &mut Criterion) {
    let mut graph = construct_base_graph(get_pattern());
    let mut bu_graph = construct_cached_graph(get_pattern());
    let storage = Storage::new();
    let mut lib_graph = construct_libgraph(&storage, get_pattern());

    bu_graph
        .as_uml_diagram("title", false)
        .render_to_file("output/bench/graph.puml")
        .unwrap();
    bu_graph
        .as_mmd_diagram("title", false)
        .render_to_file("output/bench/graph.md")
        .unwrap();
    lib_graph
        .render_to("output/bench/libgraph.mmd", RenderSettings::default())
        .unwrap();

    let order = LabelOrderBuilder::new()
        .push(SgLabel::Declaration, SgLabel::Parent)
        .build();

    // P*D;
    let label_reg = Regex::concat(Regex::kleene(SgLabel::Parent), SgLabel::Declaration);
    let matcher = RegexAutomaton::from_regex(label_reg.clone());

    let mut group = c.benchmark_group("query");
    // group.warm_up_time(Duration::from_secs(1));
    // group.measurement_time(Duration::from_secs(1));

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
        // group.bench_function(&s3, |b| {
        //     b.iter(|| query_libgraph(&mut lib_graph, num_bench))
        // });
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

#[cfg(test)]
mod tests {

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
        lib_graph
            .render_to("output/bench/libgraph.mmd", RenderSettings::default())
            .unwrap();
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
