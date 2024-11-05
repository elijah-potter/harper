use harper_data::Token;

use super::Pattern;

pub struct WhitespacePattern;

impl Pattern for WhitespacePattern {
    fn matches(&self, tokens: &[Token], _source: &[char]) -> usize {
        tokens
            .iter()
            .position(|t| !t.kind.is_whitespace())
            .unwrap_or(tokens.len())
    }
}
