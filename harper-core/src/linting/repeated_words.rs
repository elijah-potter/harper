use super::{Lint, LintKind, PatternLinter, Suggestion};
use crate::patterns::{Pattern, SequencePattern, WordPatternGroup};
use crate::token::{Token, TokenStringExt};

pub struct RepeatedWords {
    pattern: Box<dyn Pattern>
}

impl RepeatedWords {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for RepeatedWords {
    fn default() -> Self {
        let words = [
            "the", "be", "to", "of", "and", "a", "in", "that", "have", "I", "it", "for", "not",
            "on", "with", "he", "as", "you", "do", "at", "this", "is", "but", "his", "by", "from",
            "they", "we", "say", "her", "she", "or", "an", "will", "my", "one", "all", "would",
            "there", "their", "what", "so", "up", "out", "if", "about", "who", "get", "which",
            "go", "me", "when", "make", "can", "like", "time", "no", "just", "him", "know", "take",
            "people", "into", "year", "your", "good", "some", "could", "them", "see", "other",
            "than", "then", "now", "look", "only", "come", "its", "over", "think", "also", "back",
            "after", "use", "two", "how", "our", "work", "first", "well", "way", "even", "new",
            "want", "because", "any", "these", "give", "day", "most", "us", "are"
        ];

        let mut pattern = WordPatternGroup::default();

        for word in words {
            let mut rep = SequencePattern::default();
            rep.then_exact_word(word)
                .then_whitespace()
                .then_exact_word(word);

            pattern.add(word, Box::new(rep));
        }

        Self {
            pattern: Box::new(pattern)
        }
    }
}

impl PatternLinter for RepeatedWords {
    fn pattern(&self) -> &dyn Pattern {
        self.pattern.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Lint {
        Lint {
            span: matched_tokens.span().unwrap(),
            lint_kind: LintKind::Repetition,
            suggestions: vec![Suggestion::ReplaceWith(
                matched_tokens[0].span.get_content(source).to_vec()
            )],
            message: "Did you mean to repeat this word?".to_string(),
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::tests::assert_lint_count;
    use super::RepeatedWords;

    #[test]
    fn catches_basic() {
        assert_lint_count("I wanted the the banana.", RepeatedWords::new(), 1)
    }
}
