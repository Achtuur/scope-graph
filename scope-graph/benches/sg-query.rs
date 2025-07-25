use criterion::{Criterion, criterion_group, criterion_main};
use graphing::Renderer;
use scope_graph::{bench_util::{construct_cached_graph, query_graph, query_graph_cached}, generator::GraphPattern, graph::{GraphRenderOptions, ScopeGraph}, order::LabelOrderBuilder, regex::{dfs::RegexAutomaton, Regex}, SgData, SgLabel};


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
    let mut bu_graph = construct_cached_graph(get_pattern());


    bu_graph
        .as_uml_diagram("title", &GraphRenderOptions::default())
        .render_to_file("output/bench/graph.puml")
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

    for num_bench in [2] {
        let s1 = format!("bench {}", num_bench);
        let s2 = format!("cache bench {}", num_bench);
        group.bench_function(&s1, |b| {
            b.iter(|| query_graph(&mut bu_graph, 160..250, num_bench, &order, &matcher))
        });
        group.bench_function(&s2, |b| {
            b.iter(|| query_graph_cached(&mut bu_graph, 160..250, num_bench, &order, &matcher))
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
    #[test]
    fn test_rand() {
        let (g1, _, _) = construct_graph(GraphPattern::Diamond(2));
        SEED.fetch_and(0, std::sync::atomic::Ordering::SeqCst);
        let (g2, _, _) = construct_graph(GraphPattern::Diamond(2));

        g1.as_uml_diagram("yea", false)
            .write_to_file("output/bench/graph1.puml")
            .unwrap();
        g2.as_uml_diagram("yea", false)
            .write_to_file("output/bench/graph2.puml")
            .unwrap();
    }
}
