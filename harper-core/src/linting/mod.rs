mod an_a;
mod correct_number_suffix;
mod lint;
mod lint_group;
mod long_sentences;
mod matcher;
mod multiple_sequential_pronouns;
mod number_suffix_capitalization;
mod repeated_words;
mod sentence_capitalization;
mod spaces;
mod spell_check;
mod spelled_numbers;
mod unclosed_quotes;
mod wrong_quotes;

pub use lint::{Lint, LintKind, Suggestion};
pub use lint_group::{LintGroup, LintGroupConfig};

use crate::Document;

pub trait Linter: Send + Sync {
    fn lint(&mut self, document: &Document) -> Vec<Lint>;
}

#[cfg(test)]
mod tests {
    use crate::parsers::Markdown;
    use crate::{Document, Linter};

    pub fn assert_lint_count(text: &str, mut linter: impl Linter, count: usize) {
        let test = Document::new(text, Box::new(Markdown));
        let lints = linter.lint(&test);
        dbg!(&lints);
        assert_eq!(lints.len(), count);
    }
}
