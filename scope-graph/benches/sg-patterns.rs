use scope_graph::generator::GraphPattern;

use scope_graph::bench_util::bench::{BenchmarkMap, HeadGenerator, PatternBencher, PatternGenerator};


pub fn main() {
    let diamond = PatternGenerator::with_args(
        |(n, m)| GraphPattern::Diamond(*n, *m),
        [
            (4, 1), (4, 2), (4, 4), (8, 1), (16, 1)
            // (8, 1), (8, 2), (8, 4),
            // (16, 1), (16, 2), (16, 4),
        ]
    );

    let linear = PatternGenerator::with_args(
        |n| GraphPattern::Linear(*n),
        [20, 40, 80]
    );

    let circle = PatternGenerator::with_args(
        |n| GraphPattern::Circle(*n),
        [4, 16, 64]
    );

    let tree = PatternGenerator::with_args(
        |n| GraphPattern::Tree(*n),
        [40, 80, 160]
    );

    let heads = [
        HeadGenerator::linear(25),
        // HeadGenerator::fan_chain(15, 10),
    ];

    let results = [
        // PatternBencher::new("sg_circle", circle).bench(&heads),
        // PatternBencher::new("sg_tree", tree).bench(&heads),
        PatternBencher::new("sg_linear", linear).bench(&heads),
        // PatternBencher::new("sg_diamond", diamond).bench(&heads),
    ]
    .into_iter()
    .fold(BenchmarkMap::default(), |mut acc, bench| {
        acc.extend(bench);
        acc
    });

    let _ = std::fs::create_dir_all("output/benches");
    let file = std::fs::File::create("output/benches/results.json").unwrap();
    let mut writer = std::io::BufWriter::new(file);
    serde_json::to_writer_pretty(&mut writer, &results).unwrap();
}
