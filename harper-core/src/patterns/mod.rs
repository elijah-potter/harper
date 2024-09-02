use crate::Token;

mod naive_pattern_group;
mod repeating_pattern;
mod token_kind_pattern_group;
mod token_pattern;
mod token_sequence_pattern;
mod word_pattern_group;

pub use naive_pattern_group::NaivePatternGroup;
pub use repeating_pattern::RepeatingPattern;
pub use token_kind_pattern_group::TokenKindPatternGroup;
pub use token_pattern::TokenPattern;
pub use token_sequence_pattern::SequencePattern;
pub use word_pattern_group::WordPatternGroup;

#[cfg(not(feature = "concurrent"))]
pub trait Pattern {
    fn matches(&self, tokens: &[Token], source: &[char]) -> usize;
}

#[cfg(feature = "concurrent")]
pub trait Pattern: Send + Sync {
    fn matches(&self, tokens: &[Token], source: &[char]) -> usize;
}
