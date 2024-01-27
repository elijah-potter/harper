use criterion::{black_box, criterion_group, criterion_main, Criterion};
use harper_core::{Dictionary, Document, LintSet, Linter};

fn parse_demo(c: &mut Criterion) {
    let demo = include_str!("../../demo.md");

    c.bench_function("parse_demo", |b| {
        b.iter(|| {
            let _document = Document::new_markdown(demo);
        })
    });

    let dictionary = Dictionary::new();

    c.bench_function("create_lint_set", |b| {
        b.iter(|| {
            let _lint_set = LintSet::new().with_standard(dictionary.clone());
        })
    });

    let mut lint_set = LintSet::new().with_standard(dictionary);
    let document = Document::new_markdown(demo);

    c.bench_function("lint_demo", |b| {
        b.iter(|| {
            lint_set.lint(&document);
        })
    });
}

criterion_group!(benches, parse_demo);
criterion_main!(benches);
