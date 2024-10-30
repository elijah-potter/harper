use crate::linting::{LintKind, PatternLinter, Suggestion};
use crate::patterns::{EitherPattern, Pattern, SequencePattern, WordPatternGroup};
use crate::{Lint, Lrc, Token};

// Looks for places where the genitive case _isn't_ being used, and should be.
pub struct UseGenitive {
    pattern: Box<dyn Pattern>,
}

impl UseGenitive {
    fn new() -> Self {
        // Define the environment in which the genitive case should be used in.
        let environment = Lrc::new(SequencePattern::default().then_whitespace().then(Box::new(
            EitherPattern::new(vec![
                                Box::new(
                                    SequencePattern::default()
                                        .then_one_or_more_adjectives()
                                        .then_whitespace()
                                        .then_noun(),
                                ),
                                Box::new(SequencePattern::default().then_noun()),
                            ]),
        )));

        let trigger_words = ["there", "they're"];

        let mut pattern = WordPatternGroup::default();

        for word in trigger_words {
            pattern.add(
                word,
                Box::new(
                    SequencePattern::default()
                        .then_exact_word(word)
                        .then(Box::new(environment.clone())),
                ),
            )
        }

        Self {
            pattern: Box::new(pattern),
        }
    }
}

impl PatternLinter for UseGenitive {
    fn pattern(&self) -> &dyn crate::patterns::Pattern {
        self.pattern.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], _source: &[char]) -> Lint {
        Lint {
            span: matched_tokens[0].span,
            lint_kind: LintKind::Miscellaneous,
            suggestions: vec![Suggestion::ReplaceWith(vec!['t', 'h', 'e', 'i', 'r'])],
            message: "Use the genitive case.".to_string(),
            priority: 31,
        }
    }
}

impl Default for UseGenitive {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_lint_count, assert_suggestion_result};

    use super::UseGenitive;

    #[test]
    fn catches_adjective_noun() {
        assert_suggestion_result(
            "What are there big problems?",
            UseGenitive::default(),
            "What are their big problems?",
        )
    }

    #[test]
    fn catches_just_noun() {
        assert_suggestion_result(
            "What are there problems?",
            UseGenitive::default(),
            "What are their problems?",
        )
    }

    #[test]
    fn allows_clause_termination() {
        assert_lint_count("Look there!", UseGenitive::default(), 0)
    }

    #[test]
    fn allows_there_are() {
        assert_lint_count(
            "Since there are people here, we should be socially aware.",
            UseGenitive::default(),
            0,
        )
    }

    #[test]
    fn allows_there_at_beginning() {
        assert_lint_count(
            "There is a cute cat sitting on the chair at home.",
            UseGenitive::default(),
            0,
        )
    }

    #[test]
    fn catches_they_are() {
        assert_suggestion_result(
            "The students received they're test results today.",
            UseGenitive::default(),
            "The students received their test results today.",
        )
    }
}
