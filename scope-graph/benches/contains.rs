use std::{hash::Hash, hint::black_box, mem::MaybeUninit};

use criterion::{Criterion, criterion_group, criterion_main};
use rand::{Rng, SeedableRng, rngs::SmallRng};
use scope_graph::util::ContainsContainer;

fn bench_fn<'a, const N: usize>(
    c: &mut ContainsContainer<'a, usize, N>,
    list: &'a [usize],
) -> usize {
    let mut num_dup = 0;

    for i in list {
        if c.insert(i) {
            num_dup += 1;
        }
    }
    num_dup
}

pub fn criterion_benchmark(c: &mut Criterion) {
    const NUM_ITER: usize = 1000;
    #[derive(PartialEq, Eq, Hash)]
    pub enum Label {
        A,
        B,
        C,
    }

    // let mut arr2: ContainsContainer<'_, _, 16> = ContainsContainer::new();
    // let mut arr3: ContainsContainer<'_, _, 32> = ContainsContainer::new();

    let mut group = c.benchmark_group("contains");
    let mut rng = SmallRng::seed_from_u64(42);
    let v = (0..NUM_ITER)
        .map(|_| rng.random_range(0..NUM_ITER))
        .collect::<Vec<_>>();

    group.bench_function("arr 8", |b| {
        let mut arr: ContainsContainer<'_, _, 8> = ContainsContainer::new();
        b.iter(black_box(|| bench_fn(&mut arr, &v)));
    });

    group.bench_function("arr 16", |b| {
        let mut arr: ContainsContainer<'_, _, 16> = ContainsContainer::new();
        b.iter(black_box(|| bench_fn(&mut arr, &v)));
    });

    group.bench_function("arr 32", |b| {
        let mut arr: ContainsContainer<'_, _, 32> = ContainsContainer::new();
        b.iter(black_box(|| bench_fn(&mut arr, &v)));
    });

    group.bench_function("arr 64", |b| {
        let mut arr: ContainsContainer<'_, _, 64> = ContainsContainer::new();
        b.iter(black_box(|| bench_fn(&mut arr, &v)));
    });

    group.bench_function("arr 128", |b| {
        let mut arr: ContainsContainer<'_, _, 128> = ContainsContainer::new();
        b.iter(black_box(|| bench_fn(&mut arr, &v)));
    });

    group.bench_function("arr 256", |b| {
        let mut arr: ContainsContainer<'_, _, 256> = ContainsContainer::new();
        b.iter(black_box(|| bench_fn(&mut arr, &v)));
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
