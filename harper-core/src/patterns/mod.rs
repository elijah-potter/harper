use crate::Token;

mod any_pattern;
mod consumes_remaining_pattern;
mod naive_pattern_group;
mod repeating_pattern;
mod sequence_pattern;
mod token_kind_pattern_group;
mod whitespace_pattern;
mod word_pattern_group;

pub use any_pattern::AnyPattern;
pub use consumes_remaining_pattern::ConsumesRemainingPattern;
pub use naive_pattern_group::NaivePatternGroup;
pub use repeating_pattern::RepeatingPattern;
pub use sequence_pattern::SequencePattern;
pub use token_kind_pattern_group::TokenKindPatternGroup;
pub use whitespace_pattern::WhitespacePattern;
pub use word_pattern_group::WordPatternGroup;

#[cfg(not(feature = "concurrent"))]
pub trait Pattern {
    fn matches(&self, tokens: &[Token], source: &[char]) -> usize;
}

#[cfg(feature = "concurrent")]
pub trait Pattern: Send + Sync {
    fn matches(&self, tokens: &[Token], source: &[char]) -> usize;
}

#[cfg(feature = "concurrent")]
impl<F> Pattern for F
where
    F: Fn(&Token, &[char]) -> bool,
    F: Send + Sync
{
    fn matches(&self, tokens: &[Token], source: &[char]) -> usize {
        if tokens.is_empty() {
            return 0;
        }

        let tok = &tokens[0];

        if self(tok, source) {
            1
        } else {
            0
        }
    }
}

#[cfg(not(feature = "concurrent"))]
impl<F> Pattern for F
where
    F: Fn(&Token, &[char]) -> bool
{
    fn matches(&self, tokens: &[Token], source: &[char]) -> usize {
        if tokens.is_empty() {
            return 0;
        }

        let tok = &tokens[0];

        if self(tok, source) {
            1
        } else {
            0
        }
    }
}
