use hashbrown::HashMap;

use super::Pattern;
use crate::{Token, TokenKind};

pub struct TokenKindPatternGroup {
    /// These are patterns whose first token's kind must be strictly equal.
    strict_patterns: HashMap<TokenKind, Box<dyn Pattern>>,
}

impl Pattern for TokenKindPatternGroup {
    fn matches(&self, tokens: &[Token], source: &[char]) -> usize {
        let Some(first_kind) = &tokens.first().map(|t| t.kind) else {
            return 0;
        };

        let Some(pattern) = self.strict_patterns.get(first_kind) else {
            return 0;
        };

        pattern.matches(tokens, source)
    }
}
