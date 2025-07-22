pub mod bench;

use std::sync::{atomic::AtomicUsize, Arc, Mutex};

use rand::{Rng, SeedableRng, rngs::SmallRng};
use crate::{
    LibGraph, SgData, SgLabel, SgProjection,
    generator::{GraphGenerator, GraphPattern},
    graph::{CachedScopeGraph, QueryResult, ScopeGraph},
    order::LabelOrder,
    regex::dfs::RegexAutomaton,
    scope::Scope,
};
use scopegraphs::{
    Storage, completeness::UncheckedCompleteness, label_order, query_regex, resolve::Resolve,
};

const HEAD_RANGE: std::ops::RangeInclusive<usize> = 1..=20;
const TAIL_RANGE: std::ops::RangeInclusive<usize> = 1..=20;

pub static SEED: AtomicUsize = AtomicUsize::new(0);

pub type Graph = CachedScopeGraph<SgLabel, SgData>;

pub fn construct_graph(pattern: GraphPattern) -> (CachedScopeGraph<SgLabel, SgData>, usize, usize) {
    let mut rand =
        SmallRng::seed_from_u64(SEED.fetch_add(1, std::sync::atomic::Ordering::Relaxed) as u64);
    let head_size = rand.random_range(HEAD_RANGE);
    let tail_size = rand.random_range(TAIL_RANGE);
    let pattern = [
        GraphPattern::Decl(SgData::var("x", "int")),
        // GraphPattern::Linear(head_size),
        GraphPattern::LinearDecl(head_size),
        pattern,
        GraphPattern::Linear(tail_size),
    ];
    let graph = construct_cached_graph(pattern);
    (graph, head_size, tail_size)
}

// pub fn construct_libgraph(storage: &Storage, pattern: Vec<GraphPattern>) -> LibGraph<'_> {
//     let lib_graph: LibGraph = unsafe { LibGraph::new(storage, UncheckedCompleteness::new()) };
//     GraphGenerator::new(lib_graph)
//         .with_patterns(pattern)
//         .build_sg()
// }

static SG_CREATION_LOCK: Mutex<()> = Mutex::new(());

pub fn construct_cached_graph(
    pattern: impl IntoIterator<Item = GraphPattern>,
) -> CachedScopeGraph<SgLabel, SgData> {
    let _lock = SG_CREATION_LOCK.lock().unwrap();
    let graph = CachedScopeGraph::<SgLabel, SgData>::new();
    let g = GraphGenerator::new(graph).with_patterns(pattern).build();
    Scope::reset_counter();
    g
}

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
    start_scope_range: std::ops::Range<usize>,
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
        let start_scope = Scope(thread_rng.random_range(start_scope_range.clone()));
        // let start_scope = Scope(START_SCOPE);

        // let m: Arc<str> = Arc::from("x");
        // let m = matches[thread_rng.random_range(0..matches.len())].clone();
        let x = thread_rng.random_range(HEAD_RANGE.clone());
        let m = format!("x_{}", x);

        envs = graph.query(
            start_scope,
            reg,
            order,
            |d1, d2| d1.name() == d2.name(),
            |data: &SgData| data.name() == m.as_str(),
        );
    }
    envs
}

pub fn query_graph_cached<Sg>(
    graph: &mut Sg,
    start_scope_range: std::ops::Range<usize>,
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
        let start_scope = Scope(thread_rng.random_range(start_scope_range.clone()));

        let x = thread_rng.random_range(HEAD_RANGE.clone());
        let m = format!("x_{}", x);
        let m_wfd: Arc<str> = Arc::from(m.as_str());
        // let m = matches[thread_rng.random_range(0..matches.len())].clone();

        envs = graph.query_proj(start_scope, reg, order, SgProjection::VarName, m_wfd);
    }
    graph.reset_cache(); // make next benchmark run from scratch
    envs
}
