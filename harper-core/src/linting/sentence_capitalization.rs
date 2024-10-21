use itertools::Itertools;

use super::lint::Suggestion;
use super::{Lint, LintKind, Linter};
use crate::document::Document;
use crate::TokenStringExt;

#[derive(Debug, Clone, Copy, Default)]
pub struct SentenceCapitalization;

impl Linter for SentenceCapitalization {
    /// A linter that checks to make sure the first word of each sentence is
    /// capitalized.
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut lints = Vec::new();

        for sentence in document.iter_sentences() {
            if let Some(first_word) = sentence.first_non_whitespace() {
                if !first_word.kind.is_word() {
                    continue;
                }

                let letters = document.get_span_content(first_word.span);

                if let Some(first_letter) = letters.first() {
                    if first_letter.is_alphabetic() && !first_letter.is_uppercase() {
                        lints.push(Lint {
                            span: first_word.span.with_len(1),
                            lint_kind: LintKind::Capitalization,
                            suggestions: vec![Suggestion::ReplaceWith(
                                first_letter.to_uppercase().collect_vec(),
                            )],
                            priority: 31,
                            message: "This sentence does not start with a capital letter"
                                .to_string(),
                        })
                    }
                }
            }
        }

        lints
    }
}

#[cfg(test)]
mod tests {
    use super::super::tests::assert_lint_count;
    use super::SentenceCapitalization;

    #[test]
    fn catches_basic() {
        assert_lint_count("there is no way.", SentenceCapitalization, 1)
    }

    #[test]
    fn no_period() {
        assert_lint_count("there is no way", SentenceCapitalization, 1)
    }

    #[test]
    fn two_sentence() {
        assert_lint_count(
            "i have complete conviction. she is guilty",
            SentenceCapitalization,
            2,
        )
    }

    #[test]
    fn start_with_number() {
        assert_lint_count(
            "53 is the length of the longest word.",
            SentenceCapitalization,
            0,
        );
    }

    #[test]
    fn ignores_unlintable() {
        assert_lint_count(
            "[`misspelled_word`] is assumed to be quite small (n < 100). ",
            SentenceCapitalization,
            0,
        )
    }

    #[test]
    fn unphased_unlintable() {
        assert_lint_count(
            "the linter should not be affected by `this` unlintable.",
            SentenceCapitalization,
            1,
        )
    }

    #[test]
    fn unphased_ellipsis() {
        assert_lint_count(
            "the linter should not be affected by... that ellipsis.",
            SentenceCapitalization,
            1,
        )
    }
}
