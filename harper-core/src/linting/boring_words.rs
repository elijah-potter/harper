use crate::{
    patterns::{Pattern, WordPatternGroup},
    Token, TokenStringExt,
};

use super::{Lint, LintKind, PatternLinter};

pub struct BoringWords {
    pattern: Box<dyn Pattern>,
}

impl Default for BoringWords {
    fn default() -> Self {
        let mut pattern = WordPatternGroup::default();

        pattern.add_word("very");
        pattern.add_word("interesting");

        Self {
            pattern: Box::new(pattern),
        }
    }
}

impl PatternLinter for BoringWords {
    fn pattern(&self) -> &dyn Pattern {
        self.pattern.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Lint {
        let matched_word = matched_tokens.span().unwrap().get_content_string(source);

        Lint {
            span: matched_tokens.span().unwrap(),
            lint_kind: LintKind::Enhancement,
            suggestions: vec![],
            message: format!(
                "“{}” is a boring word. Try something a little more exotic.",
                matched_word
            ),
            priority: 127,
        }
    }
}
