mod lint;
mod lint_set;
mod long_sentences;
mod repeated_words;
mod sentence_capitalization;
mod spaces;
mod spell_check;
mod unclosed_quotes;
mod wrong_quotes;

pub use lint::{Lint, LintKind, Suggestion};
pub use lint_set::LintSet;

use crate::Document;

pub trait Linter: Send + Sync {
    fn lint(&mut self, document: &Document) -> Vec<Lint>;
}

#[cfg(test)]
mod tests {
    use crate::{parsers::PlainEnglishParser, Document, Linter};

    pub fn assert_lint_count(text: &str, mut linter: impl Linter, count: usize) {
        let test = Document::new(text, Box::new(PlainEnglishParser));
        let lints = linter.lint(&test);
        assert_eq!(lints.len(), count);
    }
}
