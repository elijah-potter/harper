use harper_data::{Token, TokenStringExt};
use harper_patterns::{Pattern, SequencePattern, WordPatternGroup};
use hashbrown::HashMap;

use super::{Lint, LintKind, PatternLinter, Suggestion};

pub struct DotInitialisms {
    pattern: Box<dyn Pattern>,
    corrections: HashMap<&'static str, &'static str>,
}

impl Default for DotInitialisms {
    fn default() -> Self {
        let mut patterns = WordPatternGroup::default();

        let mut corrections = HashMap::new();
        corrections.insert("ie", "i.e.");
        corrections.insert("eg", "e.g.");

        for target in corrections.keys() {
            let pattern = SequencePattern::default()
                .then_exact_word(target)
                .then_punctuation();

            patterns.add(target, Box::new(pattern));
        }

        Self {
            pattern: Box::new(patterns),
            corrections,
        }
    }
}

impl PatternLinter for DotInitialisms {
    fn pattern(&self) -> &dyn Pattern {
        self.pattern.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Lint {
        let found_word_tok = matched_tokens.first().unwrap();
        let found_word = found_word_tok.span.get_content_string(source);

        let correction = self.corrections.get(found_word.as_str()).unwrap();

        Lint {
            span: matched_tokens.span().unwrap(),
            lint_kind: LintKind::Formatting,
            suggestions: vec![Suggestion::ReplaceWith(correction.chars().collect())],
            message: "Initialisms should have dot-separated letters.".to_owned(),
            priority: 63,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::DotInitialisms;
    use crate::tests::assert_suggestion_result;

    #[test]
    fn matches_eg() {
        assert_suggestion_result(
            "Some text here (eg. more text).",
            DotInitialisms::default(),
            "Some text here (e.g. more text).",
        )
    }
}
