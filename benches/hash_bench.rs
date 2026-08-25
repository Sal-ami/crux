use criterion::{black_box, criterion_group, criterion_main, Criterion};
use crux::store::hash::content_hash;

fn bench_hash_small(c: &mut Criterion) {
    c.bench_function("hash_100b", |b| {
        b.iter(|| content_hash(black_box("x".repeat(100).as_str())))
    });
}

fn bench_hash_large(c: &mut Criterion) {
    c.bench_function("hash_10kb", |b| {
        b.iter(|| content_hash(black_box("x".repeat(10000).as_str())))
    });
}

criterion_group!(benches, bench_hash_small, bench_hash_large);
criterion_main!(benches);
