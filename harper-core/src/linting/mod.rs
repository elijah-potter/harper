mod an_a;
mod correct_number_suffix;
mod linking_verbs;
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

pub use an_a::AnA;
pub use correct_number_suffix::CorrectNumberSuffix;
pub use linking_verbs::LinkingVerbs;
pub use lint::{Lint, LintKind, Suggestion};
pub use lint_group::{LintGroup, LintGroupConfig};
pub use long_sentences::LongSentences;
pub use matcher::Matcher;
pub use multiple_sequential_pronouns::MultipleSequentialPronouns;
pub use number_suffix_capitalization::NumberSuffixCapitalization;
pub use repeated_words::RepeatedWords;
pub use sentence_capitalization::SentenceCapitalization;
pub use spaces::Spaces;
pub use spell_check::SpellCheck;
pub use spelled_numbers::SpelledNumbers;
pub use unclosed_quotes::UnclosedQuotes;
pub use wrong_quotes::WrongQuotes;

use crate::Document;

#[cfg(not(feature = "concurrent"))]
pub trait Linter {
    fn lint(&mut self, document: &Document) -> Vec<Lint>;
}
#[cfg(feature = "concurrent")]
pub trait Linter: Send + Sync {
    fn lint(&mut self, document: &Document) -> Vec<Lint>;
}

#[cfg(test)]
mod tests {
    use super::Linter;
    use crate::Document;

    pub fn assert_lint_count(text: &str, mut linter: impl Linter, count: usize) {
        let test = Document::new_markdown_curated(text);
        let lints = linter.lint(&test);
        dbg!(&lints);
        assert_eq!(lints.len(), count);
    }
}
