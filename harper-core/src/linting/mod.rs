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

use crate::Document;

pub trait Linter: Send + Sync {
    fn lint(&mut self, document: &Document) -> Vec<Lint>;
}
