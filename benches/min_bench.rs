use criterion::{black_box, criterion_group, criterion_main, Criterion};
use crux::min::ddmin::ddmin_set;
use crux::slice::ast;
use crux::store::hash::content_hash;

fn bench_ddmin(c: &mut Criterion) {
    let input: Vec<String> = (0..100).map(|i| i.to_string()).collect();
    c.bench_function("ddmin_100", |b| {
        b.iter(|| ddmin_set(black_box(&input), &|s| s.len() < 100))
    });
}

fn bench_slice(c: &mut Criterion) {
    let code = "let x = 1;\nlet y = x + 1;\nlet z = y;\nprintln!(\"{}\", z);";
    c.bench_function("slice_vars", |b| {
        b.iter(|| ast::extract_changed_vars(black_box(code)))
    });
}

fn bench_hash(c: &mut Criterion) {
    let content = "x".repeat(10000);
    c.bench_function("hash_10k", |b| {
        b.iter(|| content_hash(black_box(&content)))
    });
}

criterion_group!(benches, bench_ddmin, bench_slice, bench_hash);
criterion_main!(benches);
