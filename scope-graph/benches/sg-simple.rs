use std::sync::atomic::Ordering;

use criterion::{Criterion, criterion_group, criterion_main};
use scope_graph::{
    SgLabel,
    generator::GraphPattern,
    order::LabelOrderBuilder,
    regex::{Regex, dfs::RegexAutomaton},
};

use crate::common::{SEED, construct_graph, query_graph, query_graph_cached};

mod common;

const NUM_QUERIES: [usize; 3] = [1, 2, 5];
// const SIZES: [usize; 6] = [2, 4, 8, 16, 32, 64];
// const SIZES: [usize; 3] = [4, 16, 64];
const SIZES: [usize; 1] = [16];

pub fn benchmark_pattern(c: &mut Criterion, name: &str, pattern_fn: fn(usize) -> GraphPattern) {
    let order = LabelOrderBuilder::new()
        .push(SgLabel::Declaration, SgLabel::Parent)
        .build();

    let label_reg = Regex::concat(Regex::kleene(SgLabel::Parent), SgLabel::Declaration);
    let matcher = RegexAutomaton::from_regex(label_reg.clone());

    let mut group = c.benchmark_group(name);
    // group.sample_size(10);
    for num_queries in NUM_QUERIES {
        for n in SIZES {
            SEED.fetch_and(0, Ordering::SeqCst);
            group.bench_function(format!("{name}_{num_queries}_{n}"), |b| {
                let (mut graph, head_size, tail_size) = construct_graph(pattern_fn(n));
                let start_range = (head_size + n)..(head_size + n + tail_size);

                b.iter(|| {
                    query_graph(
                        &mut graph,
                        start_range.clone(),
                        num_queries,
                        &order,
                        &matcher,
                    )
                });
            });

            SEED.fetch_and(0, Ordering::SeqCst);
            group.bench_function(format!("{name}_cached_{num_queries}_{n}"), |b| {
                let (mut graph, head_size, tail_size) = construct_graph(pattern_fn(n));
                let start_range = (head_size + n)..(head_size + n + tail_size);
                b.iter(|| {
                    query_graph_cached(
                        &mut graph,
                        start_range.clone(),
                        num_queries,
                        &order,
                        &matcher,
                    )
                });
            });
        }
    }
    group.finish();
}

pub fn criterion_benchmark(c: &mut Criterion) {
    benchmark_pattern(c, "sg_diamond", GraphPattern::Diamond);
    benchmark_pattern(c, "sg_linear", GraphPattern::Linear);
    benchmark_pattern(c, "sg_circle", GraphPattern::Circle);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
