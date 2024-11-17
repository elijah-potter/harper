use itertools::Itertools;

use crate::{
    patterns::{Pattern, SequencePattern, WordPatternGroup},
    Token, TokenStringExt,
};

use super::{Lint, LintKind, PatternLinter, Suggestion};

pub struct ThatWhich {
    pattern: Box<dyn Pattern>,
}

impl Default for ThatWhich {
    fn default() -> Self {
        let mut pattern = WordPatternGroup::default();

        let matching_pattern = crate::Lrc::new(
            SequencePattern::default()
                .then_exact_word_or_lowercase("That")
                .then_whitespace()
                .then_exact_word("that"),
        );

        pattern.add("that", Box::new(matching_pattern.clone()));
        pattern.add("That", Box::new(matching_pattern));

        Self {
            pattern: Box::new(pattern),
        }
    }
}

impl PatternLinter for ThatWhich {
    fn pattern(&self) -> &dyn Pattern {
        self.pattern.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Lint {
        let suggestion = format!(
            "{} which",
            matched_tokens[0]
                .span
                .get_content(source)
                .iter()
                .collect::<String>()
        )
        .chars()
        .collect_vec();

        Lint {
            span: matched_tokens.span().unwrap(),
            lint_kind: LintKind::Repetition,
            suggestions: vec![Suggestion::ReplaceWith(suggestion)],
            message: "“that that” sometimes means “that which”, which is clearer.".to_string(),
            priority: 126,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::tests::assert_lint_count;
    use super::ThatWhich;

    #[test]
    fn catches_lowercase() {
        assert_lint_count(
            "To reiterate, that that is cool is not uncool.",
            ThatWhich::default(),
            1,
        );
    }

    #[test]
    fn catches_different_cases() {
        assert_lint_count("That that is cool is not uncool.", ThatWhich::default(), 1);
    }

    #[test]
    fn likes_correction() {
        assert_lint_count(
            "To reiterate, that which is cool is not uncool.",
            ThatWhich::default(),
            0,
        );
    }
}
