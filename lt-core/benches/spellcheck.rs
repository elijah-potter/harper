use criterion::{criterion_group, criterion_main, Criterion};
use lt_core::{suggest_correct_spelling_str, Dictionary};

fn spellcheck(dictionary: &Dictionary) {
    suggest_correct_spelling_str("hello", 5, 3, dictionary);
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("spellcheck");
    group.sample_size(10000);

    let dictionary = Dictionary::new();

    group.bench_function("hello 5", |b| b.iter(|| spellcheck(&dictionary)));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
