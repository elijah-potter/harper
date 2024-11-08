use harper_data::Token;

use super::Pattern;

/// A [`Pattern`] that will match any single token.
pub struct AnyPattern;

impl Pattern for AnyPattern {
    fn matches(&self, tokens: &[Token], _source: &[char]) -> usize {
        if tokens.is_empty() {
            0
        } else {
            1
        }
    }
}
