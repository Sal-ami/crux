use criterion::{black_box, criterion_group, criterion_main, Criterion};
use crux::slice::ast;

fn bench_extract_vars(c: &mut Criterion) {
    let code = "let x = 1;\nlet y = x + 1;\nlet z = y;\nprintln!(\"{}\", z);";
    c.bench_function("extract_vars", |b| {
        b.iter(|| ast::extract_changed_vars(black_box(code)))
    });
}

fn bench_extract_calls(c: &mut Criterion) {
    let code = "foo(bar(), baz(1), qux(x))";
    c.bench_function("extract_calls", |b| {
        b.iter(|| ast::extract_function_calls(black_box(code)))
    });
}

criterion_group!(benches, bench_extract_vars, bench_extract_calls);
criterion_main!(benches);
