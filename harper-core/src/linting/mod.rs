mod lint;
mod lint_set;
mod long_sentences;
mod repeated_words;
mod sentence_capitalization;
mod spell_check;
mod unclosed_quotes;
mod wrong_quotes;

pub use lint::{Lint, LintKind, Suggestion};
pub use lint_set::LintSet;

use crate::{Dictionary, Document};

pub fn run_lint_set(lint_set: &LintSet, document: &Document, dictionary: &Dictionary) -> Vec<Lint> {
    let mut lints = Vec::new();

    for linter in &lint_set.linters {
        lints.append(&mut linter(document, dictionary));
    }

    lints.sort_by_key(|lint| lint.span.start);

    lints
}
