use criterion::{criterion_group, criterion_main, Criterion};
use harper_core::{suggest_correct_spelling_str, Dictionary};

fn spellcheck(dictionary: &Dictionary) {
    suggest_correct_spelling_str("hello", 5, 3, dictionary);
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("spellcheck");

    let dictionary = Dictionary::new();

    group.bench_function("dict create", |b| b.iter(Dictionary::new));
    group.bench_function("hello 5", |b| b.iter(|| spellcheck(dictionary)));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
