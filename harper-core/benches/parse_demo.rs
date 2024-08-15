use divan::{black_box, AllocProfiler, Bencher};
use harper_core::{Document, FullDictionary, LintGroup, LintGroupConfig, Linter};

#[global_allocator]
static ALLOC: AllocProfiler = AllocProfiler::system();

static DEMO: &str = include_str!("../../demo.md");

#[divan::bench]
fn parse_demo(bencher: Bencher) {
    bencher.bench_local(|| {
        let _document = Document::new_markdown_curated(black_box(DEMO));
    });
}

#[divan::bench]
fn lint_demo(bencher: Bencher) {
    let dictionary = FullDictionary::curated();
    let mut lint_set = LintGroup::new(Default::default(), dictionary);
    let document = Document::new_markdown_curated(black_box(DEMO));

    bencher.bench_local(|| {
        lint_set.lint(&document);
    });
}

#[divan::bench]
fn lint_demo_uncached(bencher: Bencher) {
    let dictionary = FullDictionary::curated();
    bencher.bench_local(|| {
        let mut lint_set = LintGroup::new(LintGroupConfig::default(), dictionary.clone());
        let document = Document::new_markdown_curated(black_box(DEMO));

        lint_set.lint(&document);
    });
}

fn main() {
    divan::main();
}
