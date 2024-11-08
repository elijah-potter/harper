use harper_data::{Lrc, Token, TokenStringExt};
use hashbrown::HashSet;

use super::pattern_linter::PatternLinter;
use super::Suggestion;
use crate::Lint;
use crate::LintKind;
use harper_patterns::{Pattern, SequencePattern};

/// Linter that checks if multiple pronouns are being used right after each
/// other. This is a common mistake to make during the revision process.
pub struct MultipleSequentialPronouns {
    pattern: Box<dyn Pattern>,
}

impl MultipleSequentialPronouns {
    fn new() -> Self {
        let pronouns: HashSet<_> = [
            "me", "my", "I", "we", "you", "he", "him", "her", "she", "it", "they",
        ]
        .into_iter()
        .collect();

        let pronouns = Lrc::new(pronouns);

        Self {
            pattern: Box::new(
                SequencePattern::default()
                    .then_any_word_in(pronouns.clone())
                    .then_one_or_more(Box::new(
                        SequencePattern::default()
                            .then_whitespace()
                            .then_any_word_in(pronouns.clone()),
                    )),
            ),
        }
    }
}

impl PatternLinter for MultipleSequentialPronouns {
    fn pattern(&self) -> &dyn Pattern {
        self.pattern.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Lint {
        let mut suggestions = Vec::new();

        if matched_tokens.len() == 3 {
            suggestions.push(Suggestion::ReplaceWith(
                matched_tokens[0].span.get_content(source).to_vec(),
            ));
            suggestions.push(Suggestion::ReplaceWith(
                matched_tokens[2].span.get_content(source).to_vec(),
            ));
        }

        Lint {
            span: matched_tokens.span().unwrap(),
            lint_kind: LintKind::Repetition,
            message: "There are too many personal pronouns in sequence here.".to_owned(),
            priority: 63,
            suggestions,
        }
    }
}

impl Default for MultipleSequentialPronouns {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::MultipleSequentialPronouns;
    use crate::tests::assert_lint_count;

    #[test]
    fn can_detect_two_pronouns() {
        assert_lint_count(
            "...little bit about my I want to do.",
            MultipleSequentialPronouns::new(),
            1,
        )
    }

    #[test]
    fn can_detect_three_pronouns() {
        assert_lint_count(
            "...little bit about my I you want to do.",
            MultipleSequentialPronouns::new(),
            1,
        )
    }

    #[test]
    fn allows_single_pronouns() {
        assert_lint_count(
            "...little bit about I want to do.",
            MultipleSequentialPronouns::new(),
            0,
        )
    }

    #[test]
    fn detects_multiple_pronouns_at_end() {
        assert_lint_count(
            "...little bit about I want to do to me you.",
            MultipleSequentialPronouns::new(),
            1,
        )
    }

    #[test]
    fn comma_separated() {
        assert_lint_count("To prove it, we...", MultipleSequentialPronouns::new(), 0)
    }
}
