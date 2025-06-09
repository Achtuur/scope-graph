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

pub fn construct_libgraph<'a>(storage: &'a Storage, pattern: Vec<GraphPattern>) -> LibGraph<'a> {
    let lib_graph: LibGraph = unsafe {LibGraph::new(storage, UncheckedCompleteness::new()) };
    GraphGenerator::new(lib_graph)
        .with_patterns(pattern)
        .build_sg()
}

pub fn construct_cached_graph(pattern: Vec<GraphPattern>) -> CachedScopeGraph<SgLabel, SgData> {
    let graph = CachedScopeGraph::<SgLabel, SgData>::new();
    let g = GraphGenerator::new(graph)
        .with_patterns(pattern)
        .build();
    Scope::reset_counter();
    g
}

pub fn construct_base_graph(pattern: Vec<GraphPattern>) -> BaseScopeGraph<SgLabel, SgData> {
    let graph = BaseScopeGraph::<SgLabel, SgData>::new();
    let g = GraphGenerator::new(graph)
        .with_patterns(pattern)
        .build();
    Scope::reset_counter();
    g
}

const START_SCOPE: usize = 280;

pub fn query_libgraph(graph: &mut LibGraph, num_queries: usize) {
    let mut thread_rng = rand::rng();
    // let start_scope = scopegraphs::Scope(START_SCOPE);
    let start_scope = scopegraphs::Scope(thread_rng.random_range(200..300));

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

pub fn query_graph<Sg>(
    graph: &mut Sg,
    num_queries: usize,
    order: &LabelOrder<SgLabel>,
    reg: &RegexAutomaton<SgLabel>,
) -> Vec<QueryResult<SgLabel, SgData>>
where
    Sg: ScopeGraph<SgLabel, SgData>,
{
    let mut thread_rng = rand::rng();
    let mut envs = Vec::new();
    for _ in 0..num_queries {
        let start_scope = Scope(START_SCOPE);
        // let start_scope = Scope(thread_rng.random_range(200..250));

        let m: Arc<str> = Arc::from("x");
        // let m = matches[thread_rng.random_range(0..matches.len())].clone();

        envs = graph.query_proj(start_scope, &reg, &order, SgProjection::VarName, m);
    }
    graph.reset_cache(); // make next benchmark run from scratch
    envs
}
