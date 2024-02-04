use divan::{black_box, AllocProfiler, Bencher};
use harper_core::{Dictionary, Document, LintSet, Linter};

#[global_allocator]
static ALLOC: AllocProfiler = AllocProfiler::system();

static DEMO: &str = include_str!("../../demo.md");

#[divan::bench]
fn parse_demo(bencher: Bencher) {
    bencher.bench_local(|| {
        let _document = Document::new_markdown(black_box(DEMO));
    });
}

#[divan::bench]
fn create_lint_set(bencher: Bencher) {
    let dictionary = Dictionary::create_from_curated();

    bencher.bench_local(|| {
        let _lint_set = LintSet::new().with_standard(dictionary.clone());
    });
}

#[divan::bench]
fn lint_demo(bencher: Bencher) {
    let dictionary = Dictionary::create_from_curated();
    let mut lint_set = LintSet::new().with_standard(dictionary);
    let document = Document::new_markdown(black_box(DEMO));

    bencher.bench_local(|| {
        lint_set.lint(&document);
    });
}

fn main() {
    divan::main();
}
