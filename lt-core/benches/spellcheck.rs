use criterion::{black_box, criterion_group, criterion_main, Criterion};
use lt_core::suggest_correct_spelling_str;

fn spellcheck() {
    suggest_correct_spelling_str("hello", 5, 3);
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("hello 5", |b| b.iter(|| spellcheck()));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
