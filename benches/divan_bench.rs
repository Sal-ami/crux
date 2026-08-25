//! Divan terminal benchmarks: cargo bench --bench divan_bench
use divan::{Bencher, black_box};

fn main() {
    divan::main();
}

#[divan::bench(name = "hash_100b")]
fn hash_100b(b: Bencher) {
    let s = "x".repeat(100);
    b.bench(|| crux::store::hash::content_hash(black_box(&s)));
}

#[divan::bench(name = "hash_10kb")]
fn hash_10kb(b: Bencher) {
    let s = "x".repeat(10000);
    b.bench(|| crux::store::hash::content_hash(black_box(&s)));
}

#[divan::bench(name = "ddmin_100")]
fn ddmin_100(b: Bencher) {
    let input: Vec<String> = (0..100).map(|i| i.to_string()).collect();
    b.bench(|| crux::min::ddmin::ddmin_set(black_box(&input), &|s| s.len() < 100));
}

#[divan::bench(name = "extract_vars")]
fn extract_vars(b: Bencher) {
    let code = "let x = 1;\nlet y = x + 1;\nlet z = y;\nprintln!(\"{}\", z);";
    b.bench(|| crux::slice::ast::extract_changed_vars(black_box(code)));
}

#[divan::bench(name = "extract_calls")]
fn extract_calls(b: Bencher) {
    let code = "foo(bar(), baz(1), qux(x))";
    b.bench(|| crux::slice::ast::extract_function_calls(black_box(code)));
}
