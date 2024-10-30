mod an_a;
mod avoid_curses;
mod boring_words;
mod correct_number_suffix;
mod dot_initialisms;
mod ellipsis_length;
mod linking_verbs;
mod lint;
mod lint_group;
mod long_sentences;
mod matcher;
mod multiple_sequential_pronouns;
mod number_suffix_capitalization;
mod pattern_linter;
mod repeated_words;
mod sentence_capitalization;
mod spaces;
mod spell_check;
mod spelled_numbers;
mod terminating_conjunctions;
mod unclosed_quotes;
mod use_genitive;
mod wrong_quotes;

pub use an_a::AnA;
pub use avoid_curses::AvoidCurses;
pub use boring_words::BoringWords;
pub use correct_number_suffix::CorrectNumberSuffix;
pub use dot_initialisms::DotInitialisms;
pub use ellipsis_length::EllipsisLength;
pub use linking_verbs::LinkingVerbs;
pub use lint::{Lint, LintKind, Suggestion};
pub use lint_group::{LintGroup, LintGroupConfig};
pub use long_sentences::LongSentences;
pub use matcher::Matcher;
pub use multiple_sequential_pronouns::MultipleSequentialPronouns;
pub use number_suffix_capitalization::NumberSuffixCapitalization;
pub use pattern_linter::PatternLinter;
pub use repeated_words::RepeatedWords;
pub use sentence_capitalization::SentenceCapitalization;
pub use spaces::Spaces;
pub use spell_check::SpellCheck;
pub use spelled_numbers::SpelledNumbers;
pub use terminating_conjunctions::TerminatingConjunctions;
pub use unclosed_quotes::UnclosedQuotes;
pub use use_genitive::UseGenitive;
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

    /// Runs a provided linter on text, applies the first suggestion from each
    /// lint and asserts that the result is equal to a given value.
    pub fn assert_suggestion_result(text: &str, mut linter: impl Linter, expected_result: &str) {
        let test = Document::new_markdown_curated(text);
        let lints = linter.lint(&test);

        let mut text: Vec<char> = text.chars().collect();

        for lint in lints {
            if let Some(sug) = lint.suggestions.first() {
                sug.apply(lint.span, &mut text);
            }
        }

        let transformed_str: String = text.iter().collect();

        assert_eq!(transformed_str.as_str(), expected_result);
    }
}
