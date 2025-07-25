use std::{cell::RefCell, collections::HashMap, iter::Flatten, ops::{Range, RangeInclusive}, sync::{Arc, Mutex}};

use graphing::Renderer;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use rand::{rngs::{SmallRng, ThreadRng}, Rng, SeedableRng};
use crate::{bench_util::{construct_cached_graph, Graph, HEAD_RANGE}, generator::GraphPattern, graph::{CachedScopeGraph, GraphRenderOptions, QueryResult, QueryStats, ScopeGraph}, order::{LabelOrder, LabelOrderBuilder}, regex::{dfs::RegexAutomaton, Regex}, scope::Scope, SgData, SgLabel, SgProjection};
use serde::Serialize;

const QUERY_SIZES: &[usize] = &[1, 2, 5];
const NUM_SUBJECTS: usize = 100;
const NUM_RUNS: usize = 10;
const NUM_WARMUP: usize = 5;

const NUM_DATA: std::ops::RangeInclusive<usize> = 0..=20;
const TAIL_LENGTH: std::ops::RangeInclusive<usize> = 10..=20;
const TAIL_TREE_CHILDS: std::ops::RangeInclusive<usize> = 5..=10;


#[derive(Serialize, Debug, Clone)]
pub enum HeadKind {
    // Chain with length
    Linear(usize),
    // chain of fanouts
    FanChain {
        length: usize,
        num_decl: usize,
    },
}

impl std::fmt::Display for HeadKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HeadKind::Linear(len) => write!(f, "linear-{len}"),
            HeadKind::FanChain { length, num_decl } => write!(f, "fanchain-{length}-{num_decl}"),
        }
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct HeadGenerator {
    kind: HeadKind,
}

impl HeadGenerator {
    pub fn linear(length: usize) -> Self {
        Self { kind: HeadKind::Linear(length) }
    }

    pub fn fan_chain(length: usize, num_decl: usize) -> Self {
        Self { kind: HeadKind::FanChain { length, num_decl } }
    }

    pub fn num_scopes(&self) -> usize {
        match self.kind {
            HeadKind::Linear(len) => len * 2, // every scope has data, so length is len*2
            HeadKind::FanChain { length, num_decl } => length + length * num_decl,
        }
    }

    pub fn pattern(&self) -> Vec<GraphPattern> {
        match self.kind {
            HeadKind::Linear(len) => {
                vec![GraphPattern::LinearDeclLabel(len, SgLabel::Extend)]
            }
            HeadKind::FanChain { length, num_decl } => {
                let mut v = Vec::new();
                let mut cntr = 0;
                for _ in 0..length {
                    v.push(GraphPattern::Linear(1));
                    for _ in 0..num_decl {
                        let x = format!("x_{cntr}");
                        cntr = (cntr + 1) % NUM_DATA.end();
                        v.push(GraphPattern::Decl(SgData::var(x, "int")));
                    }
                }
                v
            }
        }
    }

    pub fn var_range(&self) -> Range<usize> {
        match self.kind {
            HeadKind::Linear(len) => 0..len,
            HeadKind::FanChain { length, num_decl } => 0..(length * num_decl),
        }
    }

    pub fn order(&self) -> LabelOrder<SgLabel> {
        match self.kind {
            // HeadKind::Linear(_) => LabelOrderBuilder::new()
            //     .push(SgLabel::Declaration, SgLabel::Parent)
            //     .build(),
            // HeadKind::FanChain { .. } => LabelOrderBuilder::new()
            //     .push(SgLabel::Declaration, SgLabel::Parent)
            //     .push(SgLabel::Extend, SgLabel::Parent)
                // .build(),
            _ => LabelOrderBuilder::new()
                .push(SgLabel::Declaration, SgLabel::Parent)
                .push(SgLabel::Declaration, SgLabel::Extend)
                .push(SgLabel::Declaration, SgLabel::Implement)
                .push(SgLabel::Extend, SgLabel::Parent)
                .push(SgLabel::Implement, SgLabel::Parent)
                .build(),
        }
    }

    pub fn reg(&self) -> Regex<SgLabel> {
        match self.kind {
            // // P*D
            // HeadKind::Linear(_) => Regex::concat(
            //     Regex::kleene(SgLabel::Parent),
            //     SgLabel::Declaration,
            // ),
            // P*E*D
            // HeadKind::FanChain { .. } => Regex::concat(
            //     Regex::concat_iter([
            //         Regex::kleene(SgLabel::Parent),
            //         Regex::kleene(SgLabel::Extend),
            //     ]),
            //     SgLabel::Declaration,
            // ),

            // complex thing from .stx
            // boils down to P*E*I*D
            _ => Regex::concat(
                Regex::concat_iter([
                    Regex::kleene(SgLabel::Parent),
                    Regex::kleene(SgLabel::Extend),
                    Regex::kleene(SgLabel::Implement),
                ]),
                SgLabel::Declaration,
            ),
        }
    }
}

#[derive(Debug)]
struct TailIndex {
    range: Range<usize>,
    tail_size: usize,
    branches: usize,
}

impl TailIndex {
    pub fn new(branches: usize, head_size: usize, pat: &GraphPattern, tail_size: usize) -> Self {
        // num_branches * chains, and we "skip" tail_size scopes.
        // the first "row" of branches is numbered left to right,
        // while the rest is numbered top to bottom. We want the top-bottom numbering.
        let start_idx = head_size + pat.size() + pat.n_child() * branches + 1;
        let tail_len = branches * tail_size * pat.n_child();
        let range = start_idx..(start_idx + tail_len - 1); // -1 to account for root scope
        Self {
            range,
            tail_size,
            branches: branches * pat.n_child(),
        }
    }

    pub fn sample_branch<R: Rng>(&self, branch: usize, rng: &mut R) -> usize {
        let branch = branch % self.branches;
        let start = self.range.start + branch * self.tail_size;
        let end = (start + self.tail_size).min(self.range.end);
        if start >= end {
            panic!("Invalid range: start {start} >= end {end}");
        }
        rng.random_range(start..end)
    }
}

pub struct PatternGenerator<Args>
{
    generator: fn(&Args) -> GraphPattern,
    args: Vec<Args>,
}

impl<Args> PatternGenerator<Args>
{
    pub fn new(generator: fn(&Args) -> GraphPattern) -> Self {
        Self { generator, args: Vec::new() }
    }

    pub fn with_args(generator: fn(&Args) -> GraphPattern, args: impl IntoIterator<Item = Args>) -> Self {
        let args = args.into_iter().collect();
        Self { generator, args }
    }

    pub fn push(&mut self, arg: Args) {
        self.args.push(arg);
    }

    pub fn generate(&self, arg: &Args) -> GraphPattern {
        (self.generator)(arg)
    }

    pub fn pattern_iter(&self) -> impl IntoIterator<Item = GraphPattern> + '_ {
        self.args.iter().map(|arg| (self.generator)(arg))
    }
}

#[derive(Serialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum QueryType {
    Base,
    Cached,
}

#[derive(Default, Serialize)]
#[serde(transparent)]
pub struct BenchmarkMap {
    /// [name -> head -> arg -> query_type -> stats]
    #[allow(clippy::type_complexity)] // whatever bro
    map: HashMap<String, HashMap<String, HashMap<String, HashMap<QueryType, HashMap<usize, QueryStats>>>>>,
}

impl BenchmarkMap {
    pub fn insert(&mut self, q_type: QueryType, name: impl Into<String>, head: &HeadGenerator, arg: impl Into<String>, stats: Vec<BenchStats>) {
        let named = self.map.entry(name.into()).or_default();
        let head = named.entry(format!("{}", head.kind)).or_default();
        let arg = head.entry(arg.into()).or_default();
        let q_type = arg.entry(q_type).or_default();
        q_type.extend(stats.into_iter().map(|s| (s.num_queries, s.stats)));
    }
    pub fn insert_cached(&mut self, name: impl Into<String>, head: &HeadGenerator, arg: impl Into<String>, stats: Vec<BenchStats>) {
        self.insert(QueryType::Cached, name, head, arg, stats);
    }

    pub fn insert_base(&mut self, name: impl Into<String>, head: &HeadGenerator, arg: impl Into<String>, stats: Vec<BenchStats>) {
        self.insert(QueryType::Base, name, head, arg, stats);
    }

    pub fn extend(&mut self, other: Self) {
        for (name, head_map) in other.map {
            let named = self.map.entry(name).or_default();
            for (head, arg_map) in head_map {
                let head_entry = named.entry(head).or_default();
                for (arg, stats_map) in arg_map {
                    let arg_entry = head_entry.entry(arg).or_default();
                    for (query_type, stats) in stats_map {
                        arg_entry.insert(query_type, stats);
                    }
                }
            }
        }
    }
}


pub struct PatternBencher<'a, Args: std::fmt::Debug + Send + Sync> {
    name: &'a str,
    generator: PatternGenerator<Args>,
}

impl<'a, Args: std::fmt::Debug + Send + Sync> PatternBencher<'a, Args> {
    pub fn new(name: &'a str, generator: PatternGenerator<Args>) -> Self {
        Self {
            generator,
            name,
        }
    }

    pub fn bench<'b>(self, heads: impl IntoIterator<Item = &'b HeadGenerator>) -> BenchmarkMap {
        let multi = MultiProgress::new();
        multi.println(format!("Benchmarking {}", self.name)).unwrap();

        let mut results = BenchmarkMap::default();
        let mut handles = Vec::new();
        let name: Arc<str> = Arc::from(self.name);
        for head in heads {
            for pattern in self.generator.pattern_iter() {
                let name_c = name.clone();
                let h_clone = head.clone();
                let m_bar = multi.clone();
                let h = std::thread::spawn(move || {
                    Self::bench_thread(&name_c, &h_clone, pattern, m_bar)
                });
                handles.push(h);
            }
        }

        for h in handles {
            match h.join() {
                Ok(bench_results) => results.extend(bench_results),
                Err(e) => {
                    println!("Error joining thread: {e:?}");
                    continue;
                }
            }
        }
        results
    }


    fn bench_thread(name: &str, head: &HeadGenerator, pat: GraphPattern, multi_bar: MultiProgress) -> BenchmarkMap {
        let subject_bar = indicatif::ProgressBar::new(NUM_SUBJECTS as u64);
        subject_bar.set_style(ProgressStyle::with_template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
            .unwrap()
            .progress_chars("##-"));

        // total_bar.set_message(format!("head: {:?}, {}: {arg:?}", &head, self.name));
        multi_bar.add(subject_bar.clone());
        subject_bar.set_message(format!("head: {}, {name}: {pat}", &head.kind));
        let mut results = BenchmarkMap::default();
        for n in 0..NUM_SUBJECTS {
            subject_bar.inc(1);
            subject_bar.tick();
            let mut rng = SmallRng::seed_from_u64(n as u64);
            let (graph, start_range) = Self::construct_variation(&mut rng, pat.clone(), head);
            let bencher = VariationBencher::new(graph, name, &mut rng, head, start_range);
            let (res, cached_res) = bencher.bench();
            results.insert_base(name, head, format!("{pat}"), res);
            results.insert_cached(name, head, format!("{pat}"), cached_res);
        }
        results
    }


    fn construct_variation<R: Rng>(rng: &mut R, pattern: GraphPattern, head: &HeadGenerator) -> (Graph, TailIndex) {
        let head_size = head.num_scopes();
        let tail_size = rng.random_range(TAIL_LENGTH);
        let tail_branches = rng.random_range(TAIL_TREE_CHILDS);
        let tail_range = TailIndex::new(
            tail_branches,
            head_size,
            &pattern,
            tail_size,
        );
        let mut pat = head.pattern();
        pat.push(pattern);
        pat.push(GraphPattern::Tree(tail_branches));
        pat.push(GraphPattern::Linear(tail_size));
        let graph = construct_cached_graph(pat);
        (graph, tail_range)
    }
}

struct GraphParams {
    pub order: LabelOrder<SgLabel>,
    pub matcher: RegexAutomaton<SgLabel>,
    pub x: String,
}

impl GraphParams {
    pub fn new(rng: &mut impl Rng, head: &HeadGenerator) -> Self {
        let i = rng.random_range(head.var_range());
        let x = format!("x_{i}");

        let order = head.order();
        let label_reg = head.reg();

        let matcher = RegexAutomaton::from_regex(label_reg.clone());
        Self {
            order,
            matcher,
            x,
        }
    }
}

#[derive(Debug, serde::Serialize)]
pub struct BenchStats {
    pub stats: QueryStats,
    pub num_queries: usize,
}

impl BenchStats {
    pub fn from_stat_iter(num_queries: usize, stats: impl IntoIterator<Item = QueryStats>) -> Self {
        let mut total = QueryStats::default();
        let stats = stats.into_iter().collect::<Vec<_>>();
        let len = stats.len();
        for stats in stats {
            total = total + stats;
        }

        total = total / len;

        Self { stats: total, num_queries }
    }

    fn from_map(map: HashMap<usize, Vec<QueryStats>>) -> Vec<Self> {
        map.into_iter().map(|(n_queries, stats)| {
            Self::from_stat_iter(n_queries, stats)
        }).collect()
    }
}

// type BenchStats = HashMap<usize, Vec<QueryStats>>;
pub(crate) struct VariationBencher<'a, R: Rng> {
    variation: Graph,
    name: &'a str,
    head: &'a HeadGenerator,
    rng: &'a mut R,
    tail_idx: TailIndex,
    runs: HashMap<usize, Vec<QueryStats>>,
    cached_runs: HashMap<usize, Vec<QueryStats>>,
}

impl<'a, R: Rng> VariationBencher<'a, R> {
    fn new(variation: Graph, name: &'a str, rng: &'a mut R, head: &'a HeadGenerator, tail_idx: TailIndex) -> Self {
        Self {
            variation,
            name,
            head,
            rng,
            tail_idx,
            runs: HashMap::new(),
            cached_runs: HashMap::new(),
        }
    }

    fn get_start_scope(&mut self, branch: usize) -> Scope {
        Scope(self.tail_idx.sample_branch(branch, &mut self.rng))
    }

    fn bench(mut self) -> (Vec<BenchStats>, Vec<BenchStats>) {
        for _ in 0..NUM_WARMUP {
            let params = GraphParams::new(&mut self.rng, self.head);
            let start_scope = self.get_start_scope(0);
            self.perform_query(start_scope, &params);
        }
        self.variation.reset_cache();

        for n in QUERY_SIZES {
            for _ in 0..NUM_RUNS {
                let (base, cached) = self.perform_n_queries(*n);
                self.runs.entry(*n).or_default().push(base);
                self.cached_runs.entry(*n).or_default().push(cached);
            }
        }

        (BenchStats::from_map(self.runs), BenchStats::from_map(self.cached_runs))
    }

    fn perform_n_queries(&mut self, num_queries: usize) -> (QueryStats, QueryStats) {
        let (mut base_stat_total, mut cached_stat_total) = (QueryStats::default(), QueryStats::default());
        self.variation.reset_cache();
        for i in 0..num_queries {
            let start_scope = self.get_start_scope(i * 3);
            let params = GraphParams::new(&mut self.rng, self.head);
            let (base_stats, cached_stats) = self.perform_query(start_scope, &params);
            base_stat_total = base_stat_total + base_stats;
            cached_stat_total = cached_stat_total + cached_stats;
        }

        (base_stat_total, cached_stat_total)
    }

    fn perform_query(&mut self, start_scope: Scope, params: &GraphParams) -> (QueryStats, QueryStats) {
        let (base_envs, base_stats)  = self.variation.query_stats(
            start_scope,
            &params.matcher,
            &params.order,
            |d1, d2| d1.name() == d2.name(),
            |data: &SgData| data.name() == params.x.as_str(),
        );

        let x_wfd = Arc::from(params.x.as_str());
        let (cached_envs, cached_stats) = self.variation.query_proj_stats(
            start_scope,
            &params.matcher,
            &params.order,
            SgProjection::VarName,
            x_wfd
        );

        self.cmp_envs(start_scope, params, base_envs, cached_envs);

        (base_stats, cached_stats)
    }

    fn cmp_envs(&self, start_scope: Scope, params: &GraphParams, base: Vec<QueryResult<SgLabel, SgData>>, cached: Vec<QueryResult<SgLabel, SgData>>) {
        if base == cached {
            return;
        }
        println!("Base and cached queries returned different results");
        println!("self.name: {0:?}", self.name);
        println!("start_scope: {start_scope:?}");
        for e in &base {
            println!("Base env: {e:?}");
        }
        for e in &cached {
            println!("Cached env: {e:?}");
        }
        println!("params: {0:?}", params.x);
        let options = GraphRenderOptions {
            draw_caches: true,
            ..Default::default()
        };
        self.variation.as_uml_diagram("error graph", &options)
            .render_to_file("output/benches/error_graph.puml")
            .unwrap();

        panic!("Base and cached queries returned different results");
    }

}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_var() {
        let mut rng = SmallRng::seed_from_u64(0);
        let (g, t) = PatternBencher::<()>::construct_variation(&mut rng, GraphPattern::Tree(3), &HeadGenerator::linear(5));

        println!("t: {0:?}", t);
        g.as_uml_diagram("var", &GraphRenderOptions::default())
            .render_to_file("output/benches/var.puml")
            .unwrap();

        let mut rng = rand::rng();
        println!("{:?}", (
            t.sample_branch(0, &mut rng),
            t.sample_branch(1, &mut rng),
            t.sample_branch(2, &mut rng),
            t.sample_branch(3, &mut rng),
        ))
    }
}