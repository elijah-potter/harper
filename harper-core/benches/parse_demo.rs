use criterion::{black_box, criterion_group, criterion_main, Criterion};
use harper_core::linting::{LintGroup, LintGroupConfig, Linter};
use harper_core::{Document, FstDictionary};

static DEMO: &str = include_str!("../../demo.md");

fn parse_demo(c: &mut Criterion) {
    c.bench_function("parse_demo", |b| {
        b.iter(|| Document::new_markdown_curated(black_box(DEMO)))
    });
}

fn lint_demo(c: &mut Criterion) {
    let dictionary = FstDictionary::curated();
    let mut lint_set = LintGroup::new(Default::default(), dictionary);
    let document = Document::new_markdown_curated(black_box(DEMO));

    c.bench_function("lint_demo", |b| {
        b.iter(|| lint_set.lint(&document, None));
    });
}

fn lint_demo_uncached(c: &mut Criterion) {
    c.bench_function("lint_demo_uncached", |b| {
        b.iter(|| {
            let dictionary = FstDictionary::curated();
            let mut lint_set = LintGroup::new(LintGroupConfig::default(), dictionary.clone());
            let document = Document::new_markdown(black_box(DEMO), &dictionary);
            lint_set.lint(&document, None)
        })
    });
}

pub fn criterion_benchmark(c: &mut Criterion) {
    parse_demo(c);
    lint_demo(c);
    lint_demo_uncached(c);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
